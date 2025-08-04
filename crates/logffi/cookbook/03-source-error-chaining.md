# Source Error Chaining

LogFFI integrates seamlessly with thiserror's `#[source]` attribute to provide proper error chaining that works with the Rust ecosystem.

## Basic Source Error Chaining

```rust
use logffi::define_errors;
use std::io;

define_errors! {
    pub enum FileError {
        #[error("Failed to read file: {path}")]
        ReadError {
            path: String,
            #[source]
            source: io::Error,
        },
        
        #[error("Failed to write file: {path}")]
        WriteError {
            path: String,
            #[source]
            source: io::Error,
        },
        
        #[error("File not found: {path}")]
        NotFound {
            path: String,
        },
    }
}

// Usage - the source error is preserved
fn read_config(path: &str) -> Result<String, FileError> {
    std::fs::read_to_string(path)
        .map_err(|source| FileError::ReadError {
            path: path.to_string(),
            source,
        })
}

// Error chain is accessible
fn handle_error(error: FileError) {
    eprintln!("Error: {}", error);
    
    // Walk the error chain
    let mut current = &error as &dyn std::error::Error;
    while let Some(source) = current.source() {
        eprintln!("  Caused by: {}", source);
        current = source;
    }
}
```

**Output:**

```
Error: Failed to read file: /etc/app.conf
  Caused by: No such file or directory (os error 2)
```

## Chaining Multiple Error Types

```rust
use logffi::define_errors;
use std::io;

define_errors! {
    pub enum AppError {
        #[error("Configuration error")]
        Config {
            #[source]
            source: ConfigError,
        },
        
        #[error("Database error")]
        Database {
            #[source]
            source: DatabaseError,
        },
        
        #[error("Validation failed: {message}")]
        Validation {
            message: String,
        },
    }
}

define_errors! {
    pub enum ConfigError {
        #[error("Failed to load config from {path}")]
        LoadError {
            path: String,
            #[source]
            source: io::Error,
        },
        
        #[error("Invalid config format")]
        ParseError {
            #[source]
            source: serde_json::Error,
        },
    }
}

define_errors! {
    pub enum DatabaseError {
        #[error("Connection failed to {host}:{port}")]
        ConnectionError {
            host: String,
            port: u16,
            #[source]
            source: io::Error,
        },
        
        #[error("Query failed: {query}")]
        QueryError {
            query: String,
        },
    }
}

// Multi-level error chaining
fn load_app_config() -> Result<Config, AppError> {
    let config_path = "app.json";
    
    let content = std::fs::read_to_string(config_path)
        .map_err(|source| ConfigError::LoadError {
            path: config_path.to_string(),
            source,
        })
        .map_err(|source| AppError::Config { source })?;
    
    let config: Config = serde_json::from_str(&content)
        .map_err(|source| ConfigError::ParseError { source })
        .map_err(|source| AppError::Config { source })?;
    
    Ok(config)
}
```

## Working with Dynamic Errors

```rust
use logffi::define_errors;
use std::error::Error;

define_errors! {
    pub enum FlexibleError {
        #[error("External service error: {service}")]
        ExternalService {
            service: String,
            #[source]
            source: Box<dyn Error + Send + Sync>,
        },
        
        #[error("Plugin error: {plugin}")]
        PluginError {
            plugin: String,
            #[source]
            source: Box<dyn Error + Send + Sync>,
        },
    }
}

// Can wrap any error type
fn call_external_service(name: &str) -> Result<Response, FlexibleError> {
    match external_api::call() {
        Ok(response) => Ok(response),
        Err(api_error) => Err(FlexibleError::ExternalService {
            service: name.to_string(),
            source: Box::new(api_error),
        }),
    }
}

// Works with different error types
fn load_plugin(name: &str) -> Result<Plugin, FlexibleError> {
    plugin_loader::load(name)
        .map_err(|load_error| FlexibleError::PluginError {
            plugin: name.to_string(),
            source: Box::new(load_error),
        })
}
```

## Integration with Popular Error Libraries

### With anyhow

```rust
use anyhow::{Context, Result};
use logffi::define_errors;

define_errors! {
    pub enum DomainError {
        #[error("User {id} not found")]
        UserNotFound { id: u64 },
        
        #[error("Permission denied for user {id}")]
        PermissionDenied { id: u64 },
    }
}

// anyhow can wrap our errors
fn process_user_request(id: u64) -> Result<()> {
    let user = get_user(id)
        .context("Failed to fetch user from database")?;
    
    if !user.is_admin {
        return Err(DomainError::new_permission_denied(id).into());
    }
    
    Ok(())
}

// Our errors provide context to anyhow
fn get_user(id: u64) -> Result<User, DomainError> {
    database::find(id)
        .ok_or_else(|| DomainError::new_user_not_found(id))
}
```

### With thiserror (advanced patterns)

```rust
use logffi::define_errors;
use thiserror::Error;

// Mix LogFFI errors with pure thiserror errors
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("DNS resolution failed")]
    DnsError(#[source] std::io::Error),
    
    #[error("TLS handshake failed")]
    TlsError(#[source] native_tls::Error),
}

define_errors! {
    pub enum ServiceError {
        #[error("Network operation failed", level = error, target = "network")]
        Network {
            #[source]
            source: NetworkError,
        },
        
        #[error("Timeout after {seconds}s", level = warn)]
        Timeout {
            seconds: u64,
        },
    }
}
```

## Best Practices for Error Chaining

```rust
define_errors! {
    pub enum BestPracticeError {
        // DO: Specific error messages with context
        #[error("Failed to parse config file '{file}' at line {line}")]
        ConfigParseError {
            file: String,
            line: usize,
            #[source]
            source: serde_json::Error,
        },
        
        // DON'T: Generic messages without context
        // #[error("Parse error")]
        // BadParseError {
        //     #[source]
        //     source: serde_json::Error,
        // },
        
        // DO: Preserve error chains for debugging
        #[error("Database migration failed for version {version}")]
        MigrationError {
            version: String,
            #[source]
            source: sqlx::Error,
        },
        
        // DO: Use specific error types when possible
        #[error("HTTP request failed with status {status}")]
        HttpError {
            status: u16,
            #[source]
            source: reqwest::Error,
        },
    }
}

// Helper for rich error context
fn parse_config_with_context(path: &str) -> Result<Config, BestPracticeError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| std::io::Error::new(
            e.kind(),
            format!("reading config file '{}'", path)
        ))?;
    
    // Track line numbers for better errors
    for (line_num, line) in content.lines().enumerate() {
        if let Err(e) = validate_line(line) {
            return Err(BestPracticeError::ConfigParseError {
                file: path.to_string(),
                line: line_num + 1,
                source: e,
            });
        }
    }
    
    Ok(config)
}
```

## Logging Error Chains

```rust
use std::error::Error;

// Helper function to log full error chain
fn log_error_chain(error: &dyn Error) {
    logffi::error!("Error: {}", error);
    
    let mut current = error;
    let mut depth = 1;
    while let Some(source) = current.source() {
        logffi::error!("  {}: Caused by: {}", depth, source);
        current = source;
        depth += 1;
    }
}

// Automatic logging with chains
fn process_with_logging() -> Result<(), AppError> {
    match dangerous_operation() {
        Ok(result) => Ok(result),
        Err(error) => {
            // Log the full chain
            log_error_chain(&error);
            
            // Or use the built-in log() which logs the primary error
            error.log();
            
            Err(error)
        }
    }
}
```
