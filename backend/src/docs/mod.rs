use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::Redoc;

// Import all the schemas and handlers we want to document
use crate::handlers::{clients, tickets, assets, auth, m365, azure, bitwarden, network};
use crate::models;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Resolve API",
        version = "1.0.0",
        description = "Resolve MSP Management Platform API Documentation",
        license(
            name = "GPL-3.0",
            url = "https://www.gnu.org/licenses/gpl-3.0.html"
        ),
        contact(
            name = "Resolve Support",
            url = "https://github.com/ghostkellz/resolve",
            email = "support@resolve.sh"
        )
    ),
    servers(
        (url = "/api/v1", description = "Production API"),
        (url = "http://localhost:8080/api/v1", description = "Development API")
    ),
    paths(
        // Authentication endpoints
        auth::register,
        auth::login,
        auth::logout,
        auth::refresh_token,
        auth::me,
        auth::change_password,
        auth::forgot_password,
        auth::reset_password,
        auth::enable_mfa,
        auth::disable_mfa,
        auth::verify_mfa,
        
        // Client management endpoints
        clients::list_clients,
        clients::create_client,
        clients::get_client,
        clients::update_client,
        clients::delete_client,
        clients::get_client_contacts,
        clients::create_client_contact,
        clients::update_client_contact,
        clients::delete_client_contact,
        
        // Ticket management endpoints
        tickets::list_tickets,
        tickets::create_ticket,
        tickets::get_ticket,
        tickets::update_ticket,
        tickets::delete_ticket,
        tickets::assign_ticket,
        tickets::close_ticket,
        tickets::reopen_ticket,
        tickets::add_ticket_comment,
        tickets::get_ticket_comments,
        tickets::update_ticket_status,
        tickets::add_time_entry,
        
        // Asset management endpoints
        assets::list_assets,
        assets::create_asset,
        assets::get_asset,
        assets::update_asset,
        assets::delete_asset,
        assets::get_asset_history,
        assets::add_asset_file,
        assets::get_asset_files,
        assets::delete_asset_file,
        
        // Microsoft 365 integration endpoints
        m365::list_tenants,
        m365::create_tenant,
        m365::get_tenant,
        m365::update_tenant,
        m365::delete_tenant,
        m365::sync_tenant,
        m365::get_tenant_users,
        m365::get_tenant_groups,
        m365::get_tenant_licenses,
        m365::get_tenant_security,
        
        // Azure integration endpoints
        azure::list_subscriptions,
        azure::create_subscription,
        azure::get_subscription,
        azure::update_subscription,
        azure::delete_subscription,
        azure::sync_subscription,
        azure::get_subscription_resources,
        azure::get_subscription_costs,
        azure::get_resource_groups,
        azure::get_virtual_networks,
        
        // Bitwarden integration endpoints
        bitwarden::list_servers,
        bitwarden::create_server,
        bitwarden::get_server,
        bitwarden::update_server,
        bitwarden::delete_server,
        bitwarden::sync_server,
        bitwarden::get_server_organizations,
        bitwarden::get_server_collections,
        bitwarden::get_server_items,
        
        // Network integration endpoints
        network::list_controllers,
        network::create_controller,
        network::get_controller,
        network::update_controller,
        network::delete_controller,
        network::sync_controller,
        network::get_controller_devices,
        network::get_controller_sites,
        network::get_dns_zones,
        network::get_dns_records,
    ),
    components(
        schemas(
            // Authentication schemas
            models::User,
            models::LoginRequest,
            models::LoginResponse,
            models::RegisterRequest,
            models::TokenRefreshRequest,
            models::ChangePasswordRequest,
            models::ForgotPasswordRequest,
            models::ResetPasswordRequest,
            models::MfaSetupResponse,
            models::MfaVerificationRequest,
            
            // Client schemas
            models::Client,
            models::CreateClientRequest,
            models::UpdateClientRequest,
            models::Contact,
            models::CreateContactRequest,
            models::UpdateContactRequest,
            
            // Ticket schemas
            models::Ticket,
            models::CreateTicketRequest,
            models::UpdateTicketRequest,
            models::TicketComment,
            models::CreateTicketCommentRequest,
            models::TicketStatusUpdate,
            models::TicketAssignment,
            models::TimeEntry,
            models::CreateTimeEntryRequest,
            
            // Asset schemas
            models::Asset,
            models::CreateAssetRequest,
            models::UpdateAssetRequest,
            models::AssetFile,
            models::AssetHistory,
            
            // Microsoft 365 schemas
            models::M365Tenant,
            models::CreateM365TenantRequest,
            models::UpdateM365TenantRequest,
            models::M365User,
            models::M365Group,
            models::M365License,
            models::M365SecurityReport,
            
            // Azure schemas
            models::AzureSubscription,
            models::CreateAzureSubscriptionRequest,
            models::UpdateAzureSubscriptionRequest,
            models::AzureResource,
            models::AzureResourceGroup,
            models::AzureVirtualNetwork,
            models::AzureCostSummary,
            
            // Bitwarden schemas
            models::BitwardenServer,
            models::CreateBitwardenServerRequest,
            models::UpdateBitwardenServerRequest,
            models::BitwardenOrganization,
            models::BitwardenCollection,
            models::BitwardenItem,
            
            // Network schemas
            models::NetworkController,
            models::CreateNetworkControllerRequest,
            models::UpdateNetworkControllerRequest,
            models::NetworkDevice,
            models::NetworkSite,
            models::DnsZone,
            models::DnsRecord,
            
            // Common schemas
            models::ApiResponse,
            models::ValidationError,
            models::PaginationParams,
            models::PaginatedResponse,
            models::ErrorResponse,
        )
    ),
    tags(
        (name = "Authentication", description = "User authentication and authorization"),
        (name = "Clients", description = "Client and contact management"),
        (name = "Tickets", description = "Support ticket management"),
        (name = "Assets", description = "IT asset and infrastructure management"),
        (name = "Time Tracking", description = "Time tracking and billing"),
        (name = "Microsoft 365", description = "Microsoft 365 tenant integration and management"),
        (name = "Azure", description = "Azure subscription and resource monitoring"),
        (name = "Bitwarden", description = "Bitwarden/Vaultwarden password management integration"),
        (name = "Network", description = "Network infrastructure management (UniFi, FortiGate, DNS)"),
        (name = "Invoicing", description = "Invoice generation and billing management"),
        (name = "Reporting", description = "Reports and analytics"),
        (name = "Settings", description = "System configuration and settings"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "jwt",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
            components.add_security_scheme(
                "api_key",
                utoipa::openapi::security::SecurityScheme::ApiKey(
                    utoipa::openapi::security::ApiKeyBuilder::new()
                        .location(utoipa::openapi::security::ApiKeyLocation::Header)
                        .name("X-API-Key")
                        .build(),
                ),
            );
        }
    }
}

pub fn create_docs_routes() -> axum::Router {
    axum::Router::new()
        .merge(SwaggerUi::new("/docs/swagger").url("/docs/openapi.json", ApiDoc::openapi()))
        .merge(RapiDoc::new("/docs/openapi.json").path("/docs/rapidoc"))
        .merge(Redoc::new("/docs/openapi.json").path("/docs/redoc"))
}