//! Fluent builder extension trait for enhanced Figment functionality
//!
//! The FluentExt trait provides builder-style methods for common configuration patterns
//! using SuperFigment's enhanced providers with automatic array merging.
//!
//! ## ⚠️ Important Note
//!
//! This trait automatically includes array merging functionality (`ExtendExt`).
//! All `with_*` methods use `merge_extend` internally, which means you get
//! `_add` and `_remove` pattern support automatically.

use super::ExtendExt;
use crate::providers::{Empty, Hierarchical, Nested, Universal};
use figment::{Figment, Provider};
use std::path::Path; // Import array merging functionality

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
/// let cli_args = Some(CliArgs { verbose: true });
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
/// let cli_args = Some(CliArgs { debug: true });
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

    /// Add optional CLI arguments with empty value filtering and array merging
    ///
    /// Uses the Empty provider internally to filter out empty values that could
    /// mask meaningful configuration from files or environment variables.
    ///
    /// # Examples
    /// ```rust
    /// use figment::Figment;
    /// use superconfig::prelude::*; // Import FluentExt trait
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct CliArgs { verbose: bool }
    ///
    /// let cli_args = Some(CliArgs { verbose: true });
    /// let config = Figment::new()
    ///     .with_cli_opt(cli_args);     // Only merged if Some(), empty values filtered
    /// ```
    fn with_cli_opt<T: serde::Serialize>(self, cli: Option<T>) -> Self;

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

    /// Add default configuration values with automatic array merging
    ///
    /// # Examples
    /// ```rust
    /// use figment::Figment;
    /// use superconfig::prelude::*;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Config {
    ///     host: String,
    ///     port: u16,
    /// }
    ///
    /// let defaults = Config {
    ///     host: "localhost".to_string(),
    ///     port: 8080,
    /// };
    ///
    /// let config = Figment::new()
    ///     .with_defaults(defaults);
    /// ```
    fn with_defaults<T: serde::Serialize>(self, defaults: T) -> Self;

    /// Add optional file-based configuration with automatic format detection and array merging
    ///
    /// Only adds the file if the Option is Some. Useful for conditional configuration loading.
    ///
    /// # Examples
    /// ```rust
    /// use figment::Figment;
    /// use superconfig::prelude::*;
    ///
    /// let custom_config: Option<&str> = Some("custom.toml");
    /// let config = Figment::new()
    ///     .with_file_opt(custom_config);  // Only adds if Some
    /// ```
    fn with_file_opt<P: AsRef<Path>>(self, path: Option<P>) -> Self;

    /// Add environment variable configuration with empty value filtering and array merging
    ///
    /// Similar to `with_env` but filters out empty values (empty strings, arrays, objects)
    /// to prevent meaningless overrides from masking meaningful configuration values.
    ///
    /// Uses both the Nested provider for advanced environment variable parsing and the
    /// Empty provider for filtering, combined with ExtendExt for array merging support.
    ///
    /// **Filtered Values:**
    /// - Empty strings: `""`
    /// - Empty arrays: `[]`
    /// - Empty objects: `{}`
    ///
    /// **Preserved Values:**
    /// - Meaningful falsy values: `false`, `0`
    /// - Non-empty strings, arrays, objects
    /// - JSON arrays with array merging: `MYAPP_FEATURES_ADD=["new_item"]`
    ///
    /// # Examples
    /// ```rust
    /// use figment::Figment;
    /// use superconfig::prelude::*;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Debug, Deserialize, Serialize)]
    /// struct Config {
    ///     debug: bool,
    ///     host: String,
    ///     features: Vec<String>,
    /// }
    ///
    /// // Environment variables:
    /// // APP_DEBUG=""              <- filtered out (empty string)
    /// // APP_HOST="localhost"       <- preserved (non-empty)  
    /// // APP_FEATURES="[]"          <- filtered out (empty array)
    /// // APP_FEATURES_ADD=["auth"]  <- merged with existing features
    ///
    /// let config = Figment::new()
    ///     .with_env_ignore_empty("APP_");  // Empty values filtered, meaningful ones applied
    /// ```
    ///
    /// # When to Use
    /// - Use `with_env_ignore_empty()` when you want clean config overrides without empty noise
    /// - Use `with_env()` when you need maximum flexibility and explicit empty values matter
    fn with_env_ignore_empty<S: AsRef<str>>(self, prefix: S) -> Self;

    /// Add CLI arguments with empty value filtering and array merging
    ///
    /// Uses the Empty provider internally to filter out empty values.
    ///
    /// # Examples
    /// ```rust
    /// use figment::Figment;
    /// use superconfig::prelude::*;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct CliArgs { verbose: bool }
    ///
    /// let config = Figment::new()
    ///     .with_cli(CliArgs { verbose: true });
    /// ```
    fn with_cli<T: serde::Serialize>(self, cli: T) -> Self;

}

impl FluentExt for Figment {
    fn with_file<P: AsRef<Path>>(self, path: P) -> Self {
        self.merge_extend(Universal::file(path)) // Uses ExtendExt::merge_extend
    }

    fn with_env<S: AsRef<str>>(self, prefix: S) -> Self {
        self.merge_extend(Nested::prefixed(prefix)) // Uses ExtendExt::merge_extend
    }

    fn with_cli_opt<T: serde::Serialize>(self, cli: Option<T>) -> Self {
        match cli {
            Some(c) => self.with_cli(c),
            None => self,
        }
    }

    fn with_hierarchical_config<S: AsRef<str>>(self, base_name: S) -> Self {
        self.merge_extend(Hierarchical::new(base_name)) // Uses ExtendExt::merge_extend
    }

    fn with_provider<P: Provider>(self, provider: P) -> Self {
        self.merge_extend(provider) // Uses ExtendExt::merge_extend
    }

    fn with_defaults<T: serde::Serialize>(self, defaults: T) -> Self {
        self.merge_extend(figment::providers::Serialized::defaults(defaults))
    }

    fn with_file_opt<P: AsRef<Path>>(self, path: Option<P>) -> Self {
        match path {
            Some(p) => self.with_file(p),
            None => self,
        }
    }

    fn with_env_ignore_empty<S: AsRef<str>>(self, prefix: S) -> Self {
        self.merge_extend(Empty::new(Nested::prefixed(prefix)))
    }

    fn with_cli<T: serde::Serialize>(self, cli: T) -> Self {
        let provider = figment::providers::Serialized::defaults(cli);
        self.merge_extend(Empty::new(provider))
    }
}
