//! LogFusion Source Chaining - Automatic Detection
//!
//! This example demonstrates LogFusion v0.2's automatic source chaining feature:
//! - Fields named "source" are automatically detected and become #[source]
//! - No manual #[source] attribute needed
//! - Works with any Error type (std::io::Error, Box<dyn Error>, custom errors)
//! - Compare with traditional thiserror manual #[source] attributes
//!
//! Run with:
//! cargo run --example logfusion_source_chaining
//!
//! Or with debug logs:
//! RUST_LOG=debug cargo run --example logfusion_source_chaining

use logfusion::define_errors;
use std::error::Error;

fn main() {
    println!("⛓️ LogFusion Automatic Source Chaining Example");
    println!("=============================================");
    println!("Demonstrating automatic source detection for fields named 'source'\n");

    demonstrate_logfusion_auto_source_chaining();
    demonstrate_traditional_source_chaining();
    demonstrate_mixed_source_types();
    show_source_chain_walking();

    println!("\n🎯 LogFusion Auto Source Chaining Benefits:");
    println!("   ✅ No manual #[source] attributes needed");
    println!("   ✅ Fields named 'source' automatically detected");
    println!("   ✅ Works with any Error type");
    println!("   ✅ Cleaner syntax than thiserror");
    println!("   ✅ Full backward compatibility");
    println!("   ✅ Proper error chain preservation");
    println!("\n✨ LogFusion makes error chaining effortless!");
}

fn demonstrate_logfusion_auto_source_chaining() {
    println!("🆕 LogFusion Format - Automatic Source Detection");
    println!("----------------------------------------------");

    // 🆕 LogFusion Format - Fields named "source" automatically become #[source]
    define_errors! {
        LogFusionChainedError {
            // Single source field - automatically detected!
            IoFailure {
                operation: String,
                source: std::io::Error
            } : "IO operation '{operation}' failed" [level = error, target = "io::ops"],

            // Multiple fields with automatic source detection
            NetworkError {
                endpoint: String,
                method: String,
                timeout_ms: u64,
                source: Box<dyn std::error::Error + Send + Sync>
            } : "Network {method} to {endpoint} failed (timeout: {timeout_ms}ms)" [level = error, target = "network"],

            // Source with JSON parsing
            ConfigParseError {
                config_file: String,
                section: String,
                source: serde_json::Error
            } : "Failed to parse {section} section in config file {config_file}" [level = error, target = "config::parse"],

            // Complex business logic with source
            PaymentProcessingError {
                transaction_id: String,
                amount: f64,
                processor: String,
                source: std::io::Error,
                retry_count: u32
            } : "Payment processing failed for transaction {transaction_id} (${amount} via {processor}) after {retry_count} retries" [level = error, target = "payment::processing"]
        }
    }

    println!("🔧 IO Error with automatic source chaining:");
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "Config file not found");
    let io_failure = LogFusionChainedError::IoFailure {
        operation: "read_config".to_string(),
        source: io_err,
    };

    println!("   Primary: {}", io_failure);
    println!("   Code: {}", io_failure.code());
    println!("   Has source: {}", io_failure.source().is_some());
    if let Some(source) = io_failure.source() {
        println!("   Source: {}", source);
    }
    io_failure.log();
    println!();

    println!("🔧 Network error with boxed source:");
    let timeout_err = std::io::Error::new(std::io::ErrorKind::TimedOut, "Connection timeout");
    let net_err = LogFusionChainedError::NetworkError {
        endpoint: "https://api.payment-processor.com/v1/charges".to_string(),
        method: "POST".to_string(),
        timeout_ms: 30000,
        source: Box::new(timeout_err),
    };

    println!("   Primary: {}", net_err);
    println!("   Has source: {}", net_err.source().is_some());
    if let Some(source) = net_err.source() {
        println!("   Source: {}", source);
    }
    net_err.log();
    println!();

    println!("🔧 JSON parse error with automatic chaining:");
    let json_err = serde_json::from_str::<serde_json::Value>("{invalid json}").unwrap_err();
    let parse_err = LogFusionChainedError::ConfigParseError {
        config_file: "app.json".to_string(),
        section: "database".to_string(),
        source: json_err,
    };

    println!("   Primary: {}", parse_err);
    println!("   Has source: {}", parse_err.source().is_some());
    parse_err.log();
    println!();

    println!("🔧 Complex payment error with source:");
    let payment_io_err = std::io::Error::new(
        std::io::ErrorKind::ConnectionRefused,
        "Payment gateway unreachable",
    );
    let payment_err = LogFusionChainedError::PaymentProcessingError {
        transaction_id: "txn_abc123def".to_string(),
        amount: 299.99,
        processor: "Stripe".to_string(),
        source: payment_io_err,
        retry_count: 3,
    };

    println!("   Primary: {}", payment_err);
    println!("   Has source: {}", payment_err.source().is_some());
    payment_err.log();
    println!();
}

fn demonstrate_traditional_source_chaining() {
    println!("🔧 Traditional Thiserror Format - Manual #[source] Attributes");
    println!("--------------------------------------------------------------");

    // Traditional thiserror format with manual #[source] attributes for comparison
    define_errors! {
        pub enum ThiserrorChainedError {
            #[error("IO operation '{operation}' failed", level = error, target = "io::ops")]
            IoFailure {
                operation: String,
                #[source]
                source: std::io::Error,
            },

            #[error("Network {method} to {endpoint} failed (timeout: {timeout_ms}ms)", level = error, target = "network")]
            NetworkError {
                endpoint: String,
                method: String,
                timeout_ms: u64,
                #[source]
                source: Box<dyn std::error::Error + Send + Sync>,
            },

            #[error("Failed to parse {section} section in config file {config_file}", level = error, target = "config::parse")]
            ConfigParseError {
                config_file: String,
                section: String,
                #[source]
                source: serde_json::Error,
            },
        }
    }

    println!("🔧 Traditional thiserror with manual #[source]:");
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission denied");
    let traditional_err = ThiserrorChainedError::IoFailure {
        operation: "write_log".to_string(),
        source: io_err,
    };

    println!("   Primary: {}", traditional_err);
    println!("   Has source: {}", traditional_err.source().is_some());
    traditional_err.log();
    println!();
}

fn demonstrate_mixed_source_types() {
    println!("🔄 Mixed Source Types - All Automatically Detected");
    println!("---------------------------------------------------");

    // Different source types - all automatically detected by field name
    define_errors! {
        MixedSourceError {
            // std::io::Error source
            FileSystemError {
                path: String,
                operation: String,
                source: std::io::Error
            } : "File system {operation} failed for path: {path}",

            // Custom error type as source
            DatabaseError {
                table: String,
                query: String,
                source: Box<dyn std::error::Error + Send + Sync>
            } : "Database query failed on table {table}: {query}",

            // HTTP error source
            HttpError {
                url: String,
                status: u16,
                source: Box<dyn std::error::Error + Send + Sync>
            } : "HTTP request to {url} failed with status {status}",

            // Validation error with multiple sources possible
            ValidationError {
                field: String,
                value: String,
                source: Box<dyn std::error::Error + Send + Sync>
            } : "Validation failed for field {field} with value '{value}'"
        }
    }

    println!("🔧 File system error:");
    let fs_err = std::io::Error::new(std::io::ErrorKind::NotFound, "Directory not found");
    let fs_error = MixedSourceError::FileSystemError {
        path: "/var/log/app".to_string(),
        operation: "create_directory".to_string(),
        source: fs_err,
    };
    println!("   {}", fs_error);
    fs_error.log();
    println!();

    println!("🔧 Database error with custom source:");
    let custom_err = std::io::Error::new(
        std::io::ErrorKind::ConnectionRefused,
        "DB connection refused",
    );
    let db_error = MixedSourceError::DatabaseError {
        table: "users".to_string(),
        query: "SELECT * FROM users WHERE active = true".to_string(),
        source: Box::new(custom_err),
    };
    println!("   {}", db_error);
    println!("   Source available: {}", db_error.source().is_some());
    db_error.log();
    println!();
}

fn show_source_chain_walking() {
    println!("🚶 Walking the Error Chain");
    println!("---------------------------");

    define_errors! {
        ChainWalkerError {
            ServiceError {
                service: String,
                source: std::io::Error
            } : "Service {service} encountered an error"
        }
    }

    // Create a chained error
    let root_cause = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Broken pipe");
    let service_err = ChainWalkerError::ServiceError {
        service: "payment-processor".to_string(),
        source: root_cause,
    };

    println!("🔧 Walking error chain:");
    println!("   Primary error: {}", service_err);

    // Walk the error chain
    let mut current_error: &dyn std::error::Error = &service_err;
    let mut depth = 0;

    while let Some(source) = current_error.source() {
        depth += 1;
        println!("   └─ Cause {}: {}", depth, source);
        current_error = source;
    }

    if depth == 0 {
        println!("   └─ No source errors");
    }

    println!("   ✓ Error chain depth: {}", depth);
    service_err.log();
    println!();

    println!("🔧 Source chain provides full debugging context!");
    println!("   • Primary error shows business-level message");
    println!("   • Source errors provide technical details");
    println!("   • Full chain preserved for debugging tools");
    println!("   • Works with error reporting crates (eyre, anyhow)");
}
