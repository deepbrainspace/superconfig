# LogFFI Cookbook: Configuring Destinations

This guide shows how to configure different logging destinations for each backend in LogFFI.

## Table of Contents

- [Log Backend Destinations](#log-backend-destinations)
- [Tracing Backend Destinations](#tracing-backend-destinations)
- [Slog Backend Destinations](#slog-backend-destinations)
- [Important Notes](#important-notes)

## Log Backend Destinations

### File + Console Logging with Fern

```toml
[dependencies]
logffi = { version = "0.2", default-features = false, features = ["log"] }
fern = "0.6"
chrono = "0.4"
```

```rust
use logffi::{info, error, warn};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure fern before any LogFFI usage
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;

    // Now use LogFFI - it automatically uses fern
    info!("Application started");
    warn!("This is a warning");
    error!("This is an error");
    
    Ok(())
}
```

### Rolling File Logs with log4rs

```toml
[dependencies]
logffi = { version = "0.2", default-features = false, features = ["log"] }
log4rs = "1.2"
```

```rust
use logffi::{info, error};
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = ConsoleAppender::builder().build();
    let file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build("log/app.log")?;

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file)))
        .build(Root::builder()
            .appender("stdout")
            .appender("file")
            .build(log::LevelFilter::Info))?;

    log4rs::init_config(config)?;
    
    info!("Logs to console and file");
    error!("Errors are logged too");
    
    Ok(())
}
```

### Syslog Integration

```toml
[dependencies]
logffi = { version = "0.2", default-features = false, features = ["log"] }
syslog = "6.1"
```

```rust
use logffi::{info, warn, error};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let formatter = syslog::Formatter3164::default();
    let logger = syslog::unix(formatter)?;
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(log::LevelFilter::Info);
    
    info!("Application started");
    warn!("Warning message to syslog");
    error!("Error message to syslog");
    
    Ok(())
}
```

## Tracing Backend Destinations

### Multiple Layers: Console + JSON File + Jaeger

```toml
[dependencies]
logffi = { version = "0.2", features = ["tracing"] }
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-opentelemetry = "0.21"
opentelemetry = "0.21"
opentelemetry-jaeger = "0.20"
```

```rust
use logffi::{info, error, debug};
use tracing_subscriber::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Console output layer
    let console_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_thread_ids(true);
    
    // JSON file output layer
    let file = std::fs::File::create("app.json")?;
    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(file);
    
    // Jaeger tracing layer
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("my-app")
        .install_simple()?;
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    
    // Combine all layers
    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .with(telemetry_layer)
        .init();
    
    // Use LogFFI - logs go to all destinations
    info!("Application started");
    error!("Error occurred", error_code = 500);
    debug!("Debug information");
    
    Ok(())
}
```

### Elasticsearch with Structured Logging

```toml
[dependencies]
logffi = { version = "0.2", features = ["tracing"] }
tracing-subscriber = "0.3"
tracing-elastic-apm = "3.0"
```

```rust
use logffi::{info, error};
use tracing_subscriber::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let apm_layer = tracing_elastic_apm::new_layer(
        tracing_elastic_apm::Config::new("http://localhost:8200")
            .with_service_name("logffi-app")
            .with_service_version("1.0.0")
    )?;
    
    tracing_subscriber::registry()
        .with(apm_layer)
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    info!("User logged in", user_id = 12345, ip = "192.168.1.1");
    error!("Payment failed", order_id = "ORD-789", amount = 99.99);
    
    Ok(())
}
```

### AWS CloudWatch Logs

```toml
[dependencies]
logffi = { version = "0.2", features = ["tracing"] }
tracing-subscriber = "0.3"
tracing-cloudwatch = "0.1"
```

```rust
use logffi::{info, warn};
use tracing_subscriber::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cloudwatch_layer = tracing_cloudwatch::layer()
        .with_log_group("/aws/lambda/my-function")
        .with_log_stream("instance-1")
        .with_region("us-east-1");
    
    tracing_subscriber::registry()
        .with(cloudwatch_layer)
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    info!("Lambda function started");
    warn!("High memory usage detected", usage_percent = 85);
    
    Ok(())
}
```

## Slog Backend Destinations

### Multiple Drains: Terminal + File + Syslog

```toml
[dependencies]
logffi = { version = "0.2", default-features = false, features = ["slog"] }
slog-term = "2.9"
slog-syslog = "0.12"
slog-async = "2.8"
```

```rust
use logffi::{info, error, logger};
use slog::{Drain, Logger, o};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Terminal drain with color
    let term_drain = slog_term::FullFormat::new(
        slog_term::TermDecorator::new().build()
    ).build().fuse();
    
    // File drain
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("app.log")?;
    let file_drain = slog_term::FullFormat::new(
        slog_term::PlainSyncDecorator::new(file)
    ).build().fuse();
    
    // Syslog drain
    let syslog_drain = slog_syslog::unix_3164(slog_syslog::Facility::LOG_USER)?
        .fuse();
    
    // Combine all drains
    let drain = slog::Duplicate::new(
        term_drain,
        slog::Duplicate::new(file_drain, syslog_drain)
    );
    
    // Make it async for better performance
    let drain = slog_async::Async::new(drain).build().fuse();
    
    // Create custom root logger
    let custom_logger = Logger::root(drain, o!("version" => "1.0"));
    
    // Now we need to use slog macros directly with this logger
    slog::info!(custom_logger, "Application started");
    slog::error!(custom_logger, "Critical error"; "code" => 500);
    
    Ok(())
}
```

### Elasticsearch with Slog

```toml
[dependencies]
logffi = { version = "0.2", default-features = false, features = ["slog"] }
slog-elasticsearch = "0.1"
slog-async = "2.8"
```

```rust
use logffi::logger;
use slog::{Drain, Logger, o};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let es_drain = slog_elasticsearch::ElasticsearchDrain::new(
        "http://localhost:9200",
        "logs",
        slog_elasticsearch::Options::default()
    )?;
    
    let drain = slog_async::Async::new(es_drain).build().fuse();
    let custom_logger = Logger::root(drain, o!("app" => "logffi"));
    
    // Use with slog macros
    slog::info!(custom_logger, "Indexed in Elasticsearch"; 
        "user_id" => 12345, 
        "action" => "login"
    );
    
    Ok(())
}
```

### Hierarchical Logging

```toml
[dependencies]
logffi = { version = "0.2", default-features = false, features = ["slog"] }
slog-term = "2.9"
slog-async = "2.8"
```

```rust
use logffi::logger;
use slog::{Drain, Logger, o, info, error};

fn main() {
    // Create root logger
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    
    let root = Logger::root(drain, o!("app" => "my-service"));
    
    // Create hierarchical loggers
    let db_logger = root.new(o!("component" => "database"));
    let api_logger = root.new(o!("component" => "api"));
    let auth_logger = api_logger.new(o!("subcomponent" => "auth"));
    
    // Use hierarchical loggers
    info!(db_logger, "Connected to database");
    error!(auth_logger, "Authentication failed"; "user" => "admin");
}
```

## Important Notes

### Current Limitations

⚠️ **The current LogFFI implementation requires you to import and configure the underlying logging libraries yourself.** This is a known limitation that should be addressed.

### Why This Happens

1. **Log Backend**: LogFFI initializes with `env_logger` by default, but if you initialize another logger first (like `fern`), the log crate will use that instead.

2. **Tracing Backend**: You must configure subscribers yourself because tracing needs specific configuration for different outputs.

3. **Slog Backend**: LogFFI creates a basic root logger, but for advanced features you need to create your own logger instances.

### Recommended Improvements

LogFFI should provide wrapper APIs to configure destinations without requiring direct imports:

```rust
// This is what we SHOULD have:
use logffi::backends::log::destinations::Fern;

logffi::configure_log_backend(
    Fern::new()
        .console()
        .file("app.log")
        .level(logffi::Level::Debug)
)?;

// Not requiring:
use fern; // Should not be needed!
```

This would provide true auto-deref functionality where LogFFI handles all the complexity internally.
