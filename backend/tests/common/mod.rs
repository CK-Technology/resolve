// Common test utilities that are shared across integration tests
use std::sync::Once;
use tracing_subscriber;

static INIT: Once = Once::new();

pub fn init_test_logging() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter("debug")
            .try_init()
            .ok();
    });
}