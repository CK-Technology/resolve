# Client Endpoints

Manage your MSP clients and their information.

## List Clients

```http
GET /api/v1/clients
```

### Query Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `limit` | integer | Number of results to return (default: 20, max: 100) |
| `offset` | integer | Number of results to skip (default: 0) |
| `search` | string | Search clients by name, email, or identifier |
| `status` | string | Filter by client status (`active`, `inactive`, `suspended`) |
| `sort` | string | Sort field (`name`, `created_at`, `-created_at` for desc) |

### Example Request

```bash
curl -H "Authorization: Bearer <token>" \
  "https://api.resolve.example.com/v1/clients?limit=10&search=acme&status=active"
```

### Example Response

```json
{
  "data": [
    {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "name": "Acme Corporation",
      "identifier": "ACME",
      "primary_contact_email": "john@acme.com",
      "primary_contact_phone": "+1-555-0123",
      "status": "active",
      "address": "123 Business St",
      "city": "New York",
      "state": "NY",
      "zip_code": "10001",
      "country": "US",
      "website": "https://acme.com",
      "notes": "Major enterprise client",
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-01-16T14:22:00Z",
      "stats": {
        "total_assets": 45,
        "open_tickets": 3,
        "monthly_revenue": 15000.00
      }
    }
  ],
  "pagination": {
    "limit": 10,
    "offset": 0,
    "total": 150,
    "has_more": true
  }
}
```

## Get Client

```http
GET /api/v1/clients/{id}
```

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | UUID | Client ID |

### Example Request

```bash
curl -H "Authorization: Bearer <token>" \
  "https://api.resolve.example.com/v1/clients/123e4567-e89b-12d3-a456-426614174000"
```

### Example Response

```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "name": "Acme Corporation",
  "identifier": "ACME",
  "primary_contact_email": "john@acme.com",
  "primary_contact_phone": "+1-555-0123",
  "status": "active",
  "address": "123 Business St",
  "city": "New York",
  "state": "NY",
  "zip_code": "10001",
  "country": "US",
  "website": "https://acme.com",
  "notes": "Major enterprise client",
  "billing_contact": {
    "name": "Jane Smith",
    "email": "billing@acme.com",
    "phone": "+1-555-0124"
  },
  "custom_fields": {
    "account_manager": "Bob Johnson",
    "contract_end_date": "2024-12-31"
  },
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-16T14:22:00Z",
  "stats": {
    "total_assets": 45,
    "total_contacts": 8,
    "open_tickets": 3,
    "closed_tickets": 127,
    "monthly_revenue": 15000.00,
    "total_revenue": 180000.00,
    "last_ticket_date": "2024-01-14T09:15:00Z",
    "avg_response_time": "2h 15m"
  }
}
```

## Create Client

```http
POST /api/v1/clients
```

### Request Body

```json
{
  "name": "Acme Corporation",
  "identifier": "ACME",
  "primary_contact_email": "john@acme.com",
  "primary_contact_phone": "+1-555-0123",
  "address": "123 Business St",
  "city": "New York", 
  "state": "NY",
  "zip_code": "10001",
  "country": "US",
  "website": "https://acme.com",
  "notes": "Major enterprise client",
  "billing_contact": {
    "name": "Jane Smith",
    "email": "billing@acme.com",
    "phone": "+1-555-0124"
  },
  "custom_fields": {
    "account_manager": "Bob Johnson",
    "contract_end_date": "2024-12-31"
  }
}
```

### Required Fields

- `name` - Client company name
- `primary_contact_email` - Primary contact email address

### Optional Fields

- `identifier` - Unique client identifier (auto-generated if not provided)
- `primary_contact_phone` - Primary contact phone number
- `address` - Street address
- `city` - City
- `state` - State/Province
- `zip_code` - ZIP/Postal code
- `country` - Country code (ISO 3166-1 alpha-2)
- `website` - Company website URL
- `notes` - Internal notes about the client
- `billing_contact` - Billing contact information
- `custom_fields` - Custom field values

### Example Request

```bash
curl -X POST \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Acme Corporation",
    "primary_contact_email": "john@acme.com",
    "primary_contact_phone": "+1-555-0123"
  }' \
  "https://api.resolve.example.com/v1/clients"
```

### Example Response

```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "name": "Acme Corporation",
  "identifier": "ACME",
  "primary_contact_email": "john@acme.com",
  "primary_contact_phone": "+1-555-0123",
  "status": "active",
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:30:00Z"
}
```

## Update Client

```http
PUT /api/v1/clients/{id}
```

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | UUID | Client ID |

### Request Body

Same as create client, but all fields are optional. Only provided fields will be updated.

### Example Request

```bash
curl -X PUT \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Acme Corporation Ltd",
    "website": "https://www.acme.com"
  }' \
  "https://api.resolve.example.com/v1/clients/123e4567-e89b-12d3-a456-426614174000"
```

### Example Response

```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "name": "Acme Corporation Ltd",
  "identifier": "ACME",
  "primary_contact_email": "john@acme.com",
  "primary_contact_phone": "+1-555-0123",
  "status": "active",
  "website": "https://www.acme.com",
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-16T14:22:00Z"
}
```

## Delete Client

```http
DELETE /api/v1/clients/{id}
```

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | UUID | Client ID |

⚠️ **Warning**: This action permanently deletes the client and all associated data (tickets, assets, contacts, etc.). This cannot be undone.

### Example Request

```bash
curl -X DELETE \
  -H "Authorization: Bearer <token>" \
  "https://api.resolve.example.com/v1/clients/123e4567-e89b-12d3-a456-426614174000"
```

### Example Response

```http
HTTP/1.1 204 No Content
```

## Client Statistics

```http
GET /api/v1/clients/{id}/stats
```

Get detailed statistics for a specific client.

### Example Response

```json
{
  "client_id": "123e4567-e89b-12d3-a456-426614174000",
  "assets": {
    "total": 45,
    "by_type": {
      "server": 12,
      "workstation": 25,
      "network": 8
    },
    "by_status": {
      "active": 42,
      "maintenance": 2,
      "retired": 1
    }
  },
  "tickets": {
    "total": 130,
    "open": 3,
    "in_progress": 2,
    "closed": 127,
    "avg_resolution_time": "4h 32m",
    "satisfaction_score": 4.7
  },
  "financial": {
    "monthly_revenue": 15000.00,
    "total_revenue": 180000.00,
    "outstanding_invoices": 2500.00,
    "last_payment_date": "2024-01-10T00:00:00Z"
  },
  "time_tracking": {
    "total_hours": 1250.5,
    "billable_hours": 1100.0,
    "current_month_hours": 85.5,
    "avg_hours_per_ticket": 3.2
  }
}
```

## Bulk Operations

### Bulk Update Clients

```http
PATCH /api/v1/clients/bulk
```

Update multiple clients at once.

### Request Body

```json
{
  "client_ids": [
    "123e4567-e89b-12d3-a456-426614174000",
    "987fcdeb-51a2-43d7-8f9a-123456789abc"
  ],
  "updates": {
    "status": "inactive",
    "custom_fields": {
      "bulk_updated": "2024-01-15"
    }
  }
}
```

### Export Clients

```http
GET /api/v1/clients/export
```

Export clients in various formats.

### Query Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `format` | string | Export format (`csv`, `xlsx`, `json`) |
| `fields` | string | Comma-separated list of fields to include |

### Example Request

```bash
curl -H "Authorization: Bearer <token>" \
  "https://api.resolve.example.com/v1/clients/export?format=csv&fields=name,email,status"
```