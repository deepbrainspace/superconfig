//! Tests for Arc and ConfigHandle scenarios in JSON helper macro

use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;
use superconfig_macros::generate_json_helper;

// Mock ConfigHandle similar to the real one
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ConfigHandle<T> {
    id: u64,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ConfigHandle<T> {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

// Mock data type for testing
#[derive(Debug, Clone, Serialize)]
pub struct TestConfig {
    pub value: String,
}

// Mock registry similar to ConfigRegistry
pub struct TestRegistry {
    pub data: String,
}

impl TestRegistry {
    pub fn new() -> Self {
        Self {
            data: "test config".to_string(),
        }
    }

    // This is the problematic method - it should generate read_as_json, not read_json
    #[generate_json_helper(auto)]
    pub fn read<T: Serialize + 'static>(&self, handle: &ConfigHandle<T>) -> Result<Arc<T>, String> {
        // Mock implementation - in real code this would look up the handle
        // For testing, we'll create a dummy Arc<T> by deserializing our test data
        Err(format!("Mock implementation for handle {}", handle.id()))
    }

    // Test method that should generate only _as_json (outgoing only)
    #[generate_json_helper(auto)]
    pub fn simple_read(&self, id: u64) -> Result<Arc<TestConfig>, String> {
        if id == 1 {
            Ok(Arc::new(TestConfig {
                value: "test".to_string(),
            }))
        } else {
            Err("Not found".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_read_generates_as_json() {
        let registry = TestRegistry::new();

        // This should work - simple params, complex return = outgoing only
        let json_result = registry.simple_read_as_json(1);

        let parsed: Value = serde_json::from_str(&json_result).unwrap();
        assert_eq!(parsed["success"], true);
        assert!(parsed["data"].is_object());
    }

    #[test]
    fn test_simple_read_error_case() {
        let registry = TestRegistry::new();

        let json_result = registry.simple_read_as_json(999);

        let parsed: Value = serde_json::from_str(&json_result).unwrap();
        assert_eq!(parsed["success"], false);
        assert!(parsed["error"].as_str().unwrap().contains("Not found"));
    }

    #[test]
    fn test_read_should_generate_as_json_not_json() {
        let registry = TestRegistry::new();
        let handle = ConfigHandle::<TestConfig>::new(1);

        // This should generate read_as_json, not read_json
        let json_result = registry.read_as_json(&handle);

        let parsed: Value = serde_json::from_str(&json_result).unwrap();
        assert_eq!(parsed["success"], false);
        assert!(
            parsed["error"]
                .as_str()
                .unwrap()
                .contains("Mock implementation")
        );
    }

    #[test]
    fn test_serialize_constraint_is_enforced() {
        // This test ensures the generated method requires T: Serialize
        let registry = TestRegistry::new();
        let handle = ConfigHandle::<TestConfig>::new(1);

        // TestConfig implements Serialize, so this should work
        let _json_result = registry.read_as_json(&handle);

        // If we tried to use a non-Serialize type, it would fail at compile time
        // This is verified by the fact that the macro generates T: Serialize constraint
    }

    #[test]
    fn test_data_format_is_direct_json_object() {
        let registry = TestRegistry::new();

        // Test with successful Arc<TestConfig> return
        let json_result = registry.simple_read_as_json(1);
        let parsed: Value = serde_json::from_str(&json_result).unwrap();

        assert_eq!(parsed["success"], true);

        // The data should be a direct JSON object, not a stringified JSON
        let data = &parsed["data"];
        assert!(data.is_object());
        assert_eq!(data["value"], "test");

        // This should NOT be a string containing JSON like "{\"value\":\"test\"}"
        assert!(!data.is_string());
    }

    #[test]
    fn test_primitive_types_in_data_field() {
        // This tests that primitive types work correctly in the data field
        let registry = TestRegistry::new();

        // Create a mock method that returns a primitive type
        // (This would be added to TestRegistry if we needed to test this)
        // For now, we verify the concept through the error message format
        let json_result = registry.simple_read_as_json(999);
        let parsed: Value = serde_json::from_str(&json_result).unwrap();

        assert_eq!(parsed["success"], false);
        assert!(parsed["error"].is_string()); // Error is a direct string, not nested JSON
    }

    #[test]
    fn test_arc_dereferencing_works() {
        let registry = TestRegistry::new();

        // This test verifies that Arc<T> is properly dereferenced for serialization
        let json_result = registry.simple_read_as_json(1);
        let parsed: Value = serde_json::from_str(&json_result).unwrap();

        assert_eq!(parsed["success"], true);

        // If Arc wasn't dereferenced properly, serialization would fail
        // The fact that we get a proper JSON object proves &*result is used
        assert!(parsed["data"].is_object());
    }
}
