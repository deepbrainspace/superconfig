//! Access and export methods for SuperConfig
//!
//! This module provides convenient methods for accessing configuration values,
//! exporting to different formats, and debugging configuration state.

use figment::Error;
use figment::error::Actual;

impl crate::SuperConfig {
    /// Export configuration as pretty-formatted JSON string
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::SuperConfig;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Config { name: String }
    ///
    /// let config = SuperConfig::new()
    ///     .with_defaults(Config { name: "test".to_string() });
    ///     
    /// let json_str = config.as_json()?;
    /// println!("{}", json_str); // Pretty-printed JSON
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn as_json(&self) -> Result<String, Error> {
        let value = self.figment.extract::<serde_json::Value>()?;
        serde_json::to_string_pretty(&value).map_err(|e| {
            Error::from(figment::error::Kind::InvalidType(
                Actual::Other(e.to_string()),
                "valid JSON".into(),
            ))
        })
    }

    /// Export configuration as YAML string
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::SuperConfig;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Config { name: String }
    ///
    /// let config = SuperConfig::new()
    ///     .with_defaults(Config { name: "test".to_string() });
    ///     
    /// let yaml_str = config.as_yaml()?;
    /// println!("{}", yaml_str);
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn as_yaml(&self) -> Result<String, Error> {
        let value = self.figment.extract::<serde_json::Value>()?;
        serde_yml::to_string(&value).map_err(|e| {
            Error::from(figment::error::Kind::InvalidType(
                Actual::Other(e.to_string()),
                "valid YAML".into(),
            ))
        })
    }

    /// Export configuration as TOML string
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::SuperConfig;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Config { name: String }
    ///
    /// let config = SuperConfig::new()
    ///     .with_defaults(Config { name: "test".to_string() });
    ///     
    /// let toml_str = config.as_toml()?;
    /// println!("{}", toml_str);
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn as_toml(&self) -> Result<String, Error> {
        let value = self.figment.extract::<toml::Value>()?;
        toml::to_string_pretty(&value).map_err(|e| {
            Error::from(figment::error::Kind::InvalidType(
                Actual::Other(e.to_string()),
                "valid TOML".into(),
            ))
        })
    }

    /// Get a string value from configuration
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::SuperConfig;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Config { database: Database }
    ///
    /// #[derive(Serialize)]
    /// struct Database { host: String }
    ///
    /// let config = SuperConfig::new()
    ///     .with_defaults(Config {
    ///         database: Database { host: "localhost".to_string() }
    ///     });
    ///     
    /// let host = config.get_string("database.host")?;
    /// println!("Database host: {}", host);
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn get_string<K: AsRef<str>>(&self, key: K) -> Result<String, Error> {
        self.figment.extract_inner(key.as_ref())
    }

    /// Get an array value from configuration
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::SuperConfig;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Config { cors: Cors }
    ///
    /// #[derive(Serialize)]
    /// struct Cors { allowed_origins: Vec<String> }
    ///
    /// let config = SuperConfig::new()
    ///     .with_defaults(Config {
    ///         cors: Cors { allowed_origins: vec!["https://example.com".to_string()] }
    ///     });
    ///     
    /// let origins = config.get_array::<String>("cors.allowed_origins")?;
    /// println!("CORS origins: {:?}", origins);
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn get_array<T>(&self, key: &str) -> Result<Vec<T>, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        self.figment.extract_inner(key)
    }

    /// Check if a configuration key exists
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::SuperConfig;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Config { redis: Redis }
    ///
    /// #[derive(Serialize)]
    /// struct Redis { enabled: bool }
    ///
    /// let config = SuperConfig::new()
    ///     .with_defaults(Config {
    ///         redis: Redis { enabled: true }
    ///     });
    ///     
    /// if config.has_key("redis.enabled")? {
    ///     println!("Redis is configured");
    /// }
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn has_key(&self, key: &str) -> Result<bool, Error> {
        match self.figment.find_value(key) {
            Ok(_) => Ok(true),
            Err(Error {
                kind: figment::error::Kind::MissingField(_),
                ..
            }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Get all top-level configuration keys
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::SuperConfig;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Config { database: String, redis: String }
    ///
    /// let config = SuperConfig::new()
    ///     .with_defaults(Config {
    ///         database: "postgres://localhost".to_string(),
    ///         redis: "redis://localhost".to_string()
    ///     });
    ///     
    /// let keys = config.keys()?;
    /// println!("Config sections: {:?}", keys);
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn keys(&self) -> Result<Vec<String>, Error> {
        let value = self.figment.extract::<serde_json::Value>()?;
        match value {
            serde_json::Value::Object(map) => Ok(map.keys().cloned().collect()),
            _ => Ok(vec![]),
        }
    }

    /// Debug configuration with pretty-printed values and source information
    ///
    /// Returns a formatted string showing the final configuration values
    /// along with metadata about which providers contributed each value.
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::SuperConfig;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Config { name: String }
    ///
    /// let config = SuperConfig::new()
    ///     .with_defaults(Config { name: "test".to_string() });
    ///     
    /// println!("{}", config.debug_config()?);
    /// // Output shows merged config with provider information
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn debug_config(&self) -> Result<String, Error> {
        let json_value = self.figment.extract::<serde_json::Value>()?;
        let pretty_json = serde_json::to_string_pretty(&json_value).map_err(|e| {
            Error::from(figment::error::Kind::InvalidType(
                Actual::Other(e.to_string()),
                "valid JSON".into(),
            ))
        })?;

        Ok(format!(
            "=== SuperConfig Debug ===\n\nWarnings: {:?}\n\nFinal Configuration:\n{pretty_json}\n\nProvider Chain:\n{:#?}",
            self.warnings, self.figment
        ))
    }

    /// Get debug information about configuration sources
    ///
    /// Returns metadata about the providers that contributed to the configuration.
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::SuperConfig;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Config { name: String }
    ///
    /// let config = SuperConfig::new()
    ///     .with_defaults(Config { name: "test".to_string() });
    ///     
    /// let sources = config.debug_sources();
    /// println!("Configuration sources: {:#?}", sources);
    /// ```
    pub fn debug_sources(&self) -> Vec<figment::Metadata> {
        self.figment.metadata().cloned().collect()
    }
}
