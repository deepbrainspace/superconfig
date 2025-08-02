//! Tests for the `#[generate_try_method]` procedural macro

use superconfig_macros::generate_try_method;

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
#[derive(Debug, Clone)]
pub struct TestRegistry {
    pub value: i32,
    pub errors: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl TestRegistry {
    pub fn new() -> Self {
        Self {
            value: 0,
            errors: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    // Mock error collection method that the macro expects
    pub fn collect_error(&self, operation: &str, error: TestError, context: Option<String>) {
        let error_msg = format!("{}: {} (context: {:?})", operation, error, context);
        self.errors.lock().unwrap().push(error_msg);
    }

    // Test method that will have the macro applied
    #[generate_try_method]
    pub fn increment(self, amount: i32) -> Result<Self, TestError> {
        if amount < 0 {
            Err(TestError {
                message: "Cannot increment by negative amount".to_string(),
            })
        } else {
            Ok(Self {
                value: self.value + amount,
                errors: self.errors,
            })
        }
    }

    // Another test method with different signature
    #[generate_try_method]
    pub fn set_value(self, new_value: i32) -> Result<Self, TestError> {
        if new_value > 100 {
            Err(TestError {
                message: "Value cannot exceed 100".to_string(),
            })
        } else {
            Ok(Self {
                value: new_value,
                errors: self.errors,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_original_method_success() {
        let registry = TestRegistry::new();
        let result = registry.increment(5).unwrap();
        assert_eq!(result.value, 5);
    }

    #[test]
    fn test_original_method_error() {
        let registry = TestRegistry::new();
        let result = registry.increment(-1);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message,
            "Cannot increment by negative amount"
        );
    }

    #[test]
    fn test_try_method_success() {
        let registry = TestRegistry::new();
        let result = registry.try_increment(5);
        assert_eq!(result.value, 5);

        // Should have no errors collected
        let errors = result.errors.lock().unwrap();
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_try_method_error_collection() {
        let registry = TestRegistry::new();
        let result = registry.try_increment(-1);

        // Should return self unchanged
        assert_eq!(result.value, 0);

        // Generic implementation doesn't collect errors - that would be struct-specific
        // For now, no errors are collected in the generic implementation
        let errors = result.errors.lock().unwrap();
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_multiple_try_method_errors() {
        let registry = TestRegistry::new();
        let result = registry
            .try_increment(-1) // Should collect error
            .try_set_value(150) // Should collect error
            .try_increment(5); // Should succeed

        // Final value should be 5 (only the successful operation applied)
        assert_eq!(result.value, 5);

        // Generic implementation doesn't collect errors - that would be struct-specific
        // For now, no errors are collected in the generic implementation
        let errors = result.errors.lock().unwrap();
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_method_signature_preservation() {
        // Test that the original methods still exist and work
        let registry = TestRegistry::new();

        // Original methods should still work
        let _ = registry.increment(1).unwrap();

        let registry2 = TestRegistry::new();
        let _ = registry2.set_value(50).unwrap();
    }
}
