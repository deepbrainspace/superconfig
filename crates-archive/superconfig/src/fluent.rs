//! Fluent builder methods for SuperConfig
//!
//! This module provides the fluent builder methods that were previously
//! available as extension traits. Now they're native SuperConfig methods.

use crate::SuperConfig;
use crate::verbosity;
use crate::verbosity::DebugCollector;
use figment::providers::Serialized;

impl SuperConfig {
    /// Set the verbosity level for configuration debugging
    ///
    /// Controls the amount of debug information displayed during configuration loading.
    /// Higher levels include all information from lower levels.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use superconfig::{SuperConfig, verbosity};
    ///
    /// let config = SuperConfig::new()
    ///     .with_verbosity(verbosity::DEBUG)  // -vv level debugging
    ///     .with_file("config.toml");
    /// ```
    pub fn with_verbosity(mut self, level: u8) -> Self {
        self.verbosity = level;
        self.debug(
            verbosity::INFO,
            "verbosity",
            &format!("Set verbosity level to: {level}"),
        );
        self
    }

    /// Enable basic configuration loading progress (equivalent to -v)
    ///
    /// Shows which providers are being loaded and final success/failure.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use superconfig::SuperConfig;
    ///
    /// let config = SuperConfig::new()
    ///     .with_info_verbosity()  // -v level
    ///     .with_file("config.toml");
    /// ```
    pub fn with_info_verbosity(self) -> Self {
        self.with_verbosity(verbosity::INFO)
    }

    /// Enable detailed step-by-step information (equivalent to -vv)
    ///
    /// Shows file discovery, individual provider results, and merge operations.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use superconfig::SuperConfig;
    ///
    /// let config = SuperConfig::new()
    ///     .with_debug_verbosity()  // -vv level
    ///     .with_hierarchical_config("myapp");
    /// ```
    pub fn with_debug_verbosity(self) -> Self {
        self.with_verbosity(verbosity::DEBUG)
    }

    /// Enable full introspection with configuration values (equivalent to -vvv)
    ///
    /// Shows actual configuration values at each step and final merged result.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use superconfig::SuperConfig;
    ///
    /// let config = SuperConfig::new()
    ///     .with_trace_verbosity()  // -vvv level
    ///     .with_env("APP_");
    /// ```
    pub fn with_trace_verbosity(self) -> Self {
        self.with_verbosity(verbosity::TRACE)
    }

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
        let path_str = path.as_ref().to_string_lossy();
        let step = self.next_step();

        self.debug_step(
            verbosity::INFO,
            "file",
            step,
            &format!("Loading configuration file: {path_str}"),
        );

        // Check if file exists for debug output
        if path.as_ref().exists() {
            self.debug_step_result(
                verbosity::DEBUG,
                "file",
                step,
                &format!("File found: {path_str}"),
                true,
            );
        } else {
            self.debug_step_result(
                verbosity::DEBUG,
                "file",
                step,
                &format!("File not found: {path_str}"),
                false,
            );
        }

        // Add trace-level content inspection
        if self.verbosity >= verbosity::TRACE && path.as_ref().exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let preview = if content.len() > 200 {
                    format!("{}... ({} chars total)", &content[..200], content.len())
                } else {
                    content
                };
                self.debug(
                    verbosity::TRACE,
                    "file",
                    &format!("File content preview: {preview}"),
                );
            }
        }

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
        let prefix_str = prefix.as_ref();
        let step = self.next_step();

        self.debug_step(
            verbosity::INFO,
            "env",
            step,
            &format!("Loading environment variables with prefix: {prefix_str}"),
        );

        // Collect matching environment variables for debug output
        let env_vars: Vec<(String, String)> = std::env::vars()
            .filter(|(key, _)| key.starts_with(prefix_str))
            .collect();

        if env_vars.is_empty() {
            self.debug_step_result(
                verbosity::DEBUG,
                "env",
                step,
                &format!("No environment variables found with prefix: {prefix_str}"),
                false,
            );
        } else {
            self.debug_step_result(
                verbosity::DEBUG,
                "env",
                step,
                &format!(
                    "Found {} environment variables with prefix: {prefix_str}",
                    env_vars.len()
                ),
                true,
            );

            // Show individual env vars at trace level
            if self.verbosity >= verbosity::TRACE {
                for (key, value) in &env_vars {
                    // Mask sensitive values (anything with 'password', 'secret', 'token', 'key' in name)
                    let display_value = if key.to_lowercase().contains("password")
                        || key.to_lowercase().contains("secret")
                        || key.to_lowercase().contains("token")
                        || key.to_lowercase().contains("key")
                    {
                        "***MASKED***".to_string()
                    } else {
                        value.clone()
                    };
                    self.debug(verbosity::TRACE, "env", &format!("  {key}={display_value}"));
                }
            }
        }

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
        let base_name_str = base_name.as_ref();
        let step = self.next_step();

        self.debug_step(
            verbosity::INFO,
            "hierarchical",
            step,
            &format!("Loading hierarchical config for: {base_name_str}"),
        );

        // Show the discovery paths at debug level
        if self.verbosity >= verbosity::DEBUG {
            let potential_paths = vec![
                format!("/etc/{base_name_str}/config.toml"),
                format!("/etc/{base_name_str}.toml"),
                dirs::config_dir()
                    .map(|p| {
                        p.join(base_name_str)
                            .join("config.toml")
                            .to_string_lossy()
                            .to_string()
                    })
                    .unwrap_or_else(|| "~/.config/<app>/config.toml".to_string()),
                format!("./{base_name_str}.toml"),
                format!("./config/{base_name_str}.toml"),
            ];

            self.debug(
                verbosity::DEBUG,
                "hierarchical",
                "Checking hierarchical config paths:",
            );
            for path in &potential_paths {
                let exists = std::path::Path::new(path).exists();
                self.debug_result(
                    verbosity::DEBUG,
                    "hierarchical",
                    &format!("  - {path}"),
                    exists,
                );
            }
        }

        self.merge(crate::providers::Wildcard::hierarchical(
            "config",
            base_name_str,
        ))
    }

    /// Add default configuration values from a serializable struct
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

    /// Add default configuration from a raw string (TOML, JSON, or YAML)
    ///
    /// Uses the Universal provider to parse the string content with automatic
    /// format detection. Perfect for embedded configuration strings.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use superconfig::SuperConfig;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Deserialize, Serialize)]
    /// struct CliArgs { host: Option<String> }
    ///
    /// // Embedded default configuration (common pattern)
    /// const DEFAULT_CONFIG: &str = r#"
    /// host = "localhost"
    /// port = 8080
    ///
    /// [database]
    /// url = "postgres://localhost"
    /// timeout = 30
    /// "#;
    ///
    /// let cli_args = CliArgs { host: None };
    /// let config = SuperConfig::new()
    ///     .with_defaults_string(DEFAULT_CONFIG)  // Load TOML string as defaults
    ///     .with_hierarchical_config("myapp")     // Apply hierarchical configs
    ///     .with_env("APP_")                      // Apply env variables
    ///     .with_cli_opt(Some(cli_args));         // Apply CLI overrides
    /// ```
    pub fn with_defaults_string(self, content: &str) -> Self {
        let step = self.next_step();

        self.debug_step(
            verbosity::INFO,
            "defaults",
            step,
            "Loading embedded default configuration",
        );

        if self.verbosity >= verbosity::DEBUG {
            let lines = content.lines().count();
            let chars = content.len();
            self.debug(
                verbosity::DEBUG,
                "defaults",
                &format!("Embedded config: {lines} lines, {chars} characters"),
            );
        }

        if self.verbosity >= verbosity::TRACE {
            let preview = if content.len() > 300 {
                format!("{}... ({} chars total)", &content[..300], content.len())
            } else {
                content.to_string()
            };
            self.debug(
                verbosity::TRACE,
                "defaults",
                &format!("Default config content:\n{preview}"),
            );
        }

        self.merge(crate::providers::Universal::string(content))
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
        let step = self.next_step();

        if let Some(cli_values) = cli_opt {
            self.debug_step(
                verbosity::INFO,
                "cli",
                step,
                "Loading CLI argument overrides",
            );

            // Try to serialize to JSON for debug inspection
            if self.verbosity >= verbosity::TRACE {
                if let Ok(json_value) = serde_json::to_value(&cli_values) {
                    if let Ok(pretty_json) = serde_json::to_string_pretty(&json_value) {
                        self.debug(
                            verbosity::TRACE,
                            "cli",
                            &format!("CLI overrides:\n{pretty_json}"),
                        );
                    }
                }
            }

            self.merge(crate::providers::Empty::new(Serialized::defaults(
                cli_values,
            )))
        } else {
            self.debug_step_result(
                verbosity::DEBUG,
                "cli",
                step,
                "No CLI arguments provided",
                false,
            );
            self
        }
    }

    /// Add an optional configuration file with smart format detection
    ///
    /// Uses the Universal provider for automatic format detection and caching.
    /// Only adds the file if the path is Some, otherwise returns self unchanged.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use superconfig::SuperConfig;
    /// use std::path::PathBuf;
    ///
    /// let optional_config: Option<PathBuf> = Some("config.toml".into());
    /// let config = SuperConfig::new()
    ///     .with_file_opt(optional_config)    // Only loads if Some
    ///     .with_env("APP_");
    /// ```
    pub fn with_file_opt<P: AsRef<std::path::Path>>(self, path: Option<P>) -> Self {
        if let Some(file_path) = path {
            self.with_file(file_path)
        } else {
            self
        }
    }

    /// Add environment variables with a prefix and empty value filtering
    ///
    /// Combines the Nested provider for JSON parsing and automatic nesting
    /// with the Empty provider to filter out empty values.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use superconfig::SuperConfig;
    ///
    /// let config = SuperConfig::new()
    ///     .with_env_ignore_empty("APP_");  // Filters empty env vars
    /// ```
    pub fn with_env_ignore_empty<S: AsRef<str>>(self, prefix: S) -> Self {
        let prefix_str = prefix.as_ref();
        let step = self.next_step();

        self.debug_step(
            verbosity::INFO,
            "env",
            step,
            &format!("Loading environment variables with prefix: {prefix_str} (ignore empty)"),
        );

        // Collect matching environment variables for debug output
        let env_vars: Vec<(String, String)> = std::env::vars()
            .filter(|(key, _)| key.starts_with(prefix_str))
            .collect();

        if env_vars.is_empty() {
            self.debug_step_result(
                verbosity::DEBUG,
                "env",
                step,
                &format!("No environment variables found with prefix: {prefix_str}"),
                false,
            );
        } else {
            self.debug_step_result(
                verbosity::DEBUG,
                "env",
                step,
                &format!(
                    "Found {} environment variables with prefix: {prefix_str}",
                    env_vars.len()
                ),
                true,
            );

            // Show individual env vars at trace level
            if self.verbosity >= verbosity::TRACE {
                for (key, value) in &env_vars {
                    // Mask sensitive values (anything with 'password', 'secret', 'token', 'key' in name)
                    let display_value = if key.to_lowercase().contains("password")
                        || key.to_lowercase().contains("secret")
                        || key.to_lowercase().contains("token")
                        || key.to_lowercase().contains("key")
                    {
                        "***MASKED***".to_string()
                    } else {
                        value.clone()
                    };
                    self.debug(verbosity::TRACE, "env", &format!("  {key}={display_value}"));
                }
            }
        }

        self.merge(crate::providers::Empty::new(
            crate::providers::Nested::prefixed(prefix),
        ))
    }
}
