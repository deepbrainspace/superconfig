# LogFFI

[![Crates.io](https://img.shields.io/crates/v/logffi.svg)](https://crates.io/crates/logffi)
[![Documentation](https://docs.rs/logffi/badge.svg)](https://docs.rs/logffi)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Enhanced logging for Rust that complements `log` and `tracing` with unified macros, FFI support, and advanced error handling.

## What LogFFI Provides

LogFFI **complements** existing logging solutions rather than replacing them:

1. **🔄 Unified Macros** - Use the same logging macros whether your dependencies use `log` or `tracing`
2. **🌉 FFI Bridge** - Essential for Rust libraries that need Python, Node.js, or C bindings
3. **🎯 Enhanced Error Handling** - `define_errors!` macro combines `thiserror` with automatic logging
4. **📦 Zero Dependencies** - Works with `log` or `tracing` compatible logger you choose

## When to Use LogFFI

- ✅ Building a Rust library that needs FFI bindings
- ✅ Want automatic error logging with proper error types
- ✅ Need to support multiple of `log` and `tracing` ecosystems
- ✅ Building cross-language applications

## When NOT to Use LogFFI

- ❌ Just need basic logging (use `log` or `tracing` directly)
- ❌ Want to replace your current logger (we complement, not replace)
- ❌ Looking for a logging implementation (we're a bridge, not a logger)

## 📚 Documentation

- **[API Documentation](https://docs.rs/logffi)** - Full API reference
- **[Cookbook](cookbook/)** - Real-world examples and patterns
- **[Examples](examples/)** - Runnable example code

## 🚀 Quick Start

### Step 1: Add LogFFI to your project

```toml
[dependencies]
# Choose your backend via features
logffi = { version = "0.2", features = ["log"] } # For log-based loggers
# OR
logffi = { version = "0.2", features = ["tracing"] } # For tracing-based loggers

# Also add your preferred logger implementation
env_logger = "0.11" # Or any log/tracing/slog compatible logger
```

### Popular Logger Choices

**For `log` backend:**

- `env_logger` - Simple, environment-based configuration
- `fern` - Flexible, composable logging
- `log4rs` - Powerful, Java-inspired logging
- `simplelog` - Easy to use with good defaults
- [See all 30+ options](https://docs.rs/log/latest/log/#available-logging-implementations)

**For `tracing` backend:**

- `tracing-subscriber` - The standard choice for most applications
- `tracing-bunyan-formatter` - JSON output for cloud environments
- `tracing-opentelemetry` - Distributed tracing support
- [See all 40+ options](https://github.com/tokio-rs/tracing#related-crates)

### Step 2: Initialize your logger (not LogFFI!)

```rust
use logffi::{error, warn, info, debug, trace};

fn main() {
    // Initialize YOUR CHOSEN logger (LogFFI doesn't provide loggers)
    env_logger::init();  // Or tracing_subscriber::fmt::init() etc.
    
    // Now use LogFFI's unified macros
    info!("Starting application");
    debug!("Configuration loaded");
    
    if let Err(e) = dangerous_operation() {
        error!("Operation failed: {}", e);
    }
}

fn dangerous_operation() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
```

### Error Handling with Automatic Logging

LogFFI provides **all thiserror features** plus automatic logging and FFI integration:

```rust
use logffi::define_errors;

define_errors! {
    pub enum AppError {
        #[error("Configuration not found: {path}", level = error)]
        ConfigNotFound {
            path: String,
        },
        
        #[error("Failed to connect to database", level = error, target = "db")]
        DatabaseConnection,
        
        #[error("Invalid user input: {field}", level = warn)]
        ValidationError {
            field: String,
        },
    }
}

// ✅ Gets ALL thiserror features: Display, Error, Debug, From conversions
// ✅ PLUS automatic logging integration
// ✅ PLUS constructor methods with auto-logging
// ✅ PLUS FFI-friendly error mapping

fn load_config(path: &str) -> Result<Config, AppError> {
    if !Path::new(path).exists() {
        // Creates error + logs automatically + works across languages
        return Err(AppError::new_config_not_found(path.to_string()));
    }
    // ...
}
```

### Source Error Chaining

```rust
use logffi::define_errors;
use std::io;

define_errors! {
    pub enum DataError {
        #[error("Failed to read file: {path}")]
        ReadError {
            path: String,
            #[source]
            source: io::Error,  // Proper error chaining!
        },
        
        #[error("Failed to parse JSON")]
        ParseError {
            #[source]
            source: serde_json::Error,
        },
    }
}

// Errors maintain the full chain for debugging
fn load_data(path: &str) -> Result<Data, DataError> {
    let content = std::fs::read_to_string(path)
        .map_err(|source| DataError::ReadError {
            path: path.to_string(),
            source,  // Original IO error is preserved
        })?;
    
    serde_json::from_str(&content)
        .map_err(|source| DataError::ParseError { source })
}
```

### FFI Integration using Callbacks

```rust
use logffi::callback;

// Bridge to Python logging
set_callback(Box::new(|level, target, message| {
    Python::with_gil(|py| {
        let logging = py.import("logging").unwrap();
        let logger = logging.call_method1("getLogger", (target,)).unwrap();
        logger.call_method1(level.to_lowercase(), (message,)).unwrap();
    });
}));

// Now all Rust logs appear in Python!
```

## 🎯 Key Benefits

### For Library Authors

- Provide rich error types with zero boilerplate
- Automatic logging at error creation sites
- FFI-friendly error codes and messages
- Works with any logging backend supported by log/tracing/slog.

### For Application Developers

- Structured errors with proper context
- Automatic error logging with appropriate levels
- Easy integration with monitoring systems
- Flexible backend configuration

### For FFI Users

- Bridge Rust logs to any language's native logging libraries.
- Preserve error context across language boundaries
- Structured error information for better debugging

## 📖 Learn More

Check out the **[Cookbook](cookbook/)** for detailed guides:

- [Basic Logging Patterns](cookbook/01-basic-logging.md)
- [Advanced Error Handling](cookbook/02-error-handling.md)
- [Source Error Chaining](cookbook/03-source-error-chaining.md)
- [FFI Integration Examples](cookbook/04-ffi-integration.md)
- [Backend Configuration](cookbook/05-backend-configuration.md)
- [Backend Selection Guide](cookbook/06-backend-selection.md)

## 🔧 Advanced Usage

### Custom Error Codes

```rust
define_errors! {
    pub enum ApiError {
        #[error("Authentication failed", code = "AUTH_001")]
        AuthFailed,
        
        #[error("Rate limit exceeded", code = "RATE_001")]
        RateLimited,
        
        #[error("Invalid request", code = "REQ_001")]
        BadRequest,
    }
}

// Use error codes for monitoring
match api_call() {
    Err(e) => {
        metric_counter!("api.errors", "code" => e.code());
        Err(e)
    }
    Ok(result) => Ok(result),
}
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. This crate is part of the [SuperConfig](https://github.com/deepbrain/superconfig) project.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.
