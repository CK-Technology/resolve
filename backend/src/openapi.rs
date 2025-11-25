use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenAPI 3.0 Specification Builder for Resolve MSP Platform
/// This provides a programmatic way to generate OpenAPI documentation

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: Info,
    pub servers: Vec<Server>,
    pub paths: HashMap<String, PathItem>,
    pub components: Components,
    pub security: Vec<HashMap<String, Vec<String>>>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    pub title: String,
    pub description: String,
    pub version: String,
    pub contact: Contact,
    pub license: License,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct License {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Server {
    pub url: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Operation {
    pub tags: Vec<String>,
    pub summary: String,
    pub description: String,
    #[serde(rename = "operationId")]
    pub operation_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Parameter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "requestBody")]
    pub request_body: Option<RequestBody>,
    pub responses: HashMap<String, Response>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub security: Vec<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: String,
    pub description: String,
    pub required: bool,
    pub schema: Schema,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestBody {
    pub description: String,
    pub required: bool,
    pub content: HashMap<String, MediaType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaType {
    pub schema: Schema,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<HashMap<String, MediaType>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Schema {
    Ref {
        #[serde(rename = "$ref")]
        reference: String,
    },
    Object {
        #[serde(rename = "type")]
        schema_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        properties: Option<HashMap<String, Schema>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        required: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        items: Option<Box<Schema>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "enum")]
        enum_values: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        nullable: Option<bool>,
    },
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Components {
    pub schemas: HashMap<String, Schema>,
    #[serde(rename = "securitySchemes")]
    pub security_schemes: HashMap<String, SecurityScheme>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityScheme {
    #[serde(rename = "type")]
    pub scheme_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bearerFormat")]
    pub bearer_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "in")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl OpenApiSpec {
    /// Generate the complete OpenAPI specification for Resolve MSP Platform
    pub fn generate() -> Self {
        let mut spec = Self {
            openapi: "3.0.3".to_string(),
            info: Info {
                title: "Resolve MSP Platform API".to_string(),
                description: "Comprehensive API for Managed Service Provider operations including ticketing, asset management, billing, and client portal".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                contact: Contact {
                    name: "Resolve Support".to_string(),
                    email: "support@resolve.local".to_string(),
                },
                license: License {
                    name: "Proprietary".to_string(),
                    url: "https://resolve.local/license".to_string(),
                },
            },
            servers: vec![
                Server {
                    url: "http://localhost:3001".to_string(),
                    description: "Development server".to_string(),
                },
                Server {
                    url: "https://api.resolve.local".to_string(),
                    description: "Production server".to_string(),
                },
            ],
            paths: HashMap::new(),
            components: Components::default(),
            security: vec![{
                let mut map = HashMap::new();
                map.insert("bearerAuth".to_string(), vec![]);
                map
            }],
            tags: vec![
                Tag { name: "Auth".to_string(), description: "Authentication and authorization".to_string() },
                Tag { name: "Clients".to_string(), description: "Client management".to_string() },
                Tag { name: "Tickets".to_string(), description: "Ticket management and SLA tracking".to_string() },
                Tag { name: "Assets".to_string(), description: "Asset inventory and monitoring".to_string() },
                Tag { name: "Invoices".to_string(), description: "Invoice and billing management".to_string() },
                Tag { name: "Time".to_string(), description: "Time tracking and entries".to_string() },
                Tag { name: "Projects".to_string(), description: "Project management".to_string() },
                Tag { name: "Knowledge Base".to_string(), description: "Knowledge base articles".to_string() },
                Tag { name: "Users".to_string(), description: "User management".to_string() },
                Tag { name: "Analytics".to_string(), description: "Reporting and analytics".to_string() },
                Tag { name: "Teams".to_string(), description: "Microsoft Teams integration".to_string() },
                Tag { name: "System".to_string(), description: "System health and metrics".to_string() },
            ],
        };

        spec.add_security_schemes();
        spec.add_common_schemas();
        spec.add_auth_paths();
        spec.add_client_paths();
        spec.add_ticket_paths();
        spec.add_asset_paths();
        spec.add_invoice_paths();
        spec.add_time_paths();
        spec.add_analytics_paths();
        spec.add_system_paths();

        spec
    }

    fn add_security_schemes(&mut self) {
        self.components.security_schemes.insert(
            "bearerAuth".to_string(),
            SecurityScheme {
                scheme_type: "http".to_string(),
                scheme: Some("bearer".to_string()),
                bearer_format: Some("JWT".to_string()),
                name: None,
                location: None,
                description: Some("JWT Authentication token".to_string()),
            },
        );

        self.components.security_schemes.insert(
            "apiKeyAuth".to_string(),
            SecurityScheme {
                scheme_type: "apiKey".to_string(),
                scheme: None,
                bearer_format: None,
                name: Some("X-API-Key".to_string()),
                location: Some("header".to_string()),
                description: Some("API Key for integrations".to_string()),
            },
        );
    }

    fn add_common_schemas(&mut self) {
        // UUID schema
        self.components.schemas.insert(
            "UUID".to_string(),
            Schema::Object {
                schema_type: "string".to_string(),
                format: Some("uuid".to_string()),
                properties: None,
                required: None,
                items: None,
                description: Some("Universally unique identifier".to_string()),
                enum_values: None,
                nullable: None,
            },
        );

        // Pagination meta schema
        self.components.schemas.insert(
            "PaginationMeta".to_string(),
            Schema::Object {
                schema_type: "object".to_string(),
                format: None,
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert("page".to_string(), Schema::Object {
                        schema_type: "integer".to_string(),
                        format: None,
                        properties: None,
                        required: None,
                        items: None,
                        description: Some("Current page number".to_string()),
                        enum_values: None,
                        nullable: None,
                    });
                    props.insert("per_page".to_string(), Schema::Object {
                        schema_type: "integer".to_string(),
                        format: None,
                        properties: None,
                        required: None,
                        items: None,
                        description: Some("Items per page".to_string()),
                        enum_values: None,
                        nullable: None,
                    });
                    props.insert("total".to_string(), Schema::Object {
                        schema_type: "integer".to_string(),
                        format: None,
                        properties: None,
                        required: None,
                        items: None,
                        description: Some("Total number of items".to_string()),
                        enum_values: None,
                        nullable: None,
                    });
                    props.insert("total_pages".to_string(), Schema::Object {
                        schema_type: "integer".to_string(),
                        format: None,
                        properties: None,
                        required: None,
                        items: None,
                        description: Some("Total number of pages".to_string()),
                        enum_values: None,
                        nullable: None,
                    });
                    props
                }),
                required: Some(vec!["page".to_string(), "per_page".to_string(), "total".to_string(), "total_pages".to_string()]),
                items: None,
                description: Some("Pagination metadata".to_string()),
                enum_values: None,
                nullable: None,
            },
        );

        // Error response schema
        self.components.schemas.insert(
            "ErrorResponse".to_string(),
            Schema::Object {
                schema_type: "object".to_string(),
                format: None,
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert("error".to_string(), Schema::Object {
                        schema_type: "string".to_string(),
                        format: None,
                        properties: None,
                        required: None,
                        items: None,
                        description: Some("Error message".to_string()),
                        enum_values: None,
                        nullable: None,
                    });
                    props.insert("code".to_string(), Schema::Object {
                        schema_type: "string".to_string(),
                        format: None,
                        properties: None,
                        required: None,
                        items: None,
                        description: Some("Error code".to_string()),
                        enum_values: None,
                        nullable: Some(true),
                    });
                    props
                }),
                required: Some(vec!["error".to_string()]),
                items: None,
                description: Some("Standard error response".to_string()),
                enum_values: None,
                nullable: None,
            },
        );

        // Client schema
        self.components.schemas.insert(
            "Client".to_string(),
            Schema::Object {
                schema_type: "object".to_string(),
                format: None,
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert("id".to_string(), Schema::Ref { reference: "#/components/schemas/UUID".to_string() });
                    props.insert("name".to_string(), Schema::Object {
                        schema_type: "string".to_string(),
                        format: None,
                        properties: None,
                        required: None,
                        items: None,
                        description: Some("Client name".to_string()),
                        enum_values: None,
                        nullable: None,
                    });
                    props.insert("email".to_string(), Schema::Object {
                        schema_type: "string".to_string(),
                        format: Some("email".to_string()),
                        properties: None,
                        required: None,
                        items: None,
                        description: Some("Primary contact email".to_string()),
                        enum_values: None,
                        nullable: Some(true),
                    });
                    props.insert("is_active".to_string(), Schema::Object {
                        schema_type: "boolean".to_string(),
                        format: None,
                        properties: None,
                        required: None,
                        items: None,
                        description: Some("Whether client is active".to_string()),
                        enum_values: None,
                        nullable: None,
                    });
                    props.insert("created_at".to_string(), Schema::Object {
                        schema_type: "string".to_string(),
                        format: Some("date-time".to_string()),
                        properties: None,
                        required: None,
                        items: None,
                        description: Some("Creation timestamp".to_string()),
                        enum_values: None,
                        nullable: None,
                    });
                    props
                }),
                required: Some(vec!["id".to_string(), "name".to_string(), "is_active".to_string()]),
                items: None,
                description: Some("Client entity".to_string()),
                enum_values: None,
                nullable: None,
            },
        );

        // Ticket schema
        self.components.schemas.insert(
            "Ticket".to_string(),
            Schema::Object {
                schema_type: "object".to_string(),
                format: None,
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert("id".to_string(), Schema::Ref { reference: "#/components/schemas/UUID".to_string() });
                    props.insert("number".to_string(), Schema::Object {
                        schema_type: "integer".to_string(),
                        format: None,
                        properties: None,
                        required: None,
                        items: None,
                        description: Some("Ticket number".to_string()),
                        enum_values: None,
                        nullable: None,
                    });
                    props.insert("subject".to_string(), Schema::Object {
                        schema_type: "string".to_string(),
                        format: None,
                        properties: None,
                        required: None,
                        items: None,
                        description: Some("Ticket subject".to_string()),
                        enum_values: None,
                        nullable: None,
                    });
                    props.insert("status".to_string(), Schema::Object {
                        schema_type: "string".to_string(),
                        format: None,
                        properties: None,
                        required: None,
                        items: None,
                        description: Some("Ticket status".to_string()),
                        enum_values: Some(vec!["open".to_string(), "in_progress".to_string(), "pending".to_string(), "resolved".to_string(), "closed".to_string()]),
                        nullable: None,
                    });
                    props.insert("priority".to_string(), Schema::Object {
                        schema_type: "string".to_string(),
                        format: None,
                        properties: None,
                        required: None,
                        items: None,
                        description: Some("Ticket priority".to_string()),
                        enum_values: Some(vec!["low".to_string(), "medium".to_string(), "high".to_string(), "critical".to_string()]),
                        nullable: None,
                    });
                    props.insert("client_id".to_string(), Schema::Ref { reference: "#/components/schemas/UUID".to_string() });
                    props.insert("assigned_to".to_string(), Schema::Ref { reference: "#/components/schemas/UUID".to_string() });
                    props.insert("created_at".to_string(), Schema::Object {
                        schema_type: "string".to_string(),
                        format: Some("date-time".to_string()),
                        properties: None,
                        required: None,
                        items: None,
                        description: Some("Creation timestamp".to_string()),
                        enum_values: None,
                        nullable: None,
                    });
                    props
                }),
                required: Some(vec!["id".to_string(), "number".to_string(), "subject".to_string(), "status".to_string(), "priority".to_string(), "client_id".to_string()]),
                items: None,
                description: Some("Support ticket".to_string()),
                enum_values: None,
                nullable: None,
            },
        );
    }

    fn add_auth_paths(&mut self) {
        // Login endpoint
        self.paths.insert("/api/v1/auth/login".to_string(), PathItem {
            post: Some(Operation {
                tags: vec!["Auth".to_string()],
                summary: "User login".to_string(),
                description: "Authenticate a user and receive a JWT token".to_string(),
                operation_id: "login".to_string(),
                parameters: vec![],
                request_body: Some(RequestBody {
                    description: "Login credentials".to_string(),
                    required: true,
                    content: {
                        let mut content = HashMap::new();
                        content.insert("application/json".to_string(), MediaType {
                            schema: Schema::Object {
                                schema_type: "object".to_string(),
                                format: None,
                                properties: Some({
                                    let mut props = HashMap::new();
                                    props.insert("email".to_string(), Schema::Object {
                                        schema_type: "string".to_string(),
                                        format: Some("email".to_string()),
                                        properties: None,
                                        required: None,
                                        items: None,
                                        description: None,
                                        enum_values: None,
                                        nullable: None,
                                    });
                                    props.insert("password".to_string(), Schema::Object {
                                        schema_type: "string".to_string(),
                                        format: Some("password".to_string()),
                                        properties: None,
                                        required: None,
                                        items: None,
                                        description: None,
                                        enum_values: None,
                                        nullable: None,
                                    });
                                    props
                                }),
                                required: Some(vec!["email".to_string(), "password".to_string()]),
                                items: None,
                                description: None,
                                enum_values: None,
                                nullable: None,
                            },
                        });
                        content
                    },
                }),
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert("200".to_string(), Response {
                        description: "Successful authentication".to_string(),
                        content: None,
                    });
                    responses.insert("401".to_string(), Response {
                        description: "Invalid credentials".to_string(),
                        content: None,
                    });
                    responses
                },
                security: vec![],
            }),
            ..Default::default()
        });
    }

    fn add_client_paths(&mut self) {
        // List clients
        self.paths.insert("/api/v1/clients".to_string(), PathItem {
            get: Some(Operation {
                tags: vec!["Clients".to_string()],
                summary: "List all clients".to_string(),
                description: "Retrieve a paginated list of clients".to_string(),
                operation_id: "listClients".to_string(),
                parameters: vec![
                    Parameter {
                        name: "page".to_string(),
                        location: "query".to_string(),
                        description: "Page number".to_string(),
                        required: false,
                        schema: Schema::Object {
                            schema_type: "integer".to_string(),
                            format: None,
                            properties: None,
                            required: None,
                            items: None,
                            description: None,
                            enum_values: None,
                            nullable: None,
                        },
                    },
                    Parameter {
                        name: "per_page".to_string(),
                        location: "query".to_string(),
                        description: "Items per page".to_string(),
                        required: false,
                        schema: Schema::Object {
                            schema_type: "integer".to_string(),
                            format: None,
                            properties: None,
                            required: None,
                            items: None,
                            description: None,
                            enum_values: None,
                            nullable: None,
                        },
                    },
                ],
                request_body: None,
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert("200".to_string(), Response {
                        description: "List of clients".to_string(),
                        content: None,
                    });
                    responses
                },
                security: vec![],
            }),
            post: Some(Operation {
                tags: vec!["Clients".to_string()],
                summary: "Create a new client".to_string(),
                description: "Create a new client record".to_string(),
                operation_id: "createClient".to_string(),
                parameters: vec![],
                request_body: Some(RequestBody {
                    description: "Client data".to_string(),
                    required: true,
                    content: {
                        let mut content = HashMap::new();
                        content.insert("application/json".to_string(), MediaType {
                            schema: Schema::Ref { reference: "#/components/schemas/Client".to_string() },
                        });
                        content
                    },
                }),
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert("201".to_string(), Response {
                        description: "Client created".to_string(),
                        content: None,
                    });
                    responses
                },
                security: vec![],
            }),
            ..Default::default()
        });
    }

    fn add_ticket_paths(&mut self) {
        // List tickets
        self.paths.insert("/api/v1/tickets".to_string(), PathItem {
            get: Some(Operation {
                tags: vec!["Tickets".to_string()],
                summary: "List all tickets".to_string(),
                description: "Retrieve a paginated list of tickets with optional filters".to_string(),
                operation_id: "listTickets".to_string(),
                parameters: vec![
                    Parameter {
                        name: "status".to_string(),
                        location: "query".to_string(),
                        description: "Filter by status".to_string(),
                        required: false,
                        schema: Schema::Object {
                            schema_type: "string".to_string(),
                            format: None,
                            properties: None,
                            required: None,
                            items: None,
                            description: None,
                            enum_values: Some(vec!["open".to_string(), "in_progress".to_string(), "pending".to_string(), "resolved".to_string(), "closed".to_string()]),
                            nullable: None,
                        },
                    },
                    Parameter {
                        name: "client_id".to_string(),
                        location: "query".to_string(),
                        description: "Filter by client ID".to_string(),
                        required: false,
                        schema: Schema::Ref { reference: "#/components/schemas/UUID".to_string() },
                    },
                ],
                request_body: None,
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert("200".to_string(), Response {
                        description: "List of tickets".to_string(),
                        content: None,
                    });
                    responses
                },
                security: vec![],
            }),
            post: Some(Operation {
                tags: vec!["Tickets".to_string()],
                summary: "Create a new ticket".to_string(),
                description: "Create a new support ticket".to_string(),
                operation_id: "createTicket".to_string(),
                parameters: vec![],
                request_body: Some(RequestBody {
                    description: "Ticket data".to_string(),
                    required: true,
                    content: {
                        let mut content = HashMap::new();
                        content.insert("application/json".to_string(), MediaType {
                            schema: Schema::Ref { reference: "#/components/schemas/Ticket".to_string() },
                        });
                        content
                    },
                }),
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert("201".to_string(), Response {
                        description: "Ticket created".to_string(),
                        content: None,
                    });
                    responses
                },
                security: vec![],
            }),
            ..Default::default()
        });
    }

    fn add_asset_paths(&mut self) {
        self.paths.insert("/api/v1/assets".to_string(), PathItem {
            get: Some(Operation {
                tags: vec!["Assets".to_string()],
                summary: "List all assets".to_string(),
                description: "Retrieve a paginated list of assets".to_string(),
                operation_id: "listAssets".to_string(),
                parameters: vec![],
                request_body: None,
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert("200".to_string(), Response {
                        description: "List of assets".to_string(),
                        content: None,
                    });
                    responses
                },
                security: vec![],
            }),
            ..Default::default()
        });
    }

    fn add_invoice_paths(&mut self) {
        self.paths.insert("/api/v1/invoices".to_string(), PathItem {
            get: Some(Operation {
                tags: vec!["Invoices".to_string()],
                summary: "List all invoices".to_string(),
                description: "Retrieve a paginated list of invoices".to_string(),
                operation_id: "listInvoices".to_string(),
                parameters: vec![],
                request_body: None,
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert("200".to_string(), Response {
                        description: "List of invoices".to_string(),
                        content: None,
                    });
                    responses
                },
                security: vec![],
            }),
            ..Default::default()
        });
    }

    fn add_time_paths(&mut self) {
        self.paths.insert("/api/v1/time".to_string(), PathItem {
            get: Some(Operation {
                tags: vec!["Time".to_string()],
                summary: "List time entries".to_string(),
                description: "Retrieve time entries with optional filters".to_string(),
                operation_id: "listTimeEntries".to_string(),
                parameters: vec![],
                request_body: None,
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert("200".to_string(), Response {
                        description: "List of time entries".to_string(),
                        content: None,
                    });
                    responses
                },
                security: vec![],
            }),
            post: Some(Operation {
                tags: vec!["Time".to_string()],
                summary: "Create time entry".to_string(),
                description: "Log a new time entry".to_string(),
                operation_id: "createTimeEntry".to_string(),
                parameters: vec![],
                request_body: None,
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert("201".to_string(), Response {
                        description: "Time entry created".to_string(),
                        content: None,
                    });
                    responses
                },
                security: vec![],
            }),
            ..Default::default()
        });
    }

    fn add_analytics_paths(&mut self) {
        self.paths.insert("/api/v1/analytics/utilization".to_string(), PathItem {
            get: Some(Operation {
                tags: vec!["Analytics".to_string()],
                summary: "Get technician utilization".to_string(),
                description: "Retrieve utilization metrics for technicians".to_string(),
                operation_id: "getUtilization".to_string(),
                parameters: vec![
                    Parameter {
                        name: "start_date".to_string(),
                        location: "query".to_string(),
                        description: "Start date for the report".to_string(),
                        required: true,
                        schema: Schema::Object {
                            schema_type: "string".to_string(),
                            format: Some("date".to_string()),
                            properties: None,
                            required: None,
                            items: None,
                            description: None,
                            enum_values: None,
                            nullable: None,
                        },
                    },
                    Parameter {
                        name: "end_date".to_string(),
                        location: "query".to_string(),
                        description: "End date for the report".to_string(),
                        required: true,
                        schema: Schema::Object {
                            schema_type: "string".to_string(),
                            format: Some("date".to_string()),
                            properties: None,
                            required: None,
                            items: None,
                            description: None,
                            enum_values: None,
                            nullable: None,
                        },
                    },
                ],
                request_body: None,
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert("200".to_string(), Response {
                        description: "Utilization metrics".to_string(),
                        content: None,
                    });
                    responses
                },
                security: vec![],
            }),
            ..Default::default()
        });

        self.paths.insert("/api/v1/analytics/profitability".to_string(), PathItem {
            get: Some(Operation {
                tags: vec!["Analytics".to_string()],
                summary: "Get client profitability".to_string(),
                description: "Retrieve profitability metrics by client".to_string(),
                operation_id: "getProfitability".to_string(),
                parameters: vec![],
                request_body: None,
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert("200".to_string(), Response {
                        description: "Profitability metrics".to_string(),
                        content: None,
                    });
                    responses
                },
                security: vec![],
            }),
            ..Default::default()
        });
    }

    fn add_system_paths(&mut self) {
        self.paths.insert("/health".to_string(), PathItem {
            get: Some(Operation {
                tags: vec!["System".to_string()],
                summary: "Health check".to_string(),
                description: "Basic health check endpoint".to_string(),
                operation_id: "healthCheck".to_string(),
                parameters: vec![],
                request_body: None,
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert("200".to_string(), Response {
                        description: "Service is healthy".to_string(),
                        content: None,
                    });
                    responses
                },
                security: vec![],
            }),
            ..Default::default()
        });

        self.paths.insert("/health/detailed".to_string(), PathItem {
            get: Some(Operation {
                tags: vec!["System".to_string()],
                summary: "Detailed health check".to_string(),
                description: "Detailed health check with service status".to_string(),
                operation_id: "detailedHealthCheck".to_string(),
                parameters: vec![],
                request_body: None,
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert("200".to_string(), Response {
                        description: "All services healthy".to_string(),
                        content: None,
                    });
                    responses.insert("503".to_string(), Response {
                        description: "One or more services unhealthy".to_string(),
                        content: None,
                    });
                    responses
                },
                security: vec![],
            }),
            ..Default::default()
        });

        self.paths.insert("/metrics".to_string(), PathItem {
            get: Some(Operation {
                tags: vec!["System".to_string()],
                summary: "Get system metrics".to_string(),
                description: "Retrieve system performance metrics".to_string(),
                operation_id: "getMetrics".to_string(),
                parameters: vec![],
                request_body: None,
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert("200".to_string(), Response {
                        description: "System metrics".to_string(),
                        content: None,
                    });
                    responses
                },
                security: vec![],
            }),
            ..Default::default()
        });
    }
}

/// Handler to serve the OpenAPI specification
pub async fn openapi_spec_handler() -> axum::Json<OpenApiSpec> {
    axum::Json(OpenApiSpec::generate())
}

/// Handler to serve Swagger UI HTML
pub async fn swagger_ui_handler() -> axum::response::Html<String> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Resolve MSP API Documentation</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui.css">
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui-bundle.js"></script>
    <script>
        window.onload = function() {
            SwaggerUIBundle({
                url: "/api/v1/docs/openapi.json",
                dom_id: '#swagger-ui',
                presets: [SwaggerUIBundle.presets.apis, SwaggerUIBundle.SwaggerUIStandalonePreset],
                layout: "BaseLayout"
            });
        }
    </script>
</body>
</html>
"#;
    axum::response::Html(html.to_string())
}

/// Create routes for OpenAPI documentation
pub fn openapi_routes() -> axum::Router<std::sync::Arc<crate::AppState>> {
    use axum::routing::get;

    axum::Router::new()
        .route("/", get(swagger_ui_handler))
        .route("/openapi.json", get(openapi_spec_handler))
}
