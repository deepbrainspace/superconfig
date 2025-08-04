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

    println!("\n=== What the define_errors! Macro Generates ===\n");

    println!("1. Error enum with thiserror::Error derive");
    println!("2. Display trait implementation (via thiserror)");
    println!("3. Debug trait implementation");
    println!("4. code() method for error codes");
    println!("5. kind() method for FFI error type mapping");
    println!("6. full_message_chain() method");
    println!("7. log() method with proper level and target");
    println!("8. new_<variant_name>() constructor methods that auto-log\n");

    println!("=== Example Usage - Manual Creation ===\n");

    // Example 1: Simple error (manual creation)
    let error = ConfigError::KeyNotFound {
        key: "database.host".to_string(),
        profile: "production".to_string(),
    };

    println!("Error Display: {}", error);
    println!("Error Code: {}", error.code());
    println!("Error Kind: {}", error.kind());

    // This logs to the configured backend with:
    // - Level: WARN
    // - Target: "superconfig::registry"
    // - Message: "[KeyNotFound] Key 'database.host' not found in profile 'production'"
    error.log();

    println!("\n=== NEW: Constructor Methods (Recommended) ===\n");

    // Much cleaner! The constructor automatically logs the error
    let error = ConfigError::new_key_not_found(
        "database.port".to_string(),
        "staging".to_string(),
    );
    println!("Created and logged: {}", error);

    // Profile not found - constructor handles everything
    let error = ConfigError::new_profile_not_found("development".to_string());
    println!("Created and logged: {}", error);

    println!("\n=== IO Error Pattern - Constructor Method ===\n");

    // Example 2: IO error handling with constructor
    let file_path = "/etc/app/config.toml";
    let io_result = std::fs::read_to_string(file_path);

    if let Err(io_err) = io_result {
        // One line creates AND logs the error!
        let error = ConfigError::new_file_read_error(
            file_path.to_string(),
            io_err.to_string(),
        );

        println!("Error: {}", error);
        println!("Code: {}", error.code());
        // No need to call error.log() - constructor already did it!
    }

    println!("\n=== JSON Parse Error - Constructor Method ===\n");

    // Example 3: Parse error handling with constructor
    let json_result = serde_json::from_str::<serde_json::Value>("invalid json");

    if let Err(parse_err) = json_result {
        // Constructor creates, logs, and returns the error
        let error = ConfigError::new_json_parse_error(
            "app.json".to_string(),
            format!("at line {}, column {}", parse_err.line(), parse_err.column()),
        );

        println!("Error: {}", error);
        println!("Code: {} (auto-generated)", error.code());
    }

    println!("\n=== YAML Error with Custom Code - Constructor ===\n");

    // Constructor automatically logs at INFO level to "superconfig::parse"
    let error = ConfigError::new_yaml_parse_error(
        "config.yaml".to_string(),
        "invalid indentation at line 5".to_string(),
    );

    println!("Error: {}", error);
    println!("Code: {} (custom)", error.code()); // Will show "YAML_001"

    println!("\n=== Constructor Benefits ===\n");
    println!("1. Single line error creation and logging");
    println!("2. Method names are snake_case versions of variant names");
    println!("3. No need to remember to call .log() - it's automatic");
    println!("4. Type-safe parameter passing");
    println!("5. IDE autocomplete for constructor methods");

    println!("\n=== What Gets Logged ===\n");
    println!("All errors are logged to the configured backend (log/tracing/slog)");
    println!("Format: [ERROR_CODE] Error message");
    println!("With appropriate log level and target module");
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
