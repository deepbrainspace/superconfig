# SuperConfig

**Universal Configuration Management Platform supporting Multiple Popular Languages and configuration formats** - Starting with 100% Figment compatibility, evolving into the ultimate config solution.

[![Crates.io](https://img.shields.io/crates/v/superconfig)](https://crates.io/crates/superconfig)
[![Documentation](https://docs.rs/superconfig/badge.svg)](https://docs.rs/superconfig)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 🚀 Why SuperConfig?

While [Figment](https://github.com/SergioBenitez/Figment) is excellent for Rust configuration, modern applications need more. SuperConfig builds on Figment's foundation to create a **universal configuration management platform** that:

- **Solves enterprise configuration challenges** with hierarchical config cascades
- **Provides multi-language support** for popular languages through WebAssembly and API interfaces  
- **Offers advanced array merging** with `_add`/`_remove` patterns
- **Delivers intelligent format handling** with content-based detection and caching
- **Maintains 100% Figment compatibility** for seamless migration

## ✨ Core Capabilities

### 🎯 Enterprise-Grade Configuration Management

- **Hierarchical Configuration Cascade**: Git-like config inheritance across system → user → project levels
- **Advanced Array Merging**: Compose configurations with `features_add`/`features_remove` patterns
- **Intelligent Format Detection**: Content-based parsing with fallback strategies and caching
- **Environment Variable Intelligence**: JSON parsing, nested structure creation, smart type detection
- **Production-Ready Performance**: Optimized caching, lazy loading, graceful error handling

### 🔧 Enhanced Providers (Beyond Figment)

- **Universal** - Smart format detection with caching and content analysis
- **Nested** - Advanced environment variable parsing with JSON arrays and type detection  
- **Empty** - Automatic empty value filtering while preserving meaningful falsy values
- **Hierarchical** - Configuration cascade system across directory hierarchy

### 🚀 Extension Traits (Supercharge existing Figment code)

- **ExtendExt** - Array merging with `_add`/`_remove` patterns across all sources
- **FluentExt** - Builder methods (`.with_file()`, `.with_env()`, `.with_hierarchical_config()`)
- **AccessExt** - Convenience methods (`.as_json()`, `.get_string()`, `.debug_config()`)

### 💫 SuperConfig Builder (All-in-one solution)

- Built-in methods combining all enhancements
- Zero import complexity for new projects
- Direct Figment compatibility through Deref

## 🎯 Quick Start

### For New Projects (Recommended)
```rust
use superconfig::SuperConfig;
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
    timeout: u32,
}

// One-liner configuration with all enhancements
let config: AppConfig = SuperConfig::new()
    .with_hierarchical_config("myapp")  // System → user → project cascade
    .with_file("config")                // Auto-detects .toml/.json/.yaml
    .with_env("APP_")                   // Nested env vars with JSON parsing
    .with_cli_opt(cli_args)             // Filtered CLI args (if Some)
    .extract()?;

# Ok::<(), figment::Error>(())
```

### For Existing Figment Projects
```rust
use figment::Figment;
use superconfig::prelude::*;  // Add superpowers to existing Figment code

let config = Figment::new()                     // Keep existing code
    .merge_extend(Universal::file("config"))    // Enhanced format detection
    .merge_extend(Nested::prefixed("APP_"))     // Advanced env parsing
    .with_hierarchical_config("myapp");         // Add hierarchy support
```

## 🏗️ Enterprise Configuration Patterns

### Hierarchical Configuration Cascade
```rust
// Automatically searches and merges configs from:
// 1. ~/.config/myapp/config.*
// 2. ~/.myapp/config.*  
// 3. ../config.*, ../../config.*, etc.
// 4. ./config.*
let config = SuperConfig::new()
    .with_defaults(AppConfig::default())
    .with_hierarchical_config("myapp")
    .with_env("MYAPP_")
    .with_cli_opt(cli_args);
```

### Advanced Array Composition
```toml
# base.toml
features = ["auth", "logging"]
allowed_origins = ["https://app.com"]

# override.toml  
features_add = ["metrics", "tracing"]
features_remove = ["logging"]
allowed_origins_add = ["https://admin.com"]
allowed_origins_remove = ["https://app.com"]

# Result after merging:
# features = ["auth", "metrics", "tracing"]
# allowed_origins = ["https://admin.com"]
```

### Environment Variable Intelligence
```bash
# Smart parsing of complex environment variables
export APP_DATABASE_HOST="localhost"           # → database.host
export APP_FEATURES='["auth", "cache"]'        # → features (JSON array)
export APP_SETTINGS='{"debug": true}'          # → settings (JSON object)
export APP_TIMEOUT=30                          # → timeout (number)
export APP_ENABLED=true                        # → enabled (boolean)
```

## 🌟 Real-World Use Cases

### ✅ Enterprise Application Configuration
Perfect for applications that need configuration inheritance, environment-specific overrides, and team collaboration.

### ✅ Multi-Format Development Teams  
Automatically handles different format preferences (JSON for frontend, TOML for Rust, YAML for DevOps).

### ✅ CI/CD Pipeline Configuration
Smart environment variable parsing with empty value filtering for clean deployment configs.

### ✅ Microservice Configuration Management
Hierarchical configs enable shared base configurations with service-specific overrides.

## 🔮 Vision: Universal Configuration Platform

SuperConfig's roadmap extends far beyond Rust:

### 🌐 Multi-Language Support (Planned)
- **WebAssembly Integration**: Use SuperConfig from JavaScript, Python, .NET, and other popular languages
- **REST API Server**: Centralized configuration management for distributed systems
- **CLI Tool**: Generate and validate configurations across supported languages

### 🏢 Enterprise Features (Planned)  
- **Database Backends**: Persistent configuration storage with versioning
- **HashiCorp Vault Integration**: Secure secret management
- **Remote Configuration**: Centralized config distribution and hot-reloading
- **MCP Integration**: Model Context Protocol for AI-powered configuration

### 🛠️ Developer Tooling (Planned)
- **Schema Generation**: Auto-generate configuration schemas from structs
- **Configuration Validation**: Rich validation with helpful error messages  
- **Hot Reloading**: Runtime configuration updates without restarts

## 📚 Documentation

- **[API Documentation](https://docs.rs/superconfig)** - Complete API reference
- **[Figment Compatibility Guide](docs/figment-compatibility.md)** - Migration from Figment
- **[Enterprise Patterns](docs/enterprise-patterns.md)** - Advanced configuration architectures
- **[Multi-Language Roadmap](docs/multi-language.md)** - WebAssembly and API plans

## 🤝 Contributing

SuperConfig is building the future of configuration management. We welcome contributions across:

- **Core Library**: Rust enhancements and optimizations
- **Multi-Language Bindings**: WebAssembly implementations  
- **Enterprise Features**: Database, vault, and API integrations
- **Developer Tooling**: CLI tools and schema generators

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## 📄 License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

**SuperConfig**: From Figment-compatible library to universal configuration platform. 🚀# CI test run
