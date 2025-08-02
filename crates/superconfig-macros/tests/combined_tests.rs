//! Tests for using both macros together on the same methods

use serde_json::Value;
use superconfig_macros::{generate_json_helper, generate_try_method};

// Mock error type for testing
#[derive(Debug, Clone)]
pub struct CombinedError {
    pub message: String,
}

impl std::fmt::Display for CombinedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CombinedError: {}", self.message)
    }
}

impl std::error::Error for CombinedError {}

// Mock struct that will have both macros applied
#[derive(Debug, Clone, serde::Serialize)]
pub struct CombinedService {
    pub value: String,
    #[serde(skip)]
    pub errors: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl CombinedService {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            errors: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    // Mock error collection method
    pub fn collect_error(&self, operation: &str, error: CombinedError, context: Option<String>) {
        let error_msg = format!("{}: {} (context: {:?})", operation, error, context);
        self.errors.lock().unwrap().push(error_msg);
    }

    // Method with both macros applied - JSON helper should be processed first
    #[generate_json_helper]
    #[generate_try_method]
    pub fn set_text(self, text: String) -> Result<Self, CombinedError> {
        if text.is_empty() {
            Err(CombinedError {
                message: "Text cannot be empty".to_string(),
            })
        } else if text.len() > 50 {
            Err(CombinedError {
                message: "Text too long".to_string(),
            })
        } else {
            Ok(Self {
                value: text,
                errors: self.errors,
            })
        }
    }

    // Another method with both macros but in different order
    #[generate_try_method]
    #[generate_json_helper]
    pub fn append_text(self, suffix: String) -> Result<Self, CombinedError> {
        if suffix.contains("banned") {
            Err(CombinedError {
                message: "Suffix contains banned content".to_string(),
            })
        } else {
            Ok(Self {
                value: format!("{}{}", self.value, suffix),
                errors: self.errors,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_three_methods_exist() {
        let service = CombinedService::new();

        // Original method should exist
        let result1 = service.set_text("hello".to_string());
        assert!(result1.is_ok());

        let service2 = CombinedService::new();
        // try_* method should exist
        let result2 = service2.try_set_text("world".to_string());
        assert_eq!(result2.value, "world");

        let service3 = CombinedService::new();
        // *_as_json method should exist
        let json_result = service3.set_text_as_json("json_test".to_string());
        let parsed: Value = serde_json::from_str(&json_result).unwrap();
        assert_eq!(parsed["success"], true);
    }

    #[test]
    fn test_try_method_error_collection() {
        let service = CombinedService::new();
        let result = service.try_set_text("".to_string()); // Empty string should fail

        // With generic try_method implementation, on error it returns the original self
        // Value should remain unchanged (original empty string)
        assert_eq!(result.value, "");

        // Generic implementation doesn't collect errors - that would be struct-specific
        // For now, no errors are collected in the generic implementation
        let errors = result.errors.lock().unwrap();
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_json_helper_with_errors() {
        let service = CombinedService::new();
        let json_result = service.set_text_as_json(
            "this text is way too long to be accepted by our validation rules".to_string(),
        );

        let parsed: Value = serde_json::from_str(&json_result).unwrap();
        assert_eq!(parsed["success"], false);
        assert!(parsed["error"].as_str().unwrap().contains("Text too long"));
    }

    #[test]
    fn test_macro_order_independence() {
        // Both methods should work the same regardless of macro order
        let service1 = CombinedService::new();
        let service2 = CombinedService::new();

        // set_text has json_helper first, then try_method
        let result1 = service1.try_set_text("test1".to_string());
        let json1 = service2.set_text_as_json("test1".to_string());

        let service3 = CombinedService::new();
        let service4 = CombinedService::new();

        // append_text has try_method first, then json_helper
        let result2 = service3.try_append_text("_suffix".to_string());
        let json2 = service4.append_text_as_json("_suffix".to_string());

        // Both should work correctly
        assert_eq!(result1.value, "test1");
        assert_eq!(result2.value, "_suffix");

        let parsed1: Value = serde_json::from_str(&json1).unwrap();
        let parsed2: Value = serde_json::from_str(&json2).unwrap();
        assert_eq!(parsed1["success"], true);
        assert_eq!(parsed2["success"], true);
    }

    #[test]
    fn test_chaining_try_methods() {
        let service = CombinedService::new();
        let result = service
            .try_set_text("hello".to_string())
            .try_append_text(" world".to_string())
            .try_append_text("!".to_string());

        assert_eq!(result.value, "hello world!");

        // No errors should be collected
        let errors = result.errors.lock().unwrap();
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_chaining_with_errors() {
        let service = CombinedService::new();
        let result = service
            .try_set_text("hello".to_string()) // Success
            .try_append_text("banned".to_string()) // Error - generic impl returns original self
            .try_append_text(" world".to_string()); // Success - appends to "hello"

        // With generic try_method: first succeeds ("hello"), second fails (returns "hello"), third succeeds ("hello world")
        assert_eq!(result.value, "hello world");

        // Generic implementation doesn't collect errors - that would be struct-specific
        let errors = result.errors.lock().unwrap();
        assert_eq!(errors.len(), 0);
    }
}
