# SuperConfig

A multilingual configuration management toolkit with advanced hierarchical cascading, intelligent array merging, smart format detection, and optimized patterns for high-performance applications.

## Crates in this Workspace

| Crate                  | Version                                                                                                         | Docs                                                                                                 | Description                                                                                 |
| ---------------------- | --------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| **superconfig**        | [![Crates.io](https://img.shields.io/crates/v/superconfig)](https://crates.io/crates/superconfig)               | [![Documentation](https://docs.rs/superconfig/badge.svg)](https://docs.rs/superconfig)               | Advanced configuration management with hierarchical cascading and performance optimizations |
| **superconfig-macros** | [![Crates.io](https://img.shields.io/crates/v/superconfig-macros)](https://crates.io/crates/superconfig-macros) | [![Documentation](https://docs.rs/superconfig-macros/badge.svg)](https://docs.rs/superconfig-macros) | Procedural macros for fluent API error handling and FFI integration                         |
| **logffi**             | [![Crates.io](https://img.shields.io/crates/v/logffi)](https://crates.io/crates/logffi)                         | [![Documentation](https://docs.rs/logffi/badge.svg)](https://docs.rs/logffi)                         | Drop-in replacement for log crate with FFI callback support                                 |
| **multiffi**           | [![Crates.io](https://img.shields.io/crates/v/multiffi)](https://crates.io/crates/multiffi)                     | [![Documentation](https://docs.rs/multiffi/badge.svg)](https://docs.rs/multiffi)                     | Multi-language FFI binding generator for Python, Node.js, and WebAssembly                   |

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

**SuperConfig** is a comprehensive configuration management ecosystem designed for modern multilingual applications. The core `superconfig` crate provides advanced configuration handling with hierarchical cascading and intelligent merging, while companion crates enable seamless integration across Python, Node.js, WebAssembly, and other languages through optimized FFI patterns.

Built from the ground up for maximum performance and flexibility, SuperConfig delivers enterprise-grade configuration management with zero-overhead abstractions.

## 🚀 Core Features

### Core Capabilities

- **🏗️ Hierarchical Configuration**: Git-like config inheritance across system → user → project levels
- **🔄 Advanced Array Merging**: Compose configurations with `_add`/`_remove` patterns across all sources
- **🧠 Intelligent Format Detection**: Content-based parsing with caching and performance optimizations
- **🌐 Enhanced Environment Variables**: JSON parsing, nested structures, and smart type detection
- **🔧 Production Optimizations**: Lazy loading, modification time caching, and optimized data structures
- **🔍 Configuration Debugging**: Built-in introspection, source tracking, and validation tools
- **🗣️ Verbosity System**: CLI-style debugging with `-v`, `-vv`, `-vvv` levels for troubleshooting configuration issues

- **Production-Ready**: Performance optimized for real-world applications
- **Figment Compatible**: Seamless upgrade path for existing Figment users

## 🔌 Enhanced Providers

### Universal Provider - Intelligent Format Detection

- **4-Scenario Detection Strategy**: Handles standard files, misnamed files, unknown extensions, and auto-extension search
- **Performance Optimized**: Content-based detection with modification time caching
- **Format Support**: JSON, TOML, YAML with automatic fallback chains
- **Example**: `Universal::file("config")` tries `config.toml`, `config.yaml`, `config.json` automatically

### Nested Provider - Advanced Environment Variables

- **JSON Parsing**: `APP_FEATURES='["auth", "cache"]'` → `features` array
- **Automatic Nesting**: `APP_DATABASE_HOST=localhost` → `database.host`
- **Smart Type Detection**: Strings, numbers, booleans, arrays, objects
- **Performance Caching**: Optimized parsing with intelligent caching

### Empty Provider - Clean Configuration

- **Smart Filtering**: Removes empty strings, arrays, objects while preserving meaningful falsy values
- **CLI Integration**: Perfect for filtering meaningless CLI arguments
- **Preserves Intent**: Keeps `false`, `0`, and other intentional values

### Hierarchical Provider - Configuration Cascade

- **Search Hierarchy**: `~/.config/app/`, `~/.app/`, `~/`, ancestor directories, current directory
- **Automatic Merging**: Later configs override earlier ones with array merging support
- **Git-like Behavior**: Similar to `.gitconfig` hierarchical resolution

## 🚀 Built-In Features

### Array Merging & Export

```rust
// Built into SuperConfig - no extension traits needed
let config = SuperConfig::new()
    .with_file("config")                    // Smart format detection
    .with_env("APP_")                       // Enhanced environment variables
    .with_hierarchical_config("myapp");     // Git-style discovery

// Rich export and debugging capabilities
let json = config.as_json()?;              // Export as JSON
let yaml = config.as_yaml()?;              // Export as YAML
let host = config.get_string("db.host")?;   // Extract values
let exists = config.has_key("redis")?;      // Check existence
let debug = config.debug_config()?;        // Full debug output
```

## 🚀 Quick Start

```rust
use superconfig::SuperConfig;  // Only import you need
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
struct AppConfig {
    name: String,
    port: u16,
    features: Vec<String>,
    database: DatabaseConfig,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct DatabaseConfig {
    host: String,
    port: u16,
}

let cli_args = AppConfig {
    name: "myapp".to_string(),
    port: 3000,
    ..Default::default()
};

let config: AppConfig = SuperConfig::new()
    .with_defaults(AppConfig::default())        // Set smart defaults
    .with_verbosity(VerbosityLevel::Debug)      // Enable configuration debugging
    .with_hierarchical_config("myapp")          // System → user → project cascade
    .with_file("config")                        // Auto-detects .toml/.json/.yaml
    .with_env("APP_")                           // JSON parsing + nesting
    .with_cli_opt(Some(cli_args))               // Filtered CLI overrides
    .extract()?;                                // Direct extraction

# Ok::<(), figment::Error>(())
```

## 💡 Real-World Examples

### Production Configuration Setup

```rust
use superconfig::SuperConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
struct AppConfig {
    server: ServerConfig,
    database: DatabaseConfig,
    features: Vec<String>,
    cors: CorsConfig,
    logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct ServerConfig {
    host: String,
    port: u16,
    workers: u32,
}

let config: AppConfig = SuperConfig::new()
    .with_defaults(AppConfig {
        server: ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            workers: 4,
        },
        features: vec!["basic".to_string()],
        ..Default::default()
    })
    .with_hierarchical_config("myapp")      // System-wide configs
    .with_file("config")                    // Project config
    .with_env("MYAPP_")                     // Environment overrides
    .with_cli_opt(cli_args)                 // Runtime overrides
    .extract()?;

# Ok::<(), figment::Error>(())
```

### Hierarchical Configuration Discovery

```rust
// with_hierarchical_config("myapp") searches in priority order:
//
// 1. ~/.config/myapp/myapp.{toml,yaml,json}    (XDG system config)
// 2. ~/.myapp/myapp.{toml,yaml,json}           (User home config)
// 3. ~/myapp.{toml,yaml,json}                  (Home root config)
// 4. ../../myapp.{toml,yaml,json}              (Ancestor directories)
// 5. ../myapp.{toml,yaml,json}                 (Parent directory)
// 6. ./myapp.{toml,yaml,json}                  (Current directory)
//
// Later configs override earlier ones with smart array merging

let config = SuperConfig::new()
    .with_hierarchical_config("myapp")
    .with_env("MYAPP_")
    .extract()?;
```

### Advanced Array Composition

```toml
# ~/.config/myapp/myapp.toml (system defaults)
[server]
features = ["auth", "logging", "metrics"]
ignore_paths = ["*.tmp", "*.log"]

[cors]
allowed_origins = ["https://app.example.com"]
```

```toml
# ./myapp.toml (project overrides)
[server]
features_add = ["debug", "hot-reload"] # Adds to existing array
features_remove = ["metrics"] # Removes from array
ignore_paths_add = ["*.cache", "build/*"] # Extends ignore patterns

[cors]
allowed_origins_add = ["http://localhost:3000"] # Add dev origin

# Final result:
# features = ["auth", "logging", "debug", "hot-reload"]
# ignore_paths = ["*.tmp", "*.log", "*.cache", "build/*"]
# allowed_origins = ["https://app.example.com", "http://localhost:3000"]
```

### Advanced Environment Variable Scenarios

```bash
# Simple nesting
export MYAPP_DATABASE_HOST="localhost"              # → database.host
export MYAPP_DATABASE_PORT="5432"                   # → database.port

# JSON arrays and objects
export MYAPP_FEATURES='["auth", "cache", "metrics"]' # → features (parsed as array)
export MYAPP_REDIS_CONFIG='{"host": "redis.example.com", "pool_size": 10}' # → redis.config

# Array composition via environment
export MYAPP_FEATURES_ADD='["debug"]'               # Adds "debug" to features array
export MYAPP_FEATURES_REMOVE='["cache"]'            # Removes "cache" from features

# Nested object construction
export MYAPP_SERVER_TLS_CERT_PATH="/etc/ssl/cert.pem"
export MYAPP_SERVER_TLS_KEY_PATH="/etc/ssl/key.pem"
# → server.tls.cert_path and server.tls.key_path
```

### Configuration Debugging & Introspection

```rust
use superconfig::{SuperConfig, AccessExt};

let config = SuperConfig::new()
    .with_hierarchical_config("myapp")
    .with_file("config")
    .with_env("MYAPP_");

// Export in different formats
let json_config = config.as_json()?;           // Pretty JSON
let yaml_config = config.as_yaml()?;           // YAML format
let toml_config = config.as_toml()?;           // TOML format

// Value extraction and validation
let db_host = config.get_string("database.host")?;
let features = config.get_array::<String>("features")?;
let has_redis = config.has_key("redis.enabled")?;
let all_keys = config.keys()?;

// Full debug output with source tracking
let debug_output = config.debug_config()?;
println!("{}", debug_output);
// Shows final merged config + which providers contributed each value

// Source metadata for troubleshooting
let sources = config.debug_sources();
for source in sources {
    println!("Provider: {:?}", source);
}

# Ok::<(), figment::Error>(())
```

### Performance Features

```rust
let config = SuperConfig::new()
    .with_hierarchical_config("prod-app")
    .with_file("config")
    .with_env_ignore_empty("APP_")
    .extract()?;

# Ok::<(), figment::Error>(())
```

## 🎯 When to Use SuperConfig

### ✅ Use Cases

- Complex applications with multiple environments
- Advanced configuration patterns and array merging
- Performance-critical systems
- Multi-source configuration loading
- Development teams needing debugging capabilities
- Production deployments requiring robust error handling

### 🔄 Figment Migration

Existing Figment users can easily migrate to SuperConfig's enhanced capabilities.

## ⚡ Performance

- **Lazy Loading**: Files cached by modification time
- **Smart Detection**: Content-based format detection
- **Conditional Processing**: Array merging only when needed
- **Efficient Caching**: Parsed data cached for reuse

## 🛠️ Advanced Features

### Configuration Validation

```rust
use superconfig::{SuperConfig, AccessExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
    #[serde(default = "default_max_connections")]
    max_connections: u32,
}

fn default_max_connections() -> u32 { 100 }

let config = SuperConfig::new()
    .with_hierarchical_config("myapp")
    .with_env("APP_");

// Validate configuration exists and is accessible
if !config.has_key("database.host")? {
    eprintln!("Warning: No database host configured");
}

// Extract with validation
let db_config: DatabaseConfig = config.extract_inner("database")?;
```

### Error Handling & Diagnostics

```rust
use superconfig::{SuperConfig, AccessExt};

let config = SuperConfig::new()
    .with_file("config")
    .with_env("APP_");

match config.extract::<AppConfig>() {
    Ok(cfg) => println!("Configuration loaded successfully"),
    Err(e) => {
        eprintln!("Configuration error: {}", e);

        // Debug what went wrong
        eprintln!("Debug info:\n{}", config.debug_config()?);

        // Show all sources
        for source in config.debug_sources() {
            eprintln!("Source: {:?}", source);
        }
    }
}
```

### Custom Provider Integration

```rust
use superconfig::{SuperConfig, ExtendExt};
use figment::Provider;

// Your custom provider
struct DatabaseProvider {
    connection_string: String,
}

impl Provider for DatabaseProvider {
    // Implementation details...
}

let config = SuperConfig::new()
    .with_hierarchical_config("myapp")
    .with_provider(DatabaseProvider { /* ... */ })  // Automatic array merging
    .with_env("APP_");
```

## 📚 Documentation & Resources

- **[API Documentation](https://docs.rs/superconfig)** - Complete API reference with examples
- **[Examples](crates/superconfig/examples/)** - Practical examples including verbosity system usage
- **[Figment Documentation](https://docs.rs/figment)** - Core concepts (SuperConfig is compatible)
- **[GitHub Repository](https://github.com/deepbrainspace/superconfig)** - Source code and issue tracking

## 🤝 Contributing

We welcome contributions! SuperConfig is designed to become the universal configuration standard.

- **Issues**: Bug reports, feature requests
- **Pull Requests**: Code improvements, documentation, examples
- **Discussions**: Architecture decisions, use cases, integrations

## 📄 License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

**SuperConfig** - Configuration management that scales with your application.
