//! Integration tests for error analytics and business intelligence
//!
//! Tests collecting error_info() across multiple error types and scenarios

use logfusion::define_errors;
use serde_json::{Value, json};
use std::collections::HashMap;

#[test]
fn test_error_analytics_pipeline() {
    // Test collecting error_info() across multiple error types
    // for analytics and business intelligence

    define_errors! {
        BusinessError {
            PaymentFailed { transaction_id: String, amount: f64 } : "Payment failed for transaction {transaction_id}: ${amount}" [level = error, target = "business::payment"],
            UserRegistration { email: String } : "User registration failed: {email}" [level = warn, target = "business::user"],
            OrderProcessing { order_id: u64 } : "Order processing delayed: {order_id}" [level = info, target = "business::order"]
        }

        TechnicalError {
            DatabaseConnection {} : "Database connection failed" [level = error, target = "tech::db"],
            CacheTimeout { key: String } : "Cache timeout for key: {key}" [level = warn, target = "tech::cache"],
            ApiResponse { status: u16 } : "API returned status {status}" [level = info, target = "tech::api"]
        }
    }

    // Simulate business and technical errors
    let business_errors = vec![
        BusinessError::PaymentFailed {
            transaction_id: "tx_123".to_string(),
            amount: 99.99,
        },
        BusinessError::UserRegistration {
            email: "user@example.com".to_string(),
        },
        BusinessError::OrderProcessing { order_id: 12345 },
    ];

    let technical_errors = vec![
        TechnicalError::DatabaseConnection,
        TechnicalError::CacheTimeout {
            key: "user_session_456".to_string(),
        },
        TechnicalError::ApiResponse { status: 429 },
    ];

    // Analytics pipeline: collect error_info from different error types
    let mut analytics_data = HashMap::new();

    // Collect business error analytics
    for error in &business_errors {
        let (code, level, target) = error.error_info();
        let key = format!("business_{}_{}", target.replace("::", "_"), code);
        analytics_data.insert(
            key,
            json!({
                "error_code": code,
                "level": level,
                "target": target,
                "category": "business",
                "message": error.to_string()
            }),
        );
    }

    // Collect technical error analytics
    for error in &technical_errors {
        let (code, level, target) = error.error_info();
        let key = format!("technical_{}_{}", target.replace("::", "_"), code);
        analytics_data.insert(
            key,
            json!({
                "error_code": code,
                "level": level,
                "target": target,
                "category": "technical",
                "message": error.to_string()
            }),
        );
    }

    // Verify analytics pipeline collected all errors
    assert_eq!(analytics_data.len(), 6);

    // Verify business analytics
    let payment_analytics = &analytics_data["business_business_payment_PaymentFailed"];
    assert_eq!(payment_analytics["level"], "error");
    assert_eq!(payment_analytics["category"], "business");

    let user_analytics = &analytics_data["business_business_user_UserRegistration"];
    assert_eq!(user_analytics["level"], "warn");
    assert_eq!(user_analytics["target"], "business::user");

    // Verify technical analytics
    let db_analytics = &analytics_data["technical_tech_db_DatabaseConnection"];
    assert_eq!(db_analytics["level"], "error");
    assert_eq!(db_analytics["category"], "technical");

    let cache_analytics = &analytics_data["technical_tech_cache_CacheTimeout"];
    assert_eq!(cache_analytics["level"], "warn");
    assert_eq!(cache_analytics["target"], "tech::cache");

    // Generate business intelligence summary
    let mut level_counts = HashMap::new();
    let mut category_counts = HashMap::new();
    let mut target_counts = HashMap::new();

    for data in analytics_data.values() {
        let level = data["level"].as_str().unwrap();
        let category = data["category"].as_str().unwrap();
        let target = data["target"].as_str().unwrap();

        *level_counts.entry(level).or_insert(0) += 1;
        *category_counts.entry(category).or_insert(0) += 1;
        *target_counts.entry(target).or_insert(0) += 1;
    }

    // Verify business intelligence metrics
    assert_eq!(level_counts["error"], 2); // PaymentFailed + DatabaseConnection
    assert_eq!(level_counts["warn"], 2); // UserRegistration + CacheTimeout  
    assert_eq!(level_counts["info"], 2); // OrderProcessing + ApiResponse

    assert_eq!(category_counts["business"], 3);
    assert_eq!(category_counts["technical"], 3);

    // Verify target-specific insights
    assert!(target_counts.contains_key("business::payment"));
    assert!(target_counts.contains_key("tech::db"));
    assert!(target_counts.contains_key("tech::cache"));
}

#[test]
fn test_error_info_serialization() {
    // Test that error_info() tuples serialize correctly
    // for external monitoring systems

    define_errors! {
        SerializationError {
            JsonParsing { file: String, line: u32 } : "JSON parsing failed in {file} at line {line}" [level = error, target = "parser::json"],
            ConfigValidation { key: String, expected: String } : "Config validation failed for {key}: expected {expected}" [level = warn, target = "config::validation"],
            DataTransform {} : "Data transformation completed" [level = info, target = "transform::data"]
        }
    }

    let errors = vec![
        SerializationError::JsonParsing {
            file: "config.json".to_string(),
            line: 42,
        },
        SerializationError::ConfigValidation {
            key: "database_url".to_string(),
            expected: "string".to_string(),
        },
        SerializationError::DataTransform,
    ];

    // Test serializing error_info() tuples for monitoring systems
    let mut monitoring_payload = Vec::new();

    for error in &errors {
        let (code, level, target) = error.error_info();

        // Create serializable monitoring entry
        let monitoring_entry = json!({
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "error_info": {
                "code": code,
                "level": level,
                "target": target
            },
            "message": error.to_string(),
            "metadata": {
                "service": "logfusion-test",
                "version": "0.1.0",
                "environment": "test"
            }
        });

        monitoring_payload.push(monitoring_entry);
    }

    // Verify serialization worked correctly
    assert_eq!(monitoring_payload.len(), 3);

    // Test JSON serialization/deserialization
    let json_payload = serde_json::to_string(&monitoring_payload).unwrap();
    let deserialized: Vec<Value> = serde_json::from_str(&json_payload).unwrap();

    assert_eq!(deserialized.len(), 3);

    // Verify first error serialization
    let first_error = &deserialized[0];
    assert_eq!(first_error["error_info"]["code"], "JsonParsing");
    assert_eq!(first_error["error_info"]["level"], "error");
    assert_eq!(first_error["error_info"]["target"], "parser::json");
    assert!(
        first_error["message"]
            .as_str()
            .unwrap()
            .contains("config.json")
    );

    // Verify second error serialization
    let second_error = &deserialized[1];
    assert_eq!(second_error["error_info"]["code"], "ConfigValidation");
    assert_eq!(second_error["error_info"]["level"], "warn");
    assert_eq!(second_error["error_info"]["target"], "config::validation");

    // Verify third error serialization
    let third_error = &deserialized[2];
    assert_eq!(third_error["error_info"]["code"], "DataTransform");
    assert_eq!(third_error["error_info"]["level"], "info");
    assert_eq!(third_error["error_info"]["target"], "transform::data");

    // Test compatibility with external monitoring formats

    // Prometheus-style metrics
    let prometheus_metrics: Vec<String> = deserialized
        .iter()
        .map(|entry| {
            let code = entry["error_info"]["code"].as_str().unwrap();
            let level = entry["error_info"]["level"].as_str().unwrap();
            let target = entry["error_info"]["target"].as_str().unwrap();

            format!(
                "logfusion_errors_total{{error_code=\"{}\",level=\"{}\",target=\"{}\"}} 1",
                code, level, target
            )
        })
        .collect();

    assert_eq!(prometheus_metrics.len(), 3);
    assert!(prometheus_metrics[0].contains("error_code=\"JsonParsing\""));
    assert!(prometheus_metrics[1].contains("level=\"warn\""));
    assert!(prometheus_metrics[2].contains("target=\"transform::data\""));

    // ELK Stack format
    let elk_entries: Vec<Value> = deserialized
        .iter()
        .map(|entry| {
            json!({
                "@timestamp": entry["timestamp"],
                "level": entry["error_info"]["level"],
                "logger": entry["error_info"]["target"],
                "message": entry["message"],
                "fields": {
                    "error_code": entry["error_info"]["code"],
                    "service": entry["metadata"]["service"],
                    "environment": entry["metadata"]["environment"]
                }
            })
        })
        .collect();

    assert_eq!(elk_entries.len(), 3);
    assert_eq!(elk_entries[0]["level"], "error");
    assert_eq!(elk_entries[0]["logger"], "parser::json");
    assert_eq!(elk_entries[0]["fields"]["error_code"], "JsonParsing");
}

#[test]
fn test_multi_crate_error_info_consistency() {
    // Test error_info() behavior across different crate boundaries
    // This simulates how errors would behave when used across multiple crates

    // Define errors that might come from different logical crates/modules
    define_errors! {
        CoreError {
            SystemFailure {} : "Core system failure" [level = error, target = "core::system"],
            ConfigMissing { key: String } : "Missing configuration: {key}" [level = error, target = "core::config"]
        }

        ApiError {
            RequestTimeout { endpoint: String } : "Request timeout: {endpoint}" [level = warn, target = "api::request"],
            ValidationFailed { field: String } : "Validation failed: {field}" [level = warn, target = "api::validation"]
        }

        DataError {
            ParseFailure { format: String } : "Parse failure: {format}" [level = error, target = "data::parser"],
            TransformError { operation: String } : "Transform error: {operation}" [level = warn, target = "data::transform"]
        }
    }

    // Test that error_info() returns consistent format across different "crates"
    let core_errors = vec![
        CoreError::SystemFailure,
        CoreError::ConfigMissing {
            key: "database_url".to_string(),
        },
    ];

    let api_errors = vec![
        ApiError::RequestTimeout {
            endpoint: "/api/users".to_string(),
        },
        ApiError::ValidationFailed {
            field: "email".to_string(),
        },
    ];

    let data_errors = vec![
        DataError::ParseFailure {
            format: "JSON".to_string(),
        },
        DataError::TransformError {
            operation: "normalize".to_string(),
        },
    ];

    // Verify error_info() consistency across all "crate" boundaries
    let mut all_error_info = Vec::new();

    // Collect from core errors
    for error in &core_errors {
        let (code, level, target) = error.error_info();
        all_error_info.push((code, level, target, "core"));
    }

    // Collect from API errors
    for error in &api_errors {
        let (code, level, target) = error.error_info();
        all_error_info.push((code, level, target, "api"));
    }

    // Collect from data errors
    for error in &data_errors {
        let (code, level, target) = error.error_info();
        all_error_info.push((code, level, target, "data"));
    }

    // Verify consistency: all error_info() tuples have the expected structure
    assert_eq!(all_error_info.len(), 6);

    for (code, level, target, crate_name) in &all_error_info {
        // All codes should be non-empty strings
        assert!(!code.is_empty());

        // All levels should be valid log levels
        assert!(matches!(
            *level,
            "error" | "warn" | "info" | "debug" | "trace"
        ));

        // All targets should follow the expected pattern
        assert!(target.contains("::"));
        assert!(target.starts_with(crate_name));

        // Targets should be consistent with their crate
        match *crate_name {
            "core" => assert!(target.starts_with("core::")),
            "api" => assert!(target.starts_with("api::")),
            "data" => assert!(target.starts_with("data::")),
            _ => panic!("Unexpected crate name: {}", crate_name),
        }
    }

    // Test cross-crate error aggregation
    let mut cross_crate_analytics = HashMap::new();

    for (code, level, target, crate_name) in &all_error_info {
        let analytics_key = format!("{}::{}", crate_name, level);
        let entry = cross_crate_analytics
            .entry(analytics_key)
            .or_insert_with(|| {
                json!({
                    "crate": crate_name,
                    "level": level,
                    "count": 0,
                    "errors": Vec::<Value>::new()
                })
            });

        entry["count"] = json!(entry["count"].as_u64().unwrap() + 1);
        entry["errors"].as_array_mut().unwrap().push(json!({
            "code": code,
            "target": target
        }));
    }

    // Verify cross-crate analytics
    assert_eq!(cross_crate_analytics.len(), 4); // core::error, api::warn, data::error, data::warn

    // Verify specific crate analytics
    let core_errors_analytics = &cross_crate_analytics["core::error"];
    assert_eq!(core_errors_analytics["count"], 2);
    assert_eq!(core_errors_analytics["crate"], "core");
    assert_eq!(core_errors_analytics["level"], "error");

    let api_warn_analytics = &cross_crate_analytics["api::warn"];
    assert_eq!(api_warn_analytics["count"], 2);
    assert_eq!(api_warn_analytics["crate"], "api");

    // Test that error_info() maintains consistency when errors are moved between collections
    let mut mixed_errors = Vec::new();

    // Mix errors from different sources
    mixed_errors.push(core_errors[0].error_info());
    mixed_errors.push(api_errors[0].error_info());
    mixed_errors.push(data_errors[0].error_info());

    // Convert to owned tuples for easier comparison
    let mixed_errors: Vec<(String, String, String)> = mixed_errors
        .into_iter()
        .map(|(c, l, t)| (c.to_string(), l.to_string(), t.to_string()))
        .collect();

    // Verify mixed collection maintains proper structure
    assert_eq!(mixed_errors.len(), 3);
    assert_eq!(mixed_errors[0].0, "SystemFailure");
    assert_eq!(mixed_errors[1].0, "RequestTimeout");
    assert_eq!(mixed_errors[2].0, "ParseFailure");

    // All should maintain their original targets despite being in the same collection
    assert_eq!(mixed_errors[0].2, "core::system");
    assert_eq!(mixed_errors[1].2, "api::request");
    assert_eq!(mixed_errors[2].2, "data::parser");
}
