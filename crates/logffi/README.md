# LogFFI

[![Crates.io](https://img.shields.io/crates/v/logffi.svg)](https://crates.io/crates/logffi)
[![Documentation](https://docs.rs/logffi/badge.svg)](https://docs.rs/logffi)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Tracing-native logging macros with structured logging support and enhanced error handling for Rust.**

LogFFI provides a clean, modern logging interface built on top of the `tracing` ecosystem, offering structured logging, automatic initialization, and powerful error handling capabilities.

## ✨ Key Features

- **🎯 Tracing-Native** - Built on the modern `tracing` ecosystem for superior observability
- **📊 Structured Logging** - First-class support for structured fields and metadata
- **🔄 Auto-Initialization** - Automatic tracing subscriber setup with smart defaults
- **🌉 Log Crate Bridge** - Seamless compatibility with libraries using the `log` crate
- **🔗 FFI Callback Support** - Optional integration with Python, Node.js, C/C++, and WebAssembly
- **⚡ Enhanced Error Handling** - Dual-syntax `define_errors!` macro with LogFFI format + thiserror compatibility
- **🛠️ Zero Config** - Works out of the box with sensible defaults
- **🔧 Spans & Instrumentation** - Full support for tracing spans and `#[instrument]`

## 🚀 Quick Start

### Add LogFFI to your project

```toml
[dependencies]
logffi = "0.2"

# Optional: Add tracing-subscriber for advanced configuration
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

### Basic Usage

```rust
use logffi::{info, warn, error, debug, trace};

fn main() {
    // LogFFI auto-initializes tracing - no setup needed!
    
    info!("Application starting");
    warn!("This is a warning");
    error!("Something went wrong");
    
    // Structured logging with fields
    info!(
        user_id = 12345,
        action = "login",
        ip_address = "192.168.1.1",
        duration_ms = 145,
        "User authentication successful"
    );
}
```

### Enhanced Error Handling

**New LogFFI Format** (Simplified & Powerful):

```rust
use logffi::define_errors;

// 🆕 LogFFI Format - Clean, attribute-based syntax
define_errors! {
    AppError {
        DatabaseConnection { host: String, port: u16 } : "Database connection failed: {host}:{port}" [level = error, target = "app::db"],
        UserNotFound { user_id: u64 } : "User not found: {user_id}" [level = warn],
        InvalidConfig {} : "Invalid configuration detected" [level = error],
        NetworkTimeout { source: std::io::Error } : "Network operation timed out"  // Auto source chaining
    }
}

// Multiple error types in one macro
define_errors! {
    ApiError {
        BadRequest { field: String } : "Invalid field: {field}" [level = warn],
        Unauthorized {} : "Access denied" [level = error]
    }
    
    DatabaseError {
        ConnectionFailed { host: String } : "Failed to connect to {host}" [level = error]
    }
}

fn example() -> Result<(), AppError> {
    // Errors are automatically logged with structured tracing
    let err = AppError::UserNotFound { user_id: 12345 };
    err.log(); // Logs: WARN app::module: [UserNotFound] User not found: 12345
    
    // 🆕 New in v0.2: Error introspection for monitoring & debugging
    let (code, level, target) = err.error_info();
    println!("Code: {}, Level: {}, Target: {}", code, level, target);
    // Output: Code: UserNotFound, Level: warn, Target: app::module
    
    Err(err)
}
```

**Traditional Thiserror Syntax** (Fully Compatible):

```rust
define_errors! {
    pub enum AppError {
        #[error("Database connection failed: {host}:{port}", level = error, target = "app::db")]
        DatabaseConnection { host: String, port: u16 },
        
        #[error("User not found: {user_id}", level = warn)]
        UserNotFound { user_id: u64 },
    }
}
```

### 🔍 Error Introspection & Monitoring

**New in v0.2**: All generated error enums include an `error_info()` method for monitoring and debugging:

```rust
use logffi::define_errors;
use std::collections::HashMap;

define_errors! {
    ApiError {
        DatabaseTimeout { query: String } : "Query timed out: {query}" [level = error, target = "db::query"],
        RateLimited {} : "API rate limit exceeded" [level = warn, target = "api::rate"],
        UserNotFound { user_id: u64 } : "User {user_id} not found" [level = info]
    }
}

fn main() {
    let error = ApiError::DatabaseTimeout { query: "SELECT * FROM users".to_string() };
    
    // Get structured error information
    let (code, level, target) = error.error_info();
    println!("Error Code: {}", code);     // "DatabaseTimeout"
    println!("Log Level: {}", level);     // "error" 
    println!("Target: {}", target);       // "db::query"
    
    // Perfect for metrics collection
    let mut error_metrics = HashMap::new();
    *error_metrics.entry(code).or_insert(0) += 1;
    
    // Ideal for monitoring dashboards
    match level {
        "error" => send_alert_to_pagerduty(&error),
        "warn" => increment_warning_counter(),
        _ => log_for_debugging(&error)
    }
}
```

**Use Cases for `error_info()`:**

- 📊 **Metrics Collection** - Build error dashboards and SLA monitoring
- 🚨 **Alerting Systems** - Set up automated alerts based on error patterns
- 🔍 **Debugging Tools** - Analyze error patterns in production
- 📈 **Business Intelligence** - Track error rates by component/severity

## 🎯 Why LogFFI?

### 🆕 Dual-Syntax Error Handling

LogFFI v0.2 introduces a **revolutionary dual-syntax approach** to error definitions:

**LogFFI Format Benefits:**

- ✅ **Cleaner Syntax** - No repetitive `#[error(...)]` attributes
- ✅ **Attribute-Based Logging** - `[level = warn, target = "app::db"]` syntax
- ✅ **Multiple Types** - Define multiple error enums in one macro call
- ✅ **Auto Source Detection** - Fields named `source` automatically become `#[source]`
- ✅ **Mixed Variants** - Unit (`{}`) and struct variants in same enum
- ✅ **Field Interpolation** - `"Failed to connect to {host}:{port}"` syntax

**Performance & Maintainability:**

- 🚀 **64% Macro Optimization** - Reduced from 998 to 358 lines while adding features
- 🧹 **11 Comprehensive Tests** - Every scenario covered with battle-tested reliability
- 🔄 **Full Backward Compatibility** - Existing thiserror syntax continues to work
- 📊 **Structured Logging Integration** - Seamless tracing ecosystem integration

### Modern Tracing Ecosystem

Built on `tracing`, the modern standard for Rust observability:

- **Structured logging** - Attach key-value metadata to log events
- **Spans** - Track operations across async boundaries
- **Instrumentation** - Automatic span creation with `#[instrument]`
- **Rich ecosystem** - Compatible with OpenTelemetry, Jaeger, Datadog, and more

### Auto-Initialization

No boilerplate setup required:

```rust
use logffi::info;

fn main() {
    // This works immediately - no initialization needed!
    info!("Hello, world!");
}
```

### Structured Logging Made Easy

```rust
use logffi::info;

// Rich, structured metadata
info!(
    request_id = "req-123",
    user_id = 12345,
    method = "POST", 
    path = "/api/users",
    status_code = 201,
    duration_ms = 45,
    "API request completed"
);
```

### Backward Compatibility

Works seamlessly with existing `log` crate usage:

```rust
// These both work and are captured by LogFFI
log::info!("Legacy log message");
logffi::info!("Modern LogFFI message");
```

## 📊 Structured Logging

LogFFI excels at structured logging, making your logs machine-readable and perfect for modern observability platforms:

```rust
use logffi::{info, error, info_span};

// User authentication
info!(
    user_id = 12345,
    username = "alice",
    ip_address = "192.168.1.100",
    mfa_enabled = true,
    "User login successful"
);

// Payment processing
error!(
    transaction_id = "txn-abc-123",
    amount_cents = 2999,
    currency = "USD",
    decline_reason = "insufficient_funds",
    "Payment failed"
);

// Spans with structured context
let span = info_span!("process_order", order_id = "order-123", customer_id = 456);
let _enter = span.enter();

info!("Processing order");
info!("Order completed successfully");
```

## 🏗️ Spans and Instrumentation

Full support for tracing spans and automatic instrumentation:

```rust
use logffi::{info, info_span};
use tracing::instrument;

#[instrument(level = "info")]
async fn process_user_request(user_id: u64, action: &str) -> Result<String, AppError> {
    info!("Processing user request");
    
    // Nested spans
    let span = info_span!("database_query", table = "users");
    let _enter = span.enter();
    
    info!("Executing database query");
    
    Ok("Success".to_string())
}
```

## 🎛️ Configuration

### Environment Variables

LogFFI respects standard tracing environment variables:

```bash
# Set log level
RUST_LOG=debug cargo run

# Target specific modules  
RUST_LOG=myapp::database=debug,myapp::auth=info cargo run

# Filter by span names
RUST_LOG=process_order=trace cargo run
```

### Custom Initialization

For advanced use cases, you can configure tracing manually:

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().json())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
        
    // Now use LogFFI normally
    logffi::info!("Application started with custom config");
}
```

## 📚 Documentation & Examples

- **[API Documentation](https://docs.rs/logffi)** - Full API reference
- **[Cookbook](cookbook/)** - Real-world patterns and best practices
- **[Examples](examples/)** - Runnable example code

### Key Examples

- **[`logffi_format_showcase.rs`](examples/logffi_format_showcase.rs)** - Complete LogFFI format demonstration with all features
- **[`logffi_source_chaining.rs`](examples/logffi_source_chaining.rs)** - Automatic source chaining with fields named "source"
- **[`field_interpolation_demo.rs`](examples/field_interpolation_demo.rs)** - Dual-syntax field interpolation comparison
- **[`structured_logging_demo.rs`](examples/structured_logging_demo.rs)** - Manual + automatic structured logging
- **[`advanced_tracing_features.rs`](examples/advanced_tracing_features.rs)** - Spans, instrumentation, and ecosystem integration

## 🌉 FFI and Callback Support

LogFFI includes optional FFI callback support for integrating with other languages:

```toml
[dependencies]
logffi = { version = "0.2", features = ["callback"] }
```

With the `callback` feature enabled, LogFFI can route log messages to external callbacks, enabling integration with:

- **Python** (via PyO3)
- **Node.js** (via Neon)
- **C/C++** applications
- **WebAssembly** modules
- **Any FFI-compatible language**

```rust
use logffi::{info, error};

fn main() {
    // These messages are sent to both tracing AND any registered callbacks
    info!("This goes to tracing and FFI callbacks");
    error!("Error messages are bridged to external systems");
}
```

The callback system allows external applications to receive structured log data while LogFFI continues to work normally with the tracing ecosystem.

## 🔧 Feature Flags

LogFFI uses minimal feature flags:

```toml
[dependencies]
logffi = { version = "0.2", features = ["callback"] }
```

- **`callback`** - Enable FFI callback support (optional, for cross-language integrations)

## 🌟 Use Cases

### Perfect for:

- ✅ **Modern Rust applications** wanting structured observability
- ✅ **Microservices** needing rich context and tracing
- ✅ **Cross-language projects** requiring log bridging to Python, Node.js, or C/C++
- ✅ **Rust libraries** embedded in other language ecosystems
- ✅ **Applications** migrating from `log` to `tracing`
- ✅ **Projects** needing automatic error logging with proper types

### Consider alternatives if:

- ❌ You just need basic text logging (use `log` + `env_logger`)
- ❌ You're happy with your current logging setup
- ❌ You don't need structured logging or error handling

## 🤝 Ecosystem Compatibility

LogFFI works seamlessly with the entire tracing ecosystem:

- **[`tracing-subscriber`](https://docs.rs/tracing-subscriber)** - Flexible subscriber implementations
- **[`tracing-opentelemetry`](https://docs.rs/tracing-opentelemetry)** - OpenTelemetry integration
- **[`console-subscriber`](https://docs.rs/console-subscriber)** - Tokio Console integration
- **[`tracing-appender`](https://docs.rs/tracing-appender)** - File output and rotation
- **[`tracing-flame`](https://docs.rs/tracing-flame)** - Flamegraph profiling

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

Built on top of the excellent [`tracing`](https://docs.rs/tracing) and [`thiserror`](https://docs.rs/thiserror) crates. Special thanks to the Rust logging ecosystem maintainers.
