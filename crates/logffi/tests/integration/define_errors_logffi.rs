//! Comprehensive tests for LogFFI define_errors! macro
//!
//! This file tests all supported syntax patterns for the new LogFFI format.
//! Tests both standalone error types and mixed variant types.

use logffi::define_errors;
use std::error::Error;

#[test]
fn unit_variants_only() {
    define_errors! {
        SimpleError {
            NotFound {} : "Resource not found",
            InvalidInput {} : "Invalid input provided",
            Timeout {} : "Operation timed out"
        }
    }

    let err = SimpleError::NotFound;
    assert_eq!(err.to_string(), "Resource not found");
    assert_eq!(err.code(), "NotFound");
    err.log();

    let input_err = SimpleError::InvalidInput;
    assert_eq!(input_err.to_string(), "Invalid input provided");
    input_err.log();
}

#[test]
fn struct_variants_only() {
    define_errors! {
        StructError {
            DatabaseConnection { host: String, port: u16 } : "Failed to connect to {host}:{port}",
            ValidationFailed { field: String, reason: String } : "Validation failed for {field}: {reason}",
            UserNotFound { user_id: u64 } : "User {user_id} not found"
        }
    }

    let db_err = StructError::DatabaseConnection {
        host: "localhost".to_string(),
        port: 5432,
    };
    assert_eq!(db_err.to_string(), "Failed to connect to localhost:5432");
    assert_eq!(db_err.code(), "DatabaseConnection");
    db_err.log();

    let val_err = StructError::ValidationFailed {
        field: "email".to_string(),
        reason: "invalid format".to_string(),
    };
    assert_eq!(
        val_err.to_string(),
        "Validation failed for email: invalid format"
    );
    val_err.log();
}

#[test]
fn mixed_variants() {
    define_errors! {
        MixedError {
            SimpleError {} : "A simple error",
            ComplexError { value: String } : "Complex: {value}",
            AnotherSimple {} : "Another simple one",
            WithNumber { num: i32 } : "Number is {num}",
            NetworkError { source: std::io::Error } : "Network error occurred"
        }
    }

    // Test unit variants
    let err1 = MixedError::SimpleError;
    assert_eq!(err1.to_string(), "A simple error");
    assert_eq!(err1.code(), "SimpleError");
    err1.log();

    let err3 = MixedError::AnotherSimple;
    assert_eq!(err3.to_string(), "Another simple one");
    err3.log();

    // Test struct variants
    let err2 = MixedError::ComplexError {
        value: "test".to_string(),
    };
    assert_eq!(err2.to_string(), "Complex: test");
    assert_eq!(err2.code(), "ComplexError");
    err2.log();

    let err4 = MixedError::WithNumber { num: 42 };
    assert_eq!(err4.to_string(), "Number is 42");
    err4.log();

    // Test automatic source chaining
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let net_err = MixedError::NetworkError { source: io_err };
    assert_eq!(net_err.to_string(), "Network error occurred");
    assert_eq!(net_err.code(), "NetworkError");
    assert!(net_err.source().is_some()); // Should have automatic source chaining
    net_err.log();
}

#[test]
fn with_log_level_attributes() {
    define_errors! {
        AttributedError {
            ErrorLevel {} : "An error occurred" [level = error],
            WarnLevel {} : "A warning occurred" [level = warn],
            InfoLevel {} : "Info message" [level = info],
            DebugLevel {} : "Debug message" [level = debug],
            TraceLevel {} : "Trace message" [level = trace]
        }
    }

    let err = AttributedError::ErrorLevel;
    assert_eq!(err.to_string(), "An error occurred");
    err.log(); // Should log at error level

    let warn = AttributedError::WarnLevel;
    warn.log(); // Should log at warn level

    let info = AttributedError::InfoLevel;
    info.log(); // Should log at info level

    let debug = AttributedError::DebugLevel;
    debug.log(); // Should log at debug level

    let trace = AttributedError::TraceLevel;
    trace.log(); // Should log at trace level
}

#[test]
fn with_custom_targets() {
    define_errors! {
        TargetedError {
            DatabaseError {} : "Database error" [level = error, target = "app::database"],
            NetworkError {} : "Network error" [level = warn, target = "app::network"],
            AuthError {} : "Authentication failed" [level = info, target = "app::auth"],
            DefaultError {} : "Default target error" [level = error]
        }
    }

    let db_err = TargetedError::DatabaseError;
    let net_err = TargetedError::NetworkError;
    let auth_err = TargetedError::AuthError;
    let default_err = TargetedError::DefaultError;

    // These should log to their respective targets
    db_err.log();
    net_err.log();
    auth_err.log();
    default_err.log();
}

#[test]
fn automatic_source_detection() {
    define_errors! {
        SourceError {
            IoError { source: std::io::Error } : "IO operation failed",
            ChainedError {
                context: String,
                source: Box<dyn std::error::Error + Send + Sync>
            } : "Operation failed: {context}",
            MultipleFields {
                operation: String,
                source: std::io::Error,
                retry_count: u32
            } : "Operation {operation} failed after {retry_count} retries"
        }
    }

    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let err = SourceError::IoError { source: io_err };

    assert_eq!(err.to_string(), "IO operation failed");
    assert_eq!(err.code(), "IoError");
    assert!(err.source().is_some());
    err.log();

    let io_err2 = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
    let multi_err = SourceError::MultipleFields {
        operation: "read_file".to_string(),
        source: io_err2,
        retry_count: 3,
    };

    assert_eq!(
        multi_err.to_string(),
        "Operation read_file failed after 3 retries"
    );
    assert!(multi_err.source().is_some());
    multi_err.log();
}

#[test]
fn real_world_payment_example() {
    define_errors! {
        PaymentError {
            InvalidCard {} : "Invalid card number" [level = warn],
            InsufficientFunds { amount: f64, available: f64 } : "Need ${amount}, have ${available}" [level = error],
            NetworkError { source: std::io::Error } : "Network error occurred",
            ProcessingFailed { transaction_id: String, reason: String } : "Transaction {transaction_id} failed: {reason}" [level = error]
        }
    }

    // Test unit variant
    let card_err = PaymentError::InvalidCard;
    assert_eq!(card_err.to_string(), "Invalid card number");
    assert_eq!(card_err.code(), "InvalidCard");
    card_err.log();

    // Test struct variant with field interpolation
    let funds_err = PaymentError::InsufficientFunds {
        amount: 100.0,
        available: 50.0,
    };
    assert_eq!(funds_err.to_string(), "Need $100, have $50");
    assert_eq!(funds_err.code(), "InsufficientFunds");
    funds_err.log();

    // Test automatic source chaining
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let net_err = PaymentError::NetworkError { source: io_err };
    assert_eq!(net_err.to_string(), "Network error occurred");
    assert_eq!(net_err.code(), "NetworkError");
    assert!(net_err.source().is_some());
    net_err.log();

    // Test multiple fields
    let proc_err = PaymentError::ProcessingFailed {
        transaction_id: "txn_123".to_string(),
        reason: "card declined".to_string(),
    };
    assert_eq!(
        proc_err.to_string(),
        "Transaction txn_123 failed: card declined"
    );
    proc_err.log();
}

#[test]
fn multiple_error_types_in_single_macro() {
    // Test the new feature: multiple error types in one macro call
    define_errors! {
        ApiError {
            BadRequest { field: String } : "Invalid field: {field}" [level = warn],
            Unauthorized {} : "Access denied" [level = error]
        }

        DatabaseError {
            ConnectionFailed { host: String } : "Failed to connect to {host}" [level = error],
            QueryTimeout {} : "Query timed out" [level = warn]
        }
    }

    // Test ApiError
    let api_err = ApiError::BadRequest {
        field: "email".to_string(),
    };
    assert_eq!(api_err.to_string(), "Invalid field: email");
    api_err.log();

    let auth_err = ApiError::Unauthorized;
    assert_eq!(auth_err.to_string(), "Access denied");
    auth_err.log();

    // Test DatabaseError
    let db_err = DatabaseError::ConnectionFailed {
        host: "localhost".to_string(),
    };
    assert_eq!(db_err.to_string(), "Failed to connect to localhost");
    db_err.log();

    let timeout_err = DatabaseError::QueryTimeout;
    assert_eq!(timeout_err.to_string(), "Query timed out");
    timeout_err.log();
}
