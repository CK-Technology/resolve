# Google Workspace / Google Cloud Identity Integration

This guide covers setting up Google as an identity provider for Resolve using OpenID Connect (OIDC).

## Overview

Google integration allows users from Google Workspace organizations (formerly G Suite) or personal Google accounts to sign into Resolve. Resolve supports:

- **Google Workspace**: Organization accounts with admin-managed access
- **Personal Gmail**: Individual Google accounts (can be restricted)
- **Domain Restrictions**: Limit to specific email domains

## Prerequisites

- Google Cloud Console access
- Google Workspace admin access (for organization-wide deployment)
- Resolve instance accessible via HTTPS

## Step 1: Create Google Cloud Project

1. Go to [Google Cloud Console](https://console.cloud.google.com)
2. Click **Select a project** > **New Project**
3. Enter project details:
   - **Project name**: `Resolve MSP Platform`
   - **Organization**: Select your organization (if applicable)
4. Click **Create**

## Step 2: Configure OAuth Consent Screen

1. Navigate to **APIs & Services** > **OAuth consent screen**
2. Select User Type:
   - **Internal**: Only users in your Google Workspace org (recommended for enterprise)
   - **External**: Any Google user (requires verification for production)
3. Click **Create**

### App Information

| Field | Value |
|-------|-------|
| App name | `Resolve` |
| User support email | Your support email |
| App logo | Upload Resolve logo (optional) |
| App domain | `https://your-domain.com` |
| Privacy policy | `https://your-domain.com/privacy` |
| Terms of service | `https://your-domain.com/terms` |

### Developer Contact

Add your developer email addresses.

Click **Save and Continue**

### Scopes

1. Click **Add or Remove Scopes**
2. Select these scopes:
   - `openid` - Associate you with your personal info on Google
   - `userinfo.email` - See your primary email address
   - `userinfo.profile` - See your personal info (name, profile picture)

3. Click **Update** then **Save and Continue**

### Test Users (External only)

If you selected **External**, add test users for development:
- Add email addresses of testers
- These users can access the app before verification

Click **Save and Continue**

## Step 3: Create OAuth Credentials

1. Navigate to **APIs & Services** > **Credentials**
2. Click **Create Credentials** > **OAuth client ID**

### OAuth Client Configuration

| Field | Value |
|-------|-------|
| Application type | Web application |
| Name | `Resolve Web Client` |

### Authorized JavaScript Origins

Add your domain(s):
- Production: `https://your-domain.com`
- Development: `http://localhost:8080` (if needed)

### Authorized Redirect URIs

Add callback URLs:
- Production: `https://your-domain.com/api/v1/auth/callback`
- Development: `http://localhost:8080/api/v1/auth/callback`

3. Click **Create**
4. **Download the JSON** or copy the Client ID and Client Secret

## Step 4: Note Your Configuration

| Setting | Value |
|---------|-------|
| Client ID | `xxxx.apps.googleusercontent.com` |
| Client Secret | `GOCSPX-xxxx` |
| Issuer URL | `https://accounts.google.com` |
| Auth URL | `https://accounts.google.com/o/oauth2/v2/auth` |
| Token URL | `https://oauth2.googleapis.com/token` |
| Userinfo URL | `https://openidconnect.googleapis.com/v1/userinfo` |

## Step 5: Configure Resolve

### Option A: Admin Panel

1. Go to **Admin > Settings > Authentication**
2. Click **Add Provider**
3. Select **Google**
4. Enter your configuration:
   - **Name**: `google` (internal identifier)
   - **Display Name**: `Sign in with Google`
   - **Client ID**: Your OAuth client ID
   - **Client Secret**: Your client secret
   - **Allowed Domains**: (optional) e.g., `yourcompany.com`
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
    'google',
    'oidc',
    'Sign in with Google',
    'YOUR_CLIENT_ID.apps.googleusercontent.com',
    'GOCSPX-YOUR_CLIENT_SECRET',
    'https://accounts.google.com/o/oauth2/v2/auth',
    'https://oauth2.googleapis.com/token',
    'https://openidconnect.googleapis.com/v1/userinfo',
    ARRAY['openid', 'profile', 'email'],
    true,
    true
);
```

### Option C: Environment Variables

```bash
# Google Configuration
GOOGLE_ENABLED=true
GOOGLE_CLIENT_ID=your-client-id.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=GOCSPX-your-client-secret
```

## Step 6: Test Authentication

1. Go to Resolve login page
2. Click **Sign in with Google**
3. Select your Google account
4. Approve the consent screen (first time only)
5. Verify you're redirected back and logged in

## Google Workspace Integration

### Admin-Managed Deployment

For Google Workspace admins to pre-approve the app:

1. Go to [Google Admin Console](https://admin.google.com)
2. Navigate to **Security** > **API Controls** > **App Access Control**
3. Click **Manage Third-Party App Access**
4. Add the Resolve OAuth client ID
5. Set access to **Trusted**

This allows all users in your organization to use Resolve without individual consent.

### Domain-Wide Delegation (Service Account)

For backend-to-backend API access:

1. Create a Service Account in Google Cloud Console
2. Enable domain-wide delegation
3. Grant necessary OAuth scopes in Google Admin
4. Use service account for automated operations

## Advanced Configuration

### Domain Restrictions

Restrict to Google Workspace domains only:

```json
{
  "allowed_domains": ["yourcompany.com", "subsidiary.com"]
}
```

Personal Gmail accounts (`@gmail.com`) will be rejected.

### Hosted Domain Parameter

Force login to a specific Google Workspace domain:

Add `hd` parameter to authorization URL:
```
&hd=yourcompany.com
```

This pre-selects the domain and prevents personal Gmail login attempts.

### Group-Based Access (Google Workspace)

Use Google Groups for role mapping:

1. Create groups in Google Admin (e.g., `resolve-admins@yourcompany.com`)
2. Use Google Workspace Directory API to fetch group memberships
3. Map groups to Resolve roles

```json
{
  "role_mapping": {
    "group_to_role": {
      "resolve-admins@yourcompany.com": "admin-role-id",
      "resolve-techs@yourcompany.com": "technician-role-id"
    },
    "default_role_id": "readonly-role-id"
  }
}
```

## Troubleshooting

### "Error 400: redirect_uri_mismatch"

- Verify redirect URI in Google Console matches exactly
- Check for trailing slashes
- Ensure HTTP vs HTTPS matches
- Wait a few minutes after adding URIs (propagation delay)

### "Error 403: access_denied"

For Internal apps:
- User must be in the Google Workspace organization

For External apps:
- User must be added as a test user, or
- App must be verified by Google

### "This app isn't verified"

For production External apps:
1. Go to OAuth consent screen
2. Click **Publish App**
3. Submit for Google verification
4. Complete verification process (may take weeks)

### User email not received

- Ensure `email` scope is requested
- Check user has an email associated with Google account
- Verify userinfo endpoint is correct

### Profile picture not loading

Google profile pictures require the `profile` scope and may be:
- Private (user setting)
- Not set
- Behind authentication

## Security Recommendations

1. **Use Internal app type** for enterprise-only access
2. **Restrict domains** to your organization's domains
3. **Enable 2-Step Verification** in Google Workspace
4. **Review connected apps** periodically in Google Admin
5. **Monitor OAuth activity** in Google Admin reports
6. **Use short-lived tokens** and refresh when needed

## Related Documentation

- [Google OAuth 2.0 Documentation](https://developers.google.com/identity/protocols/oauth2)
- [OpenID Connect on Google](https://developers.google.com/identity/openid-connect/openid-connect)
- [Google Workspace Admin Help](https://support.google.com/a/answer/7281227)
