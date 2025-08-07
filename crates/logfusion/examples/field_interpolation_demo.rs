//! Field Interpolation Demo - Dual Syntax Showcase
//!
//! This example demonstrates field interpolation in both LogFusion v0.2 formats:
//! - 🆕 LogFusion Format: Clean, attribute-based syntax
//! - 🔧 Thiserror Format: Traditional compatibility syntax
//!
//! Both use thiserror's automatic Display formatting for field interpolation.

use logfusion::define_errors;

// 🆕 LogFusion Format - Clean, modern syntax for field interpolation
define_errors! {
    LogFusionInterpolation {
        // Simple string interpolation
        FileNotFound { filename: String, directory: String } : "File '{filename}' not found in directory '{directory}'" [level = warn],

        // Numeric interpolation with formatting
        MemoryExhausted { current: u64, limit: u64, percentage: f64 } : "Memory usage too high: {current}MB / {limit}MB ({percentage:.1}%)" [level = error],

        // Complex type interpolation
        InvalidConfig { key: String, value: serde_json::Value, section: String } : "Invalid configuration key '{key}' with value '{value:?}' in section '{section}'" [level = error, target = "config"],

        // Multiple field types with structured logging
        QueryFailed { query: String, rows: i32, duration_ms: u128 } : "Database query failed: {query} (affected {rows} rows, took {duration_ms}ms)" [level = warn, target = "database"],

        // Collection interpolation
        MissingFields { missing_fields: Vec<String> } : "Missing required fields: {missing_fields:?}" [level = warn],

        // Complex permission system with many fields
        PermissionDenied {
            user_id: u64,
            action: String,
            resource_type: String,
            resource_id: String,
            required_permission: String
        } : "User {user_id} attempted {action} on resource {resource_type}:{resource_id} but lacks permission '{required_permission}'" [level = error, target = "security"]
    }
}

// 🔧 Traditional Thiserror Format - For comparison and backward compatibility
define_errors! {
    pub enum ThiserrorInterpolation {
        // Simple string interpolation
        #[error("File '{filename}' not found in directory '{directory}'", level = warn)]
        FileNotFound {
            filename: String,
            directory: String,
        },

        // Numeric interpolation with formatting
        #[error("Memory usage too high: {current}MB / {limit}MB ({percentage:.1}%)", level = error)]
        MemoryExhausted {
            current: u64,
            limit: u64,
            percentage: f64,
        },

        // Complex type interpolation
        #[error("Invalid configuration key '{key}' with value '{value:?}' in section '{section}'", level = error, target = "config")]
        InvalidConfig {
            key: String,
            value: serde_json::Value,
            section: String,
        },

        // Multiple field types
        #[error("Database query failed: {query} (affected {rows} rows, took {duration_ms}ms)", level = warn, target = "database")]
        QueryFailed {
            query: String,
            rows: i32,
            duration_ms: u128,
        },

        // Collection interpolation
        #[error("Missing required fields: {missing_fields:?}", level = warn)]
        MissingFields {
            missing_fields: Vec<String>,
        },

        // Custom formatting with nested fields
        #[error("User {user_id} attempted {action} on resource {resource_type}:{resource_id} but lacks permission '{required_permission}'", level = error, target = "security")]
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
    println!("🔧 LogFusion Field Interpolation Demo - Dual Syntax");
    println!("==================================================");
    println!("Comparing LogFusion v0.2 format vs traditional thiserror syntax\n");

    demonstrate_logfusion_format();
    demonstrate_thiserror_format();
    show_syntax_comparison();

    println!("\n🎯 Key Points:");
    println!("   • Both formats use thiserror's {{field_name}} interpolation");
    println!("   • LogFusion format is cleaner: no repetitive #[error(...)] attributes");
    println!(
        "   • LogFusion format supports attribute-based logging: [level = warn, target = \"db\"]"
    );
    println!("   • Any type that implements Display can be interpolated");
    println!("   • Use {{field:?}} for Debug formatting (like Vec, HashMap)");
    println!("   • Use {{field:.precision}} for number formatting");
    println!("   • Complex types like JSON values work seamlessly");
    println!("   • Full backward compatibility - migrate at your own pace");
}

fn demonstrate_logfusion_format() {
    println!("🆕 LogFusion Format Examples");
    println!("-------------------------");

    // Example 1: Simple string interpolation
    println!("📁 File operations (LogFusion format):");
    let file_error = LogFusionInterpolation::FileNotFound {
        filename: "config.toml".to_string(),
        directory: "/etc/myapp".to_string(),
    };
    println!("   Error: {}", file_error);
    println!("   Code: {}", file_error.code());
    file_error.log(); // WARN level
    println!();

    // Example 2: Numeric formatting
    println!("💾 Memory monitoring (LogFusion format):");
    let memory_error = LogFusionInterpolation::MemoryExhausted {
        current: 1542,
        limit: 1024,
        percentage: (1542.0 / 1024.0) * 100.0,
    };
    println!("   Error: {}", memory_error);
    memory_error.log(); // ERROR level
    println!();

    // Example 3: Complex types with target
    println!("⚙️  Configuration with custom target (LogFusion format):");
    let json_value = serde_json::json!({
        "timeout": "invalid_duration",
        "retries": -5
    });
    let config_error = LogFusionInterpolation::InvalidConfig {
        key: "database.connection".to_string(),
        value: json_value,
        section: "production".to_string(),
    };
    println!("   Error: {}", config_error);
    config_error.log(); // ERROR level, target = "config"
    println!();

    // Example 4: Database operations with target
    println!("🗄️  Database operations with target (LogFusion format):");
    let query_error = LogFusionInterpolation::QueryFailed {
        query: "UPDATE users SET last_login = NOW() WHERE active = true".to_string(),
        rows: 0,
        duration_ms: 2847,
    };
    println!("   Error: {}", query_error);
    query_error.log(); // WARN level, target = "database"
    println!();

    // Example 5: Collections
    println!("📋 Validation with collections (LogFusion format):");
    let validation_error = LogFusionInterpolation::MissingFields {
        missing_fields: vec![
            "email".to_string(),
            "username".to_string(),
            "password".to_string(),
        ],
    };
    println!("   Error: {}", validation_error);
    validation_error.log(); // WARN level
    println!();

    // Example 6: Complex permission system with security target
    println!("🔐 Permission system with security target (LogFusion format):");
    let permission_error = LogFusionInterpolation::PermissionDenied {
        user_id: 12345,
        action: "delete".to_string(),
        resource_type: "document".to_string(),
        resource_id: "contract_2024_001".to_string(),
        required_permission: "documents:delete".to_string(),
    };
    println!("   Error: {}", permission_error);
    permission_error.log(); // ERROR level, target = "security"
    println!();
}

fn demonstrate_thiserror_format() {
    println!("🔧 Traditional Thiserror Format Examples");
    println!("-----------------------------------------");

    // Same examples using traditional syntax
    println!("📁 File operations (thiserror format):");
    let file_error = ThiserrorInterpolation::FileNotFound {
        filename: "config.toml".to_string(),
        directory: "/etc/myapp".to_string(),
    };
    println!("   Error: {}", file_error);
    println!("   Code: {}", file_error.code());
    file_error.log(); // WARN level
    println!();

    println!("💾 Memory monitoring (thiserror format):");
    let memory_error = ThiserrorInterpolation::MemoryExhausted {
        current: 1542,
        limit: 1024,
        percentage: (1542.0 / 1024.0) * 100.0,
    };
    println!("   Error: {}", memory_error);
    memory_error.log(); // ERROR level
    println!();

    println!("🔐 Permission system (thiserror format):");
    let permission_error = ThiserrorInterpolation::PermissionDenied {
        user_id: 67890,
        action: "modify".to_string(),
        resource_type: "user_profile".to_string(),
        resource_id: "profile_xyz_789".to_string(),
        required_permission: "users:modify".to_string(),
    };
    println!("   Error: {}", permission_error);
    permission_error.log(); // ERROR level, target = "security"
    println!();
}

fn show_syntax_comparison() {
    println!("⚖️  Syntax Comparison");
    println!("--------------------");
    println!("🆕 LogFusion Format:");
    println!(
        "   ValidationError {{ field: String, reason: String }} : \"Field {{field}} failed: {{reason}}\" [level = warn]"
    );
    println!();
    println!("🔧 Thiserror Format:");
    println!("   #[error(\"Field {{field}} failed: {{reason}}\", level = warn)]");
    println!("   ValidationError {{ field: String, reason: String }},");
    println!();
    println!("✅ Benefits of LogFusion Format:");
    println!("   • Cleaner syntax - no repetitive #[error(...)] attributes");
    println!("   • Attribute-based logging: [level = warn, target = \"validation\"]");
    println!("   • Mixed variants: unit {{}} and struct variants in same enum");
    println!("   • Multiple error types in single macro call");
    println!("   • Auto source detection for fields named 'source'");
    println!("   • 64% macro size reduction (998 → 358 lines)");
}
