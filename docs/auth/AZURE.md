# Azure AD / Microsoft Entra ID Integration

This guide covers setting up Azure AD (Microsoft Entra ID) as an identity provider for Resolve using OpenID Connect (OIDC).

## Overview

Azure AD integration allows users from any Microsoft 365 organization to sign into Resolve using their work accounts. Resolve supports:

- **Multi-tenant**: Users from any Azure AD tenant can authenticate
- **Single-tenant**: Restrict to a specific organization
- **B2B Guest Users**: Support for external collaborators

## Prerequisites

- Azure AD tenant with admin access (or permission to create app registrations)
- Resolve instance accessible via HTTPS

## Step 1: Create App Registration

1. Go to [Azure Portal](https://portal.azure.com)
2. Navigate to **Azure Active Directory** > **App registrations**
3. Click **New registration**

### Registration Settings

| Field | Value |
|-------|-------|
| Name | `Resolve MSP Platform` |
| Supported account types | **Accounts in any organizational directory (Multi-tenant)** |
| Redirect URI | Platform: `Web`, URI: `https://your-domain.com/api/v1/auth/callback` |

4. Click **Register**

## Step 2: Configure Authentication

### Platform Configuration

1. Go to **Authentication** in your app registration
2. Under **Web**, add redirect URIs:
   - Production: `https://your-domain.com/api/v1/auth/callback`
   - Development: `http://localhost:8080/api/v1/auth/callback` (if needed)
3. Under **Implicit grant and hybrid flows**:
   - Check **ID tokens** (for OIDC)
4. Under **Advanced settings**:
   - Allow public client flows: **No**
5. Click **Save**

### Logout Configuration (Optional)

1. Add **Front-channel logout URL**: `https://your-domain.com/api/v1/auth/logout`
2. This enables single sign-out across applications

## Step 3: Create Client Secret

1. Go to **Certificates & secrets**
2. Click **New client secret**
3. Add a description: `Resolve Production`
4. Select expiration: **24 months** (recommended)
5. Click **Add**
6. **Copy the secret value immediately** - it won't be shown again!

## Step 4: Configure API Permissions

1. Go to **API permissions**
2. Click **Add a permission**
3. Select **Microsoft Graph**
4. Choose **Delegated permissions**
5. Add these permissions:
   - `openid` - Sign in and read user profile
   - `profile` - View users' basic profile
   - `email` - View users' email address
   - `offline_access` - Maintain access to data (for refresh tokens)
   - `User.Read` - Sign in and read user profile

6. Click **Add permissions**
7. If you're a Global Admin, click **Grant admin consent** (otherwise, users will consent individually)

### Optional: Group Claims

To enable group-based role mapping:

1. Go to **Token configuration**
2. Click **Add groups claim**
3. Select **Security groups** or **Groups assigned to the application**
4. Under **ID**, check **Group ID**
5. Click **Add**

## Step 5: Note Your Configuration

Collect these values for Resolve configuration:

| Setting | Where to Find |
|---------|---------------|
| Client ID | Overview > Application (client) ID |
| Client Secret | Certificates & secrets > Client secrets |
| Tenant ID | Overview > Directory (tenant) ID |
| Issuer URL | `https://login.microsoftonline.com/{tenant-id}/v2.0` |

For **multi-tenant** apps, use:
- Issuer URL: `https://login.microsoftonline.com/common/v2.0`

## Step 6: Configure Resolve

### Option A: Admin Panel

1. Go to **Admin > Settings > Authentication**
2. Click **Add Provider**
3. Select **Microsoft Azure AD**
4. Enter your configuration:
   - **Name**: `azure_ad` (internal identifier)
   - **Display Name**: `Sign in with Microsoft`
   - **Client ID**: Your Application (client) ID
   - **Client Secret**: Your client secret
   - **Tenant ID**: `common` for multi-tenant, or specific tenant ID
   - **Allowed Domains**: (optional) Restrict to specific email domains
5. Enable the provider
6. Click **Save**

### Option B: Database Configuration

```sql
INSERT INTO auth_providers (
    id, name, provider_type, display_name,
    client_id, client_secret,
    auth_url, token_url, userinfo_url,
    scopes, enabled, allow_registration
) VALUES (
    gen_random_uuid(),
    'azure_ad',
    'oidc',
    'Sign in with Microsoft',
    'YOUR_CLIENT_ID',
    'YOUR_CLIENT_SECRET',
    'https://login.microsoftonline.com/common/oauth2/v2.0/authorize',
    'https://login.microsoftonline.com/common/oauth2/v2.0/token',
    'https://graph.microsoft.com/v1.0/me',
    ARRAY['openid', 'profile', 'email', 'offline_access'],
    true,
    true
);
```

### Option C: Environment Variables

```bash
# Azure AD Configuration
AZURE_AD_ENABLED=true
AZURE_AD_CLIENT_ID=your-client-id
AZURE_AD_CLIENT_SECRET=your-client-secret
AZURE_AD_TENANT_ID=common
```

## Step 7: Test Authentication

1. Go to Resolve login page
2. Click **Sign in with Microsoft**
3. Authenticate with your Azure AD account
4. Verify you're redirected back and logged in

## Advanced Configuration

### Single-Tenant Mode

For a single organization only:

1. Change **Supported account types** in app registration to **Single tenant**
2. Set Tenant ID to your specific tenant ID in Resolve config
3. Issuer URL: `https://login.microsoftonline.com/{your-tenant-id}/v2.0`

### Group-Based Role Mapping

Map Azure AD groups to Resolve roles:

```json
{
  "role_mapping": {
    "group_to_role": {
      "aad-group-id-1": "resolve-admin-role-id",
      "aad-group-id-2": "resolve-technician-role-id"
    },
    "default_role_id": "resolve-readonly-role-id"
  }
}
```

### Domain Restrictions

Restrict authentication to specific domains:

```json
{
  "allowed_domains": ["yourcompany.com", "partner.com"]
}
```

Users with emails outside these domains will be rejected.

## Troubleshooting

### "AADSTS50011: Reply URL does not match"

- Verify redirect URI in Azure matches exactly (including trailing slashes)
- Check for HTTP vs HTTPS mismatch
- Ensure URI is URL-encoded properly

### "AADSTS65001: User hasn't consented"

- Either grant admin consent, or direct users to consent
- Check required permissions are configured

### "AADSTS700016: Application not found"

- Verify Client ID is correct
- Check tenant configuration (common vs specific)

### "Token validation failed"

- Verify issuer URL matches tenant configuration
- Check system clock is synchronized (NTP)
- Ensure using v2.0 endpoints

### User created but no groups

- Verify groups claim is configured in Token configuration
- Check user is actually a member of the groups
- Ensure Azure AD app has permission to read groups

## Security Recommendations

1. **Use short-lived secrets**: Rotate client secrets regularly
2. **Enable conditional access**: Require MFA for Resolve access
3. **Restrict by IP**: Use Azure AD Conditional Access to limit sign-ins
4. **Monitor sign-ins**: Review Azure AD sign-in logs regularly
5. **Use Managed Identity**: If hosting on Azure, use MI instead of secrets

## Related Documentation

- [Microsoft identity platform documentation](https://docs.microsoft.com/en-us/azure/active-directory/develop/)
- [OIDC protocol reference](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-protocols-oidc)
- [Token reference](https://docs.microsoft.com/en-us/azure/active-directory/develop/access-tokens)
