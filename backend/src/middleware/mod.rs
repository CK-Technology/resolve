pub mod observability;

pub use observability::{
    observability_layer,
    detailed_health_check,
    metrics_endpoint,
    HealthCheckResponse,
    ServiceStatus,
    MetricsResponse,
};
