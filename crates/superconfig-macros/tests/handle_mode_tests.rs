//! Tests for handle_mode functionality in generate_json_helper macro

use serde::{Deserialize, Serialize};
use superconfig_macros::generate_json_helper;

// Test error type
#[derive(Debug, Clone)]
pub struct HandleError {
    pub message: String,
}

impl std::fmt::Display for HandleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HandleError: {}", self.message)
    }
}

impl std::error::Error for HandleError {}

// Complex type that would normally need serialization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComplexConfig {
    pub name: String,
    pub value: i32,
}

// Service that uses handle-based architecture
#[derive(Debug, Clone)]
pub struct HandleService {
    pub id: u64,
    pub state: String,
}

impl HandleService {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            state: "active".to_string(),
        }
    }

    // Test 1: Handle mode with simple params + Self return (typical builder pattern)
    #[generate_json_helper(outgoing, handle_mode)]
    pub fn enable_feature(self, feature_id: u32) -> Result<Self, HandleError> {
        if feature_id == 999 {
            Err(HandleError {
                message: "Invalid feature ID".to_string(),
            })
        } else {
            Ok(Self {
                id: self.id,
                state: format!("enabled_{}", feature_id),
            })
        }
    }

    // Test 2: Handle mode with complex params + Self return
    #[generate_json_helper(outgoing, handle_mode)]
    pub fn configure_with_complex(self, config: ComplexConfig) -> Result<Self, HandleError> {
        if config.name.is_empty() {
            Err(HandleError {
                message: "Config name cannot be empty".to_string(),
            })
        } else {
            Ok(Self {
                id: self.id,
                state: format!("configured_{}", config.name),
            })
        }
    }

    // Test 3: Handle mode with non-Result return
    #[generate_json_helper(outgoing, handle_mode)]
    pub fn reset_state(self) -> Self {
        Self {
            id: self.id,
            state: "reset".to_string(),
        }
    }

    // Test 4: Handle mode with complex return type
    #[generate_json_helper(outgoing, handle_mode)]
    pub fn get_config_result(&self) -> Result<ComplexConfig, HandleError> {
        Ok(ComplexConfig {
            name: "test".to_string(),
            value: 42,
        })
    }

    // Test 5: Normal mode for comparison (should serialize actual result)
    #[generate_json_helper(outgoing)]
    pub fn get_status(&self) -> Result<ComplexConfig, HandleError> {
        Ok(ComplexConfig {
            name: self.state.clone(),
            value: self.id as i32,
        })
    }

    // Test 6: Handle mode with auto detection
    #[generate_json_helper(auto, handle_mode)]
    pub fn auto_enable(self, flag: bool) -> Result<Self, HandleError> {
        if flag {
            Ok(self)
        } else {
            Err(HandleError {
                message: "Flag must be true".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_mode_success_simple_params() {
        let service = HandleService::new(123);

        let result = service.enable_feature_as_json(42);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Should return success without serializing the Self object
        assert_eq!(parsed["success"], true);
        assert!(parsed.get("data").is_none()); // No data field in handle mode
        assert!(parsed.get("error").is_none()); // No error field on success
    }

    #[test]
    fn test_handle_mode_error_simple_params() {
        let service = HandleService::new(123);

        let result = service.enable_feature_as_json(999); // Triggers error
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Should return error without serializing anything
        assert_eq!(parsed["success"], false);
        assert!(parsed.get("data").is_none()); // No data field in handle mode
        assert!(
            parsed["error"]
                .as_str()
                .unwrap()
                .contains("Invalid feature ID")
        );
    }

    #[test]
    fn test_handle_mode_success_complex_params() {
        let service = HandleService::new(456);
        let config = ComplexConfig {
            name: "test_config".to_string(),
            value: 100,
        };

        let result = service.configure_with_complex_as_json(config);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Should return success without serializing the Self object
        assert_eq!(parsed["success"], true);
        assert!(parsed.get("data").is_none()); // No data field in handle mode
    }

    #[test]
    fn test_handle_mode_error_complex_params() {
        let service = HandleService::new(456);
        let config = ComplexConfig {
            name: "".to_string(), // Empty name triggers error
            value: 100,
        };

        let result = service.configure_with_complex_as_json(config);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Should return error
        assert_eq!(parsed["success"], false);
        assert!(
            parsed["error"]
                .as_str()
                .unwrap()
                .contains("Config name cannot be empty")
        );
    }

    #[test]
    fn test_handle_mode_non_result_return() {
        let service = HandleService::new(789);

        let result = service.reset_state_as_json();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Should return success for non-Result returns
        assert_eq!(parsed["success"], true);
        assert!(parsed.get("data").is_none()); // No data field in handle mode
        assert!(parsed.get("error").is_none()); // No error field
    }

    #[test]
    fn test_handle_mode_complex_return() {
        let service = HandleService::new(999);

        let result = service.get_config_result_as_json();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Handle mode should return success without serializing the result
        assert_eq!(parsed["success"], true);
        assert!(parsed.get("data").is_none()); // No data field in handle mode
    }

    #[test]
    fn test_normal_mode_comparison() {
        let service = HandleService::new(111);

        let result = service.get_status_as_json();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Normal mode should serialize the actual result
        assert_eq!(parsed["success"], true);
        assert!(parsed.get("data").is_some()); // Should have data field
        assert_eq!(parsed["data"]["name"], "active");
        assert_eq!(parsed["data"]["value"], 111);
    }

    #[test]
    fn test_handle_mode_with_auto_detection() {
        let service = HandleService::new(222);

        // Test success case
        let result = service.auto_enable_as_json(true);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], true);
        assert!(parsed.get("data").is_none()); // Handle mode ignores data

        // Test error case
        let service2 = HandleService::new(333);
        let result2 = service2.auto_enable_as_json(false);
        let parsed2: serde_json::Value = serde_json::from_str(&result2).unwrap();

        assert_eq!(parsed2["success"], false);
        assert!(
            parsed2["error"]
                .as_str()
                .unwrap()
                .contains("Flag must be true")
        );
    }

    #[test]
    fn test_handle_mode_json_structure() {
        let service = HandleService::new(444);

        // Test that JSON structure is consistent
        let result = service.enable_feature_as_json(1);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Should have exactly the success field for success case
        assert_eq!(parsed.as_object().unwrap().len(), 1);
        assert!(parsed.get("success").is_some());

        // Test error case structure with new service instance
        let service2 = HandleService::new(445);
        let result_error = service2.enable_feature_as_json(999);
        let parsed_error: serde_json::Value = serde_json::from_str(&result_error).unwrap();

        // Should have exactly success and error fields for error case
        assert_eq!(parsed_error.as_object().unwrap().len(), 2);
        assert!(parsed_error.get("success").is_some());
        assert!(parsed_error.get("error").is_some());
    }

    #[test]
    fn test_original_methods_still_work() {
        let service = HandleService::new(555);

        // Original method should still work normally
        let result = service.enable_feature(123).unwrap();
        assert_eq!(result.state, "enabled_123");
        assert_eq!(result.id, 555);

        // Original method error handling should work with new service instance
        let service2 = HandleService::new(556);
        let error_result = service2.enable_feature(999);
        assert!(error_result.is_err());
        assert!(
            error_result
                .unwrap_err()
                .message
                .contains("Invalid feature ID")
        );
    }
}
