# Backend Configuration

LogFFI supports multiple logging backends and can dynamically switch between them at runtime.

## Available Backends

```rust
use logffi::{set_backend, Backend, info};

fn configure_backend() {
    // Option 1: Standard log crate (default)
    set_backend(Backend::Log);
    info!("Using log crate backend");
    
    // Option 2: Tracing ecosystem
    set_backend(Backend::Tracing);
    info!("Using tracing backend");
    
    // Option 3: Slog structured logging
    set_backend(Backend::Slog);
    info!("Using slog backend");
}
```

## Configuring the Log Backend

```rust
use log::LevelFilter;
use env_logger::Builder;
use logffi::info;

fn setup_env_logger() {
    // Basic setup
    env_logger::init();
    
    // Or with custom configuration
    Builder::new()
        .filter_level(LevelFilter::Debug)
        .filter_module("my_app", LevelFilter::Trace)
        .filter_module("hyper", LevelFilter::Warn)
        .format_timestamp_millis()
        .init();
    
    info!("env_logger initialized");
}

// With custom format
fn setup_custom_format() {
    use std::io::Write;
    
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .init();
}

// With file output
fn setup_file_logging() {
    use std::fs::File;
    
    let target = Box::new(File::create("app.log").unwrap());
    
    Builder::new()
        .target(env_logger::Target::Pipe(target))
        .init();
}
```

## Configuring Tracing Backend

```rust
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use logffi::{set_backend, Backend, info, debug};

fn setup_tracing() {
    // Basic setup
    tracing_subscriber::fmt::init();
    
    // Tell LogFFI to use tracing
    set_backend(Backend::Tracing);
    
    info!("Tracing initialized");
}

// Advanced tracing configuration
fn setup_advanced_tracing() {
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true);
    
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();
    
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
    
    set_backend(Backend::Tracing);
}

// JSON output for structured logging
fn setup_json_tracing() {
    let fmt_layer = fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true);
    
    tracing_subscriber::registry()
        .with(fmt_layer)
        .init();
    
    set_backend(Backend::Tracing);
    
    info!(user_id = 123, action = "login", "User logged in");
    // Output: {"timestamp":"2024-01-15T10:30:00Z","level":"INFO","target":"app","fields":{"user_id":123,"action":"login","message":"User logged in"}}
}

// Multiple outputs
fn setup_multi_output_tracing() {
    use tracing_subscriber::fmt::writer::MakeWriterExt;
    
    let file = std::fs::File::create("app.log").unwrap();
    let debug_file = std::fs::File::create("debug.log").unwrap();
    
    let all_files = file
        .and(debug_file.with_min_level(tracing::Level::DEBUG))
        .and(std::io::stdout.with_max_level(tracing::Level::INFO));
    
    tracing_subscriber::fmt()
        .with_writer(all_files)
        .init();
    
    set_backend(Backend::Tracing);
}
```

## Configuring Slog Backend

```rust
use slog::{o, Drain, Logger};
use logffi::{set_backend, Backend, init_with_slog, info};

fn setup_basic_slog() {
    // Terminal output with color
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    
    let logger = Logger::root(drain, o!("app" => "myapp"));
    
    init_with_slog(logger);
    set_backend(Backend::Slog);
    
    info!("Slog initialized");
}

// JSON structured logging
fn setup_json_slog() {
    let drain = slog_json::Json::new(std::io::stdout())
        .add_default_keys()
        .build()
        .fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    
    let logger = Logger::root(
        drain,
        o!(
            "version" => env!("CARGO_PKG_VERSION"),
            "host" => hostname::get().unwrap().to_string_lossy().to_string(),
        )
    );
    
    init_with_slog(logger);
    set_backend(Backend::Slog);
    
    info!("Structured logging enabled"; "user_id" => 123, "action" => "startup");
}

// Multiple drains
fn setup_multi_drain_slog() {
    use slog::Duplicate;
    
    // Console drain
    let term_decorator = slog_term::TermDecorator::new().build();
    let term_drain = slog_term::FullFormat::new(term_decorator).build();
    
    // File drain
    let file = std::fs::File::create("app.log").unwrap();
    let file_drain = slog_json::Json::new(file).build();
    
    // Combine drains
    let drain = Duplicate::new(term_drain, file_drain).fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    
    let logger = Logger::root(drain, o!());
    
    init_with_slog(logger);
    set_backend(Backend::Slog);
}
```

## Runtime Backend Switching

```rust
use logffi::{set_backend, Backend, info};
use std::env;

fn configure_from_environment() {
    // Read backend preference from environment
    let backend = env::var("LOG_BACKEND").unwrap_or_else(|_| "log".to_string());
    
    match backend.as_str() {
        "tracing" => {
            tracing_subscriber::fmt::init();
            set_backend(Backend::Tracing);
            info!("Using tracing backend");
        }
        "slog" => {
            setup_slog();
            set_backend(Backend::Slog);
            info!("Using slog backend");
        }
        _ => {
            env_logger::init();
            set_backend(Backend::Log);
            info!("Using log backend");
        }
    }
}

// Dynamic switching based on deployment environment
fn configure_for_deployment() {
    if cfg!(debug_assertions) {
        // Development: Pretty terminal output
        env_logger::Builder::from_default_env()
            .format_timestamp_millis()
            .init();
        set_backend(Backend::Log);
    } else {
        // Production: JSON structured logging
        let drain = slog_json::Json::new(std::io::stdout()).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let logger = slog::Logger::root(drain, o!());
        
        init_with_slog(logger);
        set_backend(Backend::Slog);
    }
}
```

## Backend-Specific Features

```rust
use logffi::{with_tracing, with_slog, info};

fn use_backend_features() {
    // Access tracing-specific features
    with_tracing!(|subscriber| {
        use tracing::span;
        
        let span = span!(tracing::Level::INFO, "operation", user_id = 123);
        let _enter = span.enter();
        
        info!("Inside span");
    });
    
    // Access slog-specific features
    with_slog!(|logger| {
        use slog::Logger;
        
        let child_logger = logger.new(o!("component" => "database"));
        slog::info!(child_logger, "Database operation"; "query" => "SELECT * FROM users");
    });
}
```

## Performance Considerations

```rust
use logffi::{set_backend, Backend};

fn optimize_for_performance() {
    // For high-throughput applications
    if high_throughput_mode() {
        // Use async slog with larger buffer
        let drain = slog_json::Json::new(std::io::stdout()).build();
        let drain = slog_async::Async::new(drain)
            .chan_size(10000)  // Larger buffer
            .overflow_strategy(slog_async::OverflowStrategy::Drop)
            .build()
            .fuse();
        
        let logger = slog::Logger::root(drain, o!());
        init_with_slog(logger);
        set_backend(Backend::Slog);
    } else {
        // Standard env_logger for normal load
        env_logger::init();
        set_backend(Backend::Log);
    }
}

// Conditional backend based on features
#[cfg(feature = "json-logs")]
fn setup_json_logs() {
    setup_json_slog();
}

#[cfg(not(feature = "json-logs"))]
fn setup_json_logs() {
    env_logger::init();
}
```

## Testing with Different Backends

```rust
#[cfg(test)]
mod tests {
    use logffi::{set_backend, Backend, info};
    
    #[test]
    fn test_with_log_backend() {
        // Use simple logger for tests
        let _ = env_logger::builder()
            .is_test(true)
            .try_init();
        
        set_backend(Backend::Log);
        info!("Test message");
    }
    
    #[test] 
    fn test_with_tracing_backend() {
        // Use tracing subscriber for tests
        let subscriber = tracing_subscriber::fmt()
            .with_test_writer()
            .finish();
        
        let _ = tracing::subscriber::set_global_default(subscriber);
        set_backend(Backend::Tracing);
        
        info!("Test with tracing");
    }
}
```
