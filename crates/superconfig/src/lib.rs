//! # SuperConfig
//!
//! **Advanced Configuration Management System** - Built on Figment's proven foundation while retaining 100% compatibility, SuperConfig provides advanced features including hierarchical cascading, intelligent array merging, pattern-based discovery, resilient loading with warnings, and advanced environment variable processing.
//!
//! SuperConfig is designed for modern applications that demand sophisticated configuration patterns, high performance, and bulletproof reliability.
//!
//! ## 🚀 Advanced Configuration Features
//!
//! SuperConfig goes far beyond basic configuration loading with advanced capabilities:
//!
//! - **🔄 Resilient Loading**: Continues loading even when some configs fail, collecting warnings instead of crashing
//! - **🌳 Hierarchical Discovery**: Git-style config inheritance across system → user → project → local levels  
//! - **🔀 Intelligent Array Merging**: Advanced composition with `_add`/`_remove` patterns across all sources
//! - **🎯 Pattern-Based Discovery**: Powerful glob patterns for flexible multi-source configuration loading
//! - **🧠 Smart Format Detection**: Content-based parsing with intelligent caching and fallback strategies
//! - **⚡ Enhanced Environment Variables**: JSON parsing, automatic nesting, and smart type detection
//! - **🔍 Advanced Debugging**: Built-in introspection, source tracking, validation, and warning collection
//! - **🗣️ Verbosity System**: CLI-style debugging with multiple verbosity levels for troubleshooting configuration issues
//! - **🚀 Production Optimizations**: Lazy loading, modification time caching, and optimized data structures
//!
//! ## 🎯 SuperConfig All-in-One Solution
//!
//! SuperConfig provides everything you need in a single, powerful interface:
//!
//! ### 🔧 Advanced Configuration Providers
//! - **Universal Provider** - Smart format detection with intelligent caching and content analysis
//! - **Nested Environment Variables** - JSON parsing, automatic nesting, and smart type detection  
//! - **Empty Value Filtering** - Automatic filtering while preserving meaningful falsy values
//! - **Wildcard Pattern Discovery** - Unified glob-based configuration discovery with advanced sorting
//!
//! ### 🛠️ Built-in Configuration Methods
//! - **Fluent Builder API** - `.with_file()`, `.with_env()`, `.with_hierarchical_config()`, `.with_defaults()`
//! - **Array Merging** - Intelligent composition with `_add`/`_remove` patterns across all sources
//! - **Access & Export** - `.as_json()`, `.as_yaml()`, `.get_string()`, `.has_key()`, `.debug_config()`
//! - **Warning System** - Resilient loading with comprehensive error collection and reporting
//!
//! ### 💯 100% Figment Compatibility  
//! - All Figment methods and functionalities work out of the box with SuperConfig
//! - Drop-in replacement for existing Figment code
//! - Gradual migration path for existing projects
//!
//! ## 🎯 Quick Start
//!
//! SuperConfig provides a clean, powerful API for all your configuration needs:
//!
//! ```rust,no_run
//! use superconfig::SuperConfig;
//!
//! // Simple configuration loading
//! let config = SuperConfig::new()
//!     .with_file("config.toml")           // Smart format detection
//!     .with_env("APP_")                   // Enhanced environment variables
//!     .with_hierarchical_config("myapp"); // Git-style discovery
//! ```
//!
//! ## 🌟 Real-World Examples
//!
//! ### Web Application Configuration
//! ```rust,no_run
//! use superconfig::SuperConfig;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize, Serialize, Default)]
//! struct WebConfig {
//!     server: ServerConfig,
//!     database: DatabaseConfig,
//!     features: Vec<String>,
//! }
//!
//! #[derive(Deserialize, Serialize, Default)]
//! struct ServerConfig {
//!     host: String,
//!     port: u16,
//! }
//!
//! #[derive(Deserialize, Serialize, Default)]
//! struct DatabaseConfig {
//!     url: String,
//!     timeout: u32,
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Load configuration with intelligent fallbacks and validation
//! let super_config = SuperConfig::new()
//!     .with_defaults(WebConfig::default())
//!     .with_file("config.toml")                    // Base configuration
//!     .with_file("config.local.toml")              // Local overrides
//!     .with_env("WEBAPP_")                         // Environment variables
//!     .with_cli_opt(Some(std::env::args().collect::<Vec<_>>())); // CLI arguments
//!
//! let config: WebConfig = super_config.extract()?;
//!
//! // Check for configuration warnings
//! if super_config.has_warnings() {
//!     super_config.print_warnings();
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Microservice with Dynamic Discovery
//! ```rust,no_run
//! use superconfig::{SuperConfig, Wildcard};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Load configuration from multiple sources with pattern matching
//! let config = SuperConfig::new()
//!     .merge(Wildcard::hierarchical("config", "myservice"))  // Git-style discovery
//!     .merge(Wildcard::new("./config/features/*.toml"))      // Feature configs
//!     .merge(Wildcard::new("/etc/myservice/**/*.yaml"))      // System configs
//!     .with_env("SERVICE_");                                 // Environment
//!
//! // Export for debugging
//! println!("Final config: {}", config.as_yaml()?);
//! println!("Sources: {:#?}", config.debug_sources());
//! # Ok(())
//! # }
//! ```
//!
//! ### Production Configuration with Array Merging
//! ```rust,no_run
//! // config/base.toml
//! // features = ["auth", "logging"]
//! // allowed_ips = ["127.0.0.1"]
//!
//! // config/production.toml  
//! // features_add = ["metrics", "tracing"]
//! // features_remove = ["logging"]
//! // allowed_ips_add = ["10.0.0.0/8", "192.168.0.0/16"]
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # use superconfig::SuperConfig;
//! let config = SuperConfig::new()
//!     .with_file("config/base.toml")
//!     .with_file("config/production.toml");
//! # Ok(())
//! # }
//!
//! // Result: features = ["auth", "metrics", "tracing"]
//! // Result: allowed_ips = ["127.0.0.1", "10.0.0.0/8", "192.168.0.0/16"]
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
//! # fn main() -> Result<(), figment::Error> {
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
//! Ok(())
//! # }
//! ```
//!
//! ### Alternative: Figment Compatibility Mode
//!
//! **For existing Figment users** - SuperConfig provides all Figment functionality while adding enhanced features:
//!
//! ```rust,no_run
//! use superconfig::{SuperConfig, Universal, Nested};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Config { name: String, features: Vec<String> }
//!
//! let cli_args = Config {
//!     name: "myapp".to_string(),
//!     features: vec!["auth".to_string()]
//! };
//!
//! let config = SuperConfig::new()                      // SuperConfig with Figment compatibility
//!     .merge(Universal::file("config"))               // Enhanced provider
//!     .merge(Nested::prefixed("APP_"))                // Enhanced provider  
//!     .with_cli_opt(Some(cli_args));                  // SuperConfig method
//!
//! // All Figment methods work seamlessly
//! let result = config.extract::<Config>()?;
//! let json_output = config.as_json()?;                // SuperConfig extension
//! let has_redis = config.has_key("redis.enabled")?;   // SuperConfig extension
//! # Ok::<(), figment::Error>(())
//! ```
//!
//! ## 🔍 Configuration Debugging
//!
//! SuperConfig provides rich debugging capabilities for development and troubleshooting:
//!
//! ```rust,no_run
//! use superconfig::SuperConfig;
//!
//! # fn main() -> Result<(), figment::Error> {
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
//! # Ok(())
//! # }
//! ```

use crate::verbosity::{DebugCollector, DebugMessage};
use figment::Figment;
use std::cell::RefCell;
use std::ops::Deref;

// Re-export figment for compatibility
pub use figment;

pub mod access;
mod fluent;
pub mod merge;
pub mod providers;
pub mod verbosity;

// Re-export enhanced providers for existing Figment users
pub use providers::{
    Empty, MergeOrder, Nested, SearchStrategy, Universal, Wildcard, WildcardBuilder,
};

// Re-export verbosity types for clients
pub use verbosity::VerbosityLevel;

/// SuperConfig is a universal configuration management platform that combines
/// advanced features with 100% Figment compatibility.
///
/// Built on Figment's solid foundation, SuperConfig adds production-ready capabilities
/// including hierarchical configuration cascades, advanced array merging, intelligent
/// format detection, verbosity debugging, and performance optimizations - while maintaining
/// seamless compatibility with existing Figment code.
///
/// ## Core Features
///
/// - **Hierarchical Configuration**: Git-like config inheritance across system → user → project levels
/// - **Advanced Array Merging**: Compose configurations with `_add`/`_remove` patterns
/// - **Intelligent Format Detection**: Content-based parsing with caching and fallback strategies
/// - **Verbosity System**: CLI-style debugging with `-v`, `-vv`, `-vvv` levels for troubleshooting
/// - **Performance Optimized**: Lazy loading, caching, and optimized data structures
///
/// ## Quick Start Example
///
/// ```rust,no_run
/// use superconfig::{SuperConfig, VerbosityLevel};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize, Serialize, Default)]
/// struct AppConfig {
///     database_url: String,
///     log_level: String,
///     port: u16,
/// }
///
/// // Basic configuration loading
/// let config: AppConfig = SuperConfig::new()
///     .with_defaults(AppConfig::default())
///     .with_file("config.toml")
///     .with_env("APP_")
///     .extract()?;
///
/// # Ok::<(), figment::Error>(())
/// ```
///
/// ## Verbosity for Debugging
///
/// Enable verbosity to debug configuration loading issues:
///
/// ```rust,no_run
/// use superconfig::{SuperConfig, VerbosityLevel};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize, Serialize, Default)]
/// struct Config {
///     host: String,
///     port: u16,
/// }
///
/// // Enable debug verbosity (equivalent to -vv in CLI)
/// let config: Config = SuperConfig::new()
///     .with_verbosity(VerbosityLevel::Debug)  // Shows detailed loading steps
///     .with_hierarchical_config("myapp")
///     .with_env("APP_")
///     .extract()?;
///
/// # Ok::<(), figment::Error>(())
/// ```
///
/// ## CLI Integration Example
///
/// Integrate verbosity with CLI argument parsing:
///
/// ```rust,no_run
/// use superconfig::{SuperConfig, VerbosityLevel};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize, Serialize, Default)]
/// struct Config {
///     host: String,
///     port: u16,
/// }
///
/// // Parse CLI verbosity (normally from clap, structopt, etc.)
/// fn parse_verbosity() -> u8 {
///     std::env::args()
///         .skip(1)
///         .filter(|arg| arg.starts_with("-v"))
///         .map(|arg| arg.matches('v').count() as u8)
///         .max()
///         .unwrap_or(0)
/// }
///
/// let verbosity = VerbosityLevel::from_cli_args(parse_verbosity());
///
/// let config: Config = SuperConfig::new()
///     .with_verbosity(verbosity)  // Respects -v, -vv, -vvv from command line
///     .with_defaults(Config::default())
///     .with_hierarchical_config("myapp")
///     .with_env("APP_")
///     .extract()?;
///
/// # Ok::<(), figment::Error>(())
/// ```
///
/// ## Advanced Features
///
/// ### Array Merging
/// ```rust,no_run
/// // config/base.toml
/// // features = ["auth", "logging"]
///
/// // config/production.toml
/// // features_add = ["metrics", "tracing"]
/// // features_remove = ["logging"]
///
/// // Result: features = ["auth", "metrics", "tracing"]
/// # use superconfig::SuperConfig;
/// let config = SuperConfig::new()
///     .with_file("config/base.toml")
///     .with_file("config/production.toml");
/// ```
///
/// ### Debug Message Collection
/// ```rust,no_run
/// use superconfig::{SuperConfig, VerbosityLevel};
///
/// let config = SuperConfig::new()
///     .with_verbosity(VerbosityLevel::Debug)
///     .with_file("config.toml");
///
/// // Access debug messages programmatically
/// let debug_messages = config.debug_messages();
/// for msg in debug_messages {
///     println!("{}: {}", msg.provider, msg.message);
/// }
/// ```
///
/// ## Universal Platform Vision
///
/// SuperConfig is designed to become the universal configuration standard across popular
/// languages through WebAssembly bindings, REST APIs, and protocol standardization.
#[derive(Debug)]
pub struct SuperConfig {
    figment: Figment,
    warnings: Vec<String>,
    verbosity: VerbosityLevel,
    // Use internal mutability for debug state to avoid requiring &mut self
    debug_state: RefCell<DebugState>,
}

#[derive(Debug, Clone)]
struct DebugState {
    debug_messages: Vec<DebugMessage>,
    step_counter: usize,
}

impl SuperConfig {
    /// Create a new SuperConfig instance
    pub fn new() -> Self {
        Self {
            figment: Figment::new(),
            warnings: Vec::new(),
            verbosity: VerbosityLevel::default(),
            debug_state: RefCell::new(DebugState {
                debug_messages: Vec::new(),
                step_counter: 0,
            }),
        }
    }

    /// Create SuperConfig from an existing Figment
    pub fn from_figment(figment: Figment) -> Self {
        Self {
            figment,
            warnings: Vec::new(),
            verbosity: VerbosityLevel::default(),
            debug_state: RefCell::new(DebugState {
                debug_messages: Vec::new(),
                step_counter: 0,
            }),
        }
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
        self.debug(
            VerbosityLevel::Info,
            "extract",
            "Extracting final configuration",
        );

        let result = self.figment.extract::<T>();

        match &result {
            Ok(_) => {
                self.debug_result(
                    VerbosityLevel::Info,
                    "extract",
                    "Configuration extraction successful",
                    true,
                );

                // Show final merged config at trace level
                if self.verbosity >= VerbosityLevel::Trace {
                    if let Ok(json_value) = self.figment.extract::<serde_json::Value>() {
                        if let Ok(pretty_json) = serde_json::to_string_pretty(&json_value) {
                            self.debug(
                                VerbosityLevel::Trace,
                                "extract",
                                &format!("Final merged configuration:\n{pretty_json}"),
                            );
                        }
                    }
                }
            }
            Err(e) => {
                self.debug_result(
                    VerbosityLevel::Info,
                    "extract",
                    &format!("Configuration extraction failed: {e}"),
                    false,
                );
            }
        }

        result
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

impl DebugCollector for SuperConfig {
    fn debug(&self, level: VerbosityLevel, provider: &str, message: &str) {
        let debug_msg = DebugMessage::new(level, provider, message);

        // Print immediately if verbosity level allows
        if self.verbosity.should_display(level) {
            eprintln!("{}", debug_msg.format());
        }

        // Always store for later access using internal mutability
        self.debug_state.borrow_mut().debug_messages.push(debug_msg);
    }

    fn debug_step(&self, level: VerbosityLevel, provider: &str, step: usize, message: &str) {
        let debug_msg = DebugMessage::new(level, provider, message).with_step(step);

        if self.verbosity.should_display(level) {
            eprintln!("{}", debug_msg.format());
        }

        self.debug_state.borrow_mut().debug_messages.push(debug_msg);
    }

    fn debug_result(&self, level: VerbosityLevel, provider: &str, message: &str, success: bool) {
        let debug_msg = DebugMessage::new(level, provider, message).with_success(success);

        if self.verbosity.should_display(level) {
            eprintln!("{}", debug_msg.format());
        }

        self.debug_state.borrow_mut().debug_messages.push(debug_msg);
    }

    fn debug_step_result(
        &self,
        level: VerbosityLevel,
        provider: &str,
        step: usize,
        message: &str,
        success: bool,
    ) {
        let debug_msg = DebugMessage::new(level, provider, message)
            .with_step(step)
            .with_success(success);

        if self.verbosity.should_display(level) {
            eprintln!("{}", debug_msg.format());
        }

        self.debug_state.borrow_mut().debug_messages.push(debug_msg);
    }
}

impl SuperConfig {
    /// Get the current verbosity level
    pub fn verbosity(&self) -> VerbosityLevel {
        self.verbosity
    }

    /// Get all collected debug messages
    pub fn debug_messages(&self) -> Vec<DebugMessage> {
        self.debug_state.borrow().debug_messages.clone()
    }

    /// Get debug messages filtered by verbosity level
    pub fn debug_messages_at_level(&self, level: VerbosityLevel) -> Vec<DebugMessage> {
        self.debug_state
            .borrow()
            .debug_messages
            .iter()
            .filter(|msg| msg.level == level)
            .cloned()
            .collect()
    }

    /// Print all debug messages at or below the current verbosity level
    pub fn print_debug_messages(&self) {
        for msg in &self.debug_state.borrow().debug_messages {
            if self.verbosity.should_display(msg.level) {
                eprintln!("{}", msg.format());
            }
        }
    }

    /// Clear all collected debug messages
    pub fn clear_debug_messages(&self) {
        self.debug_state.borrow_mut().debug_messages.clear();
    }

    /// Get the next step number for ordered operations
    fn next_step(&self) -> usize {
        let mut state = self.debug_state.borrow_mut();
        state.step_counter += 1;
        state.step_counter
    }
}

// Fluent methods are now implemented directly in fluent.rs
