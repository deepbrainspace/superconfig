//! Extension traits for enhanced Figment functionality
//!
//! This module provides three powerful extension traits that supercharge regular Figment with
//! enterprise-grade capabilities. These traits can be applied to any existing Figment instance
//! to add advanced configuration management features.
//!
//! ## Extension Trait Overview
//!
//! ### ExtendExt - Advanced Array Merging
//! Adds sophisticated array composition capabilities using `_add` and `_remove` patterns
//! across all configuration sources.
//!
//! **Key Features:**
//! - **Smart Pattern Detection**: Only processes when `_add`/`_remove` patterns detected (performance optimized)
//! - **Cross-Source Merging**: Works across files, environment variables, CLI args, etc.
//! - **Recursive Processing**: Handles nested objects and complex data structures
//! - **Deduplication**: Removes duplicates during merge operations
//!
//! **Example:**
//! ```toml
//! # base.toml
//! features = ["auth", "logging"]
//!
//! # override.toml
//! features_add = ["metrics", "cache"]
//! features_remove = ["logging"]
//! # Result: features = ["auth", "metrics", "cache"]
//! ```
//!
//! ### FluentExt - Fluent API Builder
//! Provides builder-style methods for common configuration patterns using SuperConfig's
//! enhanced providers with automatic array merging.
//!
//! **Key Features:**
//! - **Enhanced Providers**: Uses Universal, Nested, Empty, and Hierarchical providers internally
//! - **Automatic Array Merging**: All `with_*` methods include `ExtendExt` functionality
//! - **Fluent Chaining**: Clean, readable configuration setup
//! - **Optional Parameters**: Graceful handling of optional configuration sources
//!
//! **Common Methods:**
//! - `with_file(path)` - Smart format detection with caching
//! - `with_env(prefix)` - JSON parsing + automatic nesting
//! - `with_hierarchical_config(name)` - Git-like config cascade
//! - `with_cli_opt(args)` - Empty value filtering
//!
//! ### AccessExt - Configuration Introspection
//! Adds convenience methods for accessing, exporting, and debugging configuration data.
//!
//! **Key Features:**
//! - **Format Export**: Export to JSON, YAML, TOML with pretty printing
//! - **Value Extraction**: Type-safe access to nested configuration values
//! - **Validation Helpers**: Check key existence and validate structure
//! - **Debug Tools**: Source tracking and configuration introspection
//!
//! **Common Methods:**
//! - `as_json()`, `as_yaml()`, `as_toml()` - Format export
//! - `get_string(key)`, `get_array(key)` - Type-safe value access
//! - `has_key(key)`, `keys()` - Configuration validation
//! - `debug_config()`, `debug_sources()` - Development tools
//!
//! ## Usage Examples
//!
//! ### With Regular Figment (Extension Traits)
//! ```rust,no_run
//! use figment::Figment;
//! use superconfig::prelude::*;  // Import all extension traits
//!
//! let config = Figment::new()
//!     .with_file("config")    // FluentExt method
//!     .with_env("APP_");      // FluentExt method  
//! ```
//!
//! ### With SuperConfig (Built-in Methods)
//! ```rust,no_run
//! use superconfig::SuperConfig;  // No prelude needed
//!
//! let config = SuperConfig::new()
//!     .with_file("config")    // Built-in method
//!     .with_env("APP_");      // Built-in method
//! ```
//!
//! ### Selective Import (Advanced)
//! ```rust
//! use superconfig::ExtendExt;  // Just array merging
//! use superconfig::{FluentExt, AccessExt};  // Builder + convenience
//! ```
//!
//! ## Performance Considerations
//!
//! Extension traits are designed for zero-cost abstractions:
//! - **Compile-Time Optimization**: Trait methods are inlined
//! - **Lazy Processing**: Array merging only when patterns detected
//! - **Efficient Caching**: Results cached at provider level
//! - **Memory Efficient**: No additional memory overhead

pub mod access;
pub mod extend;
pub mod fluent;

// Individual extension traits
pub use access::AccessExt;
pub use extend::ExtendExt;
pub use fluent::FluentExt;

/// Prelude module for convenient imports of all SuperConfig functionality
///
/// Import this module with `use superconfig::prelude::*` to get everything:
/// - Extension traits: `ExtendExt`, `FluentExt`, `AccessExt`
/// - Enhanced providers: `Universal`, `Nested`, `Empty`, `Hierarchical`
///
/// ## Example
/// ```rust,no_run
/// use figment::Figment;
/// use superconfig::prelude::*;  // Everything you need!
///
/// let config = Figment::new()
///     .merge(Universal::file("config"))  // Enhanced provider
///     .with_env("APP_")                  // Extension trait method
///     .merge_extend(Nested::prefixed("DB_")); // Extension trait method
/// let json = config.as_json()?;          // Extension trait method
/// # Ok::<(), figment::Error>(())
/// ```
pub mod prelude {
    // Extension traits - add methods to regular Figment
    pub use super::access::AccessExt;
    pub use super::extend::ExtendExt;
    pub use super::fluent::FluentExt;

    // Enhanced providers - drop-in replacements with superpowers
    pub use crate::providers::{Empty, Hierarchical, Nested, Universal};
}
