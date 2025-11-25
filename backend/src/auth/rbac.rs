//! Role-Based Access Control (RBAC) system
//!
//! Implements hierarchical permissions for Resolve:
//! - Roles: Admin, Manager, Technician, Billing, ReadOnly, Custom
//! - Permissions: Resource-based (clients, tickets, etc.) + Action-based (read, write, delete)
//! - Client-level access control: Users can be restricted to specific clients

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::error::{AppError, ApiResult};

/// System role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    /// Is this a built-in system role?
    pub is_system: bool,
    /// Permissions granted to this role
    pub permissions: Vec<Permission>,
    /// Role hierarchy level (higher = more permissions)
    pub hierarchy_level: u8,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Permission definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Permission {
    /// Resource this permission applies to
    pub resource: Resource,
    /// Actions allowed on the resource
    pub actions: Vec<Action>,
}

/// Resources in the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Resource {
    // Core resources
    Clients,
    Contacts,
    Locations,

    // Ticketing
    Tickets,
    TicketTemplates,
    RecurringTickets,
    TimeEntries,

    // Assets & Documentation
    Assets,
    AssetTypes,
    Passwords,
    Documentation,
    KnowledgeBase,

    // Network
    Domains,
    Certificates,
    Networks,

    // Financial
    Invoices,
    Quotes,
    Products,
    Payments,
    Expenses,
    Contracts,

    // Users & Settings
    Users,
    Roles,
    Teams,
    Settings,
    Integrations,
    AuditLogs,
    ApiKeys,

    // Reports
    Reports,
    Dashboards,

    // Notifications
    Notifications,
    EmailTemplates,

    // All resources (admin)
    All,
}

impl Resource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clients => "clients",
            Self::Contacts => "contacts",
            Self::Locations => "locations",
            Self::Tickets => "tickets",
            Self::TicketTemplates => "ticket_templates",
            Self::RecurringTickets => "recurring_tickets",
            Self::TimeEntries => "time_entries",
            Self::Assets => "assets",
            Self::AssetTypes => "asset_types",
            Self::Passwords => "passwords",
            Self::Documentation => "documentation",
            Self::KnowledgeBase => "knowledge_base",
            Self::Domains => "domains",
            Self::Certificates => "certificates",
            Self::Networks => "networks",
            Self::Invoices => "invoices",
            Self::Quotes => "quotes",
            Self::Products => "products",
            Self::Payments => "payments",
            Self::Expenses => "expenses",
            Self::Contracts => "contracts",
            Self::Users => "users",
            Self::Roles => "roles",
            Self::Teams => "teams",
            Self::Settings => "settings",
            Self::Integrations => "integrations",
            Self::AuditLogs => "audit_logs",
            Self::ApiKeys => "api_keys",
            Self::Reports => "reports",
            Self::Dashboards => "dashboards",
            Self::Notifications => "notifications",
            Self::EmailTemplates => "email_templates",
            Self::All => "*",
        }
    }
}

/// Actions that can be performed on resources
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// View/read resources
    Read,
    /// Create new resources
    Create,
    /// Update existing resources
    Update,
    /// Delete resources
    Delete,
    /// Export resources
    Export,
    /// Import resources
    Import,
    /// Assign resources to others
    Assign,
    /// Approve/review resources
    Approve,
    /// All actions
    All,
}

impl Action {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Create => "create",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::Export => "export",
            Self::Import => "import",
            Self::Assign => "assign",
            Self::Approve => "approve",
            Self::All => "*",
        }
    }
}

/// User's access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccess {
    pub user_id: Uuid,
    pub role_id: Uuid,
    /// Additional permissions beyond role (additive)
    pub additional_permissions: Vec<Permission>,
    /// Permissions denied even if role grants them (subtractive)
    pub denied_permissions: Vec<Permission>,
    /// Client-level restrictions (empty = access to all clients)
    pub client_access: ClientAccessMode,
    /// Team memberships
    pub team_ids: Vec<Uuid>,
}

/// Client access mode
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientAccessMode {
    /// Access to all clients
    All,
    /// Access only to assigned clients
    Assigned { client_ids: Vec<Uuid> },
    /// Access to clients belonging to user's teams
    TeamBased,
    /// No client access (internal user)
    None,
}

/// Permission check context
#[derive(Debug)]
pub struct PermissionContext {
    pub user_id: Uuid,
    pub role: Role,
    pub user_access: UserAccess,
    /// Specific client being accessed (for client-level checks)
    pub target_client_id: Option<Uuid>,
    /// Resource owner (for ownership checks)
    pub resource_owner_id: Option<Uuid>,
}

/// Permission checker
pub struct PermissionChecker {
    /// Cache of computed permissions per user
    permission_cache: HashMap<Uuid, HashSet<(Resource, Action)>>,
}

impl PermissionChecker {
    pub fn new() -> Self {
        Self {
            permission_cache: HashMap::new(),
        }
    }

    /// Check if a user has a specific permission
    pub fn has_permission(
        &self,
        ctx: &PermissionContext,
        resource: &Resource,
        action: &Action,
    ) -> bool {
        // Check denied permissions first
        if self.is_denied(&ctx.user_access, resource, action) {
            return false;
        }

        // Check role permissions
        if self.role_has_permission(&ctx.role, resource, action) {
            return true;
        }

        // Check additional permissions
        if self.has_additional_permission(&ctx.user_access, resource, action) {
            return true;
        }

        false
    }

    /// Check if user can access a specific client
    pub fn can_access_client(&self, ctx: &PermissionContext, client_id: Uuid) -> bool {
        match &ctx.user_access.client_access {
            ClientAccessMode::All => true,
            ClientAccessMode::Assigned { client_ids } => client_ids.contains(&client_id),
            ClientAccessMode::TeamBased => {
                // Would need to check team->client mappings from database
                // For now, return true (implement in integration)
                true
            }
            ClientAccessMode::None => false,
        }
    }

    /// Check if user owns a resource (for edit/delete own resources)
    pub fn is_owner(&self, ctx: &PermissionContext) -> bool {
        ctx.resource_owner_id == Some(ctx.user_id)
    }

    /// Require a permission or return error
    pub fn require_permission(
        &self,
        ctx: &PermissionContext,
        resource: &Resource,
        action: &Action,
    ) -> ApiResult<()> {
        if self.has_permission(ctx, resource, action) {
            Ok(())
        } else {
            Err(AppError::InsufficientPermissions {
                required: format!("{}:{}", resource.as_str(), action.as_str()),
            })
        }
    }

    /// Require client access or return error
    pub fn require_client_access(
        &self,
        ctx: &PermissionContext,
        client_id: Uuid,
    ) -> ApiResult<()> {
        if self.can_access_client(ctx, client_id) {
            Ok(())
        } else {
            Err(AppError::Forbidden(
                "You do not have access to this client".to_string(),
            ))
        }
    }

    fn role_has_permission(&self, role: &Role, resource: &Resource, action: &Action) -> bool {
        for permission in &role.permissions {
            if permission.resource == Resource::All || permission.resource == *resource {
                if permission.actions.contains(&Action::All) || permission.actions.contains(action)
                {
                    return true;
                }
            }
        }
        false
    }

    fn has_additional_permission(
        &self,
        access: &UserAccess,
        resource: &Resource,
        action: &Action,
    ) -> bool {
        for permission in &access.additional_permissions {
            if permission.resource == Resource::All || permission.resource == *resource {
                if permission.actions.contains(&Action::All) || permission.actions.contains(action)
                {
                    return true;
                }
            }
        }
        false
    }

    fn is_denied(&self, access: &UserAccess, resource: &Resource, action: &Action) -> bool {
        for permission in &access.denied_permissions {
            if permission.resource == Resource::All || permission.resource == *resource {
                if permission.actions.contains(&Action::All) || permission.actions.contains(action)
                {
                    return true;
                }
            }
        }
        false
    }
}

impl Default for PermissionChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Default system roles
pub mod default_roles {
    use super::*;

    pub fn admin() -> Role {
        Role {
            id: Uuid::nil(), // Will be set on creation
            name: "admin".to_string(),
            display_name: "Administrator".to_string(),
            description: Some("Full system access".to_string()),
            is_system: true,
            permissions: vec![Permission {
                resource: Resource::All,
                actions: vec![Action::All],
            }],
            hierarchy_level: 100,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    pub fn manager() -> Role {
        Role {
            id: Uuid::nil(),
            name: "manager".to_string(),
            display_name: "Manager".to_string(),
            description: Some("Manage team and clients".to_string()),
            is_system: true,
            permissions: vec![
                // Full access to operational resources
                Permission {
                    resource: Resource::Clients,
                    actions: vec![Action::All],
                },
                Permission {
                    resource: Resource::Contacts,
                    actions: vec![Action::All],
                },
                Permission {
                    resource: Resource::Tickets,
                    actions: vec![Action::All],
                },
                Permission {
                    resource: Resource::TimeEntries,
                    actions: vec![Action::All],
                },
                Permission {
                    resource: Resource::Assets,
                    actions: vec![Action::All],
                },
                Permission {
                    resource: Resource::Documentation,
                    actions: vec![Action::All],
                },
                Permission {
                    resource: Resource::Passwords,
                    actions: vec![Action::Read, Action::Create, Action::Update],
                },
                Permission {
                    resource: Resource::Invoices,
                    actions: vec![Action::Read, Action::Create, Action::Update, Action::Approve],
                },
                Permission {
                    resource: Resource::Reports,
                    actions: vec![Action::Read, Action::Export],
                },
                // Limited user management
                Permission {
                    resource: Resource::Users,
                    actions: vec![Action::Read, Action::Assign],
                },
            ],
            hierarchy_level: 80,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    pub fn technician() -> Role {
        Role {
            id: Uuid::nil(),
            name: "technician".to_string(),
            display_name: "Technician".to_string(),
            description: Some("Technical support role".to_string()),
            is_system: true,
            permissions: vec![
                Permission {
                    resource: Resource::Clients,
                    actions: vec![Action::Read],
                },
                Permission {
                    resource: Resource::Contacts,
                    actions: vec![Action::Read, Action::Create, Action::Update],
                },
                Permission {
                    resource: Resource::Tickets,
                    actions: vec![Action::Read, Action::Create, Action::Update],
                },
                Permission {
                    resource: Resource::TimeEntries,
                    actions: vec![Action::Read, Action::Create, Action::Update],
                },
                Permission {
                    resource: Resource::Assets,
                    actions: vec![Action::Read, Action::Create, Action::Update],
                },
                Permission {
                    resource: Resource::Documentation,
                    actions: vec![Action::Read, Action::Create, Action::Update],
                },
                Permission {
                    resource: Resource::KnowledgeBase,
                    actions: vec![Action::Read],
                },
                Permission {
                    resource: Resource::Passwords,
                    actions: vec![Action::Read],
                },
            ],
            hierarchy_level: 50,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    pub fn billing() -> Role {
        Role {
            id: Uuid::nil(),
            name: "billing".to_string(),
            display_name: "Billing".to_string(),
            description: Some("Financial operations".to_string()),
            is_system: true,
            permissions: vec![
                Permission {
                    resource: Resource::Clients,
                    actions: vec![Action::Read],
                },
                Permission {
                    resource: Resource::Contacts,
                    actions: vec![Action::Read],
                },
                Permission {
                    resource: Resource::Invoices,
                    actions: vec![Action::All],
                },
                Permission {
                    resource: Resource::Quotes,
                    actions: vec![Action::All],
                },
                Permission {
                    resource: Resource::Payments,
                    actions: vec![Action::All],
                },
                Permission {
                    resource: Resource::Expenses,
                    actions: vec![Action::All],
                },
                Permission {
                    resource: Resource::Contracts,
                    actions: vec![Action::Read, Action::Create, Action::Update],
                },
                Permission {
                    resource: Resource::Products,
                    actions: vec![Action::All],
                },
                Permission {
                    resource: Resource::TimeEntries,
                    actions: vec![Action::Read, Action::Approve],
                },
                Permission {
                    resource: Resource::Reports,
                    actions: vec![Action::Read, Action::Export],
                },
            ],
            hierarchy_level: 60,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    pub fn readonly() -> Role {
        Role {
            id: Uuid::nil(),
            name: "readonly".to_string(),
            display_name: "Read Only".to_string(),
            description: Some("View-only access".to_string()),
            is_system: true,
            permissions: vec![
                Permission {
                    resource: Resource::Clients,
                    actions: vec![Action::Read],
                },
                Permission {
                    resource: Resource::Contacts,
                    actions: vec![Action::Read],
                },
                Permission {
                    resource: Resource::Tickets,
                    actions: vec![Action::Read],
                },
                Permission {
                    resource: Resource::Assets,
                    actions: vec![Action::Read],
                },
                Permission {
                    resource: Resource::Documentation,
                    actions: vec![Action::Read],
                },
                Permission {
                    resource: Resource::KnowledgeBase,
                    actions: vec![Action::Read],
                },
                Permission {
                    resource: Resource::Reports,
                    actions: vec![Action::Read],
                },
                Permission {
                    resource: Resource::Dashboards,
                    actions: vec![Action::Read],
                },
            ],
            hierarchy_level: 10,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    pub fn all_default_roles() -> Vec<Role> {
        vec![admin(), manager(), technician(), billing(), readonly()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_has_all_permissions() {
        let admin_role = default_roles::admin();
        let checker = PermissionChecker::new();

        assert!(checker.role_has_permission(&admin_role, &Resource::Clients, &Action::Read));
        assert!(checker.role_has_permission(&admin_role, &Resource::Clients, &Action::Delete));
        assert!(checker.role_has_permission(&admin_role, &Resource::Settings, &Action::Update));
    }

    #[test]
    fn test_technician_limited_permissions() {
        let tech_role = default_roles::technician();
        let checker = PermissionChecker::new();

        // Should have
        assert!(checker.role_has_permission(&tech_role, &Resource::Tickets, &Action::Read));
        assert!(checker.role_has_permission(&tech_role, &Resource::Tickets, &Action::Create));

        // Should not have
        assert!(!checker.role_has_permission(&tech_role, &Resource::Tickets, &Action::Delete));
        assert!(!checker.role_has_permission(&tech_role, &Resource::Settings, &Action::Update));
    }

    #[test]
    fn test_client_access_modes() {
        let checker = PermissionChecker::new();
        let client_id = Uuid::new_v4();
        let other_client_id = Uuid::new_v4();

        let ctx_all = PermissionContext {
            user_id: Uuid::new_v4(),
            role: default_roles::technician(),
            user_access: UserAccess {
                user_id: Uuid::new_v4(),
                role_id: Uuid::new_v4(),
                additional_permissions: vec![],
                denied_permissions: vec![],
                client_access: ClientAccessMode::All,
                team_ids: vec![],
            },
            target_client_id: None,
            resource_owner_id: None,
        };

        assert!(checker.can_access_client(&ctx_all, client_id));

        let ctx_assigned = PermissionContext {
            user_access: UserAccess {
                client_access: ClientAccessMode::Assigned {
                    client_ids: vec![client_id],
                },
                ..ctx_all.user_access.clone()
            },
            ..ctx_all
        };

        assert!(checker.can_access_client(&ctx_assigned, client_id));
        assert!(!checker.can_access_client(&ctx_assigned, other_client_id));
    }
}
