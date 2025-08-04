# LogFFI

[![Crates.io](https://img.shields.io/crates/v/logffi.svg)](https://crates.io/crates/logffi)
[![Documentation](https://docs.rs/logffi/badge.svg)](https://docs.rs/logffi)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Universal logging for Rust with compile-time backend selection, FFI support, and advanced error handling.

## ✨ Features

- 🔧 **Feature-Based Backends** - Choose `log`, `tracing`, `slog`, or `callback` via Cargo features
- 🌉 **FFI Support** - Bridge Rust logs to Python, Node.js, C/C++, and more
- 🎯 **Advanced Error Handling** - `define_errors!` macro with automatic logging
- 🔗 **Error Chaining** - Full support for source errors with `#[source]`
- 🚀 **Zero Overhead** - Only compile what you use, no runtime switching cost
- 🛡️ **Type Safe** - Leverage Rust's type system for error handling
- 📊 **Multi-Backend Support** - Use multiple backends simultaneously when needed

## 📚 Documentation

- **[API Documentation](https://docs.rs/logffi)** - Full API reference
- **[Cookbook](cookbook/)** - Real-world examples and patterns
- **[Examples](examples/)** - Runnable example code

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
logffi = "0.2"
```

### Basic Logging

```rust
use logffi::{error, warn, info, debug, trace};

fn main() {
    // Initialize env_logger (or any log-compatible backend)
    env_logger::init();
    
    // Use like standard log macros
    info!("Starting application");
    debug!("Configuration loaded: {:?}", config);
    
    if let Err(e) = dangerous_operation() {
        error!("Operation failed: {}", e);
    }
}
```

### Error Handling with Automatic Logging

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

// Use the generated constructor methods - they automatically log!
fn load_config(path: &str) -> Result<Config, AppError> {
    if !Path::new(path).exists() {
        // This creates the error AND logs it automatically
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

### FFI Integration

```rust
use logffi::set_callback;

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
- Works with any logging backend your users prefer

### For Application Developers

- Structured errors with proper context
- Automatic error logging with appropriate levels
- Easy integration with monitoring systems
- Flexible backend configuration

### For FFI Users

- Bridge Rust logs to any language
- Preserve error context across language boundaries
- Structured error information for better debugging

## 📖 Learn More

Check out the **[Cookbook](cookbook/)** for detailed guides:

- [Basic Logging Patterns](cookbook/01-basic-logging.md)
- [Advanced Error Handling](cookbook/02-error-handling.md)
- [Source Error Chaining](cookbook/03-source-error-chaining.md)
- [FFI Integration Examples](cookbook/04-ffi-integration.md)
- [Backend Configuration](cookbook/05-backend-configuration.md)

## 🔧 Advanced Usage

### Multiple Backends

```rust
use logffi::{set_backend, Backend};

// Switch backends at runtime
set_backend(Backend::Tracing);  // Use tracing
set_backend(Backend::Slog);      // Use slog
set_backend(Backend::Log);       // Use log (default)
```

### Dual-Mode Logging

```rust
use logffi::{set_callback, FORCE_NATIVE_BACKENDS};

// Enable both FFI callback AND native Rust logging
set_callback(Box::new(|level, target, msg| {
    send_to_monitoring_system(level, target, msg);
}));
FORCE_NATIVE_BACKENDS.store(true, Ordering::Relaxed);

// Now logs go to both FFI callback AND native backend!
```

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
