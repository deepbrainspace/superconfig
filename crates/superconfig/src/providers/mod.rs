//! Enhanced configuration providers with performance optimizations
//!
//! SuperConfig provides four enhanced providers that extend Figment with enterprise-grade capabilities:
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
//! ### Hierarchical Provider - Configuration Cascade
//! Git-like configuration inheritance across system → user → project levels
//! with automatic merging and array composition support.
//!
//! **Key Features:**
//! - **Search Hierarchy**: `~/.config/app/`, `~/.app/`, `~/`, ancestor dirs, current dir
//! - **Automatic Merging**: Later configs override earlier ones with array merging
//! - **Git-like Behavior**: Similar to `.gitconfig` hierarchical resolution
//!
//! **Usage:**
//! ```rust
//! use superconfig::Hierarchical;
//!
//! let provider = Hierarchical::new("myapp");  // Search config hierarchy
//! ```
//!
//! ## Performance Characteristics
//!
//! All providers implement optimization strategies:
//! - **Lazy Loading**: Resources loaded only when needed
//! - **Intelligent Caching**: Results cached by modification time and content
//! - **Efficient Parsing**: Single-pass processing with type inference
//! - **Memory Optimized**: Minimal memory footprint for large configurations

pub mod cascade;
pub mod env;
pub mod filter;
pub mod format;

pub use cascade::Hierarchical;
pub use env::Nested;
pub use filter::Empty;
pub use format::Universal;
