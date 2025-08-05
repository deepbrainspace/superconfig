//! Tracing Integration Showcase Example
//!
//! This example demonstrates LogFFI's tracing-native implementation with auto-initialization,
//! structured logging, and comprehensive error handling integration.
//!
//! Run with different log levels:
//!
//! # Default (info level)
//! cargo run --example tracing_showcase
//!
//! # Debug level
//! RUST_LOG=debug cargo run --example tracing_showcase
//!
//! # Trace level with specific targets
//! RUST_LOG=trace,logffi=debug cargo run --example tracing_showcase
//!
//! # Filter specific modules
//! RUST_LOG=tracing_showcase::database=warn cargo run --example tracing_showcase

use logffi::{debug, define_errors, error, info, trace, warn};

fn main() {
    println!("🚀 LogFFI Tracing-Native Showcase");
    println!("=================================");
    println!();

    // Auto-initialization happens automatically on first log call
    println!("📝 Basic logging (auto-initializes tracing subscriber):");
    error!("This is an error message");
    warn!("This is a warning message");
    info!("This is an info message");
    debug!("This is a debug message (filtered by RUST_LOG)");
    trace!("This is a trace message (filtered by RUST_LOG)");
    println!();

    // Targeted logging with custom targets
    println!("🎯 Targeted logging:");
    error!(target: "app::database", "Database connection failed");
    info!(target: "app::server", "Server started on port 8080");
    debug!(target: "app::auth", "User authentication successful");
    warn!(target: "external::api", "API rate limit approaching");
    println!();

    // Formatted logging
    println!("🎨 Formatted logging:");
    let user_id = 12345;
    let operation = "save";
    info!("User {} performed operation: {}", user_id, operation);
    warn!("Memory usage: {}%", 85);
    error!("Failed to process {} records", 42);
    println!();

    // Demonstrate automatic error logging with different levels
    println!("⚠️  Error handling with automatic logging:");
    demonstrate_error_levels();
    println!();

    // Demonstrate source chaining
    println!("🔗 Error source chaining:");
    demonstrate_source_chaining();
    println!();

    // Demonstrate callback integration (if enabled)
    #[cfg(feature = "callback")]
    {
        println!("📡 Callback integration:");
        demonstrate_callback_integration();
        println!();
    }

    // Show how tracing spans work with LogFFI
    println!("📊 Tracing span integration:");
    demonstrate_spans();
    println!();

    println!(
        "✨ Example complete! All logs are processed through tracing with auto-initialization."
    );
    println!("   Try different RUST_LOG levels to see filtering in action!");
}

fn demonstrate_error_levels() {
    define_errors! {
        pub enum DemoError {
            #[error("Network connection failed: {url}", level = error, target = "network")]
            NetworkError { url: String },

            #[error("Configuration missing: {key}", level = warn)]
            ConfigMissing { key: String },

            #[error("Processing completed: {count} items", level = info)]
            ProcessingComplete { count: u32 },

            #[error("Debug info: {details}", level = debug)]
            DebugInfo { details: String },

            #[error("Trace data: {trace_id}", level = trace)]
            TraceData { trace_id: String },
        }
    }

    // These automatically log at their specified levels
    let network_err = DemoError::NetworkError {
        url: "https://api.example.com".to_string(),
    };
    network_err.log();

    let config_err = DemoError::ConfigMissing {
        key: "database.host".to_string(),
    };
    config_err.log();

    let success = DemoError::ProcessingComplete { count: 150 };
    success.log();

    let debug_info = DemoError::DebugInfo {
        details: "Cache miss for user 12345".to_string(),
    };
    debug_info.log();

    let trace_data = DemoError::TraceData {
        trace_id: "req-abc123".to_string(),
    };
    trace_data.log();
}

fn demonstrate_source_chaining() {
    use std::error::Error;
    use std::io;

    define_errors! {
        pub enum ChainedError {
            #[error("Failed to read config file: {path}", level = error, target = "config")]
            ConfigReadError {
                path: String,
                #[source]
                source: io::Error,
            },

            #[error("Database transaction failed", level = warn, target = "database")]
            TransactionError {
                operation: String,
                #[source]
                source: Box<dyn std::error::Error + Send + Sync>,
            },
        }
    }

    // Create a chained error
    let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let config_err = ChainedError::ConfigReadError {
        path: "/etc/app/config.yaml".to_string(),
        source: io_err,
    };

    // Log the error (shows top-level message)
    config_err.log();

    // Source chain is still accessible for debugging
    if let Some(source) = config_err.source() {
        debug!(target: "config::debug", "Root cause: {}", source);
    }

    // Another example with boxed error
    let inner_err = io::Error::new(io::ErrorKind::ConnectionRefused, "Connection refused");
    let db_err = ChainedError::TransactionError {
        operation: "UPDATE users SET ...".to_string(),
        source: Box::new(inner_err),
    };
    db_err.log();
}

#[cfg(feature = "callback")]
fn demonstrate_callback_integration() {
    use logffi::{Callback, set_callback};

    // Set up callback to capture logs for external systems
    let callback: Callback = Box::new(|level, target, message| {
        println!(
            "📡 FFI Callback captured: [{}] {}: {}",
            level, target, message
        );
    });

    set_callback(callback);

    // This will trigger both tracing output AND the callback
    info!(target: "callback::demo", "This message goes to both tracing and callback");
    warn!(target: "callback::demo", "Warning captured by callback system");
}

fn demonstrate_spans() {
    use tracing::{Level, instrument};

    #[instrument(level = "info")]
    fn process_user_request(user_id: u64, action: &str) {
        info!("Processing request for user {}", user_id);

        // LogFFI macros work perfectly inside tracing spans
        debug!("Action details: {}", action);

        // Simulate some work that might fail
        if action == "delete" {
            error!("Delete operation not allowed for user {}", user_id);
        } else {
            info!("Request completed successfully");
        }
    }

    // Create a span manually
    let span = tracing::span!(Level::INFO, "user_session", user_id = 12345);
    let _enter = span.enter();

    info!("User session started");

    // Call instrumented function (creates nested span)
    process_user_request(12345, "save");
    process_user_request(12345, "delete");

    info!("User session ended");
}
