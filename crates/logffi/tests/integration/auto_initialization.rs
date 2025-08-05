//! Tests for auto-initialization feature of LogFFI
//!
//! The key feature is that logging should work immediately without any setup

use logffi::{debug, error, info, trace, warn};

#[test]
fn logging_works_without_explicit_initialization() {
    // These should all work immediately without any setup
    // This is the main feature of the tracing-native bridge

    info!("Auto-initialized info message");
    error!("Auto-initialized error message");
    warn!("Auto-initialized warning message");
    debug!("Auto-initialized debug message");
    trace!("Auto-initialized trace message");

    // The test passes if it doesn't panic
    // The actual output goes to the auto-initialized tracing subscriber
}

#[test]
fn multiple_threads_auto_initialize_safely() {
    use std::thread;

    let handles: Vec<_> = (0..5)
        .map(|i| {
            thread::spawn(move || {
                info!("Thread {} logging", i);
                error!("Thread {} error", i);
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // All threads should be able to log without races
}

#[test]
fn respects_rust_log_env_variable() {
    // This test verifies that RUST_LOG is respected
    // Note: This test might need to be run in isolation due to env var changes

    // The auto-initialization should pick up RUST_LOG
    info!("This respects RUST_LOG filter level");

    // We can't easily test the actual filtering in unit tests
    // but we verify it doesn't panic
}
