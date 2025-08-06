# LogFFI Cookbook

Welcome to the LogFFI cookbook! This collection of guides shows you how to use LogFFI's tracing-native logging and structured data capabilities effectively in real-world scenarios.

## 📚 Guides

### [1. Basic Logging](01-basic-logging.md)

Learn the fundamentals of LogFFI's tracing-native logging:

- Simple logging statements with auto-initialization
- Target-based logging for module organization
- Environment-based configuration with RUST_LOG
- Integration with the tracing ecosystem
- Performance considerations and best practices

### [2. Structured Logging](02-structured-logging.md)

Master structured logging for modern observability:

- Adding fields to log messages for machine-readable data
- Best practices for field naming and types
- Integration with monitoring platforms (ELK, Grafana, Datadog)
- Real-world patterns for authentication, payments, and APIs
- JSON output configuration

### [3. Error Handling](03-error-handling.md)

Master the **dual-syntax `define_errors!` macro** for sophisticated error handling:

- **🆕 LogFFI Format** - Clean, attribute-based syntax for modern error definitions
- **🔧 Thiserror Compatibility** - Full backward compatibility with existing code
- **⚡ Advanced Features** - Multiple error types, auto source chaining, mixed variants
- Field interpolation and structured logging integration
- Custom error codes for monitoring and alerting
- Testing error scenarios

### [4. Spans and Instrumentation](04-spans-instrumentation.md)

Leverage tracing spans for request correlation and performance monitoring:

- Creating and entering spans with structured context
- Using `#[instrument]` for automatic span creation
- Nested spans for complex operations
- Async/await compatibility
- Integration with distributed tracing systems

### [5. Advanced Tracing Integration](05-tracing-integration.md)

Advanced patterns for the tracing ecosystem:

- Custom tracing subscribers and layers
- OpenTelemetry integration for distributed tracing
- Log crate bridge for legacy library compatibility
- Filtering and sampling strategies
- Performance optimization techniques

### [6. FFI and Cross-Language Integration](06-ffi-integration.md)

Bridge Rust logs to other languages using callbacks:

- Python integration with PyO3
- Node.js integration with Neon
- C/C++ integration patterns
- WebAssembly support
- Callback system architecture and best practices

## 🚀 Quick Start

If you're new to LogFFI, start with:

1. **[Basic Logging](01-basic-logging.md)** - Learn the fundamentals and auto-initialization
2. **[Structured Logging](02-structured-logging.md)** - Add rich metadata to your logs
3. **[Error Handling](03-error-handling.md)** - See the power of `define_errors!`

For specific use cases:

- **Microservices/APIs** → [Structured Logging](02-structured-logging.md) + [Spans](04-spans-instrumentation.md)
- **Cross-language projects** → [FFI Integration](06-ffi-integration.md)
- **Migrating from `log`** → [Tracing Integration](05-tracing-integration.md)

## 🎯 Common Patterns

### "I want structured logs for better observability"

```rust
use logffi::info;

info!(
    user_id = 12345,
    action = "login", 
    ip_address = "192.168.1.1",
    duration_ms = 145,
    "User authentication successful"
);
```

→ See [Structured Logging](02-structured-logging.md) for comprehensive patterns

### "I need automatic error logging with proper types"

**🆕 LogFFI Format** (Recommended):

```rust
use logffi::define_errors;

define_errors! {
    ApiError {
        UserNotFound { user_id: u64 } : "User not found: {user_id}" [level = warn, target = "api::users"],
        RateLimitExceeded { requests: u32, limit: u32 } : "Rate limit exceeded: {requests}/{limit}" [level = error],
        NetworkTimeout { source: std::io::Error } : "Network operation timed out"  // Auto source chaining
    }
}

// Errors have structured logging built-in
let error = ApiError::UserNotFound { user_id: 12345 };
error.log(); // WARN api::users: [UserNotFound] User not found: 12345
```

**Traditional Thiserror Syntax** (Also supported):

```rust
define_errors! {
    pub enum ApiError {
        #[error("User not found: {user_id}", level = warn, target = "api::users")]
        UserNotFound { user_id: u64 },
        
        #[error("Rate limit exceeded: {requests}/{limit}", level = error)]
        RateLimitExceeded { requests: u32, limit: u32 },
    }
}
```

→ Check [Error Handling](03-error-handling.md) for advanced patterns

### "I want to trace requests across async operations"

```rust
use logffi::{info, info_span};
use tracing::instrument;

#[instrument(level = "info")]
async fn process_request(user_id: u64) -> Result<String, ApiError> {
    info!("Processing user request");
    
    let span = info_span!("database_query", table = "users");
    let _enter = span.enter();
    
    // Database operations are now traced within the span
    info!("Executing user lookup");
    
    Ok("Success".to_string())
}
```

→ Read [Spans and Instrumentation](04-spans-instrumentation.md) for detailed examples

### "I need to bridge Rust logs to Python/Node.js"

```rust
// Enable callback feature
logffi = { version = "0.2", features = ["callback"] }
```

```rust
use logffi::{info, error};

fn main() {
    // These messages go to both tracing AND external callbacks
    info!("This reaches both Rust tracing and Python callbacks");
    error!("Error messages are bridged to external systems");
}
```

→ See [FFI Integration](06-ffi-integration.md) for language-specific examples

### "I want JSON logs for production monitoring"

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().json())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
        
    // Now all LogFFI logs output as JSON
    logffi::info!(
        service = "user-api",
        version = "1.2.3", 
        "Service started"
    );
}
```

→ Check [Advanced Tracing Integration](05-tracing-integration.md) for production setups

## 🔧 Configuration Quick Reference

### Environment Variables

```bash
# Basic log level control
RUST_LOG=debug cargo run

# Target specific modules
RUST_LOG=myapp::database=debug,myapp::auth=info cargo run

# Filter by span names  
RUST_LOG=process_order=trace cargo run
```

### Feature Flags

```toml
[dependencies]
logffi = { version = "0.2", features = ["callback"] }
```

### Dependencies for Advanced Use Cases

```toml
[dependencies]
logffi = "0.2"

# For JSON output and advanced filtering
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# For OpenTelemetry integration
tracing-opentelemetry = "0.22"

# For file output and rotation
tracing-appender = "0.2"
```

## 💡 Best Practices

- **Use structured fields consistently** - Adopt consistent naming conventions (snake_case, clear semantics)
- **Add context with spans** - Wrap related operations in spans for better correlation
- **Leverage auto-initialization** - Let LogFFI handle basic setup, customize only when needed
- **Test error paths** - LogFFI makes it easy to test error logging scenarios
- **Use appropriate log levels** - Reserve ERROR for actual problems, INFO for business events
- **Consider field cardinality** - Avoid fields with unlimited unique values in high-volume scenarios

## 🔗 External Resources

- **[Tracing Documentation](https://docs.rs/tracing)** - Official tracing crate docs
- **[Tracing Subscriber Guide](https://docs.rs/tracing-subscriber)** - Advanced subscriber configuration
- **[OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust)** - Distributed tracing
- **[Structured Logging Best Practices](https://engineering.grab.com/structured-logging)** - Industry patterns

## 📝 Contributing

Found a great pattern or use case? We welcome contributions to the cookbook! Please submit a PR with:

- Clear, runnable examples
- Real-world context and motivation
- Performance considerations where relevant
- Integration tips for common tools

## 🆘 Getting Help

- **[GitHub Issues](https://github.com/your-org/logffi/issues)** - Bug reports and feature requests
- **[API Documentation](https://docs.rs/logffi)** - Complete API reference
- **[Examples Directory](../examples/)** - More runnable code samples
