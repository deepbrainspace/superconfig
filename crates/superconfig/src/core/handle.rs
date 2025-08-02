//! Type-safe handles for accessing configuration data

use super::registry::HandleId;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// Type-safe handle for accessing configuration data
///
/// Handles provide zero-cost type safety for registry operations.
/// They serialize as just the handle ID for efficient FFI usage.
///
/// # Examples
///
/// ```
/// use superconfig::ConfigRegistry;
///
/// #[derive(Clone, PartialEq, Debug)]
/// struct MyConfig {
///     host: String,
///     port: u16,
/// }
///
/// let registry = ConfigRegistry::new();
/// let config = MyConfig {
///     host: "localhost".to_string(),
///     port: 8080,
/// };
///
/// let handle = registry.create(config.clone()).unwrap();
/// let retrieved = registry.read(&handle).unwrap();
/// assert_eq!(*retrieved, config);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ConfigHandle<T> {
    id: HandleId,
    _phantom: PhantomData<T>,
}

impl<T> ConfigHandle<T> {
    /// Create a new handle with the given ID
    ///
    /// This is primarily used internally by the registry
    pub(crate) const fn new(id: HandleId) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }

    /// Get the handle ID
    ///
    /// # Examples
    ///
    /// ```
    /// use superconfig::ConfigRegistry;
    ///
    /// let registry = ConfigRegistry::new();
    /// let handle = registry.create("test".to_string()).unwrap();
    ///
    /// let id = handle.id();
    /// assert_eq!(id, 1); // First handle gets ID 1
    /// ```
    #[must_use]
    pub const fn id(&self) -> HandleId {
        self.id
    }
}

/// Handles serialize as just their ID for efficient FFI usage
impl<T> Serialize for ConfigHandle<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.id.serialize(serializer)
    }
}

/// Handles deserialize from their ID
impl<'de, T> Deserialize<'de> for ConfigHandle<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = HandleId::deserialize(deserializer)?;
        Ok(Self::new(id))
    }
}

// Implement common traits for ergonomic usage
impl<T> PartialEq for ConfigHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for ConfigHandle<T> {}

impl<T> std::hash::Hash for ConfigHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct TestConfig {
        value: i32,
    }

    #[test]
    fn test_handle_creation() {
        let handle = ConfigHandle::<TestConfig>::new(42);
        assert_eq!(handle.id(), 42);
    }

    #[test]
    fn test_handle_serialization() {
        let handle = ConfigHandle::<TestConfig>::new(123);

        let serialized = serde_json::to_string(&handle).unwrap();
        assert_eq!(serialized, "123");

        let deserialized: ConfigHandle<TestConfig> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(handle.id(), deserialized.id());
    }

    #[test]
    fn test_handle_equality() {
        let handle1 = ConfigHandle::<TestConfig>::new(1);
        let handle2 = ConfigHandle::<TestConfig>::new(1);
        let handle3 = ConfigHandle::<TestConfig>::new(2);

        assert_eq!(handle1, handle2);
        assert_ne!(handle1, handle3);
    }

    #[test]
    fn test_handle_hash() {
        use std::collections::HashSet;

        let handle1 = ConfigHandle::<TestConfig>::new(1);
        let handle2 = ConfigHandle::<TestConfig>::new(1);
        let handle3 = ConfigHandle::<TestConfig>::new(2);

        let mut set = HashSet::new();
        set.insert(handle1);
        set.insert(handle2); // Should not increase size (same ID)
        set.insert(handle3);

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_handle_copy() {
        let handle1 = ConfigHandle::<TestConfig>::new(42);
        let handle2 = handle1; // Should copy, not move

        assert_eq!(handle1.id(), 42);
        assert_eq!(handle2.id(), 42);
    }

    #[test]
    fn test_handle_debug() {
        let handle = ConfigHandle::<TestConfig>::new(999);
        let debug_str = format!("{handle:?}");

        assert!(debug_str.contains("999"));
        assert!(debug_str.contains("ConfigHandle"));
    }

    #[test]
    fn test_phantom_data_zero_cost() {
        use std::mem;

        // Handle should be same size as HandleId (u64)
        assert_eq!(
            mem::size_of::<ConfigHandle<TestConfig>>(),
            mem::size_of::<HandleId>()
        );
        assert_eq!(
            mem::size_of::<ConfigHandle<String>>(),
            mem::size_of::<HandleId>()
        );
    }

    #[test]
    fn test_handle_json_roundtrip() {
        let original = ConfigHandle::<TestConfig>::new(12345);

        // Serialize to JSON
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "12345");

        // Deserialize from JSON
        let restored: ConfigHandle<TestConfig> = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
        assert_eq!(original.id(), restored.id());
    }

    #[test]
    fn test_different_types_different_handles() {
        // Even with same ID, handles with different types are different types
        let handle_string = ConfigHandle::<String>::new(1);
        let handle_int = ConfigHandle::<i32>::new(1);

        // These should be different types (won't compile if we try to compare them)
        assert_eq!(handle_string.id(), handle_int.id()); // IDs are same
        // handle_string == handle_int; // This won't compile - good!
    }

    #[test]
    fn test_handle_deserialize_error_path() {
        // Test the error path in deserialize (line 85)
        // This triggers the `?` operator when HandleId::deserialize fails
        let invalid_json = "\"not_a_number\""; // String instead of number for HandleId
        let result: Result<ConfigHandle<TestConfig>, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());

        // Also test with completely malformed JSON
        let malformed_json = "{invalid}";
        let result2: Result<ConfigHandle<TestConfig>, _> = serde_json::from_str(malformed_json);
        assert!(result2.is_err());
    }
}
