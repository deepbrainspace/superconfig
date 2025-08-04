// Example: What SuperConfig Actually Needs from define_errors!
//
// This shows the practical, working approach for SuperConfig's error handling

use logffi::define_errors;

// Define SuperConfig's errors using the current working macro
define_errors! {
    #[derive(Clone)]
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

        // For errors with source context, we store the error details as strings
        #[error("Failed to read config file '{path}': {io_error}", level = error, target = "superconfig::io")]
        FileReadError {
            path: String,
            io_error: String,  // Store IO error details as string
        },

        #[error("Failed to parse JSON from '{file}': {details}", level = error, target = "superconfig::parse")]
        JsonParseError {
            file: String,
            details: String,  // Parse error details
        },

        #[error("Failed to parse YAML from '{file}': {details}", level = error, target = "superconfig::parse", code = "YAML_001")]
        YamlParseError {
            file: String,
            details: String,
        },
    }
}

fn main() {
    // Initialize logging
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║          What the define_errors! Macro Generates             ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("  ✅ Error enum with thiserror::Error derive");
    println!("  ✅ Display trait implementation (via thiserror)");
    println!("  ✅ Debug trait implementation");
    println!("  ✅ code() method for error codes");
    println!("  ✅ kind() method for FFI error type mapping");
    println!("  ✅ full_message_chain() method");
    println!("  ✅ log() method with proper level and target");
    println!("  ✅ new_<variant_name>() constructor methods that auto-log");

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
    println!("   Kind: {}", error.kind());
    println!("\n📤 Manually calling error.log()...");
    error.log();
    println!("   ↳ ⚠️  Log output appears above (WARN level)");

    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 2: Constructor Methods (NEW Recommended Way) 🎉     │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    println!("🚀 Using new_key_not_found() constructor:");
    let error = ConfigError::new_key_not_found(
        "database.port".to_string(),
        "staging".to_string(),
    );
    println!("   ✓ Created: {}", error);
    println!("   ✓ Automatically logged! (see WARN above)");

    println!("\n🚀 Using new_profile_not_found() constructor:");
    let error = ConfigError::new_profile_not_found("development".to_string());
    println!("   ✓ Created: {}", error);
    println!("   ✓ Automatically logged! (see WARN above)");

    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 3: Real-World IO Error Handling                     │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    let file_path = "/etc/app/config.toml";
    println!("📁 Attempting to read: {}", file_path);
    let io_result = std::fs::read_to_string(file_path);

    if let Err(io_err) = io_result {
        println!("   ❌ IO operation failed!");
        println!("\n🚀 Using new_file_read_error() constructor:");
        let error = ConfigError::new_file_read_error(
            file_path.to_string(),
            io_err.to_string(),
        );
        println!("   ✓ Error created and logged in one line!");
        println!("   ✓ Error: {}", error);
        println!("   ✓ Code: {}", error.code());
        println!("   ✓ Check ERROR log above");
    }

    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 4: JSON Parse Error with Constructor                │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    println!("📄 Parsing invalid JSON...");
    let json_result = serde_json::from_str::<serde_json::Value>("invalid json");

    if let Err(parse_err) = json_result {
        println!("   ❌ JSON parsing failed!");
        println!("\n🚀 Using new_json_parse_error() constructor:");
        let error = ConfigError::new_json_parse_error(
            "app.json".to_string(),
            format!("at line {}, column {}", parse_err.line(), parse_err.column()),
        );
        println!("   ✓ Error: {}", error);
        println!("   ✓ Code: {} (auto-generated)", error.code());
        println!("   ✓ Already logged! (see ERROR above)");
    }

    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 5: Custom Error Code                                │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    println!("📄 Creating YAML error with custom code...");
    let error = ConfigError::new_yaml_parse_error(
        "config.yaml".to_string(),
        "invalid indentation at line 5".to_string(),
    );
    println!("   ✓ Error: {}", error);
    println!("   ✓ Code: {} (custom code!)", error.code());
    println!("   ✓ Auto-logged at ERROR level");

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║              Constructor Method Benefits                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    println!("  🎯 Single line error creation and logging");
    println!("  🐍 Method names are snake_case versions of variant names");
    println!("  🔄 No need to remember to call .log() - it's automatic");
    println!("  🛡️  Type-safe parameter passing");
    println!("  💡 IDE autocomplete for constructor methods");

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
// 7. ✅ No complex source error chaining to manage
// 8. ✅ Simple, explicit error creation patterns
// 9. ✅ NEW: Constructor methods (new_variant_name) that auto-log errors
// 10. ✅ NEW: Single-line error creation and logging for better ergonomics
