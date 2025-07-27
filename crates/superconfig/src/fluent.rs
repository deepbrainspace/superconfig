//! Fluent builder methods for SuperConfig
//!
//! This module provides the fluent builder methods that were previously
//! available as extension traits. Now they're native SuperConfig methods.

use crate::SuperConfig;
use figment::providers::Serialized;

impl SuperConfig {
    /// Add a configuration file with smart format detection
    ///
    /// Uses the Universal provider for automatic format detection and caching.
    /// Supports .toml, .yaml/.yml, .json files with fallback chains.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use superconfig::SuperConfig;
    ///
    /// let config = SuperConfig::new()
    ///     .with_file("config")        // Auto-detects config.toml, config.yaml, etc.
    ///     .with_file("app.json");     // Explicit JSON file
    /// ```
    pub fn with_file<P: AsRef<std::path::Path>>(self, path: P) -> Self {
        self.merge(crate::providers::Universal::file(path))
    }

    /// Add environment variables with a prefix
    ///
    /// Uses the Nested provider for JSON parsing and automatic nesting.
    /// Supports complex structures like arrays and objects in environment variables.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use superconfig::SuperConfig;
    ///
    /// let config = SuperConfig::new()
    ///     .with_env("APP_")           // APP_DATABASE_HOST, APP_FEATURES, etc.
    ///     .with_env("MYAPP_");        // Multiple prefixes supported
    /// ```
    pub fn with_env<S: AsRef<str>>(self, prefix: S) -> Self {
        self.merge(crate::providers::Nested::prefixed(prefix))
    }

    /// Add hierarchical configuration cascade
    ///
    /// Uses the Wildcard provider for Git-like config inheritance:
    /// system → user → project levels with automatic discovery.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use superconfig::SuperConfig;
    ///
    /// let config = SuperConfig::new()
    ///     .with_hierarchical_config("myapp");  // Loads system, user, project configs
    /// ```
    pub fn with_hierarchical_config<S: AsRef<str>>(self, base_name: S) -> Self {
        self.merge(crate::providers::Wildcard::hierarchical("config", base_name.as_ref()))
    }

    /// Add default configuration values
    ///
    /// Uses Figment's Serialized provider to set defaults that can be overridden
    /// by other configuration sources.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use superconfig::SuperConfig;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Deserialize, Serialize, Default)]
    /// struct Config {
    ///     host: String,
    ///     port: u16,
    /// }
    ///
    /// let config = SuperConfig::new()
    ///     .with_defaults(Config::default())
    ///     .with_file("config.toml");
    /// ```
    pub fn with_defaults<T: serde::Serialize>(self, defaults: T) -> Self {
        self.merge(Serialized::defaults(defaults))
    }

    /// Add CLI option values with empty value filtering
    ///
    /// Uses the Empty provider to filter out empty values while preserving
    /// meaningful falsy values (false, 0, etc.).
    ///
    /// # Examples
    /// ```rust,no_run
    /// use superconfig::SuperConfig;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Deserialize, Serialize)]
    /// struct CliArgs {
    ///     host: Option<String>,
    ///     port: Option<u16>,
    /// }
    ///
    /// let cli_args = CliArgs { host: Some("localhost".to_string()), port: None };
    /// let config = SuperConfig::new()
    ///     .with_file("config.toml")
    ///     .with_cli_opt(Some(cli_args));
    /// ```
    pub fn with_cli_opt<T: serde::Serialize>(self, cli_opt: Option<T>) -> Self {
        if let Some(cli_values) = cli_opt {
            self.merge(crate::providers::Empty::new(
                Serialized::defaults(cli_values)
            ))
        } else {
            self
        }
    }
}