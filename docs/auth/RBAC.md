# Role-Based Access Control (RBAC)

Resolve uses a flexible role-based access control system to manage permissions across the platform.

## Overview

RBAC in Resolve consists of:
- **Roles**: Named groups of permissions (e.g., Admin, Technician)
- **Permissions**: Specific actions on resources (e.g., `tickets.create`)
- **Hierarchy**: Numeric levels determining role seniority

## Default Roles

| Role | Hierarchy | Description |
|------|-----------|-------------|
| Admin | 100 | Full system access |
| Manager | 80 | Manage team and clients |
| Billing | 60 | Financial operations |
| Technician | 50 | Technical support |
| ReadOnly | 10 | View-only access |

## Permission Structure

Permissions follow the format: `resource.action`

### Resources

| Resource | Description |
|----------|-------------|
| `clients` | Client organizations |
| `tickets` | Support tickets |
| `assets` | Hardware/software assets |
| `passwords` | Secure password storage |
| `documentation` | KB articles and docs |
| `invoices` | Billing and invoices |
| `users` | User accounts |
| `settings` | System configuration |
| `reports` | Analytics and reports |
| `all` | All resources (admin) |

### Actions

| Action | Description |
|--------|-------------|
| `read` | View resources |
| `create` | Create new resources |
| `update` | Modify existing resources |
| `delete` | Remove resources |
| `export` | Export/download data |
| `assign` | Assign to users/teams |
| `approve` | Approve requests |
| `reveal` | Reveal sensitive data |
| `all` | All actions |

### Common Permissions

| Permission | Description |
|------------|-------------|
| `clients.read` | View clients |
| `clients.create` | Create clients |
| `clients.update` | Update clients |
| `clients.delete` | Delete clients |
| `tickets.read` | View tickets |
| `tickets.create` | Create tickets |
| `tickets.update` | Update tickets |
| `tickets.assign` | Assign tickets |
| `assets.read` | View assets |
| `assets.create` | Create assets |
| `passwords.read` | View password entries |
| `passwords.reveal` | Reveal password values |
| `users.read` | View users |
| `users.create` | Create users |
| `admin.all` | Full admin access |

## Managing Roles

### Via Admin Panel

1. Go to **Admin** > **Roles**
2. Click **Create Role** or edit existing
3. Configure:
   - **Name**: Role identifier
   - **Display Name**: User-friendly name
   - **Hierarchy**: Numeric level (higher = more senior)
   - **Permissions**: Select granted permissions

### Via Database

```sql
-- Create role
INSERT INTO roles (id, name, display_name, hierarchy, description)
VALUES (
    gen_random_uuid(),
    'support_lead',
    'Support Lead',
    70,
    'Senior technician with team management'
);

-- Assign permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT
    (SELECT id FROM roles WHERE name = 'support_lead'),
    id
FROM permissions
WHERE name IN (
    'tickets.read', 'tickets.create', 'tickets.update',
    'tickets.assign', 'tickets.delete',
    'clients.read', 'assets.read'
);
```

## Assigning Roles to Users

### Via Admin Panel

1. Go to **Admin** > **Users**
2. Click on a user
3. Select **Role** from dropdown
4. Click **Save**

### Via API

```bash
curl -X PATCH https://your-domain.com/api/v1/users/:id \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"role_id": "role-uuid"}'
```

## Role Hierarchy

The hierarchy system provides implicit permissions:
- Users can manage users with lower hierarchy levels
- Higher hierarchy can override lower permissions
- Used for escalation and approval workflows

Example:
- Admin (100) can manage all users
- Manager (80) can manage Technicians (50) but not other Managers
- ReadOnly (10) cannot manage anyone

## Checking Permissions

### In Handlers (Rust)

```rust
async fn update_ticket(
    auth: AuthUserWithRole,
    State(state): State<Arc<AppState>>,
    Path(ticket_id): Path<Uuid>,
    Json(req): Json<UpdateTicketRequest>,
) -> ApiResult<impl IntoResponse> {
    // Check permission
    auth.require(Resource::Tickets, Action::Update)?;

    // Or check multiple permissions
    if !auth.has_any_permission(&["tickets.update", "admin.all"]) {
        return Err(AppError::Forbidden("Cannot update tickets".to_string()));
    }

    // Continue with update...
}
```

### In Frontend

```typescript
// React example
const canEditTicket = user.permissions.includes('tickets.update')
    || user.permissions.includes('admin.all');

{canEditTicket && (
    <Button onClick={handleEdit}>Edit Ticket</Button>
)}
```

## Client-Level Permissions

In addition to global roles, Resolve supports client-level access:

```sql
-- Grant user access to specific client
INSERT INTO user_client_access (user_id, client_id, permission_level)
VALUES (
    'user-uuid',
    'client-uuid',
    'full'  -- 'full', 'readonly', or 'none'
);
```

This allows:
- Restricting technicians to specific clients
- Giving read-only access to certain accounts
- Client portal users seeing only their organization

## Custom Roles

Create custom roles for specific needs:

```sql
-- Create "Auditor" role with specific permissions
INSERT INTO roles (id, name, display_name, hierarchy, description)
VALUES (
    gen_random_uuid(),
    'auditor',
    'Auditor',
    30,
    'Read-only access with export capabilities'
);

-- Grant read and export permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT
    (SELECT id FROM roles WHERE name = 'auditor'),
    id
FROM permissions
WHERE name LIKE '%.read' OR name LIKE '%.export';
```

## Role Inheritance (Future)

Planned feature: role inheritance for creating role hierarchies.

```json
{
  "name": "senior_technician",
  "inherits_from": "technician",
  "additional_permissions": [
    "tickets.assign",
    "tickets.delete"
  ]
}
```

## Audit Trail

Permission checks are logged:
- User attempted action
- Permission checked
- Result (allowed/denied)
- Timestamp
- Resource accessed

View in **Admin** > **Audit Log** > **Access Control**.

## Best Practices

1. **Least Privilege**: Assign minimum required permissions
2. **Use Roles**: Don't assign permissions directly to users
3. **Regular Reviews**: Audit role assignments periodically
4. **Clear Naming**: Use descriptive role names
5. **Document Custom Roles**: Keep role purposes documented
6. **Test Permissions**: Verify new roles work as expected

## API Reference

### List Roles

```
GET /api/v1/roles
```

### Get Role Details

```
GET /api/v1/roles/:id
```

### Create Role

```
POST /api/v1/roles
```

```json
{
  "name": "custom_role",
  "display_name": "Custom Role",
  "hierarchy": 40,
  "description": "A custom role",
  "permission_ids": ["perm-uuid-1", "perm-uuid-2"]
}
```

### Update Role

```
PATCH /api/v1/roles/:id
```

### Delete Role

```
DELETE /api/v1/roles/:id
```

Note: Cannot delete roles with assigned users.

### List Permissions

```
GET /api/v1/permissions
```

### Get Current User Permissions

```
GET /api/v1/auth/me
```

Response includes:
```json
{
  "user": {...},
  "role": {
    "id": "role-uuid",
    "name": "technician",
    "display_name": "Technician",
    "hierarchy": 50
  },
  "permissions": [
    "tickets.read",
    "tickets.create",
    "tickets.update",
    "clients.read",
    "assets.read"
  ]
}
```

## Related Documentation

- [Authentication Overview](./README.md)
- [API Keys](./API_KEYS.md)
- [User Management](../admin/USERS.md)
