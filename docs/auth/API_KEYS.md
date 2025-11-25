# API Keys

API Keys provide programmatic access to Resolve for integrations, automation scripts, and third-party applications.

## Overview

API Keys are:
- **Scoped**: Limited to specific permissions
- **Revocable**: Can be disabled or deleted at any time
- **Auditable**: Usage is tracked and logged
- **Secure**: Hashed before storage, original shown only once

## Key Format

```
resolve_<prefix>_<secret>
```

Example: `resolve_abc12345_x7k9m2p4q8r1s5t6u3v0w`

- `resolve_`: Fixed prefix identifying Resolve API keys
- `<prefix>`: 8-character identifier for quick lookup
- `<secret>`: 32-character random secret

## Creating API Keys

### Via Admin Panel

1. Go to **User Settings** > **API Keys**
2. Click **Create New Key**
3. Configure:
   - **Name**: Descriptive name (e.g., "Zapier Integration")
   - **Description**: Optional notes about usage
   - **Scopes**: Select required permissions
   - **Expiration**: Optional expiry date
   - **IP Whitelist**: Optional IP restrictions
   - **Rate Limit**: Requests per minute (0 = unlimited)
4. Click **Create**
5. **Copy the key immediately** - it won't be shown again!

### Via API

```bash
curl -X POST https://your-domain.com/api/v1/auth/api-keys \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "CI/CD Integration",
    "description": "Used for automated deployments",
    "scopes": ["read_tickets", "write_tickets"],
    "expires_in_days": 365,
    "allowed_ips": ["10.0.0.0/8"],
    "rate_limit": 100
  }'
```

Response:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "CI/CD Integration",
  "key": "resolve_abc12345_x7k9m2p4q8r1s5t6u3v0w",
  "key_prefix": "abc12345",
  "scopes": ["read_tickets", "write_tickets"],
  "expires_at": "2025-11-24T00:00:00Z",
  "created_at": "2024-11-24T12:00:00Z"
}
```

## Using API Keys

Include the API key in the `Authorization` header:

```bash
curl https://your-domain.com/api/v1/tickets \
  -H "Authorization: Bearer resolve_abc12345_x7k9m2p4q8r1s5t6u3v0w"
```

## Available Scopes

### Read Scopes

| Scope | Description |
|-------|-------------|
| `read_clients` | View client information |
| `read_tickets` | View tickets |
| `read_assets` | View assets |
| `read_passwords` | View passwords (requires reveal permission) |
| `read_documentation` | View documentation and KB articles |
| `read_invoices` | View invoices and billing |
| `read_reports` | View reports and analytics |

### Write Scopes

| Scope | Description |
|-------|-------------|
| `write_clients` | Create and update clients |
| `write_tickets` | Create and update tickets |
| `write_assets` | Create and update assets |
| `write_passwords` | Create and update passwords |
| `write_documentation` | Create and update documentation |
| `write_invoices` | Create and update invoices |

### Admin Scopes

| Scope | Description |
|-------|-------------|
| `manage_users` | Manage user accounts |
| `manage_settings` | Manage system settings |
| `manage_integrations` | Manage integration configs |

### Special Scopes

| Scope | Description |
|-------|-------------|
| `full_access` | All permissions (use carefully) |
| `webhooks_only` | Only send/receive webhooks |

## API Endpoints

### List API Keys

```
GET /api/v1/auth/api-keys
```

Returns all API keys for the authenticated user (without actual key values).

### Get API Key Details

```
GET /api/v1/auth/api-keys/:id
```

### Revoke API Key

```
DELETE /api/v1/auth/api-keys/:id
```

### Regenerate API Key

```
POST /api/v1/auth/api-keys/:id/regenerate
```

Generates a new key value while keeping other settings. The old key becomes invalid immediately.

## Security Features

### IP Whitelisting

Restrict API key usage to specific IP addresses or CIDR ranges:

```json
{
  "allowed_ips": [
    "192.168.1.100",
    "10.0.0.0/8",
    "2001:db8::/32"
  ]
}
```

If the list is empty, all IPs are allowed.

### Rate Limiting

Set requests per minute (RPM) limit:

```json
{
  "rate_limit": 100
}
```

- `0` = unlimited
- Exceeding the limit returns `429 Too Many Requests`
- Response includes `Retry-After` header

### Expiration

Keys can have an optional expiration date:

```json
{
  "expires_in_days": 90
}
```

Expired keys return `401 Unauthorized`.

## Best Practices

1. **Least Privilege**: Grant only necessary scopes
2. **Rotate Regularly**: Regenerate keys periodically
3. **Use IP Restrictions**: Whitelist known IPs when possible
4. **Set Expiration**: Don't create never-expiring keys for temporary integrations
5. **Monitor Usage**: Check `last_used_at` and `usage_count` regularly
6. **Secure Storage**: Store keys in secrets managers, not code
7. **Separate Keys**: Use different keys for different integrations
8. **Descriptive Names**: Use clear names to identify key purposes

## Error Responses

### Invalid Key

```json
{
  "code": "UNAUTHORIZED",
  "message": "Invalid API key",
  "timestamp": "2024-11-24T12:00:00Z"
}
```

### Expired Key

```json
{
  "code": "UNAUTHORIZED",
  "message": "API key has expired",
  "timestamp": "2024-11-24T12:00:00Z"
}
```

### Insufficient Scope

```json
{
  "code": "INSUFFICIENT_PERMISSIONS",
  "message": "Insufficient permissions. Required: write_tickets",
  "timestamp": "2024-11-24T12:00:00Z"
}
```

### Rate Limited

```json
{
  "code": "TOO_MANY_REQUESTS",
  "message": "Too many requests. Retry after 45 seconds",
  "timestamp": "2024-11-24T12:00:00Z"
}
```
Headers: `Retry-After: 45`

### IP Not Allowed

```json
{
  "code": "FORBIDDEN",
  "message": "IP 1.2.3.4 not allowed",
  "timestamp": "2024-11-24T12:00:00Z"
}
```

## Integration Examples

### Zapier

1. Create an API key with required scopes
2. In Zapier, use "Webhooks by Zapier"
3. Set Authorization header: `Bearer resolve_...`

### n8n

1. Create an API key
2. Add HTTP Request node
3. Set Authentication: Header Auth
4. Name: `Authorization`, Value: `Bearer resolve_...`

### PowerShell

```powershell
$headers = @{
    "Authorization" = "Bearer resolve_abc12345_x7k9m2p4q8r1s5t6u3v0w"
}

$response = Invoke-RestMethod -Uri "https://your-domain.com/api/v1/tickets" `
    -Headers $headers

$response | ConvertTo-Json
```

### Python

```python
import requests

API_KEY = "resolve_abc12345_x7k9m2p4q8r1s5t6u3v0w"
BASE_URL = "https://your-domain.com/api/v1"

headers = {
    "Authorization": f"Bearer {API_KEY}",
    "Content-Type": "application/json"
}

# List tickets
response = requests.get(f"{BASE_URL}/tickets", headers=headers)
tickets = response.json()

# Create ticket
new_ticket = {
    "subject": "Server down",
    "description": "Production server not responding",
    "priority": "high",
    "client_id": "client-uuid"
}

response = requests.post(f"{BASE_URL}/tickets", headers=headers, json=new_ticket)
created = response.json()
```

### Node.js

```javascript
const API_KEY = 'resolve_abc12345_x7k9m2p4q8r1s5t6u3v0w';
const BASE_URL = 'https://your-domain.com/api/v1';

const headers = {
  'Authorization': `Bearer ${API_KEY}`,
  'Content-Type': 'application/json'
};

// Using fetch
const response = await fetch(`${BASE_URL}/tickets`, { headers });
const tickets = await response.json();

// Using axios
const axios = require('axios');
const client = axios.create({
  baseURL: BASE_URL,
  headers
});

const { data } = await client.get('/tickets');
```

## Audit Trail

API key usage is logged with:
- Timestamp
- Endpoint accessed
- Response status
- Client IP
- Request duration

View audit logs in **Admin** > **Audit Log** > Filter by API Key.

## Related Documentation

- [Authentication Overview](./README.md)
- [RBAC Documentation](./RBAC.md)
- [Webhooks](../integrations/WEBHOOKS.md)
