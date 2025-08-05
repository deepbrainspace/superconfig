//! Tests for callback functionality (only available with callback feature)

use logffi::{set_callback, info, error, warn};
use std::sync::{Arc, Mutex};

#[test]
fn test_callback_basic_functionality() {
    let messages = Arc::new(Mutex::new(Vec::new()));
    let messages_clone = messages.clone();

    // Set up callback to capture messages
    set_callback(Box::new(move |level, target, message| {
        messages_clone
            .lock()
            .unwrap()
            .push(format!("[{}] {}: {}", level, target, message));
    }));

    // Generate some log messages
    info!("Test info message");
    error!(target: "test_module", "Test error message");
    warn!("Test warning with value: {}", 42);

    // Give a moment for callback to be called
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Verify callback received messages
    let captured = messages.lock().unwrap();
    assert!(captured.len() >= 3, "Should have captured at least 3 messages");
    
    // Check that messages contain expected content
    let all_messages = captured.join("\n");
    assert!(all_messages.contains("Test info message"));
    assert!(all_messages.contains("Test error message"));
    assert!(all_messages.contains("Test warning with value: 42"));
}

#[test] 
fn test_callback_with_different_levels() {
    let messages = Arc::new(Mutex::new(Vec::new()));
    let messages_clone = messages.clone();

    set_callback(Box::new(move |level, _target, message| {
        messages_clone
            .lock()
            .unwrap()
            .push(format!("{}: {}", level, message));
    }));

    // Test different log levels
    error!("Error level test");
    warn!("Warn level test");
    info!("Info level test");

    std::thread::sleep(std::time::Duration::from_millis(10));

    let captured = messages.lock().unwrap();
    let all_messages = captured.join("\n");
    
    assert!(all_messages.contains("error: Error level test"));
    assert!(all_messages.contains("warn: Warn level test"));
    assert!(all_messages.contains("info: Info level test"));
}

#[test]
fn test_callback_replacement() {
    let messages1 = Arc::new(Mutex::new(Vec::new()));
    let messages2 = Arc::new(Mutex::new(Vec::new()));
    
    let messages1_clone = messages1.clone();
    let messages2_clone = messages2.clone();

    // Set first callback
    set_callback(Box::new(move |_level, _target, message| {
        messages1_clone
            .lock()
            .unwrap()
            .push(format!("CALLBACK1: {}", message));
    }));

    info!("Message for callback 1");
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Replace with second callback
    set_callback(Box::new(move |_level, _target, message| {
        messages2_clone
            .lock()
            .unwrap()
            .push(format!("CALLBACK2: {}", message));
    }));

    info!("Message for callback 2");
    std::thread::sleep(std::time::Duration::from_millis(10));

    // First callback should have received first message
    let captured1 = messages1.lock().unwrap();
    assert!(captured1.iter().any(|msg| msg.contains("Message for callback 1")));
    
    // Second callback should have received second message
    let captured2 = messages2.lock().unwrap();
    assert!(captured2.iter().any(|msg| msg.contains("Message for callback 2")));
}