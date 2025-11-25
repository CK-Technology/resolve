# SAML 2.0 Integration

This guide covers setting up SAML 2.0 identity providers for enterprise SSO with Resolve.

## Overview

SAML 2.0 is supported for enterprise customers who use legacy identity providers or require SAML-specific features. Resolve acts as a **Service Provider (SP)** and integrates with your **Identity Provider (IdP)**.

### Supported IdPs

| IdP | Status | Notes |
|-----|--------|-------|
| Azure AD (SAML) | Supported | Use OIDC when possible |
| Okta | Supported | |
| OneLogin | Supported | |
| ADFS | Supported | Windows Server |
| Ping Identity | Supported | |
| Generic SAML 2.0 | Supported | Any compliant IdP |

> **Recommendation**: For Azure AD and Google, prefer OIDC over SAML for simpler configuration and better token handling.

## Resolve SP Configuration

### Service Provider Metadata

Resolve's SP metadata is available at:
```
https://your-domain.com/api/v1/auth/saml/metadata
```

### SP Configuration Values

| Setting | Value |
|---------|-------|
| Entity ID | `https://your-domain.com` |
| ACS URL | `https://your-domain.com/api/v1/auth/saml/callback` |
| SLO URL | `https://your-domain.com/api/v1/auth/saml/logout` (optional) |
| NameID Format | `urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress` |
| Binding | HTTP-POST (for ACS) |
| Signed Requests | Optional |
| Signed Assertions | Required |
| Encrypted Assertions | Optional |

## Azure AD SAML Setup

### Step 1: Create Enterprise Application

1. Go to [Azure Portal](https://portal.azure.com)
2. Navigate to **Azure Active Directory** > **Enterprise applications**
3. Click **New application** > **Create your own application**
4. Name: `Resolve MSP Platform`
5. Select **Integrate any other application you don't find in the gallery (Non-gallery)**
6. Click **Create**

### Step 2: Configure SAML SSO

1. Go to **Single sign-on** > Select **SAML**
2. Edit **Basic SAML Configuration**:

| Setting | Value |
|---------|-------|
| Identifier (Entity ID) | `https://your-domain.com` |
| Reply URL (ACS URL) | `https://your-domain.com/api/v1/auth/saml/callback` |
| Sign on URL | `https://your-domain.com/login` |
| Logout URL | `https://your-domain.com/api/v1/auth/saml/logout` |

3. Click **Save**

### Step 3: Configure Attributes & Claims

1. Edit **Attributes & Claims**
2. Configure these claims:

| Claim name | Source attribute |
|------------|------------------|
| `emailaddress` | user.mail |
| `givenname` | user.givenname |
| `surname` | user.surname |
| `name` | user.displayname |
| `groups` | user.groups [All] (optional) |

### Step 4: Download Federation Metadata

1. In **SAML Signing Certificate** section
2. Download **Federation Metadata XML**
3. Or note these values:
   - **Login URL**: `https://login.microsoftonline.com/{tenant}/saml2`
   - **Azure AD Identifier**: `https://sts.windows.net/{tenant}/`
   - **Certificate (Base64)**: Download and copy contents

### Step 5: Assign Users

1. Go to **Users and groups**
2. Click **Add user/group**
3. Select users or groups who should access Resolve
4. Click **Assign**

## Okta SAML Setup

### Step 1: Create Application

1. Go to Okta Admin Console
2. Navigate to **Applications** > **Applications**
3. Click **Create App Integration**
4. Select **SAML 2.0**
5. Click **Next**

### Step 2: Configure SAML Settings

**General Settings:**
- App name: `Resolve`
- App logo: Upload Resolve logo (optional)

**SAML Settings:**

| Setting | Value |
|---------|-------|
| Single sign-on URL | `https://your-domain.com/api/v1/auth/saml/callback` |
| Audience URI (SP Entity ID) | `https://your-domain.com` |
| Name ID format | EmailAddress |
| Application username | Email |

**Attribute Statements:**

| Name | Value |
|------|-------|
| email | user.email |
| firstName | user.firstName |
| lastName | user.lastName |
| displayName | user.displayName |

**Group Attribute Statements (optional):**

| Name | Filter |
|------|--------|
| groups | Matches regex: `.*` |

### Step 3: Get IdP Configuration

After creating the app:
1. Go to **Sign On** tab
2. Click **View SAML setup instructions**
3. Note:
   - Identity Provider Single Sign-On URL
   - Identity Provider Issuer
   - X.509 Certificate

### Step 4: Assign Users

1. Go to **Assignments** tab
2. Assign users or groups

## Configure Resolve

### Admin Panel Configuration

1. Go to **Admin > Settings > Authentication**
2. Click **Add Provider**
3. Select **SAML 2.0**
4. Enter configuration:

| Field | Value |
|-------|-------|
| Name | `okta_saml` or `azure_saml` |
| Display Name | `Sign in with Okta` |
| Entity ID | IdP Entity ID / Issuer |
| SSO URL | IdP Single Sign-On URL |
| SSO Binding | HTTP-Redirect or HTTP-POST |
| Certificate | IdP signing certificate (PEM format) |

5. Configure attribute mapping
6. Enable the provider
7. Click **Save**

### Database Configuration

```sql
INSERT INTO saml_providers (
    id, name, display_name,
    entity_id, sso_url, sso_binding,
    signing_cert, attribute_mapping,
    enabled, allow_registration
) VALUES (
    gen_random_uuid(),
    'okta_saml',
    'Sign in with Okta',
    'http://www.okta.com/exk123abc',
    'https://yourcompany.okta.com/app/resolve/sso/saml',
    'HTTP-POST',
    '-----BEGIN CERTIFICATE-----
MIIDpDCCAoygAwIBAgIGAX...
-----END CERTIFICATE-----',
    '{
        "email": "email",
        "first_name": "firstName",
        "last_name": "lastName",
        "groups": "groups"
    }',
    true,
    true
);
```

## Attribute Mapping

### Default Mapping

Resolve looks for these SAML attributes:

| User Field | Default Attribute Names |
|------------|------------------------|
| Email | `email`, `emailaddress`, `http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress` |
| First Name | `firstName`, `givenname`, `http://schemas.xmlsoap.org/ws/2005/05/identity/claims/givenname` |
| Last Name | `lastName`, `surname`, `http://schemas.xmlsoap.org/ws/2005/05/identity/claims/surname` |
| Display Name | `displayName`, `name`, `http://schemas.xmlsoap.org/ws/2005/05/identity/claims/name` |
| Groups | `groups`, `memberOf`, `http://schemas.microsoft.com/ws/2008/06/identity/claims/groups` |

### Custom Mapping

Override default mapping in provider configuration:

```json
{
  "attribute_mapping": {
    "email": "urn:oid:0.9.2342.19200300.100.1.3",
    "first_name": "urn:oid:2.5.4.42",
    "last_name": "urn:oid:2.5.4.4",
    "groups": "urn:oid:1.3.6.1.4.1.5923.1.5.1.1"
  }
}
```

## Group-Based Role Mapping

Map IdP groups to Resolve roles:

```json
{
  "role_mapping": {
    "group_to_role": {
      "Resolve-Admins": "admin-role-uuid",
      "Resolve-Technicians": "technician-role-uuid",
      "Resolve-Billing": "billing-role-uuid"
    },
    "default_role_id": "readonly-role-uuid"
  }
}
```

## Domain Restrictions

Restrict SAML authentication to specific email domains:

```json
{
  "allowed_domains": ["yourcompany.com", "subsidiary.com"]
}
```

## Security Considerations

### Signature Verification

Always require signed assertions:
- IdP must sign SAML assertions
- Resolve verifies signatures using IdP certificate
- Reject unsigned or tampered assertions

### Certificate Management

- **Store certificates securely** - Use secrets management
- **Monitor expiration** - IdP certificates expire (typically 1-3 years)
- **Plan rotation** - Update Resolve config before certificates expire
- **Multiple certificates** - Some IdPs support certificate rollover

### Time Validation

- Assertions have validity windows (typically 5 minutes)
- Ensure server clocks are synchronized (NTP)
- Reject expired or future-dated assertions

### Replay Protection

- Each SAML assertion has a unique ID
- Resolve tracks processed assertion IDs
- Reused assertions are rejected

## Troubleshooting

### "Invalid signature"

- Certificate mismatch between IdP and Resolve config
- Certificate expired or rotated
- Wrong certificate format (must be PEM)

### "Assertion expired"

- Clock skew between servers
- Network latency exceeded validity window
- Sync server time with NTP

### "NameID not found"

- IdP not sending NameID
- Wrong NameID format configured
- Check IdP attribute configuration

### "User email not found"

- Email attribute not mapped correctly
- Attribute name doesn't match IdP's attribute
- Check SAML response for actual attribute names

### "Invalid InResponseTo"

- SAML response doesn't match any pending request
- Request state expired (> 5 minutes)
- Possible replay attack

### Debug SAML Responses

Enable SAML debug logging:

```bash
RUST_LOG=resolve::auth::saml=debug
```

Or decode SAML response manually:
```bash
echo "BASE64_SAML_RESPONSE" | base64 -d | xmllint --format -
```

## Testing

### SAML Tracer

Use browser extensions to inspect SAML traffic:
- [SAML Tracer for Firefox](https://addons.mozilla.org/en-US/firefox/addon/saml-tracer/)
- [SAML Chrome Panel](https://chrome.google.com/webstore/detail/saml-chrome-panel/)

### Test IdPs

For development/testing:
- **Okta Developer**: Free developer account
- **Auth0**: Free tier available
- **SimpleSAMLphp**: Self-hosted test IdP

## Related Documentation

- [SAML 2.0 Specification](http://docs.oasis-open.org/security/saml/v2.0/)
- [Azure AD SAML SSO](https://docs.microsoft.com/en-us/azure/active-directory/saas-apps/saml-toolkit-tutorial)
- [Okta SAML](https://developer.okta.com/docs/concepts/saml/)
