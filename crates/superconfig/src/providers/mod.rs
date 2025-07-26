//! Enhanced configuration providers with performance optimizations
//!
//! SuperConfig provides enhanced providers that extend Figment with enterprise-grade capabilities:
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
//! **Usage:**
//! ```rust
//! use superconfig::Universal;
//!
//! let provider = Universal::file("config");  // Auto-detects format
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
//! **Usage:**
//! ```rust
//! use superconfig::Nested;
//!
//! let provider = Nested::prefixed("APP_");  // Enhanced env parsing
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
//! **Usage:**
//! ```rust
//! use superconfig::Empty;
//! use figment::providers::Serialized;
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct CliArgs { debug: bool }
//!
//! let cli_args = CliArgs { debug: true };
//! let filtered = Empty::new(Serialized::defaults(cli_args));
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
//! **Usage Examples:**
//! ```rust
//! use superconfig::{Wildcard, MergeOrder};
//!
//! # fn main() -> Result<(), figment::Error> {
//! // Traditional hierarchy (git-like config discovery)
//! let provider = Wildcard::hierarchical("config", "myapp")?;
//!
//! // Single directory with custom ordering
//! let provider = Wildcard::new("./config/*.toml")?
//!     .with_merge_order(MergeOrder::Alphabetical);
//!
//! // Complex enterprise pattern
//! let provider = Wildcard::new("{/etc/myapp,~/.config/myapp,./config}/**/*.{toml,yaml,json}")?
//!     .with_merge_order(MergeOrder::Custom(vec![
//!         "base.*".to_string(),
//!         "*.toml".to_string(),
//!         "overrides.*".to_string(),
//!     ]));
//!
//! // Recursive plugin discovery
//! let provider = Wildcard::new("./plugins/**/config.yaml")?
//!     .with_merge_order(MergeOrder::ModificationTimeDescending);
//! # Ok(())
//! # }
//! ```
//!
//! ## Performance Characteristics
//!
//! All providers implement optimization strategies:
//! - **Lazy Loading**: Resources loaded only when needed
//! - **Intelligent Caching**: Results cached by modification time and content
//! - **Efficient Parsing**: Single-pass processing with type inference
//! - **Memory Optimized**: Minimal memory footprint for large configurations

pub mod cascade;  // Keep for now - will be removed after testing
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

// Legacy export (will be removed)
pub use cascade::Hierarchical;
