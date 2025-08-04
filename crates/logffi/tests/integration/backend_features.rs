//! Tests for backend feature combinations

use logffi::{debug, error, info, logger, trace, warn};

#[test]
fn test_logger_initialization() {
    let logger = logger();
    let available = logger.available_backends();

    // At least one backend should be available
    assert!(
        !available.is_empty(),
        "At least one backend should be enabled"
    );

    println!("Available backends: {:?}", available);
}

#[test]
fn test_basic_logging_macros() {
    // These should work regardless of which backends are enabled
    error!("Test error message");
    warn!("Test warning message");
    info!("Test info message");
    debug!("Test debug message");
    trace!("Test trace message");
}

#[test]
fn test_targeted_logging() {
    // Test with custom targets
    error!(target: "custom_target", "Error with custom target");
    info!(target: "my_module", "Info with module target");
}

#[cfg(feature = "log")]
#[test]
fn test_log_backend_access() {
    let logger = logger();

    // Should be able to access log backend when enabled
    let log_backend = logger.as_log();
    assert!(
        log_backend.is_some(),
        "Log backend should be available when feature is enabled"
    );
}

#[cfg(feature = "tracing")]
#[test]
fn test_tracing_backend_access() {
    let logger = logger();

    // Should be able to access tracing backend when enabled
    let tracing_backend = logger.as_tracing();
    assert!(
        tracing_backend.is_some(),
        "Tracing backend should be available when feature is enabled"
    );
}

#[cfg(feature = "slog")]
#[test]
fn test_slog_backend_access() {
    let logger = logger();

    // Should be able to access slog backend when enabled
    let slog_backend = logger.as_slog();
    assert!(
        slog_backend.is_some(),
        "Slog backend should be available when feature is enabled"
    );

    // Test that we can access the slog logger
    if let Some(slog) = slog_backend {
        let slog_logger = slog.logger();

        // Test direct slog usage
        slog::info!(slog_logger, "Direct slog usage test");
    }
}

#[cfg(feature = "callback")]
#[test]
fn test_callback_backend_access() {
    use logffi::{call_callback, set_callback};
    use std::sync::{Arc, Mutex};

    let logger = logger();

    // Should be able to access callback backend when enabled
    let callback_backend = logger.as_callback();
    assert!(
        callback_backend.is_some(),
        "Callback backend should be available when feature is enabled"
    );

    // Test callback functionality
    let messages = Arc::new(Mutex::new(Vec::new()));
    let messages_clone = messages.clone();

    set_callback(Box::new(move |level, target, message| {
        messages_clone
            .lock()
            .unwrap()
            .push(format!("[{}] {}: {}", level, target, message));
    }));

    // Test direct callback
    call_callback("info", "test_module", "Test callback message");

    // Verify callback was called
    let captured = messages.lock().unwrap();
    assert_eq!(captured.len(), 1);
    assert!(captured[0].contains("Test callback message"));
}

#[cfg(all(feature = "tracing", feature = "log"))]
#[test]
fn test_multiple_backends() {
    let logger = logger();

    // Both backends should be available
    assert!(logger.as_tracing().is_some(), "Tracing should be available");
    assert!(logger.as_log().is_some(), "Log should be available");

    // Test that logging goes to both backends
    info!("Message that should go to both tracing and log");
}

#[test]
fn test_backend_names() {
    use logffi::Backend;

    #[cfg(feature = "log")]
    assert_eq!(Backend::Log.name(), "log");

    #[cfg(feature = "tracing")]
    assert_eq!(Backend::Tracing.name(), "tracing");

    #[cfg(feature = "slog")]
    assert_eq!(Backend::Slog.name(), "slog");

    #[cfg(feature = "callback")]
    assert_eq!(Backend::Callback.name(), "callback");
}

#[test]
fn test_available_backends_matches_features() {
    let logger = logger();
    let available = logger.available_backends();

    // Check that available backends match enabled features
    #[cfg(feature = "log")]
    {
        use logffi::Backend;
        assert!(
            available.contains(&Backend::Log),
            "Log backend should be in available list"
        );
    }

    #[cfg(feature = "tracing")]
    {
        use logffi::Backend;
        assert!(
            available.contains(&Backend::Tracing),
            "Tracing backend should be in available list"
        );
    }

    #[cfg(feature = "slog")]
    {
        use logffi::Backend;
        assert!(
            available.contains(&Backend::Slog),
            "Slog backend should be in available list"
        );
    }

    #[cfg(feature = "callback")]
    {
        use logffi::Backend;
        assert!(
            available.contains(&Backend::Callback),
            "Callback backend should be in available list"
        );
    }
}
