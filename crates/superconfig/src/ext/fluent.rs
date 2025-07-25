//! Fluent builder extension trait and macros for unified configuration interface
//!
//! This module contains all fluent API methods for building configurations, including:
//! - FluentExt trait for extending Figment with builder methods
//! - Unified macros that handle both T and Option<T> parameters
//! - SuperConfig-specific builder methods
//!
//! ## ⚠️ Important Note
//!
//! This trait automatically includes array merging functionality (`ExtendExt`).
//! All `with_*` methods use `merge_extend` internally, which means you get
//! `_add` and `_remove` pattern support automatically.

use super::ExtendExt;
use crate::providers::{Empty, Hierarchical, Nested, Universal};
use crate::SuperConfig;
use figment::{Figment, Provider};
use std::path::Path;

/// Unified trait to convert both T and Option<T> into Option<T> for macro usage
/// 
/// This trait provides flexible parameter handling for all configuration macros,
/// supporting both direct values and optional values for any type T.
pub trait IntoOptions<T> {
    fn into_option(self) -> Option<T>;
}

impl<T> IntoOptions<T> for T {
    fn into_option(self) -> Option<T> {
        Some(self)
    }
}

impl<T> IntoOptions<T> for Option<T> {
    fn into_option(self) -> Option<T> {
        self
    }
}


/// Extension trait that adds fluent builder methods to Figment
///
/// Provides convenient `with_*` methods that use SuperFigment's enhanced providers
/// internally, giving regular Figment users access to advanced functionality like
/// format auto-detection, environment variable nesting, and hierarchical config loading.
///
/// ## ⚠️ Important: Automatic Array Merging
///
/// **All methods in this trait automatically include array merging functionality.**
/// This means that when you use `FluentExt`, you also get `ExtendExt` capabilities
/// including `_add` and `_remove` pattern support across all configuration sources.
///
/// If you only want builder methods WITHOUT array merging, use the individual
/// providers directly with regular `merge()` instead of this trait.
///
/// ## Examples
///
/// ### Basic Builder Pattern
/// ```rust
/// use figment::Figment;
/// use superconfig::prelude::*; // Import all SuperFigment functionality
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct CliArgs { verbose: bool }
///
/// let cli_args = Some(figment::providers::Serialized::defaults(
///     CliArgs { verbose: true }
/// ));
///
/// let config = Figment::new()
///     .with_file("config")           // Auto-detects .toml/.json/.yaml
///     .with_env("APP_")              // Nested env vars with JSON parsing
///     .with_cli_opt(cli_args);       // Empty value filtering
/// ```
///
/// ### Hierarchical Configuration
/// ```rust
/// use figment::Figment;
/// use superconfig::prelude::*; // Import all SuperFigment functionality
///
/// // Automatically searches and merges config files from:
/// // ~/.config/myapp/config.*, ~/.myapp/config.*, ./config.*
/// let config = Figment::new()
///     .with_hierarchical_config("myapp");
/// ```
///
/// ### Full Configuration Chain
/// ```rust
/// use figment::Figment;
/// use superconfig::prelude::*; // Import all SuperFigment functionality
/// use figment::providers::Serialized;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Defaults {
///     port: u16,
///     host: String,
/// }
///
/// #[derive(Serialize)]
/// struct CliArgs { debug: bool }
///
/// let cli_args = Some(Serialized::defaults(CliArgs { debug: true }));
///
/// let config = Figment::new()
///     .merge_extend(Serialized::defaults(Defaults {
///         port: 8080,
///         host: "localhost".to_string(),
///     }))
///     .with_hierarchical_config("myapp")  // System/user/project configs
///     .with_file("config")                // Explicit config file
///     .with_env("MYAPP_")                // Environment overrides
///     .with_cli_opt(cli_args);           // Command line overrides
/// ```
pub trait FluentExt {
    /// Add file-based configuration with automatic format detection and array merging
    ///
    /// Uses the Universal provider internally for smart format detection with caching,
    /// extension fallback, and format-specific parsing attempts.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use figment::Figment;
    /// use superconfig::prelude::*; // Import FluentExt trait
    ///
    /// let config = Figment::new()
    ///     .with_file("config");        // Tries config.toml, config.yaml, etc.
    /// ```
    fn with_file<P: AsRef<Path>>(self, path: P) -> Self;

    /// Add environment variable configuration with automatic nesting and array merging
    ///
    /// Uses the Nested provider internally for advanced environment variable processing
    /// with JSON parsing, smart type detection, and nested structure creation.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use figment::Figment;
    /// use superconfig::prelude::*; // Import FluentExt trait
    ///
    /// // Environment: APP_DATABASE_HOST=localhost, APP_FEATURES=["auth","cache"]
    /// let config = Figment::new()
    ///     .with_env("APP_");           // Creates database.host and features array
    /// ```
    fn with_env<S: AsRef<str>>(self, prefix: S) -> Self;

    /// Add CLI arguments with empty value filtering and array merging (if provided)
    ///
    /// Uses the Empty provider internally to filter out empty values that could
    /// mask meaningful configuration from files or environment variables.
    ///
    /// # Examples
    /// ```rust
    /// use figment::Figment;
    /// use superconfig::prelude::*; // Import FluentExt trait
    /// use figment::providers::Serialized;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct CliArgs { verbose: bool }
    ///
    /// let cli_data = CliArgs { verbose: true };
    /// let cli_args = Some(Serialized::defaults(cli_data));
    /// let config = Figment::new()
    ///     .with_cli_opt(cli_args);     // Only merged if Some(), empty values filtered
    /// ```
    fn with_cli_opt<P: Provider>(self, provider: Option<P>) -> Self;

    /// Add hierarchical configuration files with automatic cascade merging
    ///
    /// Uses the Hierarchical provider internally for directory traversal and cascading
    /// configuration merging from system-wide to project-local levels.
    ///
    /// # Search Hierarchy (least to most specific)
    /// 1. `~/.config/{base_name}/{base_name}.*`
    /// 2. `~/.{base_name}/{base_name}.*`
    /// 3. `~/{base_name}.*`
    /// 4. Ancestor directories: `../../{base_name}.*`, `../{base_name}.*`
    /// 5. Current directory: `./{base_name}.*`
    ///
    /// # Examples
    /// ```rust,no_run
    /// use figment::Figment;
    /// use superconfig::prelude::*; // Import FluentExt trait
    ///
    /// let config = Figment::new()
    ///     .with_hierarchical_config("myapp");  // Searches config hierarchy
    /// ```
    fn with_hierarchical_config<S: AsRef<str>>(self, base_name: S) -> Self;

    /// Add any provider with automatic array merging
    ///
    /// Convenience method that applies array merging to any provider.
    /// Equivalent to calling `merge_extend(provider)` but fits better
    /// in the fluent builder chain.
    ///
    /// # Examples
    /// ```rust
    /// use figment::Figment;
    /// use figment::providers::{Json, Format};
    /// use superconfig::prelude::*; // Import FluentExt trait
    ///
    /// let config = Figment::new()
    ///     .with_provider(Json::file("config.json"));
    /// ```
    fn with_provider<P: Provider>(self, provider: P) -> Self;
}

impl FluentExt for Figment {
    fn with_file<P: AsRef<Path>>(self, path: P) -> Self {
        self.merge_extend(Universal::file(path)) // Uses ExtendExt::merge_extend
    }

    fn with_env<S: AsRef<str>>(self, prefix: S) -> Self {
        self.merge_extend(Nested::prefixed(prefix)) // Uses ExtendExt::merge_extend
    }

    fn with_cli_opt<P: Provider>(self, provider: Option<P>) -> Self {
        self.merge_extend_opt(provider.map(Empty::new)) // Uses ExtendExt::merge_extend_opt
    }

    fn with_hierarchical_config<S: AsRef<str>>(self, base_name: S) -> Self {
        self.merge_extend(Hierarchical::new(base_name)) // Uses ExtendExt::merge_extend
    }

    fn with_provider<P: Provider>(self, provider: P) -> Self {
        self.merge_extend(provider) // Uses ExtendExt::merge_extend
    }
}

/// Macro for file-based configuration with optional flags
/// 
/// # Parameters
/// - `$self`: SuperConfig instance
/// - `$path`: File path (String, &str, PathBuf, or Option of these)
/// - `$flags`: Optional bitwise flags to control behavior
///
/// # Examples
/// ```rust
/// use superconfig::{SuperConfig, with_file, flags};
/// 
/// let config1 = SuperConfig::new();
/// with_file!(config1, "config.toml");                    // Direct path, default flags
/// 
/// let config2 = SuperConfig::new();
/// with_file!(config2, "config.toml", flags::REQUIRED);   // Direct path with flags
/// 
/// let config3 = SuperConfig::new();
/// with_file!(config3, "config.toml", flags::REQUIRED | flags::FOLLOW_SYMLINKS); // Direct path with multiple flags
/// ```
#[macro_export]
macro_rules! with_file {
    // Without flags (default behavior)
    ($self:expr, $path:expr) => {{
        with_file!($self, $path, $crate::flags::DEFAULT)
    }};
    
    // With flags
    ($self:expr, $path:expr, $flags:expr) => {{
        use std::path::Path;
        use $crate::ext::fluent::IntoOptions;
        
        let actual_flags = $flags;
        
        use $crate::ext::ExtendExt;
        use $crate::providers::Universal;
        
        // TODO: Use flags to modify behavior (REQUIRED, FOLLOW_SYMLINKS, etc.)
        // For now, basic file loading regardless of flags
        SuperConfig {
            figment: $self.figment.merge_extend(Universal::file($path)),
        }
    }};
}

/// Macro for CLI configuration with optional flags
/// 
/// # Parameters
/// - `$self`: SuperConfig instance
/// - `$cli`: CLI arguments (any serializable type or Option of it)
/// - `$flags`: Optional bitwise flags to control behavior
///
/// # Examples
/// ```rust
/// use superconfig::{SuperConfig, with_cli, flags};
/// use serde::Serialize;
/// 
/// #[derive(Serialize)]
/// struct Args { verbose: bool }
/// 
/// let config1 = SuperConfig::new();
/// with_cli!(config1, Args { verbose: true });              // Direct value, default flags
/// 
/// let config2 = SuperConfig::new();
/// with_cli!(config2, Args { verbose: true }, flags::FILTER_EMPTY); // Direct value with flags
/// 
/// let config3 = SuperConfig::new();
/// let args: Option<Args> = Some(Args { verbose: true });
/// with_cli!(config3, args, flags::STRICT_MODE);            // Option<T> with flags
/// ```
#[macro_export]
macro_rules! with_cli {
    // Without flags (default behavior)
    ($self:expr, $cli:expr) => {{
        with_cli!($self, $cli, $crate::flags::DEFAULT)
    }};
    
    // With flags
    ($self:expr, $cli:expr, $flags:expr) => {{
        let actual_flags = $flags;
        
        use $crate::ext::ExtendExt;
        use $crate::providers::Empty;
        
        let provider = figment::providers::Serialized::defaults($cli);
        
        // Apply FILTER_EMPTY flag if specified
        if actual_flags.contains($crate::flags::FILTER_EMPTY) {
            SuperConfig {
                figment: $self.figment.merge_extend(Empty::new(provider)),
            }
        } else {
            SuperConfig {
                figment: $self.figment.merge_extend(provider),
            }
        }
    }};
}

/// Macro for environment variable configuration with optional flags
/// 
/// # Parameters
/// - `$self`: SuperConfig instance
/// - `$prefix`: Environment variable prefix (e.g., "APP_")
/// - `$flags`: Optional bitwise flags to control behavior
///
/// # Examples
/// ```rust
/// use superconfig::{SuperConfig, with_env, flags};
/// 
/// let config1 = SuperConfig::new();
/// with_env!(config1, "APP_");                          // Standard env parsing
/// 
/// let config2 = SuperConfig::new();
/// with_env!(config2, "APP_", flags::FILTER_EMPTY);     // Filter empty values
/// 
/// let config3 = SuperConfig::new();
/// with_env!(config3, "APP_", flags::FILTER_EMPTY | flags::STRICT_MODE); // Multiple flags
/// ```
#[macro_export]
macro_rules! with_env {
    // Without flags (default behavior)
    ($self:expr, $prefix:expr) => {{
        with_env!($self, $prefix, $crate::flags::DEFAULT)
    }};
    
    // With flags
    ($self:expr, $prefix:expr, $flags:expr) => {{
        let actual_flags = $flags;
        
        use $crate::ext::ExtendExt;
        use $crate::providers::{Empty, Nested};
        
        let provider = Nested::prefixed($prefix);
        
        // Apply FILTER_EMPTY flag if specified
        if actual_flags.contains($crate::flags::FILTER_EMPTY) {
            SuperConfig {
                figment: $self.figment.merge_extend(Empty::new(provider)),
            }
        } else {
            SuperConfig {
                figment: $self.figment.merge_extend(provider),
            }
        }
    }};
}

/// SuperConfig-specific fluent methods
impl SuperConfig {
    /// Add default configuration values with automatic array merging
    pub fn with_defaults<T: serde::Serialize>(self, defaults: T) -> Self {
        use crate::ext::ExtendExt;
        Self {
            figment: self
                .figment
                .merge_extend(figment::providers::Serialized::defaults(defaults)),
        }
    }

    /// Add environment variable configuration with automatic nesting and array merging
    pub fn with_env<S: AsRef<str>>(self, prefix: S) -> Self {
        use crate::ext::ExtendExt;
        Self {
            figment: self.figment.merge_extend(Nested::prefixed(prefix)),
        }
    }

    /// Add environment variable configuration with empty value filtering and array merging
    pub fn with_env_ignore_empty<S: AsRef<str>>(self, prefix: S) -> Self {
        use crate::ext::ExtendExt;
        Self {
            figment: self
                .figment
                .merge_extend(Empty::new(Nested::prefixed(prefix))),
        }
    }

    /// Add any provider with automatic array merging
    pub fn with_provider<P: figment::Provider>(self, provider: P) -> Self {
        use crate::ext::ExtendExt;
        Self {
            figment: self.figment.merge_extend(provider),
        }
    }

    /// Add hierarchical configuration files with automatic cascade merging
    pub fn with_hierarchical_config<S: AsRef<str>>(self, base_name: S) -> Self {
        use crate::ext::ExtendExt;
        Self {
            figment: self.figment.merge_extend(Hierarchical::new(base_name)),
        }
    }

    /// Extract configuration directly (equivalent to calling .extract() on the inner Figment)
    pub fn extract<'de, T: serde::Deserialize<'de>>(&self) -> Result<T, figment::Error> {
        self.figment.extract()
    }
}
