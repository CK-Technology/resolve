use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State, Query,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;
use crate::{AppState, auth::{verify_token}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    pub event_type: String,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsAuth {
    pub token: String,
}

#[derive(Debug, Clone)]
pub struct WsConnection {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub sender: broadcast::Sender<WsMessage>,
}

pub struct WsManager {
    connections: Arc<RwLock<HashMap<Uuid, WsConnection>>>,
    broadcast: broadcast::Sender<WsMessage>,
}

impl WsManager {
    pub fn new() -> Self {
        let (broadcast, _) = broadcast::channel(1000);
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            broadcast,
        }
    }

    pub async fn add_connection(&self, conn: WsConnection) {
        let mut connections = self.connections.write().await;
        connections.insert(conn.id, conn);
    }

    pub async fn remove_connection(&self, id: &Uuid) {
        let mut connections = self.connections.write().await;
        connections.remove(id);
    }

    pub async fn broadcast_to_user(&self, user_id: Uuid, message: WsMessage) {
        let connections = self.connections.read().await;
        for conn in connections.values() {
            if conn.user_id == Some(user_id) {
                let _ = conn.sender.send(message.clone());
            }
        }
    }

    pub async fn broadcast_to_contact(&self, contact_id: Uuid, message: WsMessage) {
        let connections = self.connections.read().await;
        for conn in connections.values() {
            if conn.contact_id == Some(contact_id) {
                let _ = conn.sender.send(message.clone());
            }
        }
    }

    pub async fn broadcast_all(&self, message: WsMessage) {
        let _ = self.broadcast.send(message);
    }
}

#[derive(Deserialize)]
pub struct WsQuery {
    pub token: Option<String>,
}

pub async fn websocket_handler(
    Query(query): Query<WsQuery>,
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state, query.token))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>, token: Option<String>) {
    let (mut sender, mut receiver) = socket.split();
    let connection_id = Uuid::new_v4();
    
    // Authenticate the connection
    let (user_id, contact_id) = if let Some(token) = token {
        match verify_token(&token) {
            Ok(claims) => {
                // This is a user token
                (Some(Uuid::parse_str(&claims.claims.sub).unwrap_or_default()), None)
            }
            Err(_) => {
                // Try as portal token
                match verify_portal_token(&state, &token).await {
                    Ok(contact_id) => (None, Some(contact_id)),
                    Err(_) => {
                        let _ = sender.send(Message::Text(
                            serde_json::json!({
                                "event_type": "error",
                                "payload": {"message": "Authentication failed"}
                            }).to_string()
                        )).await;
                        return;
                    }
                }
            }
        }
    } else {
        let _ = sender.send(Message::Text(
            serde_json::json!({
                "event_type": "error",
                "payload": {"message": "No authentication token provided"}
            }).to_string()
        )).await;
        return;
    };

    // Create broadcast channel for this connection
    let (tx, mut rx) = broadcast::channel(100);
    
    // Add connection to manager
    let connection = WsConnection {
        id: connection_id,
        user_id,
        contact_id,
        sender: tx.clone(),
    };
    
    state.ws_manager.add_connection(connection.clone()).await;
    
    // Send connection success message
    let _ = sender.send(Message::Text(
        serde_json::json!({
            "event_type": "connected",
            "payload": {
                "connection_id": connection_id,
                "user_id": user_id,
                "contact_id": contact_id
            }
        }).to_string()
    )).await;
    
    // Store connection in database
    let _ = sqlx::query!(
        "INSERT INTO websocket_connections (connection_id, user_id, contact_id, connected_at) 
         VALUES ($1, $2, $3, NOW())",
        connection_id.to_string(),
        user_id,
        contact_id
    )
    .execute(&state.db_pool)
    .await;
    
    // Create tasks for handling messages
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(serde_json::to_string(&msg).unwrap())).await.is_err() {
                break;
            }
        }
    });
    
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    // Handle incoming messages
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                        handle_client_message(&state_clone, connection_id, ws_msg).await;
                    }
                }
                Message::Ping(_) => {
                    // Update last ping time
                    let _ = sqlx::query!(
                        "UPDATE websocket_connections SET last_ping_at = NOW() WHERE connection_id = $1",
                        connection_id.to_string()
                    )
                    .execute(&state_clone.db_pool)
                    .await;
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });
    
    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
    
    // Clean up
    state.ws_manager.remove_connection(&connection_id).await;
    
    // Update disconnection time
    let _ = sqlx::query!(
        "UPDATE websocket_connections SET disconnected_at = NOW() WHERE connection_id = $1",
        connection_id.to_string()
    )
    .execute(&state.db_pool)
    .await;
}

async fn handle_client_message(state: &Arc<AppState>, connection_id: Uuid, message: WsMessage) {
    match message.event_type.as_str() {
        "ping" => {
            // Simple ping/pong
            let connections = state.ws_manager.connections.read().await;
            if let Some(conn) = connections.get(&connection_id) {
                let _ = conn.sender.send(WsMessage {
                    event_type: "pong".to_string(),
                    payload: serde_json::json!({}),
                    timestamp: chrono::Utc::now(),
                });
            }
        }
        "subscribe" => {
            // Handle subscription to specific events
            if let Some(channel) = message.payload.get("channel").and_then(|v| v.as_str()) {
                // Store subscription preference
                tracing::info!("Connection {} subscribed to channel: {}", connection_id, channel);
            }
        }
        _ => {
            tracing::warn!("Unknown message type: {}", message.event_type);
        }
    }
}

async fn verify_portal_token(state: &Arc<AppState>, token: &str) -> Result<Uuid, String> {
    // Verify portal access token
    let result = sqlx::query!(
        "SELECT contact_id FROM portal_access_tokens 
         WHERE token = $1 AND expires_at > NOW()",
        token
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| "Database error".to_string())?;
    
    match result {
        Some(record) => Ok(record.contact_id),
        None => Err("Invalid or expired portal token".to_string()),
    }
}

// Helper functions to send notifications through WebSocket
impl AppState {
    pub async fn notify_user(&self, user_id: Uuid, event_type: &str, payload: serde_json::Value) {
        let message = WsMessage {
            event_type: event_type.to_string(),
            payload,
            timestamp: chrono::Utc::now(),
        };
        self.ws_manager.broadcast_to_user(user_id, message).await;
    }

    pub async fn notify_contact(&self, contact_id: Uuid, event_type: &str, payload: serde_json::Value) {
        let message = WsMessage {
            event_type: event_type.to_string(),
            payload,
            timestamp: chrono::Utc::now(),
        };
        self.ws_manager.broadcast_to_contact(contact_id, message).await;
    }

    pub async fn broadcast_notification(&self, event_type: &str, payload: serde_json::Value) {
        let message = WsMessage {
            event_type: event_type.to_string(),
            payload,
            timestamp: chrono::Utc::now(),
        };
        self.ws_manager.broadcast_all(message).await;
    }
}