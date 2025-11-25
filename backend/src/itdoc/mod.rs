pub mod credentials;
pub mod domains;
pub mod ssl_certificates;
pub mod networks;
pub mod software_licenses;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::AppState;

pub fn itdoc_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Credentials routes
        .nest("/credentials", credentials::credential_routes())
        // Domains routes
        .nest("/domains", domains::domain_routes())
        // SSL Certificates routes
        .nest("/ssl", ssl_certificates::ssl_routes())
        // Networks routes
        .nest("/networks", networks::network_routes())
        // Software Licenses routes
        .nest("/licenses", software_licenses::license_routes())
        // Overview route
        .route("/overview/:client_id", get(get_itdoc_overview))
}

#[derive(Debug, Serialize)]
pub struct ITDocOverview {
    pub client_id: Uuid,
    pub credentials_count: i64,
    pub domains_count: i64,
    pub ssl_certificates_count: i64,
    pub networks_count: i64,
    pub software_licenses_count: i64,
    pub expiring_domains: Vec<resolve_shared::Domain>,
    pub expiring_ssl_certificates: Vec<resolve_shared::SslCertificate>,
    pub expiring_licenses: Vec<resolve_shared::SoftwareLicense>,
}

async fn get_itdoc_overview(
    State(state): State<Arc<AppState>>,
    Path(client_id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, StatusCode> {
    // Get counts for each category
    let credentials_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM credentials WHERE client_id = $1",
        client_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .unwrap_or(0);

    let domains_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM domains WHERE client_id = $1",
        client_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .unwrap_or(0);

    let ssl_certificates_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM ssl_certificates WHERE client_id = $1",
        client_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .unwrap_or(0);

    let networks_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM networks WHERE client_id = $1",
        client_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .unwrap_or(0);

    let software_licenses_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM software_licenses WHERE client_id = $1",
        client_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .unwrap_or(0);

    // Get items expiring within 30 days
    let expiring_domains = sqlx::query_as!(
        resolve_shared::Domain,
        r#"
        SELECT id, client_id, name, registrar, nameservers, registration_date,
               expiry_date, auto_renew, dns_records, notes, created_at, updated_at
        FROM domains 
        WHERE client_id = $1 AND expiry_date <= CURRENT_DATE + INTERVAL '30 days'
        ORDER BY expiry_date ASC
        LIMIT 5
        "#,
        client_id
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let expiring_ssl_certificates = sqlx::query_as!(
        resolve_shared::SslCertificate,
        r#"
        SELECT id, domain_id, client_id, name, common_name, subject_alt_names,
               issuer, issued_date, expiry_date, certificate_chain, private_key,
               auto_renew, status, created_at, updated_at
        FROM ssl_certificates 
        WHERE client_id = $1 AND expiry_date <= CURRENT_DATE + INTERVAL '30 days'
        ORDER BY expiry_date ASC
        LIMIT 5
        "#,
        client_id
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let expiring_licenses = sqlx::query_as!(
        resolve_shared::SoftwareLicense,
        r#"
        SELECT id, client_id, name, vendor, version, license_key, license_type,
               seats, used_seats, purchase_date, expiry_date, renewal_date,
               cost, notes, created_at, updated_at
        FROM software_licenses 
        WHERE client_id = $1 AND expiry_date <= CURRENT_DATE + INTERVAL '30 days'
        ORDER BY expiry_date ASC
        LIMIT 5
        "#,
        client_id
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let overview = ITDocOverview {
        client_id,
        credentials_count,
        domains_count,
        ssl_certificates_count,
        networks_count,
        software_licenses_count,
        expiring_domains,
        expiring_ssl_certificates,
        expiring_licenses,
    };

    Ok(Json(overview))
}