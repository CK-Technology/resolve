# Resolve MSP Platform - Architecture Overview

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Load Balancer / CDN                         │
│                        (Nginx / Cloudflare)                          │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                              Frontend                                │
│                     Yew + WebAssembly (WASM)                        │
│    ┌─────────────────────────────────────────────────────────┐      │
│    │  Components    │    Pages      │    Services           │      │
│    │  - Layout      │    - Dashboard │   - API Client       │      │
│    │  - Auth        │    - Tickets   │   - WebSocket        │      │
│    │  - Forms       │    - Clients   │   - Theme            │      │
│    │  - Tables      │    - Assets    │   - Auth             │      │
│    │  - Charts      │    - Time      │                       │      │
│    └─────────────────────────────────────────────────────────┘      │
└────────────────────────────────┬────────────────────────────────────┘
                                 │ HTTP/WebSocket
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                              Backend                                 │
│                      Axum + Tokio (Rust)                            │
│    ┌───────────────────────────────────────────────────────────┐    │
│    │                      HTTP Layer                            │    │
│    │  ┌─────────┐  ┌──────────┐  ┌──────────┐  ┌─────────┐    │    │
│    │  │  Auth   │  │ Handlers │  │Middleware│  │ OpenAPI │    │    │
│    │  └─────────┘  └──────────┘  └──────────┘  └─────────┘    │    │
│    └───────────────────────────────────────────────────────────┘    │
│    ┌───────────────────────────────────────────────────────────┐    │
│    │                    Services Layer                          │    │
│    │  ┌─────────┐  ┌──────────┐  ┌──────────┐  ┌─────────┐    │    │
│    │  │  Email  │  │ Workflows│  │   Jobs   │  │  Cache  │    │    │
│    │  └─────────┘  └──────────┘  └──────────┘  └─────────┘    │    │
│    └───────────────────────────────────────────────────────────┘    │
│    ┌───────────────────────────────────────────────────────────┐    │
│    │                  Integration Layer                         │    │
│    │  ┌─────────┐  ┌──────────┐  ┌──────────┐  ┌─────────┐    │    │
│    │  │  M365   │  │   Azure  │  │ Bitwarden│  │  Teams  │    │    │
│    │  └─────────┘  └──────────┘  └──────────┘  └─────────┘    │    │
│    └───────────────────────────────────────────────────────────┘    │
└─────────┬─────────────────────────────────────┬─────────────────────┘
          │                                     │
          ▼                                     ▼
┌─────────────────────┐             ┌─────────────────────┐
│    PostgreSQL       │             │       Redis         │
│    Primary DB       │             │   Cache / Queue     │
│                     │             │                     │
│  - Clients          │             │  - Session Data     │
│  - Tickets          │             │  - API Cache        │
│  - Assets           │             │  - Rate Limits      │
│  - Invoices         │             │  - Job Queue        │
│  - Users            │             │                     │
└─────────────────────┘             └─────────────────────┘
```

## Module Structure

### Backend (`/backend/src/`)

```
src/
├── main.rs                 # Application entry point
├── config.rs               # Configuration management
├── database.rs             # Database connection & migrations
├── error.rs                # Error types and handling
├── pagination.rs           # Pagination utilities
├── validation.rs           # Request validation
├── websocket.rs            # WebSocket handler
├── openapi.rs              # OpenAPI/Swagger docs
│
├── auth/                   # Authentication & Authorization
│   ├── mod.rs
│   ├── jwt.rs              # JWT token handling
│   ├── oauth.rs            # OAuth2 providers
│   ├── oidc.rs             # OpenID Connect
│   ├── saml.rs             # SAML SSO
│   ├── totp.rs             # 2FA TOTP
│   ├── api_keys.rs         # API key management
│   ├── rbac.rs             # Role-based access control
│   └── middleware.rs       # Auth middleware
│
├── handlers/               # HTTP Request Handlers
│   ├── mod.rs
│   ├── clients.rs          # Client CRUD
│   ├── tickets.rs          # Ticket management
│   ├── ticket_advanced.rs  # Advanced ticket features
│   ├── time_tracking.rs    # Time entries
│   ├── invoices.rs         # Invoice management
│   ├── billing.rs          # Billing operations
│   ├── assets.rs           # Asset management
│   ├── asset_layouts.rs    # Custom asset layouts
│   ├── asset_relationships.rs
│   ├── passwords.rs        # Password vault
│   ├── knowledge_base.rs   # KB articles
│   ├── projects.rs         # Project management
│   ├── sla_management.rs   # SLA policies & tracking
│   ├── analytics.rs        # Reporting & analytics
│   ├── teams.rs            # MS Teams integration
│   └── ...
│
├── services/               # Business Logic Services
│   ├── mod.rs
│   ├── email.rs            # Email sending
│   ├── email_processor.rs  # Inbound email processing
│   ├── bms_workflows.rs    # BMS billing workflows
│   ├── encryption.rs       # Data encryption
│   ├── password_manager.rs # Password encryption
│   ├── teams_integration.rs# Teams notifications
│   ├── domain_ssl_monitor.rs
│   ├── cache.rs            # Redis caching
│   ├── audit.rs            # Audit logging
│   └── metrics.rs          # Prometheus metrics
│
├── jobs/                   # Background Jobs
│   ├── mod.rs
│   ├── scheduler.rs        # Job scheduling
│   ├── sla_checker.rs      # SLA breach detection
│   ├── expiration_monitor.rs # Domain/SSL/License expiry
│   ├── recurring_billing.rs # Auto-invoicing
│   └── maintenance.rs      # DB cleanup & optimization
│
├── workflows/              # Workflow Automation Engine
│   ├── mod.rs
│   ├── engine.rs           # Workflow processing
│   ├── triggers.rs         # Event triggers
│   ├── conditions.rs       # Condition evaluation
│   ├── actions.rs          # Action definitions
│   └── executor.rs         # Action execution
│
├── integrations/           # External Integrations
│   ├── mod.rs
│   ├── azure.rs            # Azure AD/Entra
│   ├── google.rs           # Google Workspace
│   ├── stripe.rs           # Payment processing
│   ├── cloudflare.rs       # DNS management
│   └── github.rs           # Source control
│
├── itdoc/                  # IT Documentation
│   ├── mod.rs
│   ├── networks.rs
│   ├── domains.rs
│   ├── credentials.rs
│   ├── ssl_certificates.rs
│   └── software_licenses.rs
│
├── files/                  # File Storage
│   └── mod.rs
│
├── notifications/          # Push Notifications
│   └── mod.rs
│
├── middleware/             # HTTP Middleware
│   ├── mod.rs
│   └── observability.rs    # Logging, tracing, metrics
│
├── models/                 # Database Models
│   ├── mod.rs
│   ├── passwords.rs
│   ├── domains_ssl.rs
│   └── assets.rs
│
└── tests/                  # Test Suite
    ├── mod.rs
    ├── fixtures.rs
    ├── helpers.rs
    ├── unit/
    └── integration/
```

### Frontend (`/frontend/src/`)

```
src/
├── main.rs                 # App entry point & routing
│
├── components/             # Reusable Components
│   ├── mod.rs
│   ├── layout.rs           # Main layout with sidebar
│   └── auth/               # Auth components
│       ├── mod.rs
│       └── login.rs
│
├── pages/                  # Page Components
│   ├── mod.rs
│   ├── dashboard.rs        # Dashboard with stats
│   ├── tickets.rs          # Ticket management
│   ├── clients.rs          # Client CRM
│   ├── time_tracking.rs    # Time tracker
│   ├── passwords.rs        # Password vault
│   ├── assets.rs           # Asset management
│   ├── invoices.rs         # Invoicing
│   ├── knowledge_base.rs   # KB articles
│   ├── reports.rs          # Analytics
│   ├── admin.rs            # Settings
│   ├── m365.rs             # M365 integration
│   ├── azure.rs            # Azure portal
│   ├── bitwarden.rs        # Bitwarden sync
│   └── network.rs          # Network topology
│
├── services/               # Frontend Services
│   └── mod.rs
│
├── theme.rs                # Theme management
│
└── utils/                  # Utilities
    └── mod.rs
```

## Data Flow

### Request Flow

```
Client Request
     │
     ▼
┌─────────────┐
│   Nginx     │  ──► Static files (WASM, CSS, JS)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  CORS Layer │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Auth Middle │  ──► JWT/API Key validation
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Handler   │  ──► Request handling
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Service   │  ──► Business logic
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Database   │  ──► PostgreSQL
└─────────────┘
```

### Event Flow (Workflows)

```
Trigger Event (e.g., Ticket Created)
     │
     ▼
┌─────────────────┐
│ Workflow Engine │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Evaluate        │
│ Conditions      │  ──► Match against workflow rules
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Execute         │
│ Actions         │  ──► Send email, update ticket, notify
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Log Execution   │  ──► Audit trail
└─────────────────┘
```

### Background Job Flow

```
┌─────────────────────────────────────┐
│         Job Scheduler               │
│     (tokio-cron-scheduler)          │
└──────────────┬──────────────────────┘
               │
    ┌──────────┼──────────┐
    ▼          ▼          ▼
┌────────┐ ┌────────┐ ┌────────┐
│  SLA   │ │Expiry  │ │Billing │
│Checker │ │Monitor │ │ Jobs   │
└───┬────┘ └───┬────┘ └───┬────┘
    │          │          │
    ▼          ▼          ▼
┌─────────────────────────────────────┐
│           Database / Email          │
└─────────────────────────────────────┘
```

## Security Architecture

### Authentication Methods

1. **JWT Tokens** - Primary authentication for web UI
2. **API Keys** - Service-to-service authentication
3. **OAuth2/OIDC** - SSO with Google, Microsoft, etc.
4. **SAML** - Enterprise SSO
5. **TOTP 2FA** - Two-factor authentication

### Data Encryption

- **At Rest:** AES-256-GCM for sensitive data (passwords, credentials)
- **In Transit:** TLS 1.3 for all connections
- **Key Management:** Environment variables or secrets manager

### Access Control

```rust
// Role-Based Access Control (RBAC)
pub enum Role {
    Admin,          // Full access
    Manager,        // Manage team, view reports
    Technician,     // Handle tickets, time tracking
    ReadOnly,       // View-only access
    ClientPortal,   // Limited client access
}

pub enum Permission {
    TicketsRead,
    TicketsWrite,
    TicketsDelete,
    ClientsRead,
    ClientsWrite,
    InvoicesRead,
    InvoicesWrite,
    ReportsView,
    AdminSettings,
    // ...
}
```

## Scalability

### Horizontal Scaling

- **Backend:** Stateless design allows multiple instances
- **Database:** Read replicas for query scaling
- **Cache:** Redis cluster for distributed caching
- **Files:** Object storage (S3/MinIO) for file uploads

### Performance Optimizations

1. **Connection Pooling:** SQLx with configurable pool size
2. **Query Optimization:** Indexed queries, eager loading
3. **Caching:** Redis for frequently accessed data
4. **Async I/O:** Tokio runtime for concurrent operations
5. **WebSocket:** Real-time updates without polling

## Technology Stack

| Layer | Technology |
|-------|------------|
| Frontend Framework | Yew 0.21 |
| Frontend Build | Trunk (WASM) |
| CSS Framework | Tailwind CSS |
| Backend Framework | Axum 0.7 |
| Async Runtime | Tokio |
| Database | PostgreSQL 15 |
| Cache | Redis 7 |
| Email | Lettre (SMTP) |
| Authentication | jsonwebtoken, oauth2 |
| API Docs | utoipa (OpenAPI) |
| Job Scheduler | tokio-cron-scheduler |
| HTTP Client | reqwest |
| Serialization | serde, serde_json |
| Validation | validator |
| Encryption | aes-gcm, ring |
| Metrics | Custom + Prometheus format |
