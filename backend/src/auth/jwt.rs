use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData as JwtTokenData, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

use resolve_shared::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,    // Subject (user ID)
    pub email: String,
    pub name: String,
    pub role_id: Option<Uuid>,
    pub exp: i64,     // Expiration time
    pub iat: i64,     // Issued at
}

#[derive(Debug)]
pub struct TokenResponse {
    pub token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub fn create_jwt(user: &User) -> Result<TokenResponse, jsonwebtoken::errors::Error> {
    let secret = get_jwt_secret();
    let expires_at = Utc::now() + Duration::hours(24); // 24 hour expiration
    
    let claims = Claims {
        sub: user.id,
        email: user.email.clone(),
        name: format!("{} {}", user.first_name, user.last_name),
        role_id: user.role_id,
        exp: expires_at.timestamp(),
        iat: Utc::now().timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )?;

    Ok(TokenResponse {
        token,
        expires_at,
    })
}

pub fn verify_jwt(token: &str) -> Result<JwtTokenData<Claims>, jsonwebtoken::errors::Error> {
    let secret = get_jwt_secret();
    let validation = Validation::default();

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )
}

fn get_jwt_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| {
        tracing::warn!("JWT_SECRET not set, using default (insecure for production)");
        "your-secret-key".to_string()
    })
}