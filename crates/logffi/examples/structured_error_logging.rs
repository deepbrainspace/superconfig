use logffi::define_errors;
use serde_json::json;
use std::time::SystemTime;

// Example: Structured logging integration using error_info()

define_errors! {
    AppError {
        DatabaseConnection {} : "Database connection failed" [level = error, target = "app::db"],
        ApiRateLimit { limit: u32 } : "API rate limit exceeded: {limit}" [level = warn, target = "app::api"],
        UserAuthFailure { user_id: u64 } : "User authentication failed for ID: {user_id}" [level = info, target = "app::auth"],
        ValidationError { field: String, value: String } : "Validation failed for {field}: '{value}'" [level = warn, target = "app::validation"]
    }
}

/// Log AppError with structured context using error_info()
fn log_error_with_context(error: &AppError, context: &str) {
    let (code, level, target) = error.error_info();
    let log_entry = json!({
        "error_code": code,
        "level": level,
        "target": target,
        "context": context,
        "timestamp": SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        "message": error.to_string()
    });

    println!("{}", serde_json::to_string_pretty(&log_entry).unwrap());
}

/// Advanced logging with additional metadata for AppError
fn log_error_with_metadata(error: &AppError, context: &str, metadata: serde_json::Value) {
    let (code, level, target) = error.error_info();
    let mut log_entry = json!({
        "error_code": code,
        "level": level,
        "target": target,
        "context": context,
        "timestamp": SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        "message": error.to_string()
    });

    // Merge additional metadata
    if let serde_json::Value::Object(ref mut map) = log_entry {
        if let serde_json::Value::Object(metadata_map) = metadata {
            for (key, value) in metadata_map {
                map.insert(key, value);
            }
        }
    }

    println!("{}", serde_json::to_string_pretty(&log_entry).unwrap());
}

/// Batch logging for multiple AppErrors
fn log_error_batch(errors: &[AppError], context: &str) {
    let batch_entry = json!({
        "batch_context": context,
        "timestamp": SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        "error_count": errors.len(),
        "errors": errors.iter().map(|error| {
            let (code, level, target) = error.error_info();
            json!({
                "error_code": code,
                "level": level,
                "target": target,
                "message": error.to_string()
            })
        }).collect::<Vec<_>>()
    });

    println!("{}", serde_json::to_string_pretty(&batch_entry).unwrap());
}

fn main() {
    println!("🏗️  LogFFI Structured Error Logging Demo");
    println!("=========================================\n");

    // Initialize LogFFI for automatic error logging
    logffi::info!("Starting structured error logging demo");

    println!("1. Basic Structured Error Logging:");
    println!("----------------------------------");
    let db_error = AppError::DatabaseConnection;
    log_error_with_context(&db_error, "User registration flow");

    println!("\n2. Error with Field Data:");
    println!("-------------------------");
    let rate_error = AppError::ApiRateLimit { limit: 100 };
    log_error_with_context(&rate_error, "API endpoint throttling");

    println!("\n3. Enhanced Logging with Metadata:");
    println!("----------------------------------");
    let auth_error = AppError::UserAuthFailure { user_id: 12345 };
    let metadata = json!({
        "request_id": "req-abc-123",
        "ip_address": "192.168.1.100",
        "user_agent": "Mozilla/5.0...",
        "session_id": "sess-xyz-789"
    });
    log_error_with_metadata(&auth_error, "Login attempt", metadata);

    println!("\n4. Batch Error Logging:");
    println!("-----------------------");
    let validation_errors = vec![
        AppError::ValidationError {
            field: "email".to_string(),
            value: "not-an-email".to_string(),
        },
        AppError::ValidationError {
            field: "phone".to_string(),
            value: "invalid-phone".to_string(),
        },
        AppError::ValidationError {
            field: "age".to_string(),
            value: "-5".to_string(),
        },
    ];
    log_error_batch(&validation_errors, "Form validation");

    println!("\n5. Log Aggregation Simulation:");
    println!("------------------------------");
    simulate_log_aggregation();

    logffi::info!("Structured error logging demo completed");
}

/// Simulate how these structured logs would be processed by log aggregation systems
fn simulate_log_aggregation() {
    let errors = vec![
        AppError::DatabaseConnection,
        AppError::ApiRateLimit { limit: 50 },
        AppError::UserAuthFailure { user_id: 67890 },
        AppError::ValidationError {
            field: "password".to_string(),
            value: "weak".to_string(),
        },
    ];

    // Simulate ELK/Grafana-style aggregation
    let aggregated = json!({
        "aggregation_timestamp": SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        "time_window": "last_5_minutes",
        "summary": {
            "total_errors": errors.len(),
            "by_level": {
                "error": errors.iter().filter(|e| e.error_info().1 == "error").count(),
                "warn": errors.iter().filter(|e| e.error_info().1 == "warn").count(),
                "info": errors.iter().filter(|e| e.error_info().1 == "info").count(),
            },
            "by_target": {
                "app::db": errors.iter().filter(|e| e.error_info().2.contains("db")).count(),
                "app::api": errors.iter().filter(|e| e.error_info().2.contains("api")).count(),
                "app::auth": errors.iter().filter(|e| e.error_info().2.contains("auth")).count(),
                "app::validation": errors.iter().filter(|e| e.error_info().2.contains("validation")).count(),
            },
            "top_errors": errors.iter().map(|e| e.error_info().0).collect::<Vec<_>>()
        },
        "alert_conditions": check_alert_conditions(&errors)
    });

    println!("📊 Log Aggregation Report:");
    println!("{}", serde_json::to_string_pretty(&aggregated).unwrap());
}

/// Check for alert conditions based on error patterns
fn check_alert_conditions(errors: &[AppError]) -> Vec<String> {
    let mut alerts = Vec::new();

    let error_count = errors
        .iter()
        .filter(|e| e.error_info().1 == "error")
        .count();
    if error_count >= 1 {
        alerts.push(format!(
            "CRITICAL: {} error-level events detected",
            error_count
        ));
    }

    let db_errors = errors
        .iter()
        .filter(|e| e.error_info().2.contains("db"))
        .count();
    if db_errors >= 1 {
        alerts.push("DATABASE: Connection issues detected".to_string());
    }

    let auth_failures = errors
        .iter()
        .filter(|e| matches!(e, AppError::UserAuthFailure { .. }))
        .count();
    if auth_failures >= 1 {
        alerts.push(format!(
            "SECURITY: {} authentication failures",
            auth_failures
        ));
    }

    alerts
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_error_info_extraction() {
        let error = AppError::DatabaseConnection;
        let (code, level, target) = error.error_info();

        assert_eq!(code, "DatabaseConnection");
        assert_eq!(level, "error");
        assert_eq!(target, "app::db");
    }

    #[test]
    fn test_structured_logging_format() {
        let error = AppError::ApiRateLimit { limit: 100 };
        let (code, level, target) = error.error_info();

        // Simulate what log_error_with_context would create
        let log_entry = json!({
            "error_code": code,
            "level": level,
            "target": target,
            "context": "test_context",
            "message": error.to_string()
        });

        assert_eq!(log_entry["error_code"], "ApiRateLimit");
        assert_eq!(log_entry["level"], "warn");
        assert_eq!(log_entry["target"], "app::api");
        assert_eq!(log_entry["context"], "test_context");
    }

    #[test]
    fn test_batch_error_processing() {
        let errors = vec![
            AppError::DatabaseConnection,
            AppError::ApiRateLimit { limit: 50 },
        ];

        let error_codes: Vec<_> = errors.iter().map(|e| e.error_info().0).collect();
        assert_eq!(error_codes, vec!["DatabaseConnection", "ApiRateLimit"]);

        let error_levels: Vec<_> = errors.iter().map(|e| e.error_info().1).collect();
        assert_eq!(error_levels, vec!["error", "warn"]);
    }

    #[test]
    fn test_alert_conditions() {
        let errors = vec![
            AppError::DatabaseConnection,
            AppError::UserAuthFailure { user_id: 123 },
        ];

        let alerts = check_alert_conditions(&errors);

        assert!(alerts.iter().any(|alert| alert.contains("CRITICAL")));
        assert!(alerts.iter().any(|alert| alert.contains("DATABASE")));
        assert!(alerts.iter().any(|alert| alert.contains("SECURITY")));
    }
}
