# Resolve Authentication Documentation

Resolve supports multiple authentication methods for enterprise SSO integration:

## Supported Methods

| Method | Protocol | Status | Use Case |
|--------|----------|--------|----------|
| [Azure AD / Entra ID](./AZURE.md) | OIDC | Primary | Microsoft 365 organizations |
| [Google Workspace](./GOOGLE.md) | OIDC | Secondary | Google-based organizations |
| [SAML 2.0](./SAML.md) | SAML | Enterprise | Legacy IdPs (ADFS, Okta, etc.) |
| Local Auth | JWT | Built-in | Fallback / development |
| [API Keys](./API_KEYS.md) | Bearer | Built-in | Integrations / automation |

## Authentication Flow

### OIDC (Recommended)

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│  User    │     │ Resolve  │     │   IdP    │
│ Browser  │     │ Backend  │     │(Azure/G) │
└────┬─────┘     └────┬─────┘     └────┬─────┘
     │  1. Login      │                 │
     │──────────────>│                 │
     │               │  2. Auth URL     │
     │               │──────────────────>
     │  3. Redirect  │                 │
     │<──────────────│                 │
     │               │                 │
     │  4. User authenticates at IdP   │
     │─────────────────────────────────>
     │               │                 │
     │  5. Callback with code          │
     │<────────────────────────────────│
     │               │  6. Exchange    │
     │──────────────>│  code for       │
     │               │  tokens         │
     │               │──────────────────>
     │               │  7. ID Token    │
     │               │<─────────────────│
     │  8. JWT       │                 │
     │<──────────────│                 │
     └───────────────┴─────────────────┘
```

### Key Concepts

- **ID Token**: Contains user identity claims (email, name, etc.)
- **Access Token**: Used for API calls to IdP (optional)
- **Refresh Token**: Used to obtain new tokens without re-authentication
- **PKCE**: Proof Key for Code Exchange (required for security)

## Configuration

Authentication providers are configured in the admin panel under **Admin > Settings > Authentication**.

### Environment Variables

```bash
# JWT Configuration
JWT_SECRET=your-256-bit-secret-key
JWT_EXPIRY_HOURS=24

# OIDC Configuration
OAUTH_REDIRECT_URL=https://your-domain.com/api/v1/auth/callback

# Azure AD (Multi-tenant)
AZURE_CLIENT_ID=your-client-id
AZURE_CLIENT_SECRET=your-client-secret
AZURE_TENANT_ID=common  # or specific tenant ID

# Google
GOOGLE_CLIENT_ID=your-client-id
GOOGLE_CLIENT_SECRET=your-client-secret

# MFA Encryption
MFA_ENCRYPTION_KEY=64-hex-char-key
```

## Security Best Practices

1. **Always use HTTPS** in production
2. **Store secrets securely** (use Docker secrets or vault)
3. **Enable MFA** for admin accounts
4. **Restrict allowed domains** when possible
5. **Rotate API keys** periodically
6. **Monitor audit logs** for suspicious activity

## API Endpoints

### Authentication

```
POST   /api/v1/auth/login              # Local login
POST   /api/v1/auth/register           # Local registration
POST   /api/v1/auth/logout             # Logout
GET    /api/v1/auth/me                 # Current user
POST   /api/v1/auth/refresh            # Refresh token

# OIDC (Recommended)
GET    /api/v1/auth/oidc/providers     # List OIDC providers
GET    /api/v1/auth/oidc/login/:name   # Start OIDC flow
GET    /api/v1/auth/oidc/callback      # OIDC callback (PKCE)

# Legacy OAuth (Backwards compatibility)
GET    /api/v1/auth/oauth/providers    # List enabled providers
GET    /api/v1/auth/oauth/:provider    # Start OAuth flow
GET    /api/v1/auth/oauth/callback     # OAuth callback

# SAML 2.0
GET    /api/v1/auth/saml/providers     # List SAML providers
GET    /api/v1/auth/saml/login/:name   # Start SAML flow
POST   /api/v1/auth/saml/callback      # SAML ACS endpoint (POST binding)
GET    /api/v1/auth/saml/callback      # SAML ACS endpoint (Redirect binding)
GET    /api/v1/auth/saml/metadata      # SP metadata XML

# MFA
POST   /api/v1/auth/mfa/setup          # Setup TOTP
POST   /api/v1/auth/mfa/verify         # Verify and enable
POST   /api/v1/auth/mfa/disable        # Disable MFA
```

### API Keys

```
GET    /api/v1/auth/api-keys           # List user's API keys
POST   /api/v1/auth/api-keys           # Create new API key
GET    /api/v1/auth/api-keys/:id       # Get API key details
DELETE /api/v1/auth/api-keys/:id       # Revoke API key
POST   /api/v1/auth/api-keys/:id/regenerate  # Regenerate key
```

## Role-Based Access Control

Resolve uses RBAC with the following default roles:

| Role | Description | Hierarchy |
|------|-------------|-----------|
| Admin | Full system access | 100 |
| Manager | Manage team and clients | 80 |
| Billing | Financial operations | 60 |
| Technician | Technical support | 50 |
| ReadOnly | View-only access | 10 |

See [RBAC Documentation](./RBAC.md) for details on permissions and custom roles.
