//! Backend Showcase Example
//!
//! This example demonstrates LogFFI's feature-based backend system.
//!
//! Run with different feature combinations:
//!
//! # Default (tracing only)
//! cargo run --example backend_showcase
//!
//! # Log backend only
//! cargo run --example backend_showcase --no-default-features --features log
//!
//! # Slog backend only  
//! cargo run --example backend_showcase --no-default-features --features slog
//!
//! # All backends
//! cargo run --example backend_showcase --features all
//!
//! # Multiple specific backends
//! cargo run --example backend_showcase --no-default-features --features "log,tracing"

use logffi::{debug, error, info, logger, trace, warn};

fn main() {
    println!("🚀 LogFFI Backend Showcase");
    println!("=========================");

    // Get logger and show available backends
    let logger = logger();
    let available = logger.available_backends();

    println!("📊 Available backends: {:?}", available);
    println!();

    // Basic logging - works with any backend combination
    println!("📝 Basic logging:");
    error!("This is an error message");
    warn!("This is a warning message");
    info!("This is an info message");
    debug!("This is a debug message");
    trace!("This is a trace message");
    println!();

    // Targeted logging
    println!("🎯 Targeted logging:");
    error!(target: "database", "Database connection failed");
    info!(target: "server", "Server started on port 8080");
    debug!(target: "auth", "User authentication successful");
    println!();

    // Formatted logging
    println!("🎨 Formatted logging:");
    let user_id = 12345;
    let operation = "save";
    info!("User {} performed operation: {}", user_id, operation);
    warn!("Memory usage: {}%", 85);
    error!("Failed to process {} records", 42);
    println!();

    // Backend-specific features
    println!("🔧 Backend-specific features:");

    #[cfg(feature = "log")]
    {
        if let Some(_log_backend) = logger.as_log() {
            println!("✅ Log backend is available");

            // With log backend, you can use any log-compatible logger
            log::info!("Direct log crate usage");
        }
    }

    #[cfg(feature = "tracing")]
    {
        if let Some(_tracing_backend) = logger.as_tracing() {
            println!("✅ Tracing backend is available");

            // With tracing backend, you can use spans and structured logging
            use tracing::{Level, span};
            let span = span!(Level::INFO, "user_operation", user_id = 12345);
            let _enter = span.enter();

            tracing::info!("Inside a tracing span");
            info!("LogFFI macros work inside spans too!");
        }
    }

    #[cfg(feature = "slog")]
    {
        if let Some(slog_backend) = logger.as_slog() {
            println!("✅ Slog backend is available");

            // With slog backend, you can create child loggers
            use slog::{info as slog_info, o};
            let child = slog_backend.logger().new(o!("component" => "database"));
            slog_info!(child, "Database query executed"; "query_time" => "15ms");
        }
    }

    #[cfg(feature = "callback")]
    {
        if let Some(_callback_backend) = logger.as_callback() {
            println!("✅ Callback backend is available");

            // Set up a callback for external systems
            use logffi::set_callback;
            set_callback(Box::new(|level, target, message| {
                println!("📡 FFI Callback: [{}] {}: {}", level, target, message);
            }));

            info!("This message will go to the callback!");
        }
    }

    println!();

    // Error handling with LogFFI
    println!("⚠️  Error handling integration:");

    use logffi::define_errors;

    define_errors! {
        pub enum DemoError {
            #[error("Network connection failed: {url}", level = error, target = "network")]
            NetworkError {
                url: String,
            },

            #[error("Configuration missing: {key}", level = warn)]
            ConfigMissing {
                key: String,
            },

            #[error("Processing completed: {count} items", level = info)]
            ProcessingComplete {
                count: u32,
            },
        }
    }

    // These create errors AND log them automatically
    let _network_err = DemoError::new_network_error("https://api.example.com".to_string());
    let _config_err = DemoError::new_config_missing("database.host".to_string());
    let _success = DemoError::new_processing_complete(150);

    println!();
    println!("✨ Example complete! Check the log output above to see all the automatic logging.");

    // Show compile-time optimizations
    print_backend_info();
}

fn print_backend_info() {
    println!("🔍 Compile-time backend information:");

    #[cfg(feature = "log")]
    println!("   - Log backend: ENABLED");
    #[cfg(not(feature = "log"))]
    println!("   - Log backend: disabled (not compiled in)");

    #[cfg(feature = "tracing")]
    println!("   - Tracing backend: ENABLED");
    #[cfg(not(feature = "tracing"))]
    println!("   - Tracing backend: disabled (not compiled in)");

    #[cfg(feature = "slog")]
    println!("   - Slog backend: ENABLED");
    #[cfg(not(feature = "slog"))]
    println!("   - Slog backend: disabled (not compiled in)");

    #[cfg(feature = "callback")]
    println!("   - Callback backend: ENABLED");
    #[cfg(not(feature = "callback"))]
    println!("   - Callback backend: disabled (not compiled in)");
}
