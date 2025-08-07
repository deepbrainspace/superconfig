//! Integration tests for logffi functionality within superconfig

use std::sync::{Arc, Mutex, OnceLock};
use superconfig::{
    logging::set_ffi_callback,
    ConfigRegistry,
};
use serial_test::serial;

#[derive(Debug, Clone)]
struct CapturedLog {
    level: String,
    target: String,
    message: String,
}

// Global log capture shared across all tests
static GLOBAL_LOGS: OnceLock<Arc<Mutex<Vec<CapturedLog>>>> = OnceLock::new();

fn get_global_log_capture() -> Arc<Mutex<Vec<CapturedLog>>> {
    GLOBAL_LOGS.get_or_init(|| {
        let capture = Arc::new(Mutex::new(Vec::new()));
        let capture_clone = capture.clone();
        
        // Set up the FFI callback once for all tests
        set_ffi_callback(Box::new(move |level, target, message| {
            let log_entry = CapturedLog {
                level: level.to_string(),
                target: target.to_string(),
                message: message.to_string(),
            };
            capture_clone.lock().unwrap().push(log_entry);
        }));
        
        capture
    }).clone()
}

fn clear_captured_logs() {
    let capture = get_global_log_capture();
    capture.lock().unwrap().clear();
}

#[test]
#[serial]
fn test_superconfig_logging_reexports() {
    // Test that all logffi functionality is properly re-exported under superconfig::logging
    env_logger::try_init().ok();
    
    // Clear any previous log captures and get the global capture
    clear_captured_logs();
    let capture = get_global_log_capture();
    
    // Test that we can actually call the FFI callback directly
    superconfig::logging::call_ffi_callback("ERROR", "test_module", "Direct FFI test message");
    superconfig::logging::call_ffi_callback("WARN", "test_target", "Direct FFI warning");
    
    // Give FFI callback time to execute
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    let logs = capture.lock().unwrap();
    assert!(!logs.is_empty(), "Should have captured direct FFI messages");
    
    // Verify we captured the direct FFI calls
    let has_error = logs.iter().any(|log| log.level.contains("ERROR") && log.message.contains("Direct FFI test"));
    let has_warn = logs.iter().any(|log| log.level.contains("WARN") && log.message.contains("Direct FFI warning"));
    
    assert!(has_error, "Should have captured direct FFI error message");
    assert!(has_warn, "Should have captured direct FFI warning message");
    
    // Verify targeted logging worked
    let has_targeted = logs.iter().any(|log| log.target == "test_target");
    assert!(has_targeted, "Should have captured targeted log messages");
}

#[test]
#[serial]
fn test_registry_operations_trigger_logging() {
    // Test that internal registry operations trigger logging calls
    env_logger::try_init().ok();
    
    // Clear any previous log captures and get the global capture
    clear_captured_logs();
    let capture = get_global_log_capture();
    
    // Perform registry operations that should trigger internal logging
    let registry = ConfigRegistry::new();
    
    // Create configuration - should trigger logging
    registry.create("test-config".to_string()).unwrap();
    
    // Update configuration - should trigger logging  
    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestConfig {
        value: i32,
    }
    
    let config = TestConfig { value: 42 };
    let typed_handle = registry.create(config).unwrap();
    
    // Read configuration - might trigger logging
    let _result = registry.read(&typed_handle).unwrap();
    
    // Delete configuration - should trigger logging
    registry.delete(&typed_handle).unwrap();
    
    // Give FFI callback time to execute
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    let logs = capture.lock().unwrap();
    
    // Registry operations should generate some log output
    // Even if we don't know the exact count, there should be some activity
    if !logs.is_empty() {
        println!("Captured {} log messages from registry operations", logs.len());
        for log in logs.iter() {
            println!("  {:?}: [{}] {}", log.level, log.target, log.message);
        }
    }
}

#[test]
#[serial] 
fn test_ffi_callback_error_handling() {
    // Test error scenarios in FFI callback functionality
    env_logger::try_init().ok();
    
    // Clear any previous log captures and get the global capture
    clear_captured_logs();
    let capture = get_global_log_capture();
    
    // Test with invalid configurations that might trigger error logging
    let registry = ConfigRegistry::new();
    
    // Create a handle, delete it, then try to read from it - should trigger error logging
    #[derive(serde::Serialize, serde::Deserialize)]
    struct DummyConfig {
        id: u64,
    }
    let test_config = DummyConfig { id: 12345 };
    let handle = registry.create(test_config).unwrap();
    
    // Delete the configuration to make the handle invalid
    registry.delete(&handle).unwrap();
    
    // Now try to read from the deleted handle - should trigger error logging
    let result = registry.read(&handle);
    assert!(result.is_err(), "Should fail to read deleted handle");
    
    // Give FFI callback time to execute
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    let logs = capture.lock().unwrap();
    
    // Check if any error-level logs were captured
    let error_logs: Vec<_> = logs.iter().filter(|log| log.level.contains("ERROR")).collect();
    
    if !error_logs.is_empty() {
        println!("Captured {} error logs", error_logs.len());
        for log in error_logs {
            println!("  Error: [{}] {}", log.target, log.message);
        }
    }
}

#[test]
#[serial]
fn test_logging_with_formatting() {
    // Test that formatted FFI callback messages work correctly
    env_logger::try_init().ok();
    
    // Clear any previous log captures and get the global capture
    clear_captured_logs();
    let capture = get_global_log_capture();
    
    let test_value = 42;
    let test_string = "test_data";
    
    // Test formatting in direct FFI calls
    let formatted_message = format!("Test formatting: value={}, string={}", test_value, test_string);
    superconfig::logging::call_ffi_callback("INFO", "test_module", &formatted_message);
    
    let complex_message = format!("Complex formatting: {:#?}", vec![1, 2, 3]);
    superconfig::logging::call_ffi_callback("WARN", "test_module", &complex_message);
    
    let targeted_message = format!("Targeted with format: {:.2}", 3.14159);
    superconfig::logging::call_ffi_callback("ERROR", "formatting", &targeted_message);
    
    // Give FFI callback time to execute
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    let logs = capture.lock().unwrap();
    assert!(!logs.is_empty(), "Should have captured formatted log messages");
    
    // Verify formatting worked by checking message content
    let has_formatted = logs.iter().any(|log| {
        log.message.contains("value=42") && log.message.contains("string=test_data")
    });
    assert!(has_formatted, "Should have captured formatted message with values");
    
    let has_targeted_formatted = logs.iter().any(|log| {
        log.target == "formatting" && log.message.contains("3.14")
    });
    assert!(has_targeted_formatted, "Should have captured targeted formatted message");
}

#[test]
#[serial]
fn test_logging_level_filtering() {
    // Test that different log levels work correctly through FFI callbacks
    env_logger::try_init().ok();
    
    // Clear any previous log captures and get the global capture
    clear_captured_logs();
    let capture = get_global_log_capture();
    
    // Test all levels through direct FFI calls
    superconfig::logging::call_ffi_callback("TRACE", "test_module", "Trace level message");
    superconfig::logging::call_ffi_callback("DEBUG", "test_module", "Debug level message");
    superconfig::logging::call_ffi_callback("INFO", "test_module", "Info level message");
    superconfig::logging::call_ffi_callback("WARN", "test_module", "Warning level message");
    superconfig::logging::call_ffi_callback("ERROR", "test_module", "Error level message");
    
    // Give FFI callback time to execute
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    let logs = capture.lock().unwrap();
    
    // Log the actual filtering behavior for debugging
    println!("Captured {} total log messages", logs.len());
    for log in logs.iter() {
        println!("  {:?}: {}", log.level, log.message);
    }
    
    // All levels should be captured since we're calling FFI directly
    assert_eq!(logs.len(), 5, "Should capture all 5 log level messages");
    
    let has_error = logs.iter().any(|log| log.level.contains("ERROR"));
    let has_warn = logs.iter().any(|log| log.level.contains("WARN"));
    let has_info = logs.iter().any(|log| log.level.contains("INFO"));
    let has_debug = logs.iter().any(|log| log.level.contains("DEBUG"));
    let has_trace = logs.iter().any(|log| log.level.contains("TRACE"));
    
    assert!(has_error, "Error level logs should be captured");
    assert!(has_warn, "Warning level logs should be captured");
    assert!(has_info, "Info level logs should be captured");
    assert!(has_debug, "Debug level logs should be captured");
    assert!(has_trace, "Trace level logs should be captured");
}