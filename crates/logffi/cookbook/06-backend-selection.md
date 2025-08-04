# Backend Selection and Configuration

LogFFI supports multiple logging backends that can be selected at compile time using Cargo features. This allows for optimal binary size and performance while providing flexibility for different use cases.

## Available Backends

### Log Backend

The standard Rust logging facade, lightweight and widely compatible.

```toml
[dependencies]
logffi = { version = "0.2", default-features = false, features = ["log"] }
```

**Best for:**

- Libraries that want maximum compatibility
- Simple applications with basic logging needs
- Integration with existing log-based infrastructure

### Tracing Backend (Default)

Modern structured logging with async support and spans.

```toml
[dependencies]
logffi = "0.2" # tracing is the default
# Or explicitly:
logffi = { version = "0.2", default-features = false, features = ["tracing"] }
```

**Best for:**

- Modern applications with complex async workflows
- Applications that need structured logging
- Microservices and distributed systems

### Slog Backend

Highly structured logging with powerful composition capabilities.

```toml
[dependencies]
logffi = { version = "0.2", default-features = false, features = ["slog"] }
```

**Best for:**

- Applications requiring highly structured logs
- Complex logging hierarchies and contexts
- High-performance logging scenarios

### Callback Backend

Custom logging routing for FFI integration.

```toml
[dependencies]
logffi = { version = "0.2", default-features = false, features = ["callback"] }
```

**Best for:**

- Python/Node.js bindings
- Custom log routing systems
- Integration with external monitoring systems

## Feature Combinations

### Multiple Backends

Enable multiple backends simultaneously:

```toml
[dependencies]
# Enable log and tracing together
logffi = { version = "0.2", default-features = false, features = ["log", "tracing"] }

# Enable all backends
logffi = { version = "0.2", features = ["all"] }
```

**Use case:** Applications that need different backends for different purposes (e.g., tracing for main app logs, callback for metrics).

### Single Backend (Minimal Binary)

For size-constrained environments:

```toml
[dependencies]
# Only log backend - smallest binary
logffi = { version = "0.2", default-features = false, features = ["log"] }
```

## Backend-Specific Usage

### Using the Log Backend

```rust
use logffi::{error, info, logger};

fn main() {
    // Basic usage works automatically
    info!("Application started");
    
    // Access log backend for advanced features
    if let Some(_log_backend) = logger().as_log() {
        // Log backend is available
        // Can use any log-compatible libraries
        log::info!("Direct log crate usage");
    }
}
```

### Using the Tracing Backend

```rust
use logffi::{error, info, logger};

#[tokio::main]
async fn main() {
    // Structured logging works automatically
    info!(user_id = 123, action = "login", "User logged in");
    
    // Access tracing backend for spans
    if let Some(_tracing_backend) = logger().as_tracing() {
        use tracing::{span, Level};
        
        let span = span!(Level::INFO, "database_operation");
        let _enter = span.enter();
        
        info!("Inside database span");
        // LogFFI macros work inside spans
    }
}
```

### Using the Slog Backend

```rust
use logffi::{error, info, logger};

fn main() {
    // Basic usage works automatically
    info!("Server starting");
    
    // Access slog for advanced features
    if let Some(slog_backend) = logger().as_slog() {
        use slog::{info as slog_info, o};
        
        // Create child loggers with context
        let db_logger = slog_backend.logger().new(o!(
            "component" => "database",
            "version" => "1.2.3"
        ));
        
        slog_info!(db_logger, "Database connected"; 
                   "host" => "localhost", 
                   "port" => 5432);
    }
}
```

### Using the Callback Backend

```rust
use logffi::{error, info, logger, set_callback};

fn main() {
    // Set up callback for external systems
    if let Some(_callback_backend) = logger().as_callback() {
        set_callback(Box::new(|level, target, message| {
            // Send to external monitoring system
            send_to_datadog(level, target, message);
            
            // Or bridge to Python logging
            #[cfg(feature = "python")]
            python_log_bridge(level, target, message);
        }));
    }
    
    // Now all LogFFI logs go to the callback
    info!("This will be sent to external systems");
    error!(target: "payment", "Payment processing failed");
}

fn send_to_datadog(level: &str, target: &str, message: &str) {
    // Implementation for external logging service
    println!("DataDog: [{}] {}: {}", level, target, message);
}
```

## Multi-Backend Scenarios

### Application + Metrics

Use tracing for application logs and callback for metrics:

```toml
[dependencies]
logffi = { version = "0.2", default-features = false, features = ["tracing", "callback"] }
```

```rust
use logffi::{error, info, logger, set_callback};

fn main() {
    // Set up metrics callback
    if let Some(_callback) = logger().as_callback() {
        set_callback(Box::new(|level, target, message| {
            if target.starts_with("metrics") {
                send_to_prometheus(level, target, message);
            }
        }));
    }
    
    // Regular application logging (goes to tracing)
    info!("Application started");
    
    // Metrics logging (goes to both tracing and callback)
    info!(target: "metrics::requests", "Request count: {}", 100);
}
```

### Development vs Production

Different backends for different environments:

```rust
fn init_logging() {
    let logger = logffi::logger();
    let available = logger.available_backends();
    
    if cfg!(debug_assertions) {
        // Development: Use tracing with pretty output
        if available.contains(&logffi::Backend::Tracing) {
            std::env::set_var("LOGFFI_FORMAT", "text");
            println!("Using tracing backend for development");
        }
    } else {
        // Production: Use slog with JSON output  
        if available.contains(&logffi::Backend::Slog) {
            std::env::set_var("LOGFFI_FORMAT", "json");
            println!("Using slog backend for production");
        }
    }
}
```

## Environment Configuration

### Format Control

Control output format via environment variables:

```bash
# Text format (default)
export LOGFFI_FORMAT=text

# JSON format for structured logging
export LOGFFI_FORMAT=json

# Compact format for development
export LOGFFI_FORMAT=compact
```

### Log Level Control

Each backend respects its own environment variables:

```bash
# For log backend
export RUST_LOG=info

# For tracing backend  
export RUST_LOG=debug

# For slog backend (depends on configuration)
export SLOG_LEVEL=info
```

## Performance Considerations

### Binary Size

- **Single backend**: Smallest binary, fastest compile times
- **Multiple backends**: Larger binary, all backends always available
- **All backends**: Largest binary, maximum flexibility

### Runtime Performance

- **Log**: Fastest, minimal overhead
- **Tracing**: Fast with structured data support
- **Slog**: Fast with advanced composition features
- **Callback**: Depends on callback implementation

### Recommendations

**For Libraries:**

```toml
# Don't specify default features, let applications choose
logffi = { version = "0.2", default-features = false, features = ["log"] }
```

**For Applications:**

```toml
# Use default (tracing) for modern async apps
logffi = "0.2"

# Or choose based on specific needs
logffi = { version = "0.2", default-features = false, features = ["slog"] }
```

**For FFI/Bindings:**

```toml
# Callback-only for minimal FFI overhead
logffi = { version = "0.2", default-features = false, features = ["callback"] }
```

## Migration Guide

### From Standard log Crate

```rust
// Before
use log::{error, info};

// After (no changes needed)
use logffi::{error, info};
```

### From Tracing

```rust
// Before
use tracing::{error, info, span, Level};

// After
use logffi::{error, info};  // Basic logging
use tracing::{span, Level}; // Advanced features when needed
```

### From Slog

```rust
// Before
use slog::{info, o, Logger};

// After
use logffi::{info, logger};  // Basic logging
use slog::{info as slog_info, o}; // Advanced features via logger().as_slog()
```

## Best Practices

1. **Choose the Right Backend**: Match backend to use case (library vs application vs FFI)
2. **Minimize Dependencies**: Use single backend when possible for smaller binaries
3. **Test All Combinations**: If supporting multiple backends, test each combination
4. **Document Requirements**: Clearly document which features your crate needs
5. **Graceful Degradation**: Check backend availability before using advanced features

```rust
// Good: Check before using advanced features
if let Some(tracing) = logger().as_tracing() {
    // Use tracing-specific features
} else {
    // Fallback to basic logging
    info!("Using basic logging fallback");
}

// Bad: Assume specific backend is available
let tracing = logger().as_tracing().unwrap(); // May panic!
```
