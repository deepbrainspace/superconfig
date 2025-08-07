//! Tests for the new error_info() method
//!
//! Verifies that the error_info() method correctly returns structured information

use logfusion::define_errors;

#[test]
fn test_error_info_thiserror_format() {
    define_errors! {
        pub enum ThiserrorError {
            #[error("Simple error")]
            SimpleError,

            #[error("Database error", level = error, target = "db::core")]
            DatabaseError,

            #[error("Network warning", level = warn, target = "net::client")]
            NetworkError,

            #[error("Debug info", level = debug)]
            DebugInfo,
        }
    }

    // Test simple error (defaults)
    let err = ThiserrorError::SimpleError;
    let (code, level, target) = err.error_info();
    assert_eq!(code, "SimpleError");
    assert_eq!(level, "error");
    // module_path!() returns the current module path
    assert!(target.contains("error_info_method"));

    // Test with explicit level and target
    let err = ThiserrorError::DatabaseError;
    let (code, level, target) = err.error_info();
    assert_eq!(code, "DatabaseError");
    assert_eq!(level, "error");
    assert_eq!(target, "db::core");

    // Test with warn level
    let err = ThiserrorError::NetworkError;
    let (code, level, target) = err.error_info();
    assert_eq!(code, "NetworkError");
    assert_eq!(level, "warn");
    assert_eq!(target, "net::client");

    // Test with debug level and default target
    let err = ThiserrorError::DebugInfo;
    let (code, level, target) = err.error_info();
    assert_eq!(code, "DebugInfo");
    assert_eq!(level, "debug");
    assert!(target.contains("error_info_method"));
}

#[test]
fn test_error_info_logfusion_format() {
    define_errors! {
        LogFusionError {
            SimpleError {} : "Simple error occurred",
            DatabaseError { host: String, port: u16 } : "Database connection failed: {host}:{port}" [level = error, target = "db::connection"],
            ValidationWarning { field: String } : "Field validation failed: {field}" [level = warn],
            InfoMessage {} : "Information message" [level = info, target = "app::info"],
            DefaultError {} : "Default error with no attributes"
        }
    }

    // Test unit variant with defaults
    let err = LogFusionError::SimpleError;
    let (code, level, target) = err.error_info();
    assert_eq!(code, "SimpleError");
    assert_eq!(level, "error");
    assert!(target.contains("error_info_method"));

    // Test struct variant with level and target
    let err = LogFusionError::DatabaseError {
        host: "localhost".to_string(),
        port: 5432,
    };
    let (code, level, target) = err.error_info();
    assert_eq!(code, "DatabaseError");
    assert_eq!(level, "error");
    assert_eq!(target, "db::connection");

    // Test with warn level and default target
    let err = LogFusionError::ValidationWarning {
        field: "email".to_string(),
    };
    let (code, level, target) = err.error_info();
    assert_eq!(code, "ValidationWarning");
    assert_eq!(level, "warn");
    assert!(target.contains("error_info_method"));

    // Test info level with custom target
    let err = LogFusionError::InfoMessage;
    let (code, level, target) = err.error_info();
    assert_eq!(code, "InfoMessage");
    assert_eq!(level, "info");
    assert_eq!(target, "app::info");

    // Test defaults
    let err = LogFusionError::DefaultError;
    let (code, level, target) = err.error_info();
    assert_eq!(code, "DefaultError");
    assert_eq!(level, "error");
    assert!(target.contains("error_info_method"));
}

#[test]
fn test_error_info_mixed_variants() {
    define_errors! {
        MixedError {
            UnitError {} : "Unit error" [level = warn, target = "unit::module"],
            StructError { msg: String } : "Struct error: {msg}" [level = info],
            DefaultUnit {} : "Default unit",
            DefaultStruct { value: i32 } : "Default struct: {value}"
        }
    }

    // Test unit variant with attributes
    let err = MixedError::UnitError;
    let (code, level, target) = err.error_info();
    assert_eq!(code, "UnitError");
    assert_eq!(level, "warn");
    assert_eq!(target, "unit::module");

    // Test struct variant with level only
    let err = MixedError::StructError {
        msg: "test".to_string(),
    };
    let (code, level, target) = err.error_info();
    assert_eq!(code, "StructError");
    assert_eq!(level, "info");
    assert!(target.contains("error_info_method"));

    // Test defaults for both unit and struct
    let err = MixedError::DefaultUnit;
    let (code, level, _target) = err.error_info();
    assert_eq!(code, "DefaultUnit");
    assert_eq!(level, "error");

    let err = MixedError::DefaultStruct { value: 42 };
    let (code, level, _target) = err.error_info();
    assert_eq!(code, "DefaultStruct");
    assert_eq!(level, "error");
}

#[test]
fn test_error_info_for_metrics_and_monitoring() {
    define_errors! {
        MetricsError {
            CriticalSystemFailure {} : "Critical system failure" [level = error, target = "system::critical"],
            PerformanceWarning { duration_ms: u64, threshold_ms: u64 } : "Performance threshold exceeded: {duration_ms}ms > {threshold_ms}ms" [level = warn, target = "metrics::performance"],
            UserActivity { user_id: u64, action: String } : "User {user_id} performed {action}" [level = info, target = "analytics::user"]
        }
    }

    // Simulate collecting metrics data
    let errors = vec![
        MetricsError::CriticalSystemFailure,
        MetricsError::PerformanceWarning {
            duration_ms: 1500,
            threshold_ms: 1000,
        },
        MetricsError::UserActivity {
            user_id: 12345,
            action: "login".to_string(),
        },
    ];

    let mut error_counts = std::collections::HashMap::new();
    let mut level_counts = std::collections::HashMap::new();
    let mut target_counts = std::collections::HashMap::new();

    for error in &errors {
        let (code, level, target) = error.error_info();

        *error_counts.entry(code).or_insert(0) += 1;
        *level_counts.entry(level).or_insert(0) += 1;
        *target_counts.entry(target).or_insert(0) += 1;
    }

    // Verify metrics can be collected correctly
    assert_eq!(error_counts.get("CriticalSystemFailure"), Some(&1));
    assert_eq!(error_counts.get("PerformanceWarning"), Some(&1));
    assert_eq!(error_counts.get("UserActivity"), Some(&1));

    assert_eq!(level_counts.get("error"), Some(&1));
    assert_eq!(level_counts.get("warn"), Some(&1));
    assert_eq!(level_counts.get("info"), Some(&1));

    assert_eq!(target_counts.get("system::critical"), Some(&1));
    assert_eq!(target_counts.get("metrics::performance"), Some(&1));
    assert_eq!(target_counts.get("analytics::user"), Some(&1));
}
