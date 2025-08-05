//! Tests for define_errors! macro
//!
//! Verifies that the error definition macro works correctly with tracing

use logffi::define_errors;
use std::error::Error;

#[test]
fn basic_error_definition() {
    define_errors! {
        pub enum TestError {
            #[error("Simple error occurred")]
            SimpleError,

            #[error("Error with field: {value}")]
            WithField {
                value: String,
            },
        }
    }

    // Create errors
    let err1 = TestError::SimpleError;
    let err2 = TestError::WithField {
        value: "test".to_string(),
    };

    // Test Display trait
    assert_eq!(err1.to_string(), "Simple error occurred");
    assert_eq!(err2.to_string(), "Error with field: test");

    // Test code method
    assert_eq!(err1.code(), "SimpleError");
    assert_eq!(err2.code(), "WithField");
}

#[test]
fn error_with_log_levels() {
    define_errors! {
        pub enum LogLevelError {
            // Test that level = error now works with hygiene fix
            #[error("Critical error", level = error)]
            Critical,

            #[error("Warning condition", level = warn)]
            Warning,

            #[error("Info message", level = info)]
            Information,
        }
    }

    // Just verify they compile and can be created
    let err = LogLevelError::Critical;
    let warn = LogLevelError::Warning;
    let info = LogLevelError::Information;

    // Test that log() method exists
    err.log();
    warn.log();
    info.log();
}

#[test]
fn error_with_targets() {
    define_errors! {
        pub enum TargetedError {
            #[error("Database error", target = "app::db")]
            DatabaseError,

            #[error("Network error", level = error, target = "app::net")]
            NetworkError,
        }
    }

    let db_err = TargetedError::DatabaseError;
    let net_err = TargetedError::NetworkError;

    // Test that log() method exists and doesn't panic
    db_err.log();
    net_err.log();
}

#[test]
fn error_with_source_chain() {
    use std::io;

    define_errors! {
        pub enum ChainedError {
            #[error("IO operation failed")]
            IoError {
                #[source]
                source: io::Error,
            },

            #[error("Multiple sources")]
            MultiError {
                msg: String,
                #[source]
                source: Box<dyn std::error::Error + Send + Sync>,
            },
        }
    }

    let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let chained = ChainedError::IoError { source: io_err };

    // Verify error chain
    assert!(chained.source().is_some());
    assert_eq!(chained.to_string(), "IO operation failed");

    // Test that .log() works with chained errors
    chained.log();
}

#[test]
fn comprehensive_level_coverage() {
    define_errors! {
        pub enum AllLevelsError {
            #[error("Error level message", level = error)]
            ErrorLevel,

            #[error("Warn level message", level = warn)]
            WarnLevel,

            #[error("Info level message", level = info)]
            InfoLevel,

            #[error("Debug level message", level = debug)]
            DebugLevel,

            #[error("Trace level message", level = trace)]
            TraceLevel,
        }
    }

    // Test all levels compile and log
    AllLevelsError::ErrorLevel.log();
    AllLevelsError::WarnLevel.log();
    AllLevelsError::InfoLevel.log();
    AllLevelsError::DebugLevel.log();
    AllLevelsError::TraceLevel.log();
}

#[test]
fn source_with_different_levels() {
    use std::io;

    define_errors! {
        pub enum SourceWithLevels {
            #[error("Critical IO error", level = error)]
            CriticalIo {
                #[source]
                source: io::Error,
            },

            #[error("IO warning", level = warn)]
            WarningIo {
                #[source]
                source: io::Error,
            },

            #[error("IO info", level = info)]
            InfoIo {
                details: String,
                #[source]
                source: io::Error,
            },
        }
    }

    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");

    let critical = SourceWithLevels::CriticalIo { source: io_err };
    critical.log();

    let io_err2 = io::Error::new(io::ErrorKind::TimedOut, "Timeout");
    let warning = SourceWithLevels::WarningIo { source: io_err2 };
    warning.log();

    let io_err3 = io::Error::new(io::ErrorKind::Interrupted, "Interrupted");
    let info = SourceWithLevels::InfoIo {
        details: "Operation details".to_string(),
        source: io_err3,
    };
    info.log();
}

#[test]
fn source_with_custom_targets() {
    use std::io;

    define_errors! {
        pub enum SourceWithTargets {
            #[error("Database IO error", target = "storage::db")]
            DatabaseIo {
                operation: String,
                #[source]
                source: io::Error,
            },

            #[error("Network IO error", level = warn, target = "network::client")]
            NetworkIo {
                url: String,
                #[source]
                source: io::Error,
            },
        }
    }

    let io_err = io::Error::new(io::ErrorKind::UnexpectedEof, "Connection closed");

    let db_error = SourceWithTargets::DatabaseIo {
        operation: "INSERT".to_string(),
        source: io_err,
    };
    db_error.log();

    let io_err2 = io::Error::new(io::ErrorKind::ConnectionRefused, "Connection refused");
    let net_error = SourceWithTargets::NetworkIo {
        url: "https://api.example.com".to_string(),
        source: io_err2,
    };
    net_error.log();
}

#[test]
fn default_behavior_tests() {
    define_errors! {
        pub enum DefaultBehavior {
            // No level, no target - should default to error level, module_path!() target
            #[error("Default error")]
            DefaultError,

            // Only target specified - should default to error level
            #[error("Custom target only", target = "custom::module")]
            CustomTargetOnly,

            // Only level specified - should use module_path!() target
            #[error("Custom level only", level = debug)]
            CustomLevelOnly,
        }
    }

    // Test all default combinations
    DefaultBehavior::DefaultError.log(); // error level + module_path!()
    DefaultBehavior::CustomTargetOnly.log(); // error level + custom target  
    DefaultBehavior::CustomLevelOnly.log(); // debug level + module_path!()
}
