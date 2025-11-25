# Resolve MSP Platform - API Documentation

## Overview

The Resolve API provides programmatic access to all platform features. The API follows REST principles and uses JSON for request/response bodies.

**Base URL:** `https://your-instance.resolve.io/api/v1`

## Authentication

### JWT Authentication

Most endpoints require JWT authentication. Include the token in the `Authorization` header:

```
Authorization: Bearer <your-jwt-token>
```

### Obtaining a Token

```bash
POST /api/v1/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "your-password"
}
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
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

### API Key Authentication

For service integrations, use API keys:

```
X-API-Key: <your-api-key>
```

Create API keys in **Settings > API Keys**.

---

## Clients

### List Clients

```bash
GET /api/v1/clients
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `page` | integer | Page number (default: 1) |
| `per_page` | integer | Items per page (default: 25, max: 100) |
| `search` | string | Search by name or email |
| `status` | string | Filter by status: `active`, `inactive` |
| `sort` | string | Sort field: `name`, `created_at` |
| `order` | string | Sort order: `asc`, `desc` |

**Response:**
```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "Acme Corporation",
      "email": "contact@acme.com",
      "phone": "+1-555-0100",
      "status": "active",
      "website": "https://acme.com",
      "billing_address": "123 Main St, New York, NY 10001",
      "is_vip": false,
      "default_hourly_rate": "150.00",
      "sla_policy_id": "uuid",
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-02-20T14:45:00Z"
    }
  ],
  "meta": {
    "current_page": 1,
    "per_page": 25,
    "total": 150,
    "total_pages": 6
  }
}
```

### Get Client

```bash
GET /api/v1/clients/{id}
```

### Create Client

```bash
POST /api/v1/clients
Content-Type: application/json

{
  "name": "New Client Inc",
  "email": "contact@newclient.com",
  "phone": "+1-555-0200",
  "website": "https://newclient.com",
  "billing_address": "456 Oak Ave, Boston, MA 02101",
  "default_hourly_rate": 125.00,
  "is_vip": false,
  "notes": "Referred by existing client"
}
```

### Update Client

```bash
PUT /api/v1/clients/{id}
Content-Type: application/json

{
  "name": "Updated Name",
  "is_vip": true
}
```

### Delete Client

```bash
DELETE /api/v1/clients/{id}
```

---

## Tickets

### List Tickets

```bash
GET /api/v1/tickets
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `client_id` | uuid | Filter by client |
| `assigned_to` | uuid | Filter by assignee |
| `status` | string | `open`, `in_progress`, `pending`, `resolved`, `closed` |
| `priority` | string | `low`, `medium`, `high`, `critical` |
| `queue_id` | uuid | Filter by queue |
| `created_after` | datetime | Filter by creation date |
| `created_before` | datetime | Filter by creation date |

### Create Ticket

```bash
POST /api/v1/tickets
Content-Type: application/json

{
  "client_id": "uuid",
  "subject": "Network connectivity issues",
  "description": "Users are experiencing intermittent network drops...",
  "priority": "high",
  "category_id": "uuid",
  "assigned_to": "uuid",
  "due_date": "2024-03-01T17:00:00Z",
  "tags": ["network", "urgent"]
}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "ticket_number": "TKT-001234",
  "subject": "Network connectivity issues",
  "status": "open",
  "priority": "high",
  "client_id": "uuid",
  "client_name": "Acme Corporation",
  "assigned_to": "uuid",
  "assigned_user_name": "John Doe",
  "sla_tracking": {
    "response_due_at": "2024-02-28T11:30:00Z",
    "resolution_due_at": "2024-02-29T17:00:00Z",
    "response_breached": false,
    "resolution_breached": false
  },
  "created_at": "2024-02-28T10:30:00Z"
}
```

### Update Ticket

```bash
PUT /api/v1/tickets/{id}
Content-Type: application/json

{
  "status": "in_progress",
  "priority": "critical",
  "assigned_to": "uuid"
}
```

### Add Comment

```bash
POST /api/v1/tickets/{id}/comments
Content-Type: application/json

{
  "content": "Investigated the issue. Found router configuration problem.",
  "is_internal": false,
  "notify_client": true
}
```

### Get Ticket Comments

```bash
GET /api/v1/tickets/{id}/comments
```

---

## Time Tracking

### List Time Entries

```bash
GET /api/v1/time
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `user_id` | uuid | Filter by user |
| `client_id` | uuid | Filter by client |
| `ticket_id` | uuid | Filter by ticket |
| `project_id` | uuid | Filter by project |
| `billable` | boolean | Filter billable entries |
| `start_date` | date | Start of date range |
| `end_date` | date | End of date range |

### Create Time Entry

```bash
POST /api/v1/time
Content-Type: application/json

{
  "ticket_id": "uuid",
  "start_time": "2024-02-28T09:00:00Z",
  "end_time": "2024-02-28T11:30:00Z",
  "description": "Troubleshooting network issues",
  "billable": true,
  "hourly_rate": 150.00
}
```

### Start Timer

```bash
POST /api/v1/time/start
Content-Type: application/json

{
  "ticket_id": "uuid",
  "description": "Working on ticket"
}
```

### Stop Timer

```bash
POST /api/v1/time/stop
Content-Type: application/json

{
  "entry_id": "uuid"
}
```

### Get Active Timer

```bash
GET /api/v1/time/active
```

---

## Invoices

### List Invoices

```bash
GET /api/v1/invoices
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `client_id` | uuid | Filter by client |
| `status` | string | `draft`, `sent`, `viewed`, `paid`, `overdue` |
| `date_from` | date | Invoice date range start |
| `date_to` | date | Invoice date range end |

### Create Invoice

```bash
POST /api/v1/invoices
Content-Type: application/json

{
  "client_id": "uuid",
  "issue_date": "2024-02-28",
  "due_date": "2024-03-30",
  "line_items": [
    {
      "description": "IT Support - February 2024",
      "quantity": 40,
      "unit_price": 150.00
    },
    {
      "description": "Software License - Microsoft 365",
      "quantity": 25,
      "unit_price": 22.00
    }
  ],
  "tax_rate": 0,
  "notes": "Thank you for your business!"
}
```

### Send Invoice

```bash
POST /api/v1/invoices/{id}/send
Content-Type: application/json

{
  "email_to": "billing@client.com",
  "cc": ["manager@client.com"],
  "message": "Please find attached your invoice for February."
}
```

### Record Payment

```bash
POST /api/v1/invoices/{id}/payments
Content-Type: application/json

{
  "amount": 6550.00,
  "payment_method": "bank_transfer",
  "payment_date": "2024-03-15",
  "reference": "CHK-12345"
}
```

---

## Assets

### List Assets

```bash
GET /api/v1/assets
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `client_id` | uuid | Filter by client |
| `asset_type` | string | `server`, `workstation`, `network`, `printer`, etc. |
| `status` | string | `active`, `inactive`, `retired` |
| `warranty_expires_before` | date | Warranty expiring before date |

### Create Asset

```bash
POST /api/v1/assets
Content-Type: application/json

{
  "client_id": "uuid",
  "name": "DC-SERVER-01",
  "asset_type": "server",
  "manufacturer": "Dell",
  "model": "PowerEdge R740",
  "serial_number": "ABC123XYZ",
  "purchase_date": "2023-06-15",
  "warranty_expiry": "2026-06-15",
  "ip_address": "192.168.1.10",
  "mac_address": "00:11:22:33:44:55",
  "location": "Server Room A",
  "notes": "Domain Controller",
  "custom_fields": {
    "os": "Windows Server 2022",
    "ram": "128GB",
    "storage": "2TB SSD RAID"
  }
}
```

---

## Knowledge Base

### List Articles

```bash
GET /api/v1/kb
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `folder_id` | uuid | Filter by folder |
| `client_id` | uuid | Client-specific articles |
| `is_global` | boolean | Global or client-specific |
| `search` | string | Full-text search |

### Create Article

```bash
POST /api/v1/kb
Content-Type: application/json

{
  "title": "VPN Setup Guide",
  "slug": "vpn-setup-guide",
  "content": "# VPN Setup Guide\n\n## Prerequisites\n...",
  "folder_id": "uuid",
  "is_global": true,
  "tags": ["vpn", "network", "setup"]
}
```

---

## SLA Management

### List SLA Policies

```bash
GET /api/v1/sla/policies
```

### Create SLA Policy

```bash
POST /api/v1/sla/policies
Content-Type: application/json

{
  "name": "Standard Support",
  "description": "Standard SLA for regular clients",
  "is_global": true,
  "priority_levels": {
    "critical": {
      "response_minutes": 15,
      "resolution_hours": 4
    },
    "high": {
      "response_minutes": 60,
      "resolution_hours": 8
    },
    "medium": {
      "response_minutes": 240,
      "resolution_hours": 24
    },
    "low": {
      "response_minutes": 480,
      "resolution_hours": 72
    }
  },
  "business_hours": {
    "timezone": "America/New_York",
    "schedule": {
      "monday": {"start": "09:00", "end": "17:00"},
      "tuesday": {"start": "09:00", "end": "17:00"},
      "wednesday": {"start": "09:00", "end": "17:00"},
      "thursday": {"start": "09:00", "end": "17:00"},
      "friday": {"start": "09:00", "end": "17:00"}
    }
  },
  "auto_escalation": true
}
```

### Get SLA Tracking for Ticket

```bash
GET /api/v1/sla/tracking/{ticket_id}
```

### Pause SLA

```bash
POST /api/v1/sla/tracking/{ticket_id}/pause
```

### Resume SLA

```bash
POST /api/v1/sla/tracking/{ticket_id}/resume
```

---

## Workflows

### List Workflows

```bash
GET /api/v1/sla/workflows
```

### Create Workflow

```bash
POST /api/v1/sla/workflows
Content-Type: application/json

{
  "name": "Auto-assign Critical Tickets",
  "description": "Automatically assign critical tickets to senior technicians",
  "trigger_type": "ticket_created",
  "trigger_config": {
    "priority": "critical"
  },
  "conditions": {
    "logic": "AND",
    "conditions": [
      {"field": "client.is_vip", "operator": "equals", "value": true}
    ]
  },
  "actions": [
    {
      "action_type": "assign_to_group",
      "config": {"group_id": "uuid"}
    },
    {
      "action_type": "send_teams_notification",
      "config": {
        "channel": "critical-alerts",
        "message": "🚨 Critical VIP ticket: {{ticket.subject}}"
      }
    }
  ],
  "is_active": true,
  "execution_order": 1
}
```

---

## Webhooks

### Register Webhook

```bash
POST /api/v1/webhooks
Content-Type: application/json

{
  "url": "https://your-app.com/webhooks/resolve",
  "events": ["ticket.created", "ticket.updated", "invoice.paid"],
  "secret": "your-webhook-secret"
}
```

### Webhook Events

| Event | Description |
|-------|-------------|
| `ticket.created` | New ticket created |
| `ticket.updated` | Ticket modified |
| `ticket.resolved` | Ticket resolved |
| `ticket.sla_breach` | SLA breached |
| `client.created` | New client added |
| `invoice.created` | Invoice generated |
| `invoice.sent` | Invoice sent |
| `invoice.paid` | Payment received |
| `asset.warranty_expiring` | Asset warranty expiring |

### Webhook Payload Example

```json
{
  "event": "ticket.created",
  "timestamp": "2024-02-28T10:30:00Z",
  "data": {
    "ticket_id": "uuid",
    "ticket_number": "TKT-001234",
    "subject": "Network issue",
    "client_id": "uuid",
    "client_name": "Acme Corp",
    "priority": "high"
  },
  "signature": "sha256=..."
}
```

---

## Error Responses

All errors follow this format:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid request parameters",
    "details": [
      {"field": "email", "message": "Must be a valid email address"}
    ]
  }
}
```

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `UNAUTHORIZED` | 401 | Invalid or expired token |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `VALIDATION_ERROR` | 400 | Invalid request data |
| `CONFLICT` | 409 | Resource already exists |
| `RATE_LIMITED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Server error |

---

## Rate Limiting

API requests are rate limited:
- **Standard:** 1000 requests per minute
- **Burst:** 50 requests per second

Rate limit headers:
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 950
X-RateLimit-Reset: 1709123456
```

---

## Pagination

All list endpoints support pagination:

```json
{
  "data": [...],
  "meta": {
    "current_page": 1,
    "per_page": 25,
    "total": 150,
    "total_pages": 6,
    "has_more": true
  },
  "links": {
    "first": "/api/v1/tickets?page=1",
    "last": "/api/v1/tickets?page=6",
    "prev": null,
    "next": "/api/v1/tickets?page=2"
  }
}
```

---

## SDK Examples

### Python

```python
import requests

class ResolveClient:
    def __init__(self, base_url, api_key):
        self.base_url = base_url
        self.headers = {"X-API-Key": api_key}

    def get_tickets(self, status=None, priority=None):
        params = {}
        if status:
            params["status"] = status
        if priority:
            params["priority"] = priority

        response = requests.get(
            f"{self.base_url}/api/v1/tickets",
            headers=self.headers,
            params=params
        )
        return response.json()

    def create_ticket(self, client_id, subject, description, priority="medium"):
        response = requests.post(
            f"{self.base_url}/api/v1/tickets",
            headers=self.headers,
            json={
                "client_id": client_id,
                "subject": subject,
                "description": description,
                "priority": priority
            }
        )
        return response.json()

# Usage
client = ResolveClient("https://resolve.example.com", "your-api-key")
tickets = client.get_tickets(status="open", priority="high")
```

### JavaScript/TypeScript

```typescript
class ResolveClient {
  constructor(private baseUrl: string, private apiKey: string) {}

  private async request(endpoint: string, options: RequestInit = {}) {
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers: {
        'X-API-Key': this.apiKey,
        'Content-Type': 'application/json',
        ...options.headers,
      },
    });
    return response.json();
  }

  async getTickets(params?: { status?: string; priority?: string }) {
    const query = new URLSearchParams(params).toString();
    return this.request(`/api/v1/tickets?${query}`);
  }

  async createTicket(data: {
    client_id: string;
    subject: string;
    description: string;
    priority?: string;
  }) {
    return this.request('/api/v1/tickets', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }
}

// Usage
const client = new ResolveClient('https://resolve.example.com', 'your-api-key');
const tickets = await client.getTickets({ status: 'open' });
```

---

## OpenAPI/Swagger

Full OpenAPI 3.0 specification is available at:
- **Swagger UI:** `/api/v1/docs/swagger`
- **ReDoc:** `/api/v1/docs/redoc`
- **RapiDoc:** `/api/v1/docs/rapidoc`
- **JSON Spec:** `/api/v1/docs/openapi.json`
