# Basic Logging with LogFFI

LogFFI provides drop-in replacements for all standard Rust logging macros with enhanced capabilities.

## Simple Logging

```rust
use logffi::{error, warn, info, debug, trace};

fn main() {
    // Initialize env_logger (or any log-compatible backend)
    env_logger::init();

    // Basic logging - just like the log crate
    error!("This is an error message");
    warn!("This is a warning");
    info!("This is info");
    debug!("This is debug");
    trace!("This is trace");
}
```

**Output:**

```
[ERROR] This is an error message
[WARN ] This is a warning
[INFO ] This is info
[DEBUG] This is debug
[TRACE] This is trace
```

## Logging with Variables

```rust
use logffi::{info, error};

fn process_user(id: u64, name: &str) {
    info!("Processing user: {} (ID: {})", name, id);
    
    let result = do_something();
    if let Err(e) = result {
        error!("Failed to process user {}: {}", id, e);
    }
}
```

## Target-based Logging

```rust
use logffi::{info, debug};

fn database_operation() {
    info!(target: "database", "Connecting to database");
    debug!(target: "database::query", "Executing SELECT * FROM users");
}

fn api_handler() {
    info!(target: "api", "Handling GET /users request");
    debug!(target: "api::auth", "Validating JWT token");
}
```

**Benefits:**

- Filter logs by module: `RUST_LOG=database=debug`
- Hierarchical filtering: `RUST_LOG=api::auth=trace`
- Multiple targets: `RUST_LOG=database=info,api=debug`

## Structured Logging

```rust
use logffi::info;

#[derive(Debug)]
struct User {
    id: u64,
    name: String,
    email: String,
}

fn log_user_action(user: &User, action: &str) {
    info!(
        target: "audit",
        "User action - ID: {}, Name: {}, Action: {}",
        user.id, user.name, action
    );
    
    // Or log the entire struct
    info!(target: "audit", "User performed action: {:?}", user);
}
```

## Performance Considerations

```rust
use logffi::{debug, trace};

fn expensive_debug_info() -> String {
    // This computation only happens if debug level is enabled
    format!("Expensive calculation: {}", compute_something())
}

fn optimized_logging() {
    // ❌ Bad: Always computes the string
    debug!("Debug info: {}", expensive_debug_info());
    
    // ✅ Good: Only computes if debug is enabled
    if log::log_enabled!(log::Level::Debug) {
        debug!("Debug info: {}", expensive_debug_info());
    }
    
    // ✅ Also good: Use closures for lazy evaluation
    trace!("Trace info: {}", {
        // This block only executes if trace is enabled
        expensive_debug_info()
    });
}
```

## Integration with Backend Loggers

```rust
// With env_logger
fn with_env_logger() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();
    
    logffi::info!("Using env_logger backend");
}

// With tracing (via LogFFI's native support)
fn with_tracing() {
    use logffi::{set_backend, Backend};
    
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();
    
    // Tell LogFFI to use tracing
    set_backend(Backend::Tracing);
    
    logffi::info!("Using tracing backend");
}

// With slog
fn with_slog() {
    use logffi::{set_backend, Backend};
    use slog::{o, Drain};
    
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, o!());
    
    // Initialize LogFFI with slog
    logffi::init_with_slog(logger);
    set_backend(Backend::Slog);
    
    logffi::info!("Using slog backend");
}
```
