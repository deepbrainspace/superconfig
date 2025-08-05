//! Tests for logging macro functionality
//!
//! Verifies that all logging macros work correctly

use logffi::{debug, error, info, trace, warn};

#[test]
fn basic_logging_macros_work() {
    // These should all work without panicking
    trace!("Trace message");
    debug!("Debug message");
    info!("Info message");
    warn!("Warning message");
    error!("Error message");
}

#[test]
fn logging_with_formatting() {
    let user = "alice";
    let count = 42;

    info!("User {} logged in", user);
    warn!("User {} has {} failed attempts", user, count);
    error!("Failed to process request for user: {}", user);
}

#[test]
fn logging_with_target() {
    info!(target: "app::auth", "Authentication successful");
    error!(target: "app::db", "Database connection failed");
}
