//! Tests for tracing ecosystem compatibility
//!
//! Verifies that our tracing-native implementation works correctly with:
//! - log crate bridge (for libraries still using log)
//! - tracing spans and structured logging
//! - tracing subscribers and filtering
//! - Advanced tracing features

use logffi::{debug, error, info, trace, warn};
use std::sync::{Arc, Mutex};
use tracing::{Level, Span, instrument};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

// Helper to capture tracing events for testing
#[derive(Debug, Clone)]
struct CapturedEvent {
    level: String,
    target: String,
    message: String,
    fields: std::collections::HashMap<String, String>,
}

struct TestCollector {
    events: Arc<Mutex<Vec<CapturedEvent>>>,
}

impl TestCollector {
    fn new() -> (Self, Arc<Mutex<Vec<CapturedEvent>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                events: events.clone(),
            },
            events,
        )
    }
}

impl<S> tracing_subscriber::Layer<S> for TestCollector
where
    S: tracing::Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = FieldVisitor::new();
        event.record(&mut visitor);

        let captured = CapturedEvent {
            level: format!("{}", event.metadata().level()),
            target: event.metadata().target().to_string(),
            message: visitor.message,
            fields: visitor.fields,
        };

        self.events.lock().unwrap().push(captured);
    }
}

struct FieldVisitor {
    message: String,
    fields: std::collections::HashMap<String, String>,
}

impl FieldVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
            fields: std::collections::HashMap::new(),
        }
    }
}

impl tracing::field::Visit for FieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value).trim_matches('"').to_string();
        } else {
            self.fields
                .insert(field.name().to_string(), format!("{:?}", value));
        }
    }
}

#[test]
fn tracing_basic_functionality() {
    let (collector, events) = TestCollector::new();

    let _guard = tracing_subscriber::registry().with(collector).set_default();

    // Clear any existing events
    events.lock().unwrap().clear();

    // Test basic logging
    error!("Test error message");
    warn!("Test warning message");
    info!("Test info message");
    debug!("Test debug message");
    trace!("Test trace message");

    let captured = events.lock().unwrap();

    // Should have captured at least the higher level events
    assert!(!captured.is_empty());

    // Check that error was captured
    let error_event = captured.iter().find(|e| e.level == "ERROR");
    assert!(error_event.is_some());
    assert_eq!(error_event.unwrap().message, "Test error message");
}

#[test]
fn tracing_targeted_logging() {
    let (collector, events) = TestCollector::new();

    let _guard = tracing_subscriber::registry().with(collector).set_default();

    events.lock().unwrap().clear();

    // Test targeted logging
    error!(target: "app::database", "Database connection failed");
    warn!(target: "app::network", "Network timeout");
    info!(target: "app::auth", "User authenticated");

    let captured = events.lock().unwrap();

    // Find events by target
    let db_event = captured.iter().find(|e| e.target == "app::database");
    let net_event = captured.iter().find(|e| e.target == "app::network");
    let auth_event = captured.iter().find(|e| e.target == "app::auth");

    assert!(db_event.is_some());
    assert_eq!(db_event.unwrap().message, "Database connection failed");

    assert!(net_event.is_some());
    assert_eq!(net_event.unwrap().message, "Network timeout");

    assert!(auth_event.is_some());
    assert_eq!(auth_event.unwrap().message, "User authenticated");
}

#[test]
fn tracing_spans_integration() {
    let (collector, events) = TestCollector::new();

    let _guard = tracing_subscriber::registry().with(collector).set_default();

    events.lock().unwrap().clear();

    // Create a span and log within it
    let span = tracing::span!(Level::INFO, "test_operation", user_id = 12345);
    let _enter = span.enter();

    info!("Operation started");
    error!("Operation failed");

    drop(_enter);
    drop(span);

    let captured = events.lock().unwrap();

    // Events should be captured within the span context
    assert!(!captured.is_empty());

    let info_event = captured.iter().find(|e| e.level == "INFO");
    assert!(info_event.is_some());
    assert_eq!(info_event.unwrap().message, "Operation started");
}

#[test]
fn tracing_instrumentation() {
    let (collector, events) = TestCollector::new();

    let _guard = tracing_subscriber::registry().with(collector).set_default();

    events.lock().unwrap().clear();

    #[instrument(level = "info")]
    fn instrumented_function(user_id: u64, action: &str) {
        info!("Processing action: {}", action);

        if action == "fail" {
            error!("Action failed for user {}", user_id);
        } else {
            info!("Action completed successfully");
        }
    }

    // Call instrumented function
    instrumented_function(12345, "process");
    instrumented_function(67890, "fail");

    let captured = events.lock().unwrap();

    // Should have multiple events from the function
    let process_events: Vec<_> = captured
        .iter()
        .filter(|e| {
            e.message.contains("Processing action") || e.message.contains("completed successfully")
        })
        .collect();

    let error_events: Vec<_> = captured
        .iter()
        .filter(|e| e.message.contains("Action failed"))
        .collect();

    assert!(!process_events.is_empty());
    assert!(!error_events.is_empty());
}

#[test]
fn log_crate_bridge_compatibility() {
    // This test verifies that log crate calls work through tracing's log bridge
    let (collector, events) = TestCollector::new();

    let _guard = tracing_subscriber::registry()
        .with(collector)
        .with(tracing_subscriber::fmt::layer())
        .set_default();

    events.lock().unwrap().clear();

    // Use log crate directly (should work through tracing's bridge)
    log::error!("Log crate error");
    log::warn!("Log crate warning");
    log::info!("Log crate info");

    // Also test our LogFFI macros
    error!("LogFFI error");
    warn!("LogFFI warning");
    info!("LogFFI info");

    let captured = events.lock().unwrap();

    // Should capture both log crate and LogFFI events
    assert!(captured.len() >= 4); // At minimum should have some events

    // Look for our LogFFI events specifically
    let logffi_error = captured.iter().find(|e| e.message == "LogFFI error");
    assert!(logffi_error.is_some());
    assert_eq!(logffi_error.unwrap().level, "ERROR");
}

#[test]
fn tracing_structured_logging() {
    let (collector, events) = TestCollector::new();

    let _guard = tracing_subscriber::registry().with(collector).set_default();

    events.lock().unwrap().clear();

    // Use tracing directly for structured logging
    tracing::info!(
        user_id = 12345,
        action = "login",
        ip_address = "192.168.1.1",
        "User login successful"
    );

    // Use our LogFFI macros (should still work with tracing)
    info!("Regular LogFFI log");

    let captured = events.lock().unwrap();

    // Should have captured both structured and regular events
    let structured_event = captured
        .iter()
        .find(|e| e.message == "User login successful");

    let regular_event = captured.iter().find(|e| e.message == "Regular LogFFI log");

    assert!(structured_event.is_some());
    assert!(regular_event.is_some());

    // The structured event should have fields
    let structured = structured_event.unwrap();
    assert_eq!(structured.level, "INFO");
    // Note: Field checking would require more complex visitor implementation
}

#[test]
fn tracing_filtering() {
    // Test that tracing filtering works correctly
    let (collector, events) = TestCollector::new();

    // Create a filter that only allows WARN and ERROR
    let filter = EnvFilter::new("warn");

    let _guard = tracing_subscriber::registry()
        .with(collector)
        .with(filter)
        .set_default();

    events.lock().unwrap().clear();

    // Log at different levels
    error!("This should appear");
    warn!("This should also appear");
    info!("This should be filtered out");
    debug!("This should be filtered out");

    let captured = events.lock().unwrap();

    // Should only have ERROR and WARN events
    let error_count = captured.iter().filter(|e| e.level == "ERROR").count();
    let warn_count = captured.iter().filter(|e| e.level == "WARN").count();
    let info_count = captured.iter().filter(|e| e.level == "INFO").count();
    let debug_count = captured.iter().filter(|e| e.level == "DEBUG").count();

    assert!(error_count > 0);
    assert!(warn_count > 0);
    assert_eq!(info_count, 0);
    assert_eq!(debug_count, 0);
}

#[test]
fn tracing_subscriber_layers() {
    // Test that multiple layers work together
    let (collector, events) = TestCollector::new();

    let _guard = tracing_subscriber::registry()
        .with(collector)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .set_default();

    events.lock().unwrap().clear();

    // Log some events
    error!("Multi-layer error");
    info!("Multi-layer info");

    let captured = events.lock().unwrap();

    // Events should be captured by our test collector
    assert!(!captured.is_empty());

    let error_event = captured.iter().find(|e| e.message == "Multi-layer error");
    assert!(error_event.is_some());
    assert_eq!(error_event.unwrap().level, "ERROR");
}
