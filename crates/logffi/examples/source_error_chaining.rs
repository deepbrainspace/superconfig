//! Source Error Chaining Example
//!
//! This example demonstrates proper error source chaining using the #[source] attribute
//! with LogFFI's tracing-native implementation.
//!
//! Run with:
//! cargo run --example source_error_chaining
//!
//! Or with debug logs:
//! RUST_LOG=debug cargo run --example source_error_chaining

use logffi::define_errors;
use std::error::Error;
use std::fs;
use std::io;

// Define errors with source error chaining using thiserror's #[source] attribute
define_errors! {
    pub enum AppError {
        // Simple error without source
        #[error("Configuration missing for key: {key}", level = warn, target = "app::config")]
        ConfigMissing {
            key: String,
        },

        // Error with source - use #[source] attribute on the field
        #[error("Failed to read config file: {path}", level = error, target = "app::io")]
        ConfigReadError {
            path: String,
            #[source]
            source: io::Error,
        },

        // Another example with serde_json::Error as source
        #[error("Failed to parse JSON config from {file}", level = error, target = "app::parse")]
        JsonParseError {
            file: String,
            #[source]
            source: serde_json::Error,
        },

        // Example with Box<dyn Error> for flexible source types
        #[error("Database operation failed: {operation}", level = error, target = "app::db")]
        DatabaseError {
            operation: String,
            #[source]
            source: Box<dyn std::error::Error + Send + Sync>,
        },
    }
}

// Helper function to simulate database errors
fn simulate_db_error() -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(io::Error::new(
        io::ErrorKind::ConnectionRefused,
        "Cannot connect to database",
    ))
}

fn main() {
    println!("\n🔗 LogFFI Source Error Chaining Example");
    println!("=======================================");
    println!("Auto-initializing tracing subscriber...");

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║              Source Error Chaining Examples                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 1: IO Error with Source Chain                       │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    // Try to read a non-existent file
    let file_path = "/tmp/non_existent_config.json";
    println!("📁 Attempting to read: {}", file_path);

    match fs::read_to_string(file_path) {
        Ok(_) => println!("   ✓ File read successfully"),
        Err(io_err) => {
            println!("   ❌ IO Error occurred: {}", io_err);

            // Create error with source
            let error = AppError::ConfigReadError {
                path: file_path.to_string(),
                source: io_err,
            };

            println!("\n🔗 Error chain:");
            println!("   Primary: {}", error);

            // Walk the error chain
            let mut current_error: &dyn std::error::Error = &error;
            let mut depth = 1;
            while let Some(source) = current_error.source() {
                println!("   └─ Cause {}: {}", depth, source);
                current_error = source;
                depth += 1;
            }

            // Log the error (includes the primary message)
            error.log();
        }
    }

    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 2: JSON Parse Error with Source                     │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    let json_content = r#"{ invalid json }"#;
    println!("📄 Attempting to parse JSON...");

    match serde_json::from_str::<serde_json::Value>(json_content) {
        Ok(_) => println!("   ✓ JSON parsed successfully"),
        Err(json_err) => {
            println!("   ❌ Parse Error: {}", json_err);

            let error = AppError::JsonParseError {
                file: "config.json".to_string(),
                source: json_err,
            };

            println!("\n🔗 Error chain:");
            println!("   Primary: {}", error);
            if let Some(source) = error.source() {
                println!("   └─ Cause: {}", source);
            }

            error.log();
        }
    }

    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 3: Database Error with Boxed Source                 │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    println!("🗄️  Simulating database operation...");

    let db_error = simulate_db_error();
    let error = AppError::DatabaseError {
        operation: "connect".to_string(),
        source: db_error,
    };

    println!("   ❌ Database operation failed");
    println!("\n🔗 Error chain:");
    println!("   Primary: {}", error);
    if let Some(source) = error.source() {
        println!("   └─ Cause: {}", source);
    }

    error.log();

    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example 4: Error without Source (for comparison)            │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    let error = AppError::ConfigMissing {
        key: "database.url".to_string(),
    };

    println!("🔍 Simple error without source:");
    println!("   Error: {}", error);
    println!("   Has source: {}", error.source().is_some());

    error.log();

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    Key Benefits                               ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("  🔗 Proper error chaining with std::error::Error");
    println!("  📊 Source errors accessible via .source() method");
    println!("  🎯 Works with error reporting tools (eyre, anyhow, etc.)");
    println!("  📝 Clean thiserror integration with #[source] attribute");
    println!("  🚀 No custom implementation needed");

    println!("\n✨ Source error chaining provides full context for debugging!");
}

// Usage in real code:
//
// fn read_config(path: &str) -> Result<Config, AppError> {
//     let content = fs::read_to_string(path)
//         .map_err(|source| AppError::ConfigReadError {
//             path: path.to_string(),
//             source,
//         })?;
//
//     let config = serde_json::from_str(&content)
//         .map_err(|source| AppError::JsonParseError {
//             file: path.to_string(),
//             source,
//         })?;
//
//     Ok(config)
// }
