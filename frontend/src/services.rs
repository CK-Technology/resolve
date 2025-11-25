// API service layer for communicating with backend
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

const API_BASE_URL: &str = "/api/v1";
const AUTH_TOKEN_KEY: &str = "resolve_auth_token";

// ============================================
// ERROR HANDLING
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
    pub code: Option<String>,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

// ============================================
// HTTP CLIENT
// ============================================

pub struct ApiClient;

impl ApiClient {
    fn get_auth_token() -> Option<String> {
        LocalStorage::get::<String>(AUTH_TOKEN_KEY).ok()
    }

    pub fn set_auth_token(token: &str) {
        let _ = LocalStorage::set(AUTH_TOKEN_KEY, token);
    }

    pub fn clear_auth_token() {
        LocalStorage::delete(AUTH_TOKEN_KEY);
    }

    pub fn is_authenticated() -> bool {
        Self::get_auth_token().is_some()
    }

    async fn request<T: DeserializeOwned>(
        method: &str,
        endpoint: &str,
    ) -> ApiResult<T> {
        let url = format!("{}{}", API_BASE_URL, endpoint);

        let mut req = match method {
            "GET" => Request::get(&url),
            "DELETE" => Request::delete(&url),
            _ => return Err(ApiError { message: "Invalid method".to_string(), code: None }),
        };

        if let Some(token) = Self::get_auth_token() {
            req = req.header("Authorization", &format!("Bearer {}", token));
        }

        let response = req.send().await.map_err(|e| ApiError {
            message: e.to_string(),
            code: Some("NETWORK_ERROR".to_string()),
        })?;

        if response.ok() {
            response.json::<T>().await.map_err(|e| ApiError {
                message: e.to_string(),
                code: Some("PARSE_ERROR".to_string()),
            })
        } else {
            let error = response.json::<ApiError>().await.unwrap_or(ApiError {
                message: format!("HTTP Error: {}", response.status()),
                code: Some(format!("HTTP_{}", response.status())),
            });
            Err(error)
        }
    }

    async fn request_with_body<T: DeserializeOwned, B: Serialize>(
        method: &str,
        endpoint: &str,
        body: &B,
    ) -> ApiResult<T> {
        let url = format!("{}{}", API_BASE_URL, endpoint);

        let mut req = match method {
            "POST" => Request::post(&url),
            "PUT" => Request::put(&url),
            "PATCH" => Request::patch(&url),
            _ => return Err(ApiError { message: "Invalid method".to_string(), code: None }),
        };

        if let Some(token) = Self::get_auth_token() {
            req = req.header("Authorization", &format!("Bearer {}", token));
        }

        let response = req
            .header("Content-Type", "application/json")
            .json(body)
            .map_err(|e| ApiError {
                message: e.to_string(),
                code: Some("SERIALIZE_ERROR".to_string()),
            })?
            .send()
            .await
            .map_err(|e| ApiError {
                message: e.to_string(),
                code: Some("NETWORK_ERROR".to_string()),
            })?;

        if response.ok() {
            response.json::<T>().await.map_err(|e| ApiError {
                message: e.to_string(),
                code: Some("PARSE_ERROR".to_string()),
            })
        } else {
            let error = response.json::<ApiError>().await.unwrap_or(ApiError {
                message: format!("HTTP Error: {}", response.status()),
                code: Some(format!("HTTP_{}", response.status())),
            });
            Err(error)
        }
    }

    // GET request
    pub async fn get<T: DeserializeOwned>(endpoint: &str) -> ApiResult<T> {
        Self::request("GET", endpoint).await
    }

    // POST request
    pub async fn post<T: DeserializeOwned, B: Serialize>(endpoint: &str, body: &B) -> ApiResult<T> {
        Self::request_with_body("POST", endpoint, body).await
    }

    // PUT request
    pub async fn put<T: DeserializeOwned, B: Serialize>(endpoint: &str, body: &B) -> ApiResult<T> {
        Self::request_with_body("PUT", endpoint, body).await
    }

    // PATCH request
    pub async fn patch<T: DeserializeOwned, B: Serialize>(endpoint: &str, body: &B) -> ApiResult<T> {
        Self::request_with_body("PATCH", endpoint, body).await
    }

    // DELETE request
    pub async fn delete<T: DeserializeOwned>(endpoint: &str) -> ApiResult<T> {
        Self::request("DELETE", endpoint).await
    }
}

// ============================================
// COMMON TYPES
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
}

// ============================================
// AUTH SERVICE
// ============================================

pub mod auth {
    use super::*;

    #[derive(Debug, Clone, Serialize)]
    pub struct LoginRequest {
        pub email: String,
        pub password: String,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct LoginResponse {
        pub token: String,
        pub user: User,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct User {
        pub id: String,
        pub email: String,
        pub first_name: String,
        pub last_name: String,
        pub role: String,
    }

    pub async fn login(email: &str, password: &str) -> ApiResult<LoginResponse> {
        let req = LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
        };
        let response: LoginResponse = ApiClient::post("/auth/login", &req).await?;
        ApiClient::set_auth_token(&response.token);
        Ok(response)
    }

    pub async fn logout() {
        ApiClient::clear_auth_token();
    }

    pub async fn get_current_user() -> ApiResult<User> {
        ApiClient::get("/auth/me").await
    }
}

// ============================================
// DASHBOARD SERVICE
// ============================================

pub mod dashboard {
    use super::*;
    use rust_decimal::Decimal;

    #[derive(Debug, Clone, Deserialize)]
    pub struct DashboardStats {
        pub overview: OverviewStats,
        pub tickets: TicketStats,
        pub time: TimeStats,
        pub invoices: InvoiceStats,
        pub clients: ClientStats,
        pub assets: AssetStats,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct OverviewStats {
        pub total_clients: i64,
        pub active_tickets: i64,
        pub monthly_revenue: Decimal,
        pub unbilled_time: Decimal,
        pub overdue_invoices: i64,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct TicketStats {
        pub open: i64,
        pub in_progress: i64,
        pub pending: i64,
        pub resolved_today: i64,
        pub sla_breached: i64,
        pub avg_response_time_hours: Option<f64>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct TimeStats {
        pub hours_today: Decimal,
        pub billable_hours_today: Decimal,
        pub hours_this_week: Decimal,
        pub active_timers: i64,
        pub team_utilization: Option<f64>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct InvoiceStats {
        pub outstanding_amount: Decimal,
        pub overdue_amount: Decimal,
        pub draft_count: i64,
        pub paid_this_month: Decimal,
        pub collection_ratio: Option<f64>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct ClientStats {
        pub total_clients: i64,
        pub new_this_month: i64,
        pub top_clients_by_revenue: Vec<TopClient>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct TopClient {
        pub name: String,
        pub revenue: Decimal,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct AssetStats {
        pub total_assets: i64,
        pub critical_alerts: i64,
        pub warranty_expiring: i64,
        pub online_percentage: Option<f64>,
    }

    pub async fn get_stats() -> ApiResult<DashboardStats> {
        ApiClient::get("/dashboard").await
    }
}

// ============================================
// CLIENTS SERVICE
// ============================================

pub mod clients {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Client {
        pub id: String,
        pub name: String,
        pub identifier: Option<String>,
        pub email: Option<String>,
        pub phone: Option<String>,
        pub address: Option<String>,
        pub city: Option<String>,
        pub state: Option<String>,
        pub zip_code: Option<String>,
        pub country: Option<String>,
        pub website: Option<String>,
        pub notes: Option<String>,
        pub is_active: bool,
        pub created_at: String,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct CreateClientRequest {
        pub name: String,
        pub email: Option<String>,
        pub phone: Option<String>,
        pub address: Option<String>,
        pub city: Option<String>,
        pub state: Option<String>,
        pub zip_code: Option<String>,
        pub country: Option<String>,
        pub website: Option<String>,
        pub notes: Option<String>,
    }

    pub async fn list(page: u32, per_page: u32) -> ApiResult<PaginatedResponse<Client>> {
        ApiClient::get(&format!("/clients?page={}&per_page={}", page, per_page)).await
    }

    pub async fn get(id: &str) -> ApiResult<Client> {
        ApiClient::get(&format!("/clients/{}", id)).await
    }

    pub async fn create(client: &CreateClientRequest) -> ApiResult<Client> {
        ApiClient::post("/clients", client).await
    }

    pub async fn update(id: &str, client: &CreateClientRequest) -> ApiResult<Client> {
        ApiClient::put(&format!("/clients/{}", id), client).await
    }

    pub async fn delete(id: &str) -> ApiResult<()> {
        ApiClient::delete(&format!("/clients/{}", id)).await
    }
}

// ============================================
// TICKETS SERVICE
// ============================================

pub mod tickets {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Ticket {
        pub id: String,
        pub number: i32,
        pub subject: String,
        pub description: Option<String>,
        pub status: String,
        pub priority: String,
        pub client_id: String,
        pub client_name: Option<String>,
        pub assigned_to: Option<String>,
        pub assigned_to_name: Option<String>,
        pub queue_id: Option<String>,
        pub queue_name: Option<String>,
        pub created_at: String,
        pub updated_at: Option<String>,
        pub resolved_at: Option<String>,
        pub sla_response_due: Option<String>,
        pub sla_resolution_due: Option<String>,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct CreateTicketRequest {
        pub subject: String,
        pub description: Option<String>,
        pub priority: String,
        pub client_id: String,
        pub assigned_to: Option<String>,
        pub queue_id: Option<String>,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct UpdateTicketRequest {
        pub subject: Option<String>,
        pub description: Option<String>,
        pub status: Option<String>,
        pub priority: Option<String>,
        pub assigned_to: Option<String>,
        pub queue_id: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TicketReply {
        pub id: String,
        pub ticket_id: String,
        pub user_id: String,
        pub user_name: String,
        pub content: String,
        pub is_internal: bool,
        pub created_at: String,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct CreateReplyRequest {
        pub content: String,
        pub is_internal: bool,
    }

    pub async fn list(
        page: u32,
        per_page: u32,
        status: Option<&str>,
        client_id: Option<&str>,
    ) -> ApiResult<PaginatedResponse<Ticket>> {
        let mut url = format!("/tickets?page={}&per_page={}", page, per_page);
        if let Some(s) = status {
            url.push_str(&format!("&status={}", s));
        }
        if let Some(c) = client_id {
            url.push_str(&format!("&client_id={}", c));
        }
        ApiClient::get(&url).await
    }

    pub async fn get(id: &str) -> ApiResult<Ticket> {
        ApiClient::get(&format!("/tickets/{}", id)).await
    }

    pub async fn create(ticket: &CreateTicketRequest) -> ApiResult<Ticket> {
        ApiClient::post("/tickets", ticket).await
    }

    pub async fn update(id: &str, ticket: &UpdateTicketRequest) -> ApiResult<Ticket> {
        ApiClient::patch(&format!("/tickets/{}", id), ticket).await
    }

    pub async fn delete(id: &str) -> ApiResult<()> {
        ApiClient::delete(&format!("/tickets/{}", id)).await
    }

    pub async fn get_replies(ticket_id: &str) -> ApiResult<Vec<TicketReply>> {
        ApiClient::get(&format!("/tickets/{}/replies", ticket_id)).await
    }

    pub async fn add_reply(ticket_id: &str, reply: &CreateReplyRequest) -> ApiResult<TicketReply> {
        ApiClient::post(&format!("/tickets/{}/replies", ticket_id), reply).await
    }
}

// ============================================
// TIME TRACKING SERVICE
// ============================================

pub mod time_tracking {
    use super::*;
    use rust_decimal::Decimal;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TimeEntry {
        pub id: String,
        pub user_id: String,
        pub user_name: Option<String>,
        pub client_id: Option<String>,
        pub client_name: Option<String>,
        pub ticket_id: Option<String>,
        pub ticket_number: Option<i32>,
        pub project_id: Option<String>,
        pub project_name: Option<String>,
        pub description: String,
        pub start_time: String,
        pub end_time: Option<String>,
        pub duration_minutes: Option<i32>,
        pub billable: bool,
        pub billed: bool,
        pub hourly_rate: Option<Decimal>,
        pub total_amount: Option<Decimal>,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct CreateTimeEntryRequest {
        pub client_id: Option<String>,
        pub ticket_id: Option<String>,
        pub project_id: Option<String>,
        pub description: String,
        pub start_time: Option<String>,
        pub end_time: Option<String>,
        pub duration_minutes: Option<i32>,
        pub billable: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ActiveTimer {
        pub id: String,
        pub description: String,
        pub start_time: String,
        pub client_name: Option<String>,
        pub ticket_number: Option<i32>,
    }

    pub async fn list(
        page: u32,
        per_page: u32,
        user_id: Option<&str>,
    ) -> ApiResult<PaginatedResponse<TimeEntry>> {
        let mut url = format!("/time?page={}&per_page={}", page, per_page);
        if let Some(u) = user_id {
            url.push_str(&format!("&user_id={}", u));
        }
        ApiClient::get(&url).await
    }

    pub async fn get(id: &str) -> ApiResult<TimeEntry> {
        ApiClient::get(&format!("/time/{}", id)).await
    }

    pub async fn create(entry: &CreateTimeEntryRequest) -> ApiResult<TimeEntry> {
        ApiClient::post("/time", entry).await
    }

    pub async fn start_timer(entry: &CreateTimeEntryRequest) -> ApiResult<TimeEntry> {
        ApiClient::post("/time/start", entry).await
    }

    pub async fn stop_timer(id: &str) -> ApiResult<TimeEntry> {
        ApiClient::post(&format!("/time/{}/stop", id), &()).await
    }

    pub async fn get_active_timer() -> ApiResult<Option<ActiveTimer>> {
        ApiClient::get("/time/active").await
    }

    pub async fn delete(id: &str) -> ApiResult<()> {
        ApiClient::delete(&format!("/time/{}", id)).await
    }
}

// ============================================
// ASSETS SERVICE
// ============================================

pub mod assets {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Asset {
        pub id: String,
        pub client_id: String,
        pub client_name: Option<String>,
        pub name: String,
        pub asset_type: String,
        pub manufacturer: Option<String>,
        pub model: Option<String>,
        pub serial_number: Option<String>,
        pub status: String,
        pub location: Option<String>,
        pub ip_address: Option<String>,
        pub mac_address: Option<String>,
        pub purchase_date: Option<String>,
        pub warranty_expiry: Option<String>,
        pub notes: Option<String>,
        pub custom_fields: Option<serde_json::Value>,
        pub created_at: String,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct CreateAssetRequest {
        pub client_id: String,
        pub name: String,
        pub asset_type: String,
        pub manufacturer: Option<String>,
        pub model: Option<String>,
        pub serial_number: Option<String>,
        pub status: String,
        pub location: Option<String>,
        pub ip_address: Option<String>,
        pub mac_address: Option<String>,
        pub purchase_date: Option<String>,
        pub warranty_expiry: Option<String>,
        pub notes: Option<String>,
    }

    pub async fn list(
        page: u32,
        per_page: u32,
        client_id: Option<&str>,
    ) -> ApiResult<PaginatedResponse<Asset>> {
        let mut url = format!("/assets?page={}&per_page={}", page, per_page);
        if let Some(c) = client_id {
            url.push_str(&format!("&client_id={}", c));
        }
        ApiClient::get(&url).await
    }

    pub async fn get(id: &str) -> ApiResult<Asset> {
        ApiClient::get(&format!("/assets/{}", id)).await
    }

    pub async fn create(asset: &CreateAssetRequest) -> ApiResult<Asset> {
        ApiClient::post("/assets", asset).await
    }

    pub async fn update(id: &str, asset: &CreateAssetRequest) -> ApiResult<Asset> {
        ApiClient::put(&format!("/assets/{}", id), asset).await
    }

    pub async fn delete(id: &str) -> ApiResult<()> {
        ApiClient::delete(&format!("/assets/{}", id)).await
    }
}

// ============================================
// INVOICES SERVICE
// ============================================

pub mod invoices {
    use super::*;
    use rust_decimal::Decimal;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Invoice {
        pub id: String,
        pub invoice_number: String,
        pub client_id: String,
        pub client_name: Option<String>,
        pub status: String,
        pub date: String,
        pub due_date: String,
        pub subtotal: Decimal,
        pub tax: Decimal,
        pub total: Decimal,
        pub notes: Option<String>,
        pub created_at: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct InvoiceLineItem {
        pub id: String,
        pub invoice_id: String,
        pub description: String,
        pub quantity: Decimal,
        pub unit_price: Decimal,
        pub total: Decimal,
    }

    pub async fn list(
        page: u32,
        per_page: u32,
        status: Option<&str>,
    ) -> ApiResult<PaginatedResponse<Invoice>> {
        let mut url = format!("/invoices?page={}&per_page={}", page, per_page);
        if let Some(s) = status {
            url.push_str(&format!("&status={}", s));
        }
        ApiClient::get(&url).await
    }

    pub async fn get(id: &str) -> ApiResult<Invoice> {
        ApiClient::get(&format!("/invoices/{}", id)).await
    }

    pub async fn get_line_items(id: &str) -> ApiResult<Vec<InvoiceLineItem>> {
        ApiClient::get(&format!("/invoices/{}/items", id)).await
    }
}

// ============================================
// ANALYTICS SERVICE
// ============================================

pub mod analytics {
    use super::*;
    use rust_decimal::Decimal;

    #[derive(Debug, Clone, Deserialize)]
    pub struct TechnicianUtilization {
        pub user_id: String,
        pub user_name: String,
        pub total_hours: Decimal,
        pub billable_hours: Decimal,
        pub non_billable_hours: Decimal,
        pub utilization_rate: f64,
        pub tickets_resolved: i64,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct ClientProfitability {
        pub client_id: String,
        pub client_name: String,
        pub revenue: Decimal,
        pub cost: Decimal,
        pub margin: Decimal,
        pub margin_percentage: f64,
        pub total_tickets: i64,
        pub total_hours: Decimal,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct SlaCompliance {
        pub total_tickets: i64,
        pub met_response_sla: i64,
        pub met_resolution_sla: i64,
        pub response_compliance_rate: f64,
        pub resolution_compliance_rate: f64,
    }

    pub async fn get_utilization(start_date: &str, end_date: &str) -> ApiResult<Vec<TechnicianUtilization>> {
        ApiClient::get(&format!("/analytics/utilization?start_date={}&end_date={}", start_date, end_date)).await
    }

    pub async fn get_profitability(start_date: &str, end_date: &str) -> ApiResult<Vec<ClientProfitability>> {
        ApiClient::get(&format!("/analytics/profitability?start_date={}&end_date={}", start_date, end_date)).await
    }

    pub async fn get_sla_compliance(start_date: &str, end_date: &str) -> ApiResult<SlaCompliance> {
        ApiClient::get(&format!("/analytics/sla-compliance?start_date={}&end_date={}", start_date, end_date)).await
    }
}

// ============================================
// QUEUES SERVICE
// ============================================

pub mod queues {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TicketQueue {
        pub id: String,
        pub name: String,
        pub description: Option<String>,
        pub is_active: bool,
        pub auto_assign: bool,
        pub assignment_method: String,
    }

    pub async fn list() -> ApiResult<Vec<TicketQueue>> {
        ApiClient::get("/queues").await
    }

    pub async fn get(id: &str) -> ApiResult<TicketQueue> {
        ApiClient::get(&format!("/queues/{}", id)).await
    }
}

// ============================================
// KNOWLEDGE BASE SERVICE
// ============================================

pub mod knowledge_base {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Article {
        pub id: String,
        pub title: String,
        pub slug: String,
        pub content: String,
        pub category_id: Option<String>,
        pub category_name: Option<String>,
        pub is_published: bool,
        pub view_count: i64,
        pub created_at: String,
        pub updated_at: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Category {
        pub id: String,
        pub name: String,
        pub slug: String,
        pub description: Option<String>,
        pub parent_id: Option<String>,
        pub article_count: i64,
    }

    pub async fn list_articles(
        page: u32,
        per_page: u32,
        category_id: Option<&str>,
    ) -> ApiResult<PaginatedResponse<Article>> {
        let mut url = format!("/kb/articles?page={}&per_page={}", page, per_page);
        if let Some(c) = category_id {
            url.push_str(&format!("&category_id={}", c));
        }
        ApiClient::get(&url).await
    }

    pub async fn get_article(id: &str) -> ApiResult<Article> {
        ApiClient::get(&format!("/kb/articles/{}", id)).await
    }

    pub async fn search(query: &str) -> ApiResult<Vec<Article>> {
        ApiClient::get(&format!("/kb/search?q={}", query)).await
    }

    pub async fn list_categories() -> ApiResult<Vec<Category>> {
        ApiClient::get("/kb/categories").await
    }
}
