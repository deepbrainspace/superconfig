//! Field Interpolation Demo
//!
//! This example shows how field values are interpolated into error messages
//! using thiserror's automatic Display formatting.

use logffi::define_errors;
use std::collections::HashMap;

// Define errors with various field types and interpolation patterns
define_errors! {
    pub enum InterpolationDemo {
        // Simple string interpolation
        #[error("File '{filename}' not found in directory '{directory}'")]
        FileNotFound {
            filename: String,
            directory: String,
        },

        // Numeric interpolation with formatting
        #[error("Memory usage too high: {current}MB / {limit}MB ({percentage:.1}%)")]
        MemoryExhausted {
            current: u64,
            limit: u64,
            percentage: f64,
        },

        // Complex type interpolation
        #[error("Invalid configuration key '{key}' with value '{value:?}' in section '{section}'")]
        InvalidConfig {
            key: String,
            value: serde_json::Value,
            section: String,
        },

        // Multiple field types
        #[error("Database query failed: {query} (affected {rows} rows, took {duration_ms}ms)")]
        QueryFailed {
            query: String,
            rows: i32,
            duration_ms: u128,
        },

        // Collection interpolation
        #[error("Missing required fields: {missing_fields:?}")]
        MissingFields {
            missing_fields: Vec<String>,
        },

        // Custom formatting with nested fields
        #[error("User {user_id} attempted {action} on resource {resource_type}:{resource_id} but lacks permission '{required_permission}'")]
        PermissionDenied {
            user_id: u64,
            action: String,
            resource_type: String,
            resource_id: String,
            required_permission: String,
        },
    }
}

fn main() {
    println!("🔧 LogFFI Field Interpolation Demo");
    println!("==================================\n");

    // Example 1: Simple string interpolation
    println!("📁 Example 1: File operations");
    let file_error = InterpolationDemo::FileNotFound {
        filename: "config.toml".to_string(),
        directory: "/etc/myapp".to_string(),
    };
    println!("   Error: {}", file_error);
    println!("   Code: {}", file_error.code());
    file_error.log();
    println!();

    // Example 2: Numeric formatting
    println!("💾 Example 2: Memory monitoring");
    let memory_error = InterpolationDemo::MemoryExhausted {
        current: 1542,
        limit: 1024,
        percentage: (1542.0 / 1024.0) * 100.0,
    };
    println!("   Error: {}", memory_error);
    memory_error.log();
    println!();

    // Example 3: Complex types (JSON values)
    println!("⚙️  Example 3: Configuration validation");
    let json_value = serde_json::json!({
        "timeout": "invalid_duration",
        "retries": -5
    });
    let config_error = InterpolationDemo::InvalidConfig {
        key: "database.connection".to_string(),
        value: json_value,
        section: "production".to_string(),
    };
    println!("   Error: {}", config_error);
    config_error.log();
    println!();

    // Example 4: Database operations
    println!("🗄️  Example 4: Database operations");
    let query_error = InterpolationDemo::QueryFailed {
        query: "UPDATE users SET last_login = NOW() WHERE active = true".to_string(),
        rows: 0,
        duration_ms: 2847,
    };
    println!("   Error: {}", query_error);
    query_error.log();
    println!();

    // Example 5: Collections
    println!("📋 Example 5: Validation with collections");
    let validation_error = InterpolationDemo::MissingFields {
        missing_fields: vec![
            "email".to_string(),
            "username".to_string(),
            "password".to_string(),
        ],
    };
    println!("   Error: {}", validation_error);
    validation_error.log();
    println!();

    // Example 6: Complex permission system
    println!("🔐 Example 6: Permission system");
    let permission_error = InterpolationDemo::PermissionDenied {
        user_id: 12345,
        action: "delete".to_string(),
        resource_type: "document".to_string(),
        resource_id: "contract_2024_001".to_string(),
        required_permission: "documents:delete".to_string(),
    };
    println!("   Error: {}", permission_error);
    permission_error.log();
    println!();

    println!("🎯 Key Points:");
    println!("   • thiserror automatically interpolates {{field_name}} patterns");
    println!("   • Any type that implements Display can be interpolated");
    println!("   • Use {{field:?}} for Debug formatting (like Vec, HashMap)");
    println!("   • Use {{field:.precision}} for number formatting");
    println!("   • Complex types like JSON values work seamlessly");
    println!("   • Our LogFFI macro preserves all thiserror functionality");
}
