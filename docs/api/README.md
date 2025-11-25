# Resolve API Documentation

Welcome to the Resolve API documentation. Resolve is a comprehensive MSP (Managed Service Provider) management platform.

## Base URL
- Production: `https://your-domain.com/api/v1`
- Development: `http://localhost:8080/api/v1`

## Authentication

Resolve uses JWT (JSON Web Tokens) for authentication. Include the token in the Authorization header:

```
Authorization: Bearer <your-jwt-token>
```

### Getting a Token

```http
POST /api/v1/auth/login
Content-Type: application/json

{
    "email": "user@example.com",
    "password": "password"
}
```

Response:
```json
{
    "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "refresh_token": "...",
    "expires_in": 3600,
    "user": {
        "id": "uuid",
        "email": "user@example.com",
        "first_name": "John",
        "last_name": "Doe",
        "role": "admin"
    }
}
```

## Rate Limiting

- **General API**: 100 requests per minute per IP
- **Authentication**: 10 requests per minute per IP
- **File uploads**: 5 requests per minute per IP

Rate limit headers are included in responses:
- `X-RateLimit-Limit`: Request limit per window
- `X-RateLimit-Remaining`: Remaining requests in window
- `X-RateLimit-Reset`: Time when window resets (Unix timestamp)

## Error Handling

The API uses conventional HTTP response codes and returns errors in JSON format:

```json
{
    "error": {
        "code": "VALIDATION_ERROR",
        "message": "Invalid input data",
        "details": {
            "field_name": ["Field is required"]
        }
    },
    "timestamp": "2024-01-15T10:30:00Z",
    "path": "/api/v1/clients"
}
```

### HTTP Status Codes

- `200 OK` - Request successful
- `201 Created` - Resource created successfully
- `400 Bad Request` - Invalid request data
- `401 Unauthorized` - Authentication required
- `403 Forbidden` - Access denied
- `404 Not Found` - Resource not found
- `409 Conflict` - Resource conflict (e.g., duplicate)
- `422 Unprocessable Entity` - Validation error
- `429 Too Many Requests` - Rate limit exceeded
- `500 Internal Server Error` - Server error

## Pagination

List endpoints support pagination using query parameters:

- `limit` - Number of items to return (default: 20, max: 100)
- `offset` - Number of items to skip (default: 0)

Example:
```
GET /api/v1/clients?limit=50&offset=100
```

Response includes pagination metadata:
```json
{
    "data": [...],
    "pagination": {
        "limit": 50,
        "offset": 100,
        "total": 500,
        "has_more": true
    }
}
```

## Filtering and Search

Most list endpoints support filtering and search:

- `search` - Full-text search across relevant fields
- `filter[field]` - Filter by specific field values
- `sort` - Sort by field (prefix with `-` for descending)

Example:
```
GET /api/v1/clients?search=acme&filter[status]=active&sort=-created_at
```

## Field Selection

Use the `fields` parameter to specify which fields to return:

```
GET /api/v1/clients?fields=id,name,email
```

## API Endpoints

### Authentication
- [Authentication Endpoints](./auth.md)

### Client Management
- [Client Endpoints](./clients.md)
- [Contact Endpoints](./contacts.md)

### Ticket Management
- [Ticket Endpoints](./tickets.md)
- [Time Tracking Endpoints](./time-tracking.md)

### Asset Management
- [Asset Endpoints](./assets.md)
- [Asset Files Endpoints](./asset-files.md)

### Integration Management
- [Microsoft 365 Endpoints](./m365.md)
- [Azure Endpoints](./azure.md)
- [Bitwarden Endpoints](./bitwarden.md)
- [Network Endpoints](./network.md)

### Financial Management
- [Invoice Endpoints](./invoices.md)
- [Payment Endpoints](./payments.md)

### Reporting
- [Report Endpoints](./reports.md)
- [Dashboard Endpoints](./dashboard.md)

### System Management
- [User Endpoints](./users.md)
- [Settings Endpoints](./settings.md)

## SDKs and Libraries

### JavaScript/TypeScript
```bash
npm install @resolve-msp/api-client
```

```javascript
import { ResolveClient } from '@resolve-msp/api-client';

const client = new ResolveClient({
    baseUrl: 'https://your-domain.com/api/v1',
    apiKey: 'your-api-key'
});

// List clients
const clients = await client.clients.list();

// Create client
const newClient = await client.clients.create({
    name: 'Acme Corp',
    email: 'contact@acme.com'
});
```

### Python
```bash
pip install resolve-msp-api
```

```python
from resolve_msp_api import ResolveClient

client = ResolveClient(
    base_url='https://your-domain.com/api/v1',
    api_key='your-api-key'
)

# List clients
clients = client.clients.list()

# Create client
new_client = client.clients.create({
    'name': 'Acme Corp',
    'email': 'contact@acme.com'
})
```

### PHP
```bash
composer require resolve-msp/api-client
```

```php
use ResolveMsp\ApiClient\Client;

$client = new Client([
    'base_url' => 'https://your-domain.com/api/v1',
    'api_key' => 'your-api-key'
]);

// List clients
$clients = $client->clients()->list();

// Create client
$newClient = $client->clients()->create([
    'name' => 'Acme Corp',
    'email' => 'contact@acme.com'
]);
```

## Webhooks

Resolve supports webhooks to notify your application of events:

### Supported Events
- `client.created` - New client created
- `client.updated` - Client information updated
- `ticket.created` - New support ticket created
- `ticket.updated` - Ticket status or details changed
- `asset.created` - New asset added
- `asset.updated` - Asset information updated
- `invoice.created` - New invoice generated
- `invoice.paid` - Invoice payment received

### Webhook Configuration

Webhooks can be configured in the admin panel or via API:

```http
POST /api/v1/webhooks
Content-Type: application/json

{
    "url": "https://your-app.com/webhooks/resolve",
    "events": ["client.created", "ticket.created"],
    "secret": "your-webhook-secret"
}
```

### Webhook Payload

```json
{
    "event": "client.created",
    "timestamp": "2024-01-15T10:30:00Z",
    "data": {
        "id": "client-uuid",
        "name": "Acme Corp",
        "email": "contact@acme.com"
    }
}
```

## Postman Collection

Download our [Postman Collection](./postman/Resolve-API.json) for easy API testing.

## Support

- **Documentation Issues**: [GitHub Issues](https://github.com/CK-Technology/resolve/issues)
- **Community**: [GitHub Discussions](https://github.com/CK-Technology/resolve/discussions)
