//! # SuperConfig
//!
//! **Universal Configuration Management Platform supporting Multiple Popular Languages and configuration formats** - Starting with 100% Figment compatibility, evolving into the ultimate config solution.
//!
//! SuperConfig builds on [Figment](https://github.com/SergioBenitez/Figment)'s foundation to create a comprehensive configuration platform that solves enterprise challenges while maintaining seamless compatibility.
//!
//! ## 🚀 Why SuperConfig?
//!
//! While Figment excels at Rust configuration, modern applications need more:
//!
//! - **Enterprise-grade hierarchical configuration** with system → user → project cascades
//! - **Advanced array composition** with `_add`/`_remove` patterns across all sources
//! - **Intelligent format handling** with content-based detection and performance caching
//! - **Multi-language support** for popular languages through WebAssembly and API interfaces
//! - **Production-ready optimizations** for real-world applications
//!
//! ## 🎯 Core Capabilities
//!
//! ### 🔧 Enhanced Providers (Beyond Figment)
//! - **Universal** - Smart format detection with caching and content analysis
//! - **Nested** - Advanced environment variable parsing with JSON arrays and type detection
//! - **Empty** - Automatic empty value filtering while preserving meaningful falsy values
//! - **Hierarchical** - Configuration cascade system across directory hierarchy
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
//! ```rust
//! use superconfig::SuperConfig;  // Recommended: clean all-in-one API
//! // or
//! use superconfig::prelude::*;    // For existing Figment users: add superpowers to current setup
//! ```
//!
//! ## 🔗 100% Figment Compatibility
//!
//! SuperConfig is fully compatible with existing Figment code:
//! - All Figment methods work seamlessly
//! - Existing Figment configurations can be enhanced without changes
//! - SuperConfig can be converted to/from regular Figment instances
//! - No breaking changes to your existing Figment workflow
//!
//! ## Two Ways to Use SuperConfig
//!
//! Choose the approach that best fits your project:
//!
//! ### Approach A: Enhance Existing Figment Setup (Extension Pattern)
//!
//! **For teams with existing Figment code** - Add SuperConfig powers without changing your setup:
//!
//! ```rust
//! use figment::Figment;
//! use superconfig::prelude::*;  // Everything: traits + providers
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Config { name: String }
//!
//! let cli_args = Config { name: "test".to_string() };
//!
//! let config = Figment::new()                     // Keep existing Figment code
//!     .merge(Universal::file("config"))           // Enhanced provider
//!     .merge_extend(Nested::prefixed("APP_"))     // Extension trait method
//!     .merge(Empty::new(figment::providers::Serialized::defaults(cli_args))); // Enhanced provider
//! ```
//!
//! ### Approach B: Pure SuperConfig (All-in-One Pattern)
//!
//! **For new projects or clean rewrites** - Use SuperConfig's fluent builder directly:
//!
//! ```rust,no_run
//! use superconfig::SuperConfig;
//! use serde::{Deserialize, Serialize};
//! use figment::providers::Serialized;
//!
//! #[derive(Debug, Deserialize, Serialize, Default)]
//! struct AppConfig {
//!     name: String,
//!     port: u16,
//! }
//!
//! let cli_args = AppConfig {
//!     name: "myapp".to_string(),
//!     port: 3000,
//! };
//!
//! let config: AppConfig = SuperConfig::new()
//!     .with_defaults(AppConfig::default())  // Start with defaults
//!     .with_hierarchical_config("myapp")    // Load config hierarchy
//!     .with_env("APP_")                     // Enhanced env parsing with JSON arrays
//!     .with_provider(Serialized::defaults(cli_args))  // CLI overrides
//!     .extract()?;                          // Direct extraction with auto array merging
//!
//! # Ok::<(), figment::Error>(())
//! ```

use figment::Figment;
use std::ops::Deref;

// Re-export figment for compatibility
pub use figment;

// Core modules
pub mod ext;
pub mod flags;
pub mod providers;

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
    pub figment: Figment,
    current_flags: crate::flags::Config,
}

impl SuperConfig {
    /// Create a new SuperConfig instance
    pub fn new() -> Self {
        Self {
            figment: Figment::new(),
            current_flags: crate::flags::DEFAULT,
        }
    }

    /// Create SuperConfig from an existing Figment
    pub fn from_figment(figment: Figment) -> Self {
        Self { 
            figment,
            current_flags: crate::flags::DEFAULT,
        }
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

// Re-export enhanced providers for existing Figment users
pub use providers::{Empty, Hierarchical, Nested, Universal};

// Re-export extension traits
pub use ext::{AccessExt, ExtendExt, FluentExt};

// Re-export prelude module for convenient imports
pub use ext::prelude;
