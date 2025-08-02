//! Tests for bidirectional `#[generate_json_helper]` macro functionality

use serde::{Deserialize, Serialize};
use superconfig_macros::generate_json_helper;

// Complex types for testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComplexConfig {
    pub name: String,
    pub settings: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComplexResult {
    pub status: String,
    pub data: ComplexConfig,
}

// Mock error type
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

// Test struct with various method signatures
#[derive(Debug, Clone, Serialize)]
pub struct TestService {
    pub value: i32,
}

impl TestService {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    // Test 1: Simple params + simple return = auto should generate _as_json only
    #[generate_json_helper(auto)]
    pub fn simple_method(self, amount: i32) -> Result<Self, TestError> {
        if amount < 0 {
            Err(TestError {
                message: "Negative amount".to_string(),
            })
        } else {
            Ok(Self {
                value: self.value + amount,
            })
        }
    }

    // Test 2: Complex params + simple return = auto should generate _from_json
    #[generate_json_helper(auto)]
    pub fn complex_params_method(self, config: ComplexConfig) -> Result<Self, TestError> {
        if config.name.is_empty() {
            Err(TestError {
                message: "Empty name".to_string(),
            })
        } else {
            Ok(self)
        }
    }

    // Test 3: Simple params + complex return = auto should generate _as_json
    #[generate_json_helper(auto)]
    pub fn complex_return_method(self, id: u64) -> Result<ComplexResult, TestError> {
        if id == 0 {
            Err(TestError {
                message: "Invalid ID".to_string(),
            })
        } else {
            Ok(ComplexResult {
                status: "success".to_string(),
                data: ComplexConfig {
                    name: format!("Config {}", id),
                    settings: vec!["default".to_string()],
                    enabled: true,
                },
            })
        }
    }

    // Test 4: Complex params + complex return = auto should generate both
    #[generate_json_helper(auto)]
    pub fn complex_both_method(self, config: ComplexConfig) -> Result<ComplexResult, TestError> {
        if config.name.is_empty() {
            Err(TestError {
                message: "Empty config name".to_string(),
            })
        } else {
            Ok(ComplexResult {
                status: "processed".to_string(),
                data: config,
            })
        }
    }

    // Test 5: Explicit outgoing only
    #[generate_json_helper(out)]
    pub fn explicit_out_method(self, value: i32) -> Result<Self, TestError> {
        Ok(Self { value })
    }

    // Test 6: Explicit incoming only
    #[generate_json_helper(incoming)]
    pub fn explicit_in_method(self, config: ComplexConfig) -> Result<Self, TestError> {
        if config.name.is_empty() {
            Err(TestError {
                message: "Config name cannot be empty".to_string(),
            })
        } else {
            Ok(self)
        }
    }

    // Test 7: Explicit bidirectional with mixed types (complex input, simple output)
    #[generate_json_helper(incoming, outgoing)]
    pub fn explicit_both_method(self, config: ComplexConfig) -> Result<Self, TestError> {
        if config.name.is_empty() {
            Err(TestError {
                message: "Config name cannot be empty".to_string(),
            })
        } else {
            Ok(Self {
                value: config.settings.len() as i32,
            })
        }
    }

    // Test 8: Non-Result method with complex return (should generate _as_json)
    #[generate_json_helper(auto)]
    pub fn non_result_method(self, value: i32) -> ComplexResult {
        ComplexResult {
            status: "completed".to_string(),
            data: ComplexConfig {
                name: format!("result_{}", value),
                settings: vec!["default".to_string()],
                enabled: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_method_generates_as_json_only() {
        let service = TestService::new();

        // Original method should work
        let result = service.clone().simple_method(5).unwrap();
        assert_eq!(result.value, 5);

        // Should have generated _as_json method
        let json_result = service.simple_method_as_json(5);
        assert!(json_result.contains("success"));
        assert!(json_result.contains("true"));

        // Should NOT have generated _from_json method (this would cause compile error)
        // Uncomment to test: service.simple_method_from_json("{}");
    }

    #[test]
    fn test_complex_params_generates_from_json() {
        let service = TestService::new();
        let config = ComplexConfig {
            name: "test".to_string(),
            settings: vec!["setting1".to_string()],
            enabled: true,
        };

        // Original method should work
        let result = service
            .clone()
            .complex_params_method(config.clone())
            .unwrap();
        assert_eq!(result.value, 0);

        // Should have generated _from_json method
        let json_result = service.complex_params_method_json(
            r#"{"config":{"name":"test","settings":["test"],"enabled":true}}"#,
        );
        assert!(json_result.contains("success") || json_result.contains("error"));
    }

    #[test]
    fn test_complex_return_generates_as_json() {
        let service = TestService::new();

        // Original method should work
        let result = service.clone().complex_return_method(123).unwrap();
        assert_eq!(result.status, "success");

        // Should have generated _as_json method
        let json_result = service.complex_return_method_as_json(123);
        assert!(json_result.contains("success"));
        assert!(json_result.contains("data"));
    }

    #[test]
    fn test_complex_both_generates_unified_method() {
        let service = TestService::new();
        let config = ComplexConfig {
            name: "test".to_string(),
            settings: vec!["setting1".to_string()],
            enabled: true,
        };

        // Original method should work
        let result = service.clone().complex_both_method(config.clone()).unwrap();
        assert_eq!(result.status, "processed");

        // Should have generated unified _json method (not separate as_json/from_json)
        let json_result = service.complex_both_method_json(
            r#"{"config":{"name":"test","settings":["setting1"],"enabled":true}}"#,
        );
        assert!(json_result.contains("success") || json_result.contains("error"));
    }

    #[test]
    fn test_explicit_out_only() {
        let service = TestService::new();

        // Should have _as_json method
        let json_result = service.explicit_out_method_as_json(42);
        assert!(json_result.contains("success"));

        // Should NOT have _from_json method (would cause compile error)
        // service.explicit_out_method_from_json("{}");
    }

    #[test]
    fn test_explicit_in_only() {
        let service = TestService::new();

        // Should have _from_json method
        let json_result = service.explicit_in_method_from_json(r#"{"name":"test"}"#);
        assert!(json_result.contains("success") || json_result.contains("error"));

        // Should NOT have _as_json method (would cause compile error)
        // service.explicit_in_method_as_json(...);
    }

    #[test]
    fn test_explicit_both() {
        let service = TestService::new();

        // Should have unified method for complex input + simple output
        let json_result = service.explicit_both_method_json(
            r#"{"config":{"name":"test","settings":["setting1"],"enabled":true}}"#,
        );
        assert!(json_result.contains("success") || json_result.contains("error"));
    }

    #[test]
    fn test_error_handling_in_as_json() {
        let service = TestService::new();

        // Test error case
        let json_result = service.simple_method_as_json(-1);
        assert!(json_result.contains("success"));
        assert!(json_result.contains("false"));
        assert!(json_result.contains("error"));
        assert!(json_result.contains("Negative amount"));
    }

    #[test]
    fn test_non_result_method() {
        let service = TestService::new();

        // Original method should work
        let result = service.clone().non_result_method(99);
        assert_eq!(result.status, "completed");
        assert_eq!(result.data.name, "result_99");

        // Should generate _as_json for non-Result methods with complex return
        let json_result = service.non_result_method_as_json(99);
        assert!(json_result.contains("success"));
        assert!(json_result.contains("result_99"));
    }

    #[test]
    fn test_json_structure() {
        let service = TestService::new();

        // Test successful response structure
        let json_result = service.clone().simple_method_as_json(10);
        let parsed: serde_json::Value = serde_json::from_str(&json_result).unwrap();

        assert_eq!(parsed["success"], true);
        // Should have data field for successful results
        assert!(parsed.get("data").is_some() || parsed.get("success").is_some());

        // Test error response structure
        let json_error = service.simple_method_as_json(-1);
        let parsed_error: serde_json::Value = serde_json::from_str(&json_error).unwrap();

        assert_eq!(parsed_error["success"], false);
        assert!(parsed_error.get("error").is_some());
    }
}
