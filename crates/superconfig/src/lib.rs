//! # SuperConfig
//!
//! **Next-Generation Configuration Management Platform** - Advanced configuration management with hierarchical cascading, intelligent array composition, smart format detection, and enterprise-grade optimizations. Designed for modern applications that demand flexibility, performance, and sophisticated configuration patterns.
//!
//! Built on [Figment's](https://github.com/SergioBenitez/Figment) solid foundation, SuperConfig adds the advanced features modern applications need while maintaining 100% compatibility for existing Figment users.
//!
//! ## 🚀 Advanced Configuration Features
//!
//! SuperConfig provides enterprise-grade capabilities for sophisticated configuration management:
//!
//! - **Hierarchical Configuration**: Git-like config inheritance across system → user → project levels
//! - **Advanced Array Composition**: Intelligent merging with `_add`/`_remove` patterns across all sources
//! - **Smart Format Detection**: Content-based parsing with caching and performance optimizations
//! - **Enhanced Environment Variables**: JSON parsing, nested structures, and smart type detection
//! - **Configuration Debugging**: Built-in introspection, source tracking, and validation tools
//! - **Production Optimizations**: Lazy loading, modification time caching, and optimized data structures
//!
//! ## 🎯 Core Capabilities
//!
//! ### 🔧 Enhanced Providers (Beyond Figment)
//! - **Universal** - Smart format detection with caching and content analysis
//! - **Nested** - Advanced environment variable parsing with JSON arrays and type detection
//! - **Empty** - Automatic empty value filtering while preserving meaningful falsy values
//! - **Wildcard** - Unified pattern-based configuration discovery with globset integration
//!
//! ### 🚀 Extension Traits (Supercharge existing Figment code)
//! - **ExtendExt** - Array merging with `_add`/`_remove` patterns across all sources
//! - **FluentExt** - Builder methods (`.with_file()`, `.with_env()`, `.with_hierarchical_config()`)
//! - **AccessExt** - Convenience methods (`.as_json()`, `.get_string()`, `.debug_config()`)
//!
//! ### 💫 SuperConfig Builder (All-in-one solution)
//! - Built-in methods combining all enhancements  
//! - Zero import complexity for new projects
//! - Direct Figment compatibility through Deref
//!
//! ## 🎯 Quick Start
//! ```rust,no_run
//! use superconfig::SuperConfig;  // Recommended: clean all-in-one API
//! // or
//! use superconfig::prelude::*;    // For existing Figment users: add superpowers to current setup
//! ```
//!
//! ## ⚡ Performance Characteristics
//!
//! SuperConfig implements several optimization strategies:
//!
//! - **Lazy Loading**: Files only read when needed, cached by modification time
//! - **Smart Detection**: Content-based format detection with fallback chains  
//! - **Conditional Processing**: Array merging only when `_add`/`_remove` patterns detected
//! - **Efficient Caching**: Parsed environment variables and file contents cached
//! - **Memory Efficient**: Optimized data structures for large configurations
//!
//! ## 🔗 100% Figment Compatibility
//!
//! **Bonus Feature**: SuperConfig is fully compatible with existing Figment code:
//! - All Figment methods work seamlessly
//! - Existing Figment configurations can be enhanced without changes
//! - SuperConfig can be converted to/from regular Figment instances
//! - No breaking changes to your existing Figment workflow
//!
//! ## Usage Approaches
//!
//! Choose the approach that best fits your project:
//!
//! ### Primary: SuperConfig Platform (Recommended)
//!
//! **The recommended way** - Experience SuperConfig's full power with clean, intuitive APIs:
//!
//! ```rust,no_run
//! use superconfig::SuperConfig;  // Only import you need
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Deserialize, Serialize, Default)]
//! struct AppConfig {
//!     name: String,
//!     port: u16,
//!     features: Vec<String>,
//! }
//!
//! let cli_args = AppConfig {
//!     name: "myapp".to_string(),
//!     port: 3000,
//!     ..Default::default()
//! };
//!
//! let config: AppConfig = SuperConfig::new()
//!     .with_defaults(AppConfig::default())        // Set smart defaults
//!     .with_hierarchical_config("myapp")          // System → user → project cascade
//!     .with_file("config")                        // Auto-detects .toml/.json/.yaml
//!     .with_env("APP_")                           // JSON parsing + nesting
//!     .with_cli_opt(Some(cli_args))               // Filtered CLI overrides
//!     .extract()?;                                // Direct extraction
//!
//! # Ok::<(), figment::Error>(())
//! ```
//!
//! ### Alternative: Figment Compatibility Mode
//!
//! **For existing Figment users** - Add SuperConfig's advanced features to your current Figment setup without changing existing code:
//!
//! ```rust,no_run
//! use figment::Figment;
//! use superconfig::prelude::*;  // Add SuperConfig superpowers
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Config { name: String, features: Vec<String> }
//!
//! let cli_args = Config {
//!     name: "myapp".to_string(),
//!     features: vec!["auth".to_string()]
//! };
//!
//! let config = Figment::new()                           // Keep existing Figment code
//!     .merge_extend(Universal::file("config"))          // Enhanced provider
//!     .merge_extend(Nested::prefixed("APP_"))           // Enhanced provider  
//!     .merge_extend(Hierarchical::new("myapp"))         // Enhanced provider
//!     .merge_extend(Empty::new(                         // Enhanced provider
//!         figment::providers::Serialized::defaults(cli_args)
//!     ));
//!
//! // All extension traits available:
//! let json_output = config.as_json()?;                 // AccessExt
//! let has_redis = config.has_key("redis.enabled")?;    // AccessExt
//! # Ok::<(), figment::Error>(())
//! ```
//!
//! ## 🔍 Configuration Debugging
//!
//! SuperConfig provides rich debugging capabilities for development and troubleshooting:
//!
//! ```rust,no_run
//! use superconfig::{SuperConfig, AccessExt};
//!
//! let config = SuperConfig::new()
//!     .with_hierarchical_config("myapp")
//!     .with_env("APP_");
//!
//! // Export in different formats
//! let json_config = config.as_json()?;
//! let yaml_config = config.as_yaml()?;
//! let toml_config = config.as_toml()?;
//!
//! // Value extraction and validation  
//! let db_host = config.get_string("database.host")?;
//! let features = config.get_array::<String>("features")?;
//! let has_redis = config.has_key("redis.enabled")?;
//! let all_keys = config.keys()?;
//!
//! // Full debug output with source tracking
//! let debug_output = config.debug_config()?;
//! println!("{}", debug_output);
//!
//! // Source metadata for troubleshooting
//! let sources = config.debug_sources();
//! for source in sources {
//!     println!("Provider: {:?}", source);
//! }
//! # Ok::<(), figment::Error>(())
//! ```

use figment::Figment;
use std::ops::Deref;

// Re-export figment for compatibility
pub use figment;

pub mod ext;
pub mod providers;
mod fluent;

// Re-export enhanced providers for existing Figment users
pub use providers::{Empty, Hierarchical, MergeOrder, Nested, Universal, Wildcard, WildcardBuilder, SearchStrategy};

// Re-export extension traits
pub use ext::{AccessExt, ExtendExt};
// FluentExt removed - methods now native to SuperConfig

// Re-export prelude module for convenient imports
pub use ext::prelude;

/// SuperConfig is a universal configuration management platform that combines
/// enterprise-grade features with 100% Figment compatibility.
///
/// Built on Figment's solid foundation, SuperConfig adds production-ready capabilities
/// including hierarchical configuration cascades, advanced array merging, intelligent
/// format detection, and performance optimizations - while maintaining seamless
/// compatibility with existing Figment code.
///
/// ## Enterprise Features
///
/// - **Hierarchical Configuration**: Git-like config inheritance across system → user → project levels
/// - **Advanced Array Merging**: Compose configurations with `_add`/`_remove` patterns
/// - **Intelligent Format Detection**: Content-based parsing with caching and fallback strategies
/// - **Performance Optimized**: Lazy loading, caching, and optimized data structures
///
/// ## Universal Platform Vision
///
/// SuperConfig is designed to become the universal configuration standard across popular
/// languages through WebAssembly bindings, REST APIs, and protocol standardization.
#[derive(Debug, Clone)]
pub struct SuperConfig {
    figment: Figment,
}

impl SuperConfig {
    /// Create a new SuperConfig instance
    pub fn new() -> Self {
        Self {
            figment: Figment::new(),
        }
    }

    /// Create SuperConfig from an existing Figment
    pub fn from_figment(figment: Figment) -> Self {
        Self { figment }
    }

    /// Extract configuration directly (equivalent to calling .extract() on the inner Figment)
    ///
    /// This is a convenience method that makes the SuperConfig API more fluent by avoiding
    /// the need to dereference before extraction.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use superconfig::SuperConfig;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Deserialize, Serialize, Default)]
    /// struct Config {
    ///     #[serde(default)]
    ///     host: String,
    ///     #[serde(default)]
    ///     port: u16,
    /// }
    ///
    /// let config: Config = SuperConfig::new()
    ///     .with_defaults(Config::default())
    ///     .with_file("config.toml")
    ///     .with_env("APP_")
    ///     .extract()?;                 // Direct extraction with all enhancements
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn extract<'de, T: serde::Deserialize<'de>>(&self) -> Result<T, figment::Error> {
        self.figment.extract()
    }
}

impl Default for SuperConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Deref to Figment provides 100% compatibility - all Figment methods work seamlessly
impl Deref for SuperConfig {
    type Target = Figment;

    fn deref(&self) -> &Self::Target {
        &self.figment
    }
}

impl From<Figment> for SuperConfig {
    fn from(figment: Figment) -> Self {
        Self::from_figment(figment)
    }
}

impl From<SuperConfig> for Figment {
    fn from(super_figment: SuperConfig) -> Self {
        super_figment.figment
    }
}

// Fluent methods are now implemented directly in fluent.rs
