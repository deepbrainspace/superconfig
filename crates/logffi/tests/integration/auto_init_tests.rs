//! Tests for auto-initialization functionality

use logffi::{info, warn, error, debug, trace};

#[test]
fn test_basic_logging_macros_work() {
    // These should work immediately without any setup due to auto-initialization
    error!("Test error message");
    warn!("Test warning message");
    info!("Test info message");
    debug!("Test debug message");
    trace!("Test trace message");
    
    // Test passes if no panics occur
}

#[test]
fn test_targeted_logging() {
    // Test with custom targets
    error!(target: "custom_target", "Error with custom target");
    info!(target: "my_module", "Info with module target");
    warn!(target: "test::integration", "Warning from test");
}

#[test]
fn test_formatted_logging() {
    let user_id = 42;
    let operation = "database_query";
    
    info!("User {} performed operation: {}", user_id, operation);
    error!(target: "db", "Query failed for user {} in operation {}", user_id, operation);
}

#[test]
fn test_tracing_level_import() {
    // Verify we can import tracing types directly from logffi
    use logffi::Level;
    
    let level = Level::INFO;
    assert_eq!(level.as_str(), "INFO");
}

#[test]
fn test_auto_init_idempotent() {
    // This should be safe to call multiple times
    logffi::auto_init();
    logffi::auto_init();
    logffi::auto_init();
    
    // Should still work after multiple init calls
    info!("Logging works after multiple init calls");
}