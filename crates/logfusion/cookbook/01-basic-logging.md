# Basic Logging

This guide covers the fundamentals of logging with tracing through LogFusion, including LogFusion's auto-initialization feature, access to tracing's logging macros, and environment configuration.

## Table of Contents

- [Getting Started](#getting-started)
- [Auto-Initialization](#auto-initialization)
- [Basic Logging Macros](#basic-logging-macros)
- [Target-Based Logging](#target-based-logging)
- [Environment Configuration](#environment-configuration)
- [Formatting and Arguments](#formatting-and-arguments)
- [Performance Considerations](#performance-considerations)
- [Integration with Tracing Ecosystem](#integration-with-tracing-ecosystem)

## Getting Started

LogFusion provides a simple, zero-configuration way to access the modern `tracing` ecosystem's logging capabilities.

**What LogFusion provides:** Auto-initialization, convenient macros, and re-exported tracing-subscriber components.

**What tracing provides:** The logging implementation, filtering, formatting, and ecosystem integration.

### Add LogFusion to Your Project

```toml
[dependencies]
logfusion = "0.2"
```

### Your First Log

```rust
use logfusion::{info, warn, error, debug, trace};

fn main() {
    // No initialization needed - LogFusion handles it automatically!
    info!("Application starting");
    warn!("This is a warning");
    error!("Something went wrong");
    debug!("Debug information");
    trace!("Detailed trace information");
}
```

That's it! LogFusion's auto-initialization feature sets up tracing with sensible defaults when you use any logging macro.

## Auto-Initialization

LogFusion's auto-initialization feature means you can start logging immediately without boilerplate setup code.

### How It Works

```rust
use logfusion::info;

fn main() {
    // First call to any LogFusion macro automatically initializes tracing
    info!("This works immediately!");
    
    // Subsequent calls use the same initialized subscriber
    info!("No additional setup needed");
}
```

### What Gets Initialized

LogFusion sets up:

- **Default subscriber** - `tracing_subscriber::fmt()` with human-readable output
- **Environment filter** - Respects `RUST_LOG` environment variable
- **Thread safety** - Safe initialization even in multi-threaded applications

### Initialization Details

```rust
use logfusion::info;

fn main() {
    // This is equivalent to what LogFusion does automatically:
    /*
    let env_filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".to_string());
    
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .init();
    */
    
    // But you just need to do this:
    info!("Auto-initialization handles the setup!");
}
```

## Basic Logging Macros

LogFusion provides access to tracing's five logging levels, each corresponding to a different severity:

```rust
use logfusion::{trace, debug, info, warn, error};

fn demonstrate_log_levels() {
    // ERROR - Use for actual errors that need attention
    error!("Database connection failed");
    
    // WARN - Use for concerning situations that aren't errors
    warn!("API rate limit approaching");
    
    // INFO - Use for general informational messages
    info!("User logged in successfully");
    
    // DEBUG - Use for diagnostic information useful during development
    debug!("Cache hit for key: user_12345");
    
    // TRACE - Use for very detailed diagnostic information
    trace!("Entering function process_request()");
}
```

### When to Use Each Level

| Level    | Use Case                               | Examples                                                  |
| -------- | -------------------------------------- | --------------------------------------------------------- |
| `error!` | Actual errors requiring attention      | Database failures, API errors, crashes                    |
| `warn!`  | Concerning but non-fatal situations    | Rate limiting, deprecated API usage, performance issues   |
| `info!`  | Important business events              | User actions, system state changes, successful operations |
| `debug!` | Development and diagnostic information | Function entry/exit, cache operations, internal state     |
| `trace!` | Very detailed diagnostic information   | Request/response bodies, detailed execution flow          |

## Target-Based Logging

Organize your logs by module or component using targets:

```rust
use logfusion::{info, warn, error};

fn main() {
    // Default target (uses current module path)
    info!("Application starting");
    
    // Custom targets for organization
    info!(target: "app::database", "Connected to PostgreSQL");
    warn!(target: "app::auth", "Failed login attempt");
    error!(target: "app::payment", "Payment processing failed");
    
    // Use targets to represent different systems
    info!(target: "metrics", "CPU usage: 45%");
    info!(target: "audit", "User admin action performed");
}
```

### Target Best Practices

```rust
use logfusion::{info, error};

// Good: Use hierarchical naming
info!(target: "myapp::database::connection", "Connection established");
info!(target: "myapp::auth::login", "User authentication successful");
info!(target: "myapp::api::users", "User created");

// Good: Use consistent naming conventions
info!(target: "service::user_service", "Processing user request");
info!(target: "service::payment_service", "Processing payment");

// Avoid: Inconsistent or overly specific targets
// error!(target: "very_specific_function_name_db_conn_retry_3", "Failed");
```

## Environment Configuration

LogFusion respects the standard `RUST_LOG` environment variable for controlling log levels:

### Basic Level Control

```bash
# Show all logs at INFO level and above
RUST_LOG=info cargo run

# Show all logs at DEBUG level and above  
RUST_LOG=debug cargo run

# Show only ERROR level logs
RUST_LOG=error cargo run
```

### Module-Specific Control

```bash
# Different levels for different modules
RUST_LOG=warn,myapp::database=debug cargo run

# Detailed logging for specific components
RUST_LOG=info,myapp::auth=trace,myapp::payment=debug cargo run

# Hide noisy third-party crates
RUST_LOG=info,sqlx=warn,hyper=warn cargo run
```

### Target-Specific Filtering

```bash
# Filter by target names
RUST_LOG=info,app::database=debug cargo run

# Multiple target filters
RUST_LOG=warn,app::auth=info,app::payment=debug cargo run
```

### Advanced Filtering

```bash
# Complex filtering with multiple rules
RUST_LOG="warn,myapp=info,myapp::database=debug,sqlx::query=trace" cargo run

# Regex-based filtering (advanced)
RUST_LOG="info,sqlx::query{db.statement}=trace" cargo run
```

## Formatting and Arguments

LogFusion supports Rust's standard formatting syntax:

### Basic Formatting

```rust
use logfusion::{info, warn, error};

fn demonstrate_formatting() {
    let user_id = 12345;
    let username = "alice";
    let attempts = 3;
    
    // Simple variable interpolation
    info!("User {} logged in", username);
    
    // Multiple variables
    warn!("User {} failed login after {} attempts", username, attempts);
    
    // Named arguments
    error!("Database error for user {user} with ID {id}", user = username, id = user_id);
    
    // Different formatting options
    info!("Progress: {:.1}%", 75.456);  // 75.5%
    info!("Hex value: 0x{:X}", 255);    // 0xFF
    info!("User ID: {:<10}", user_id);  // Left-aligned in 10 chars
}
```

### Complex Data Types

```rust
use logfusion::{info, debug};
use std::collections::HashMap;

fn log_complex_data() {
    let user_data = HashMap::from([
        ("name", "Alice"),
        ("email", "alice@example.com"),
        ("role", "admin"),
    ]);
    
    // Debug formatting for complex types
    debug!("User data: {:?}", user_data);
    
    // Pretty formatting
    debug!("User data: {:#?}", user_data);
    
    // Custom types that implement Display
    info!("Processing request: {}", request_id);
}
```

## Performance Considerations

### Log Level Filtering

```rust
use logfusion::{debug, trace};

fn performance_sensitive_function() {
    // These are efficiently filtered out in release builds
    // when RUST_LOG doesn't include debug/trace levels
    debug!("Processing item {}", expensive_debug_computation());
    trace!("Very detailed trace: {:?}", expensive_trace_data());
}

// Expensive operations are only called if the log level is enabled
fn expensive_debug_computation() -> String {
    // This won't be called if debug logging is disabled
    format!("Expensive computation result")
}
```

### Conditional Logging

```rust
use logfusion::debug;
use tracing::enabled;

fn conditional_logging() {
    // Check if debug logging is enabled before expensive operations
    if enabled!(tracing::Level::DEBUG) {
        let expensive_data = compute_expensive_debug_info();
        debug!("Debug data: {:?}", expensive_data);
    }
}

fn compute_expensive_debug_info() -> Vec<String> {
    // Expensive computation that only runs when needed
    vec!["expensive".to_string(), "data".to_string()]
}
```

### Best Practices for Performance

```rust
use logfusion::{info, debug};

fn performance_best_practices() {
    // ✅ Good: Simple string literals are very efficient
    info!("User login successful");
    
    // ✅ Good: Simple variable interpolation
    let user_id = 12345;
    info!("User {} logged in", user_id);
    
    // ⚠️ Be careful: Expensive computations in log arguments
    // debug!("Data: {:?}", expensive_serialization()); // Only if needed
    
    // ✅ Better: Use conditional logging for expensive operations
    if tracing::enabled!(tracing::Level::DEBUG) {
        debug!("Data: {:?}", expensive_serialization());
    }
}

fn expensive_serialization() -> String {
    // Some expensive operation
    "expensive result".to_string()
}
```

## Integration with Tracing Ecosystem

LogFusion is built on `tracing`, so it integrates seamlessly with the entire ecosystem:

### Working with Existing Tracing Code

```rust
use logfusion::{info, warn};
use tracing::{info as tracing_info, span, Level};

fn mixed_usage() {
    // LogFusion macros and tracing macros work together
    info!("Using LogFusion macro");
    tracing_info!("Using tracing macro directly");
    
    // Both are captured by the same subscriber
    let span = span!(Level::INFO, "my_operation");
    let _enter = span.enter();
    
    warn!("This appears within the span context");
}
```

### Log Crate Bridge

LogFusion automatically works with libraries using the `log` crate:

```rust
use logfusion::info;

fn main() {
    // LogFusion automatically sets up log-to-tracing bridge
    info!("LogFusion message");
    
    // These also work through the bridge
    log::info!("Legacy log crate message");
    log::warn!("Another legacy message");
    
    // All messages appear in the same output stream
}
```

### Custom Subscriber Setup

For advanced use cases, you can disable auto-initialization and set up your own subscriber:

```rust
use logfusion::{info, registry, fmt, EnvFilter, SubscriberExt, SubscriberInitExt};

fn main() {
    // Set up custom tracing subscriber BEFORE using LogFusion macros
    // Using tracing-subscriber components re-exported by LogFusion for convenience
    registry()
        .with(fmt::layer().json())
        .with(EnvFilter::from_default_env())
        .init();
    
    // Now LogFusion macros will use your custom tracing setup
    info!("This uses the custom JSON formatter");
}
```

## Common Patterns

### Application Startup

```rust
use logfusion::{info, warn, error};

fn main() {
    info!("Application starting");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));
    
    match initialize_database() {
        Ok(_) => info!("Database initialized successfully"),
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    }
    
    info!("Application ready to accept requests");
}

fn initialize_database() -> Result<(), Box<dyn std::error::Error>> {
    // Database initialization logic
    Ok(())
}
```

### Request Processing

```rust
use logfusion::{info, warn, error, debug};

fn process_request(request_id: &str, user_id: u64) {
    info!("Processing request {} for user {}", request_id, user_id);
    
    debug!("Validating request parameters");
    
    match validate_request() {
        Ok(_) => debug!("Request validation successful"),
        Err(e) => {
            warn!("Request validation failed: {}", e);
            return;
        }
    }
    
    info!("Request {} completed successfully", request_id);
}

fn validate_request() -> Result<(), &'static str> {
    // Validation logic
    Ok(())
}
```

### Error Handling

```rust
use logfusion::{error, warn, info};

fn handle_operation_result(result: Result<String, Box<dyn std::error::Error>>) {
    match result {
        Ok(data) => {
            info!("Operation completed successfully");
            debug!("Result data: {}", data);
        }
        Err(e) => {
            error!("Operation failed: {}", e);
            
            // Log additional context based on error type
            if e.to_string().contains("network") {
                warn!("Network-related failure detected");
            } else if e.to_string().contains("timeout") {
                warn!("Timeout occurred, retrying might help");
            }
        }
    }
}
```

## Next Steps

Now that you understand basic logging, explore these advanced topics:

- **[Structured Logging](02-structured-logging.md)** - Add rich metadata to your logs
- **[Error Handling](03-error-handling.md)** - Use `define_errors!` for automatic error logging
- **[Spans and Instrumentation](04-spans-instrumentation.md)** - Track operations across async boundaries
- **[Advanced Tracing Integration](05-tracing-integration.md)** - Custom subscribers and OpenTelemetry

## Troubleshooting

### Common Issues

**Q: My debug logs aren't appearing**

```bash
# Make sure RUST_LOG includes debug level
RUST_LOG=debug cargo run
```

**Q: Too many logs from third-party crates**

```bash
# Filter out noisy crates
RUST_LOG=info,sqlx=warn,hyper=warn,tokio=warn cargo run
```

**Q: I want JSON output**

```rust
// Set up JSON formatting before using LogFusion - using tracing-subscriber via LogFusion's re-exports
use logfusion::{info, registry, fmt, SubscriberInitExt};

fn main() {
    registry()
        .with(fmt::layer().json())
        .init();
        
    info!("This will be JSON formatted");
}
```

**Q: Logs aren't showing in tests**

```rust
// Initialize tracing in test setup
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_with_logging() {
        // This ensures tracing logs appear during testing - using LogFusion's re-exported components
        let _ = logfusion::fmt::try_init();
        
        logfusion::info!("Test log message");
        // Your test code
    }
}
```
