//! Comprehensive tests for from_json functionality with real deserialization

use serde::{Deserialize, Serialize};
use superconfig_macros::generate_json_helper;

// Complex types that implement both Serialize and Deserialize
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserConfig {
    pub username: String,
    pub email: String,
    pub preferences: Vec<String>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatabaseSettings {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub ssl_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessResult {
    pub id: u64,
    pub status: String,
    pub config: UserConfig,
}

// Test error type
#[derive(Debug, Clone)]
pub struct ConfigError {
    pub message: String,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ConfigError: {}", self.message)
    }
}

impl std::error::Error for ConfigError {}

// Service that handles complex configurations
#[derive(Debug, Clone, Serialize)]
pub struct ConfigService {
    pub processed_count: u32,
}

impl ConfigService {
    pub fn new() -> Self {
        Self { processed_count: 0 }
    }

    // Test complex parameter deserialization
    #[generate_json_helper(incoming)]
    pub fn process_user_config(self, config: UserConfig) -> Result<Self, ConfigError> {
        if config.username.is_empty() {
            return Err(ConfigError {
                message: "Username cannot be empty".to_string(),
            });
        }

        if !config.email.contains('@') {
            return Err(ConfigError {
                message: "Invalid email format".to_string(),
            });
        }

        Ok(Self {
            processed_count: self.processed_count + 1,
        })
    }

    // Test multiple complex parameters
    #[generate_json_helper(incoming)]
    pub fn setup_database(
        self,
        user_config: UserConfig,
        db_settings: DatabaseSettings,
    ) -> Result<Self, ConfigError> {
        if user_config.username.is_empty() {
            return Err(ConfigError {
                message: "Username required for database setup".to_string(),
            });
        }

        if db_settings.host.is_empty() {
            return Err(ConfigError {
                message: "Database host cannot be empty".to_string(),
            });
        }

        if db_settings.port == 0 {
            return Err(ConfigError {
                message: "Invalid database port".to_string(),
            });
        }

        Ok(self)
    }

    // Test mixed simple and complex parameters
    #[generate_json_helper(incoming)]
    pub fn process_with_id(
        self,
        id: u64,
        config: UserConfig,
        enabled: bool,
    ) -> Result<Self, ConfigError> {
        if id == 0 {
            return Err(ConfigError {
                message: "ID cannot be zero".to_string(),
            });
        }

        if !enabled {
            return Err(ConfigError {
                message: "Processing is disabled".to_string(),
            });
        }

        if config.username.len() < 3 {
            return Err(ConfigError {
                message: "Username too short".to_string(),
            });
        }

        Ok(self)
    }

    // Test bidirectional with complex types
    #[generate_json_helper(incoming, outgoing)]
    pub fn transform_config(self, input_config: UserConfig) -> Result<ProcessResult, ConfigError> {
        if input_config.username.is_empty() {
            return Err(ConfigError {
                message: "Cannot transform empty username".to_string(),
            });
        }

        Ok(ProcessResult {
            id: self.processed_count as u64 + 1,
            status: "transformed".to_string(),
            config: UserConfig {
                username: format!("processed_{}", input_config.username),
                email: input_config.email,
                preferences: input_config.preferences,
                active: true,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_json_with_valid_complex_param() {
        let service = ConfigService::new();

        let json_input = r#"{
            "config": {
                "username": "john_doe",
                "email": "john@example.com",
                "preferences": ["dark_mode", "notifications"],
                "active": true
            }
        }"#;

        let result = service.process_user_config_from_json(json_input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], true);
    }

    #[test]
    fn test_from_json_with_validation_error() {
        let service = ConfigService::new();

        // Invalid email (missing @)
        let json_input = r#"{
            "config": {
                "username": "john_doe",
                "email": "invalid-email",
                "preferences": ["dark_mode"],
                "active": true
            }
        }"#;

        let result = service.process_user_config_from_json(json_input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], false);
        assert!(
            parsed["error"]
                .as_str()
                .unwrap()
                .contains("Invalid email format")
        );
    }

    #[test]
    fn test_from_json_with_missing_parameter() {
        let service = ConfigService::new();

        // Missing the "config" field entirely
        let json_input = r#"{
            "other_field": "some_value"
        }"#;

        let result = service.process_user_config_from_json(json_input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], false);
        assert!(
            parsed["error"]
                .as_str()
                .unwrap()
                .contains("Missing required parameter")
        );
    }

    #[test]
    fn test_from_json_with_invalid_json_structure() {
        let service = ConfigService::new();

        // Wrong structure for UserConfig
        let json_input = r#"{
            "config": {
                "username": "john_doe",
                "email": "john@example.com",
                "preferences": "should_be_array_not_string",
                "active": true
            }
        }"#;

        let result = service.process_user_config_from_json(json_input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], false);
        assert!(
            parsed["error"]
                .as_str()
                .unwrap()
                .contains("Failed to deserialize parameter")
        );
    }

    #[test]
    fn test_from_json_with_multiple_complex_params() {
        let service = ConfigService::new();

        let json_input = r#"{
            "user_config": {
                "username": "admin",
                "email": "admin@example.com",
                "preferences": ["full_access"],
                "active": true
            },
            "db_settings": {
                "host": "localhost",
                "port": 5432,
                "database": "myapp",
                "ssl_enabled": true
            }
        }"#;

        let result = service.setup_database_from_json(json_input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], true);
    }

    #[test]
    fn test_from_json_with_mixed_params() {
        let service = ConfigService::new();

        let json_input = r#"{
            "config": {
                "username": "testuser",
                "email": "test@example.com",
                "preferences": ["basic"],
                "active": true
            }
        }"#;

        // Note: id and enabled are simple params, passed directly to the method
        let result = service.process_with_id_from_json(42, true, json_input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], true);
    }

    #[test]
    fn test_from_json_mixed_params_validation_error() {
        let service = ConfigService::new();

        let json_input = r#"{
            "config": {
                "username": "ab",
                "email": "test@example.com",
                "preferences": ["basic"],
                "active": true
            }
        }"#;

        // Username is too short (less than 3 characters)
        let result = service.process_with_id_from_json(42, true, json_input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], false);
        assert!(
            parsed["error"]
                .as_str()
                .unwrap()
                .contains("Username too short")
        );
    }

    #[test]
    fn test_bidirectional_from_json() {
        let service = ConfigService::new();

        let json_input = r#"{
            "input_config": {
                "username": "original_user",
                "email": "user@example.com",
                "preferences": ["theme_dark", "lang_en"],
                "active": false
            }
        }"#;

        let result = service.transform_config_json(json_input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], true);
        assert!(parsed["data"].is_object());

        let data = &parsed["data"];
        assert_eq!(data["status"], "transformed");
        assert_eq!(data["config"]["username"], "processed_original_user");
        assert_eq!(data["config"]["active"], true);
    }

    #[test]
    fn test_bidirectional_as_json_still_works() {
        let service = ConfigService::new();

        // Test that the unified json method still works
        let result = service.transform_config_json(r#"{"input_config":{"username":"direct_test","email":"test@example.com","preferences":["test"],"active":true}}"#);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], true);
        assert!(parsed["data"].is_object());
    }

    #[test]
    fn test_malformed_json_input() {
        let service = ConfigService::new();

        let malformed_json = r#"{ "config": { "username": "test", }"#; // Missing closing brace

        let result = service.process_user_config_from_json(malformed_json);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], false);
        assert!(parsed["error"].as_str().unwrap().contains("Invalid JSON"));
    }
}
