//! Advanced Configuration Providers for SuperConfig
//!
//! SuperConfig includes powerful, enterprise-grade configuration providers that go far beyond basic file loading:
//!
//! ## Provider Overview
//!
//! ### Universal Provider - Smart Format Detection
//! Automatically detects configuration file formats (JSON, TOML, YAML) with intelligent
//! caching and extension fallback for optimal performance.
//!
//! **Key Features:**
//! - **4-Scenario Detection**: Standard files, misnamed files, unknown extensions, auto-extension search
//! - **Performance Optimized**: Content-based detection with modification time caching
//! - **Robust Fallbacks**: Graceful handling of missing or corrupted files
//!
//! **Usage with SuperConfig:**
//! ```rust
//! use superconfig::SuperConfig;
//!
//! let config = SuperConfig::new()
//!     .with_file("config");  // Auto-detects format internally
//! ```
//!
//! ### Nested Provider - Advanced Environment Variables
//! Enhanced environment variable parsing with JSON support, automatic nesting,
//! and smart type detection.
//!
//! **Key Features:**
//! - **JSON Parsing**: `APP_FEATURES='["auth", "cache"]'` → `features` array
//! - **Automatic Nesting**: `APP_DATABASE_HOST=localhost` → `database.host`
//! - **Smart Type Detection**: Strings, numbers, booleans, arrays, objects
//! - **Performance Caching**: Optimized parsing with intelligent caching
//!
//! **Usage with SuperConfig:**
//! ```rust
//! use superconfig::SuperConfig;
//!
//! let config = SuperConfig::new()
//!     .with_env("APP_");  // Enhanced env parsing built-in
//! ```
//!
//! ### Empty Provider - Clean Configuration
//! Filters out empty values while preserving meaningful falsy values,
//! perfect for CLI argument processing.
//!
//! **Key Features:**
//! - **Smart Filtering**: Removes empty strings, arrays, objects
//! - **Preserves Intent**: Keeps `false`, `0`, and other intentional values
//! - **CLI Integration**: Perfect for filtering meaningless CLI arguments
//!
//! **Usage with SuperConfig:**
//! ```rust
//! use superconfig::SuperConfig;
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct CliArgs { debug: bool }
//!
//! let config = SuperConfig::new()
//!     .with_cli_opt(Some(CliArgs { debug: true }));  // Automatic empty filtering
//! ```
//!
//! ### Wildcard Provider - Unified Pattern-Based Discovery
//! Revolutionary unified provider using globset patterns for flexible configuration discovery.
//! Replaces hierarchical and single-directory providers with a single, powerful solution.
//!
//! **Key Features:**
//! - **Any Glob Pattern**: Single directory, hierarchy, recursive - any pattern supported
//! - **Multiple Ordering Strategies**: Alphabetical, size, time, or custom rule-based sorting
//! - **High Performance**: Leverages OS-optimized glob matching and compiled patterns
//! - **Enterprise Ready**: Handles complex multi-source configuration scenarios
//! - **100% Figment Compatible**: Works seamlessly with both Figment and SuperConfig APIs
//!
//! **Usage with SuperConfig:**
//! ```rust
//! use superconfig::{SuperConfig, Wildcard};
//!
//! // Git-style hierarchical discovery (most common)
//! let config = SuperConfig::new()
//!     .with_hierarchical_config("myapp");  // Built-in hierarchical loading
//!
//! // Advanced pattern-based discovery
//! let config = SuperConfig::new()
//!     .merge(Wildcard::new("./config/*.toml"))                    // Directory patterns
//!     .merge(Wildcard::new("/etc/myapp/**/*.yaml"))               // Recursive patterns  
//!     .merge(Wildcard::new("./plugins/**/config.json"));          // Plugin discovery
//!
//! // Multi-source configuration  
//! # use serde::Serialize;
//! # #[derive(Serialize)]
//! # struct CliArgs { debug: bool }
//! # let cli_args = Some(CliArgs { debug: true });
//! let config = SuperConfig::new()
//!     .merge(Wildcard::hierarchical("config", "myapp"))           // Git-style discovery
//!     .with_env("MYAPP_")                                         // Environment variables
//!     .with_cli_opt(cli_args);                                    // CLI arguments
//! ```
//!
//! ## Performance Characteristics
//!
//! All providers implement optimization strategies:
//! - **Lazy Loading**: Resources loaded only when needed
//! - **Intelligent Caching**: Results cached by modification time and content
//! - **Efficient Parsing**: Single-pass processing with type inference
//! - **Memory Optimized**: Minimal memory footprint for large configurations

pub mod env;
pub mod filter;
pub mod format;
pub mod wildcard;

// New unified exports
pub use wildcard::{Wildcard, WildcardBuilder, SearchStrategy, MergeOrder};

// Existing exports
pub use env::Nested;
pub use filter::Empty;
pub use format::Universal;

