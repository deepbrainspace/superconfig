//! SuperConfig Error Handling Example
//!
//! This example demonstrates practical error handling patterns for configuration
//! management systems using LogFFI's define_errors! macro with tracing integration.
//!
//! Run with:
//! cargo run --example superconfig_errors
//!
//! Or with trace level:
//! RUST_LOG=trace cargo run --example superconfig_errors

use logffi::define_errors;

// Define SuperConfig's errors with proper source error chaining
define_errors! {
    pub enum ConfigError {
        // Simple errors - these work perfectly
        #[error("Key '{key}' not found in profile '{profile}'", level = warn, target = "superconfig::registry")]
        KeyNotFound {
            key: String,
            profile: String,
        },

        #[error("Profile '{profile}' not found", level = warn, target = "superconfig::profiles")]
        ProfileNotFound {
            profile: String,
        },

        // Errors with source chaining using #[source] attribute
        #[error("Failed to read config file '{path}'", level = error, target = "superconfig::io")]
        FileReadError {
            path: String,
            #[source]
            source: std::io::Error,  // Proper source error chaining
        },

        #[error("Failed to parse JSON from '{file}'", level = error, target = "superconfig::parse")]
        JsonParseError {
            file: String,
            #[source]
            source: serde_json::Error,  // Chain parse errors
        },

        // For cases where you need string details instead of source chaining
        #[error("Failed to parse YAML from '{file}': {details}", level = error, target = "superconfig::parse")]
        YamlParseError {
            file: String,
            details: String,
        },

        // Environment variable errors
        #[error("Environment variable '{var}' not found", level = info, target = "superconfig::env")]
        EnvVarNotFound {
            var: String,
        },

        // Network-related config errors
        #[error("Failed to fetch remote config from '{url}'", level = error, target = "superconfig::remote")]
        RemoteConfigError {
            url: String,
            #[source]
            source: Box<dyn std::error::Error + Send + Sync>,
        },
    }
}

fn main() {
    println!("\n🔧 SuperConfig Error Handling with LogFFI");
    println!("==========================================");
    println!("Auto-initializing tracing subscriber...\n");

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          What the define_errors! Macro Generates             ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("  ✅ Error enum with thiserror::Error derive");
    println!("  ✅ Display trait implementation (via thiserror)");
    println!("  ✅ Debug trait implementation");
    println!("  ✅ code() method for error codes");
    println!("  ✅ log() method with proper level and target");
    println!("  ✅ Source error chaining via #[source] attribute");

    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 1: Manual Error Creation (Old Way)                  │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    // Example 1: Simple error (manual creation)
    let error = ConfigError::KeyNotFound {
        key: "database.host".to_string(),
        profile: "production".to_string(),
    };

    println!("📝 Creating error manually...");
    println!("   Display: {}", error);
    println!("   Code: {}", error.code());
    println!("\n📤 Manually calling error.log()...");
    error.log();
    println!("   ↳ ⚠️  Log output appears above (WARN level)");

    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 2: Different Error Types and Levels                 │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    println!("🚀 Creating profile error (WARN level):");
    let error = ConfigError::ProfileNotFound {
        profile: "development".to_string(),
    };
    println!("   ✓ Created: {}", error);
    error.log();

    println!("\n🚀 Creating env var error (INFO level):");
    let env_error = ConfigError::EnvVarNotFound {
        var: "DATABASE_URL".to_string(),
    };
    println!("   ✓ Created: {}", env_error);
    env_error.log();

    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 3: Real-World IO Error Handling with Source Chain   │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    let file_path = "/etc/app/config.toml";
    println!("📁 Attempting to read: {}", file_path);
    let io_result = std::fs::read_to_string(file_path);

    if let Err(io_err) = io_result {
        println!("   ❌ IO operation failed!");
        println!("\n🚀 Creating error with source chaining:");
        let error = ConfigError::FileReadError {
            path: file_path.to_string(),
            source: io_err,
        };

        println!("   ✓ Primary error: {}", error);
        println!("   ✓ Error code: {}", error.code());

        // Show error chain
        use std::error::Error;
        if let Some(source) = error.source() {
            println!("   ✓ Source error: {}", source);
        }

        println!("\n📤 Logging the error:");
        error.log();
        println!("   ↳ Check ERROR log above");
    }

    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 4: JSON Parse Error with Constructor                │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    println!("📄 Parsing invalid JSON...");
    let json_result = serde_json::from_str::<serde_json::Value>("invalid json");

    if let Err(parse_err) = json_result {
        println!("   ❌ JSON parsing failed!");
        println!("\n🚀 Creating error with source chaining:");
        // With source errors, create directly (constructor methods need updating)
        let error = ConfigError::JsonParseError {
            file: "app.json".to_string(),
            source: parse_err,
        };

        println!("   ✓ Primary error: {}", error);
        println!("   ✓ Code: {} (auto-generated)", error.code());

        use std::error::Error;
        if let Some(source) = error.source() {
            println!("   ✓ Source error: {}", source);
        }

        println!("\n📤 Logging the error:");
        error.log();
    }

    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 5: Custom Error Code                                │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    println!("📄 Creating YAML error...");
    let error = ConfigError::YamlParseError {
        file: "config.yaml".to_string(),
        details: "invalid indentation at line 5".to_string(),
    };
    println!("   ✓ Error: {}", error);
    println!("   ✓ Code: {}", error.code());
    error.log();
    println!("   ✓ Logged at ERROR level");

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    Key Benefits                               ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    println!("  🎯 Automatic logging with proper levels and targets");
    println!("  🔗 Full source error chaining support");
    println!("  🛡️  Type-safe error creation");
    println!("  📊 Structured error codes for monitoring");
    println!("  💡 thiserror integration for clean Display messages");

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                   Log Output Format                           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    println!("  📊 Backend: log/tracing/slog (configurable)");
    println!("  📝 Format: [ERROR_CODE] Error message");
    println!("  🎯 Target: Module-specific (e.g., superconfig::io)");
    println!("  📈 Level: Configured per error (ERROR/WARN/INFO/DEBUG/TRACE)");

    println!("\n✨ Example complete! Check the log output above to see all the automatic logging.");
}

// What this gives SuperConfig:
// 1. ✅ Structured error types with automatic Display formatting
// 2. ✅ Consistent error logging without manual effort
// 3. ✅ Error codes for monitoring/alerting
// 4. ✅ Proper log levels (warn for missing keys, error for IO/parse failures)
// 5. ✅ Target-based filtering for debugging specific modules
// 6. ✅ FFI-friendly error identification via kind()
// 7. ✅ Proper source error chaining with #[source] attribute
// 8. ✅ Simple, explicit error creation patterns
// 9. ✅ Constructor methods (new_variant_name) that auto-log errors
// 10. ✅ Full error context via .source() method for debugging
