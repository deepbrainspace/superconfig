# SuperConfig

Configuration management library with hierarchical cascading, advanced array merging, and intelligent format detection. Built on [Figment](https://github.com/SergioBenitez/Figment) with 100% compatibility.

[![Crates.io](https://img.shields.io/crates/v/superconfig)](https://crates.io/crates/superconfig)
[![Documentation](https://docs.rs/superconfig/badge.svg)](https://docs.rs/superconfig)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## Features

- **Hierarchical Configuration**: System → user → project config cascade
- **Advanced Array Merging**: `_add`/`_remove` patterns for array composition  
- **Smart Format Detection**: Content-based parsing with caching
- **Enhanced Environment Variables**: JSON parsing and nested structure support
- **Figment Compatibility**: Drop-in replacement for existing Figment code

## Providers

- **Universal** - Smart format detection with caching
- **Nested** - Advanced environment variable parsing with JSON support
- **Empty** - Automatic empty value filtering
- **Hierarchical** - Configuration cascade across directory hierarchy

## Extension Traits

- **ExtendExt** - Array merging with `_add`/`_remove` patterns
- **FluentExt** - Builder methods (`.with_file()`, `.with_env()`, `.with_hierarchical_config()`)
- **AccessExt** - Convenience methods (`.as_json()`, `.get_string()`, `.debug_config()`)

## Quick Start

### SuperConfig Builder (New Projects)
```rust
use superconfig::SuperConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
struct AppConfig {
    name: String,
    port: u16,
    features: Vec<String>,
}

let config: AppConfig = SuperConfig::new()
    .with_hierarchical_config("myapp")  // System → user → project cascade
    .with_file("config")                // Auto-detects .toml/.json/.yaml
    .with_env("APP_")                   // Nested env vars with JSON parsing
    .extract()?;
# Ok::<(), figment::Error>(())
```

### Extension Traits (Existing Figment Projects)
```rust
use figment::Figment;
use superconfig::prelude::*;

let config = Figment::new()
    .merge_extend(Universal::file("config"))    // Enhanced format detection
    .merge_extend(Nested::prefixed("APP_"))     // Advanced env parsing
    .with_hierarchical_config("myapp");         // Add hierarchy support
```

## Examples

### Hierarchical Configuration
```rust
// Searches configs in order:
// 1. ~/.config/myapp/config.*
// 2. ~/.myapp/config.*  
// 3. ../config.*, ../../config.*, etc.
// 4. ./config.*
let config = SuperConfig::new()
    .with_hierarchical_config("myapp")
    .with_env("MYAPP_")
    .extract()?;
```

### Array Merging
```toml
# base.toml
features = ["auth", "logging"]

# override.toml  
features_add = ["metrics"]
features_remove = ["logging"]

# Result: features = ["auth", "metrics"]
```

### Environment Variables
```bash
export APP_DATABASE_HOST="localhost"           # → database.host
export APP_FEATURES='["auth", "cache"]'        # → features (JSON array)
export APP_SETTINGS='{"debug": true}'          # → settings (JSON object)
```

## Documentation 

- **[API Documentation](https://docs.rs/superconfig)** - Complete API reference

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.
