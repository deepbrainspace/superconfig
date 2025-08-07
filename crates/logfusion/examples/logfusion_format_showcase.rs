//! LogFusion Format Showcase
//!
//! This example demonstrates the new LogFusion format introduced in v0.2, featuring:
//! - Clean, attribute-based syntax without repetitive #[error(...)] attributes
//! - Mixed variant types (unit + struct in same enum)
//! - Automatic source chaining for fields named "source"
//! - Multiple error types in single macro call
//! - All logging levels and custom targets
//! - Field interpolation with structured logging
//!
//! Compare this with the traditional thiserror syntax in other examples.
//!
//! Run with:
//! cargo run --example logfusion_format_showcase
//!
//! Or with different log levels:
//! RUST_LOG=debug cargo run --example logfusion_format_showcase
//! RUST_LOG=trace cargo run --example logfusion_format_showcase

use logfusion::define_errors;
use std::error::Error;

fn main() {
    println!("🆕 LogFusion Format Showcase (v0.2)");
    println!("=================================");
    println!("Demonstrating the new clean LogFusion syntax!\n");

    // Auto-initialization happens on first log call
    demonstrate_basic_logfusion_format();
    demonstrate_mixed_variants();
    demonstrate_logging_levels_and_targets();
    demonstrate_automatic_source_chaining();
    demonstrate_multiple_error_types();
    demonstrate_real_world_example();

    println!("\n🎯 LogFusion Format Benefits:");
    println!("   ✅ 64% macro size reduction (998 → 358 lines)");
    println!("   ✅ Cleaner syntax - no repetitive #[error(...)] attributes");
    println!("   ✅ Mixed variant types in same enum");
    println!("   ✅ Automatic source detection for 'source' fields");
    println!("   ✅ Multiple error types in single macro call");
    println!("   ✅ Attribute-based logging: [level = warn, target = \"app::db\"]");
    println!("   ✅ Field interpolation: \"Failed to connect to {{host}}:{{port}}\"");
    println!("   ✅ Full backward compatibility with thiserror syntax");
    println!("\n✨ LogFusion v0.2 - The future of Rust error handling!");
}

fn demonstrate_basic_logfusion_format() {
    println!("📦 Basic LogFusion Format");
    println!("----------------------");

    // 🆕 LogFusion Format - Clean, attribute-based syntax
    define_errors! {
        BasicError {
            // Unit variants (empty braces)
            NotFound {} : "Resource not found" [level = warn],
            Unauthorized {} : "Access denied" [level = error],

            // Struct variants (with fields)
            ValidationFailed { field: String, reason: String } : "Validation failed for {field}: {reason}" [level = warn],
            DatabaseConnection { host: String, port: u16 } : "Failed to connect to {host}:{port}" [level = error]
        }
    }

    println!("🔧 Creating unit variant:");
    let err = BasicError::NotFound;
    println!("   Error: {}", err);
    println!("   Code: {}", err.code());
    err.log(); // WARN level

    println!("\n🔧 Creating struct variant with field interpolation:");
    let validation_err = BasicError::ValidationFailed {
        field: "email".to_string(),
        reason: "invalid format".to_string(),
    };
    println!("   Error: {}", validation_err);
    println!("   Code: {}", validation_err.code());
    validation_err.log(); // WARN level

    println!("\n🔧 Creating connection error:");
    let db_err = BasicError::DatabaseConnection {
        host: "localhost".to_string(),
        port: 5432,
    };
    println!("   Error: {}", db_err);
    db_err.log(); // ERROR level

    println!();
}

fn demonstrate_mixed_variants() {
    println!("🔀 Mixed Variant Types (Most Powerful Feature)");
    println!("-----------------------------------------------");

    // Mix unit and struct variants in the same enum - this is unique to LogFusion format!
    define_errors! {
        MixedError {
            // Unit variants for simple cases
            NetworkTimeout {} : "Network operation timed out" [level = error],
            ServiceUnavailable {} : "Service temporarily unavailable" [level = warn],

            // Struct variants for complex cases with data
            InsufficientFunds {
                requested: f64,
                available: f64
            } : "Insufficient funds: requested ${requested}, available ${available}" [level = error],

            ValidationError {
                field: String,
                value: String,
                constraint: String
            } : "Field '{field}' with value '{value}' violates constraint: {constraint}" [level = warn],

            // Mix with automatic source chaining
            ProcessingError {
                operation: String,
                source: std::io::Error,
                retry_count: u32
            } : "Operation '{operation}' failed after {retry_count} retries"
        }
    }

    println!("🔧 Unit variant:");
    let timeout_err = MixedError::NetworkTimeout;
    println!("   {}", timeout_err);
    timeout_err.log();

    println!("\n🔧 Struct variant with multiple fields:");
    let funds_err = MixedError::InsufficientFunds {
        requested: 100.50,
        available: 75.25,
    };
    println!("   {}", funds_err);
    funds_err.log();

    println!("\n🔧 Complex validation error:");
    let validation_err = MixedError::ValidationError {
        field: "age".to_string(),
        value: "-5".to_string(),
        constraint: "must be positive".to_string(),
    };
    println!("   {}", validation_err);
    validation_err.log();

    println!("\n🔧 Error with automatic source chaining:");
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
    let proc_err = MixedError::ProcessingError {
        operation: "file_write".to_string(),
        source: io_err,
        retry_count: 3,
    };
    println!("   Primary: {}", proc_err);
    println!("   Has source: {}", proc_err.source().is_some());
    proc_err.log();

    println!();
}

fn demonstrate_logging_levels_and_targets() {
    println!("📊 All Logging Levels & Custom Targets");
    println!("---------------------------------------");

    define_errors! {
        LeveledError {
            CriticalFailure {} : "System critical failure" [level = error, target = "system::critical"],
            SecurityWarning {} : "Security policy violation detected" [level = warn, target = "security::audit"],
            UserAction { user_id: u64, action: String } : "User {user_id} performed {action}" [level = info, target = "user::activity"],
            CacheOperation { operation: String, key: String } : "Cache {operation} for key '{key}'" [level = debug, target = "cache::ops"],
            TraceData { trace_id: String, span: String } : "Trace {trace_id} in span {span}" [level = trace, target = "trace::debug"]
        }
    }

    println!("🔧 All log levels demonstration:");

    let critical = LeveledError::CriticalFailure;
    println!("   ERROR (system::critical): {}", critical);
    critical.log();

    let security = LeveledError::SecurityWarning;
    println!("   WARN (security::audit): {}", security);
    security.log();

    let user_action = LeveledError::UserAction {
        user_id: 12345,
        action: "login".to_string(),
    };
    println!("   INFO (user::activity): {}", user_action);
    user_action.log();

    let cache_op = LeveledError::CacheOperation {
        operation: "miss".to_string(),
        key: "user:12345".to_string(),
    };
    println!("   DEBUG (cache::ops): {}", cache_op);
    cache_op.log();

    let trace_data = LeveledError::TraceData {
        trace_id: "req-abc123".to_string(),
        span: "process_request".to_string(),
    };
    println!("   TRACE (trace::debug): {}", trace_data);
    trace_data.log();

    println!();
}

fn demonstrate_automatic_source_chaining() {
    println!("⛓️ Automatic Source Chaining");
    println!("-----------------------------");
    println!("Fields named 'source' are automatically detected and become #[source]!\n");

    define_errors! {
        ChainedError {
            // Single source field
            IoFailure {
                operation: String,
                source: std::io::Error
            } : "IO operation '{operation}' failed",

            // Multiple fields with source
            NetworkError {
                endpoint: String,
                method: String,
                source: Box<dyn std::error::Error + Send + Sync>,
                timeout_ms: u64
            } : "Network {method} to {endpoint} failed (timeout: {timeout_ms}ms)",

            // Source with different error types
            ParseError {
                content_type: String,
                source: serde_json::Error
            } : "Failed to parse {content_type} content"
        }
    }

    use std::error::Error;

    println!("🔧 IO error with source:");
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let io_failure = ChainedError::IoFailure {
        operation: "read_config".to_string(),
        source: io_err,
    };
    println!("   Primary: {}", io_failure);
    println!(
        "   Source: {:?}",
        io_failure.source().map(|s| s.to_string())
    );
    io_failure.log();

    println!("\n🔧 Network error with boxed source:");
    let inner_err = std::io::Error::new(std::io::ErrorKind::TimedOut, "Connection timed out");
    let net_err = ChainedError::NetworkError {
        endpoint: "https://api.example.com/users".to_string(),
        method: "POST".to_string(),
        source: Box::new(inner_err),
        timeout_ms: 5000,
    };
    println!("   Primary: {}", net_err);
    println!("   Source: {:?}", net_err.source().map(|s| s.to_string()));
    net_err.log();

    println!("\n🔧 JSON parse error with source:");
    let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let parse_err = ChainedError::ParseError {
        content_type: "application/json".to_string(),
        source: json_err,
    };
    println!("   Primary: {}", parse_err);
    println!("   Source: {:?}", parse_err.source().map(|s| s.to_string()));
    parse_err.log();

    println!();
}

fn demonstrate_multiple_error_types() {
    println!("🔧 Multiple Error Types in Single Macro");
    println!("----------------------------------------");

    // Define multiple error enums in one macro call - another LogFusion exclusive feature!
    define_errors! {
        ApiError {
            BadRequest { field: String } : "Invalid field: {field}" [level = warn, target = "api::validation"],
            Unauthorized { user_id: Option<u64> } : "Unauthorized access attempt by user: {user_id:?}" [level = error, target = "api::auth"],
            RateLimited { requests: u32, limit: u32 } : "Rate limit exceeded: {requests}/{limit}" [level = warn, target = "api::rate_limit"]
        }

        DatabaseError {
            ConnectionFailed { host: String, port: u16 } : "Cannot connect to database at {host}:{port}" [level = error, target = "db::connection"],
            QueryTimeout { query: String, duration_ms: u64 } : "Query timed out after {duration_ms}ms: {query}" [level = warn, target = "db::performance"],
            TransactionFailed { operation: String } : "Transaction failed: {operation}" [level = error, target = "db::transaction"]
        }

        CacheError {
            Unavailable {} : "Cache service unavailable" [level = warn, target = "cache::service"],
            KeyExpired { key: String, expired_at: String } : "Key '{key}' expired at {expired_at}" [level = info, target = "cache::expiry"]
        }
    }

    println!("🔧 API Errors:");
    let api_err = ApiError::BadRequest {
        field: "email".to_string(),
    };
    println!("   {}", api_err);
    api_err.log();

    let auth_err = ApiError::Unauthorized {
        user_id: Some(12345),
    };
    println!("   {}", auth_err);
    auth_err.log();

    println!("\n🔧 Database Errors:");
    let db_err = DatabaseError::ConnectionFailed {
        host: "postgres.prod.example.com".to_string(),
        port: 5432,
    };
    println!("   {}", db_err);
    db_err.log();

    let query_err = DatabaseError::QueryTimeout {
        query: "SELECT * FROM users WHERE active = true".to_string(),
        duration_ms: 30000,
    };
    println!("   {}", query_err);
    query_err.log();

    println!("\n🔧 Cache Errors:");
    let cache_err = CacheError::Unavailable;
    println!("   {}", cache_err);
    cache_err.log();

    let expiry_err = CacheError::KeyExpired {
        key: "user:session:abc123".to_string(),
        expired_at: "2024-01-15T10:30:00Z".to_string(),
    };
    println!("   {}", expiry_err);
    expiry_err.log();

    println!();
}

fn demonstrate_real_world_example() {
    println!("🌍 Real-World E-commerce Example");
    println!("---------------------------------");

    // Comprehensive e-commerce error handling with LogFusion format
    define_errors! {
        PaymentError {
            // Simple cases (unit variants)
            InvalidCard {} : "Invalid credit card number" [level = warn, target = "payment::validation"],
            ExpiredCard {} : "Credit card has expired" [level = warn, target = "payment::validation"],
            NetworkTimeout {} : "Payment gateway timeout" [level = error, target = "payment::gateway"],

            // Complex cases (struct variants)
            InsufficientFunds {
                requested_amount: f64,
                available_balance: f64,
                account_id: String
            } : "Insufficient funds in account {account_id}: requested ${requested_amount}, available ${available_balance}" [level = error, target = "payment::balance"],

            ProcessorDeclined {
                transaction_id: String,
                decline_code: String,
                processor: String
            } : "Payment processor {processor} declined transaction {transaction_id}: {decline_code}" [level = warn, target = "payment::processor"],

            // With automatic source chaining
            GatewayError {
                gateway: String,
                endpoint: String,
                source: std::io::Error
            } : "Payment gateway {gateway} error at {endpoint}",

            FraudDetected {
                user_id: u64,
                risk_score: f32,
                rules_triggered: Vec<String>,
                transaction_id: String
            } : "Fraud detected for user {user_id} (risk: {risk_score}): transaction {transaction_id}" [level = error, target = "security::fraud"]
        }
    }

    println!("🔧 E-commerce payment processing scenarios:");

    // Simple validation errors
    let invalid = PaymentError::InvalidCard;
    println!("   Validation: {}", invalid);
    invalid.log();

    // Business logic with detailed context
    let insufficient = PaymentError::InsufficientFunds {
        requested_amount: 299.99,
        available_balance: 150.00,
        account_id: "acc_1234567890".to_string(),
    };
    println!("   Business logic: {}", insufficient);
    insufficient.log();

    // External service integration
    let declined = PaymentError::ProcessorDeclined {
        transaction_id: "txn_abc123def456".to_string(),
        decline_code: "insufficient_funds".to_string(),
        processor: "Stripe".to_string(),
    };
    println!("   External service: {}", declined);
    declined.log();

    // Network error with source chaining
    let io_err = std::io::Error::new(std::io::ErrorKind::TimedOut, "Connection timeout");
    let gateway_err = PaymentError::GatewayError {
        gateway: "PayPal".to_string(),
        endpoint: "/v2/payments/payment".to_string(),
        source: io_err,
    };
    println!("   Network (with source): {}", gateway_err);
    println!("   Has source chain: {}", gateway_err.source().is_some());
    gateway_err.log();

    // Security/fraud detection
    let fraud = PaymentError::FraudDetected {
        user_id: 78901,
        risk_score: 0.92,
        rules_triggered: vec![
            "velocity_check".to_string(),
            "geolocation_mismatch".to_string(),
            "device_fingerprint".to_string(),
        ],
        transaction_id: "txn_suspicious_001".to_string(),
    };
    println!("   Security: {}", fraud);
    fraud.log();

    println!();
}
