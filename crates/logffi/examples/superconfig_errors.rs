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
    println!("7. log() method with proper level and target\n");

    println!("=== Example Usage ===\n");

    // Example 1: Simple error
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

    println!("\n=== IO Error Pattern ===\n");

    // Example 2: IO error handling in SuperConfig
    let file_path = "/etc/app/config.toml";
    let io_result = std::fs::read_to_string(file_path);

    if let Err(io_err) = io_result {
        let error = ConfigError::FileReadError {
            path: file_path.to_string(),
            io_error: io_err.to_string(), // Convert source error to string
        };

        println!("Error: {}", error);
        println!("Code: {}", error.code());

        // Logs at ERROR level to "superconfig::io"
        error.log();
    }

    println!("\n=== JSON Parse Error Pattern ===\n");

    // Example 3: Parse error handling
    let json_result = serde_json::from_str::<serde_json::Value>("invalid json");

    if let Err(parse_err) = json_result {
        let error = ConfigError::JsonParseError {
            file: "app.json".to_string(),
            details: format!(
                "at line {}, column {}",
                parse_err.line(),
                parse_err.column()
            ),
        };

        println!("Error: {}", error);
        // Custom error code if specified
        println!("Code: {} (auto-generated)", error.code());

        error.log();
    }

    println!("\n=== YAML Error with Custom Code ===\n");

    let error = ConfigError::YamlParseError {
        file: "config.yaml".to_string(),
        details: "invalid indentation at line 5".to_string(),
    };

    println!("Error: {}", error);
    println!("Code: {} (custom)", error.code()); // Will show "YAML_001"

    error.log();

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
