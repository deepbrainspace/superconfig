//! Tests for callback functionality
//!
//! Verifies that logging can be bridged to external systems via callbacks

#![cfg(feature = "callback")]

use logfusion::{error, info, set_callback};
use std::sync::{Arc, Mutex};

#[test]
fn callback_receives_log_messages() {
    use std::sync::atomic::{AtomicBool, Ordering};

    // Use a flag to identify our messages
    static TEST_FLAG: AtomicBool = AtomicBool::new(false);
    TEST_FLAG.store(true, Ordering::SeqCst);

    let captured = Arc::new(Mutex::new(Vec::new()));
    let captured_clone = captured.clone();

    set_callback(Box::new(move |level, target, message| {
        // Only capture messages from this test
        if TEST_FLAG.load(Ordering::SeqCst)
            && (message.contains("Callback test message")
                || message.contains("Callback error message"))
        {
            captured_clone.lock().unwrap().push((
                level.to_string(),
                target.to_string(),
                message.to_string(),
            ));
        }
    }));

    // These should trigger the callback
    info!("Callback test message");
    error!("Callback error message");

    // Give a moment for any async operations
    std::thread::sleep(std::time::Duration::from_millis(50));

    TEST_FLAG.store(false, Ordering::SeqCst);

    let logs = captured.lock().unwrap();
    assert_eq!(logs.len(), 2, "Expected 2 log entries, got {}", logs.len());

    // Check messages contain what we expect
    assert!(
        logs.iter()
            .any(|log| log.0 == "info" && log.2.contains("Callback test message"))
    );
    assert!(
        logs.iter()
            .any(|log| log.0 == "error" && log.2.contains("Callback error message"))
    );
}
