//! Tests for the `#[generate_json_helper]` procedural macro

use serde::Serialize;
use serde_json::Value;
use superconfig_macros::generate_json_helper;

// Mock error type for testing
#[derive(Debug, Clone)]
pub struct TestError {
    pub message: String,
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TestError: {}", self.message)
    }
}

impl std::error::Error for TestError {}

// Mock struct that will have the macro applied
#[derive(Debug, Clone, Serialize)]
pub struct TestService {
    pub value: i32,
}

impl TestService {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    // Test method that will have the macro applied
    #[generate_json_helper]
    pub fn update_value(self, new_value: i32) -> Result<Self, TestError> {
        if new_value < 0 {
            Err(TestError {
                message: "Value cannot be negative".to_string(),
            })
        } else {
            Ok(Self { value: new_value })
        }
    }

    // Another test method with different parameters
    #[generate_json_helper]
    pub fn add_values(self, a: i32, b: i32) -> Result<Self, TestError> {
        let sum = a + b;
        if sum > 1000 {
            Err(TestError {
                message: "Sum exceeds maximum allowed value".to_string(),
            })
        } else {
            Ok(Self { value: sum })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_original_method_still_works() {
        let service = TestService::new();
        let result = service.update_value(42).unwrap();
        assert_eq!(result.value, 42);
    }

    #[test]
    fn test_json_helper_success() {
        let service = TestService::new();
        let json_result = service.update_value_as_json(42);

        let parsed: Value = serde_json::from_str(&json_result).unwrap();
        assert_eq!(parsed["success"], true);
    }

    #[test]
    fn test_json_helper_error() {
        let service = TestService::new();
        let json_result = service.update_value_as_json(-5);

        let parsed: Value = serde_json::from_str(&json_result).unwrap();
        assert_eq!(parsed["success"], false);
        assert!(
            parsed["error"]
                .as_str()
                .unwrap()
                .contains("Value cannot be negative")
        );
    }

    #[test]
    fn test_json_helper_with_multiple_params() {
        let service = TestService::new();

        // Test success case
        let json_success = service.add_values_as_json(10, 20);
        let parsed_success: Value = serde_json::from_str(&json_success).unwrap();
        assert_eq!(parsed_success["success"], true);

        // Test error case
        let service2 = TestService::new();
        let json_error = service2.add_values_as_json(600, 500);
        let parsed_error: Value = serde_json::from_str(&json_error).unwrap();
        assert_eq!(parsed_error["success"], false);
        assert!(
            parsed_error["error"]
                .as_str()
                .unwrap()
                .contains("Sum exceeds maximum")
        );
    }

    #[test]
    fn test_json_format_structure() {
        // Test success format
        let service1 = TestService::new();
        let success_json = service1.update_value_as_json(25);
        let success_parsed: Value = serde_json::from_str(&success_json).unwrap();
        assert!(success_parsed.is_object());
        assert!(success_parsed.get("success").is_some());
        assert_eq!(success_parsed.get("error"), None);

        // Test error format
        let service2 = TestService::new();
        let error_json = service2.update_value_as_json(-10);
        let error_parsed: Value = serde_json::from_str(&error_json).unwrap();
        assert!(error_parsed.is_object());
        assert!(error_parsed.get("success").is_some());
        assert!(error_parsed.get("error").is_some());
        assert_eq!(error_parsed["success"], false);
    }

    #[test]
    fn test_method_signature_preservation() {
        // Ensure original methods still exist and work correctly
        let service = TestService::new();

        // Original method should work
        let result1 = service.update_value(100);
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap().value, 100);

        // Original method should handle errors
        let service2 = TestService::new();
        let result2 = service2.update_value(-50);
        assert!(result2.is_err());
    }
}
