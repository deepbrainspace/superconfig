//! Tests for LogFusion's structured logging syntax
//!
//! These tests verify that LogFusion macros support structured field syntax
//! and that the re-exported tracing-subscriber components work correctly.

use logfusion::{EnvFilter, SubscriberExt, SubscriberInitExt, fmt, registry};
use logfusion::{debug, error, info, trace, warn};
use std::sync::{Arc, Mutex};

// Helper to capture structured events
#[derive(Debug, Clone)]
struct StructuredEvent {
    level: String,
    target: String,
    message: String,
    fields: std::collections::HashMap<String, String>,
}

struct StructuredCollector {
    events: Arc<Mutex<Vec<StructuredEvent>>>,
}

impl StructuredCollector {
    fn new() -> (Self, Arc<Mutex<Vec<StructuredEvent>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                events: events.clone(),
            },
            events,
        )
    }
}

impl<S> tracing_subscriber::Layer<S> for StructuredCollector
where
    S: tracing::Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = StructuredFieldVisitor::new();
        event.record(&mut visitor);

        let captured = StructuredEvent {
            level: format!("{}", event.metadata().level()),
            target: event.metadata().target().to_string(),
            message: visitor.message,
            fields: visitor.fields,
        };

        self.events.lock().unwrap().push(captured);
    }
}

struct StructuredFieldVisitor {
    message: String,
    fields: std::collections::HashMap<String, String>,
}

impl StructuredFieldVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
            fields: std::collections::HashMap::new(),
        }
    }
}

impl tracing::field::Visit for StructuredFieldVisitor {
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
fn logfusion_structured_logging_basic() {
    let (collector, events) = StructuredCollector::new();
    let _guard = registry().with(collector).set_default();
    events.lock().unwrap().clear();

    // Test LogFusion structured syntax
    info!(
        user_id = 12345,
        action = "login",
        ip_address = "192.168.1.1",
        "User authentication successful"
    );

    let captured = events.lock().unwrap();
    let event = captured
        .iter()
        .find(|e| e.message == "User authentication successful");

    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.level, "INFO");

    // Verify structured fields were captured
    assert!(event.fields.contains_key("user_id"));
    assert_eq!(event.fields.get("user_id"), Some(&"12345".to_string()));

    assert!(event.fields.contains_key("action"));
    assert_eq!(event.fields.get("action"), Some(&"\"login\"".to_string()));

    assert!(event.fields.contains_key("ip_address"));
    assert_eq!(
        event.fields.get("ip_address"),
        Some(&"\"192.168.1.1\"".to_string())
    );
}

#[test]
fn logfusion_structured_logging_all_levels() {
    let (collector, events) = StructuredCollector::new();
    let _guard = registry().with(collector).set_default();
    events.lock().unwrap().clear();

    // Test structured logging at all levels
    error!(
        error_code = 500,
        module = "database",
        "Database connection failed"
    );

    warn!(threshold = 85, current = 92, "Memory usage high");

    info!(user_id = 67890, session_id = "sess_123", "Session created");

    debug!(query_time_ms = 45, table = "users", "Query executed");

    trace!(
        trace_id = "abc-123",
        span_id = "def-456",
        "Trace point reached"
    );

    let captured = events.lock().unwrap();

    // Verify error event
    let error_event = captured
        .iter()
        .find(|e| e.level == "ERROR" && e.message == "Database connection failed");
    assert!(error_event.is_some());
    let error_event = error_event.unwrap();
    assert_eq!(
        error_event.fields.get("error_code"),
        Some(&"500".to_string())
    );
    assert_eq!(
        error_event.fields.get("module"),
        Some(&"\"database\"".to_string())
    );

    // Verify warn event
    let warn_event = captured
        .iter()
        .find(|e| e.level == "WARN" && e.message == "Memory usage high");
    assert!(warn_event.is_some());
    let warn_event = warn_event.unwrap();
    assert_eq!(warn_event.fields.get("threshold"), Some(&"85".to_string()));
    assert_eq!(warn_event.fields.get("current"), Some(&"92".to_string()));

    // Verify info event
    let info_event = captured
        .iter()
        .find(|e| e.level == "INFO" && e.message == "Session created");
    assert!(info_event.is_some());
    let info_event = info_event.unwrap();
    assert_eq!(info_event.fields.get("user_id"), Some(&"67890".to_string()));
    assert_eq!(
        info_event.fields.get("session_id"),
        Some(&"\"sess_123\"".to_string())
    );
}

#[test]
fn logfusion_structured_logging_with_targets() {
    let (collector, events) = StructuredCollector::new();
    let _guard = registry().with(collector).set_default();
    events.lock().unwrap().clear();

    // Test structured logging with custom targets
    error!(
        target: "app::payment",
        transaction_id = "txn_789",
        amount_cents = 2999,
        decline_reason = "insufficient_funds",
        "Payment declined"
    );

    info!(
        target: "app::auth",
        user_id = 11111,
        method = "oauth",
        provider = "google",
        "Authentication successful"
    );

    let captured = events.lock().unwrap();

    // Verify payment event
    let payment_event = captured.iter().find(|e| e.target == "app::payment");
    assert!(payment_event.is_some());
    let payment_event = payment_event.unwrap();
    assert_eq!(payment_event.message, "Payment declined");
    assert_eq!(
        payment_event.fields.get("transaction_id"),
        Some(&"\"txn_789\"".to_string())
    );
    assert_eq!(
        payment_event.fields.get("amount_cents"),
        Some(&"2999".to_string())
    );

    // Verify auth event
    let auth_event = captured.iter().find(|e| e.target == "app::auth");
    assert!(auth_event.is_some());
    let auth_event = auth_event.unwrap();
    assert_eq!(auth_event.message, "Authentication successful");
    assert_eq!(auth_event.fields.get("user_id"), Some(&"11111".to_string()));
    assert_eq!(
        auth_event.fields.get("method"),
        Some(&"\"oauth\"".to_string())
    );
}

#[test]
fn logfusion_structured_logging_mixed_types() {
    let (collector, events) = StructuredCollector::new();
    let _guard = registry().with(collector).set_default();
    events.lock().unwrap().clear();

    // Test structured logging with various data types
    info!(
        string_field = "hello world",
        integer_field = 42,
        float_field = 3.14159,
        boolean_field = true,
        "Mixed type fields"
    );

    let captured = events.lock().unwrap();
    let event = captured.iter().find(|e| e.message == "Mixed type fields");

    assert!(event.is_some());
    let event = event.unwrap();

    assert_eq!(
        event.fields.get("string_field"),
        Some(&"\"hello world\"".to_string())
    );
    assert_eq!(event.fields.get("integer_field"), Some(&"42".to_string()));
    assert_eq!(
        event.fields.get("float_field"),
        Some(&"3.14159".to_string())
    );
    assert_eq!(event.fields.get("boolean_field"), Some(&"true".to_string()));
}

#[test]
fn logfusion_reexports_work() {
    // Test that LogFusion's re-exports of tracing-subscriber work correctly
    let (collector, events) = StructuredCollector::new();

    // This should work using only LogFusion re-exports
    let _guard = registry()
        .with(collector)
        .with(EnvFilter::new("info"))
        .set_default();

    events.lock().unwrap().clear();

    // Test basic logging
    info!("Test message");
    debug!("This should be filtered out by EnvFilter");

    let captured = events.lock().unwrap();

    // Should have info but not debug due to filter
    let info_event = captured.iter().find(|e| e.message == "Test message");
    assert!(info_event.is_some());

    let debug_event = captured
        .iter()
        .find(|e| e.message == "This should be filtered out by EnvFilter");
    assert!(debug_event.is_none());
}

#[test]
fn logfusion_json_output_configuration() {
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::fmt::MakeWriter;

    // Create a shared buffer to capture output
    #[derive(Clone)]
    struct TestWriter {
        buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.buffer.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for TestWriter {
        type Writer = TestWriter;

        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    let buffer = Arc::new(Mutex::new(Vec::new()));
    let writer = TestWriter {
        buffer: buffer.clone(),
    };

    // Set up JSON formatting using LogFusion re-exports
    let _guard = registry()
        .with(fmt::layer().json().with_writer(writer))
        .set_default();

    // Log a structured message
    info!(
        service = "test-service",
        version = "1.0.0",
        environment = "test",
        "Service started"
    );

    // Give it a moment to write and flush
    std::thread::sleep(std::time::Duration::from_millis(100));

    let output = buffer.lock().unwrap();
    let json_str = String::from_utf8_lossy(&output);

    // Should contain JSON-like structure
    assert!(
        json_str.contains("level"),
        "Expected 'level' in output: {}",
        json_str
    );
    assert!(
        json_str.contains("Service started"),
        "Expected 'Service started' in output: {}",
        json_str
    );
    assert!(
        json_str.contains("service"),
        "Expected 'service' field in output: {}",
        json_str
    );
    assert!(
        json_str.contains("test-service"),
        "Expected 'test-service' value in output: {}",
        json_str
    );
}

#[test]
fn logfusion_backwards_compatibility_with_simple_syntax() {
    let (collector, events) = StructuredCollector::new();
    let _guard = registry().with(collector).set_default();
    events.lock().unwrap().clear();

    // Test that simple syntax still works alongside structured syntax
    info!("Simple message");
    info!(target: "custom", "Targeted message");
    info!(user_id = 123, "Structured message");
    info!("Formatted message: {}", 42);

    let captured = events.lock().unwrap();

    assert!(captured.iter().any(|e| e.message == "Simple message"));
    assert!(
        captured
            .iter()
            .any(|e| e.message == "Targeted message" && e.target == "custom")
    );
    assert!(
        captured
            .iter()
            .any(|e| e.message == "Structured message" && e.fields.contains_key("user_id"))
    );
    assert!(
        captured
            .iter()
            .any(|e| e.message == "Formatted message: 42")
    );
}
