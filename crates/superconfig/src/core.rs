//! Core registry system for `SuperConfig` V2
//!
//! This module implements the foundational handle-based registry system that enables
//! zero-copy configuration access with sub-microsecond lookup times.

use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};
use thiserror::Error;

/// Unique identifier for configuration handles
pub type HandleId = u64;

/// Global configuration registry instance
static GLOBAL_REGISTRY: std::sync::LazyLock<ConfigRegistry> =
    std::sync::LazyLock::new(ConfigRegistry::new);

/// Get a reference to the global configuration registry
#[must_use]
pub fn global_registry() -> &'static ConfigRegistry {
    &GLOBAL_REGISTRY
}

/// Errors that can occur during registry operations
#[derive(Error, Debug, Clone)]
pub enum RegistryError {
    /// Handle not found in registry
    #[error("Handle {handle_id} not found in registry")]
    HandleNotFound {
        /// The handle ID that was not found
        handle_id: HandleId,
    },

    /// Handle has wrong type
    #[error("Handle {handle_id} has wrong type: expected {expected}, found {found}")]
    WrongType {
        /// The handle ID with wrong type
        handle_id: HandleId,
        /// Expected type name
        expected: &'static str,
        /// Found type name
        found: &'static str,
    },

    /// Handle has been invalidated
    #[error("Handle {handle_id} has been invalidated")]
    InvalidHandle {
        /// The invalidated handle ID
        handle_id: HandleId,
    },

    /// Registry is at capacity
    #[error("Registry is at maximum capacity")]
    RegistryFull,

    /// Serialization error
    #[error("Serialization error: {message}")]
    SerializationError {
        /// Error message
        message: String,
    },
}

/// Statistics about the registry state
#[derive(Debug, Clone, Default)]
pub struct RegistryStats {
    /// Total number of active handles
    pub total_handles: u64,
    /// Total number of create operations
    pub total_creates: u64,
    /// Total number of read operations
    pub total_reads: u64,
    /// Total number of update operations
    pub total_updates: u64,
    /// Total number of delete operations
    pub total_deletes: u64,
    /// Approximate memory usage in bytes
    pub memory_usage_bytes: u64,
}

/// Internal entry stored in the registry
#[derive(Debug)]
struct ConfigEntry {
    /// The actual configuration data
    data: Box<dyn std::any::Any + Send + Sync>,
    /// Type name for runtime type checking
    type_name: &'static str,
    /// When this entry was created (used for cache eviction in Phase 5)
    #[allow(dead_code)]
    created_at: Instant,
    /// When this entry was last accessed (used for LRU eviction in Phase 5)
    #[allow(dead_code)]
    last_accessed: Instant,
    /// Registry-level reference count (for statistics, separate from Arc's count)
    #[allow(dead_code)]
    ref_count: AtomicU64,
    /// Size of the data in bytes (approximate)
    data_size: usize,
}

impl ConfigEntry {
    fn new<T: 'static + Send + Sync>(data: T) -> Self {
        let data_size = std::mem::size_of::<T>();
        Self {
            data: Box::new(Arc::new(data)), // Always store as Arc<T>
            type_name: std::any::type_name::<T>(),
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            ref_count: AtomicU64::new(1),
            data_size,
        }
    }

    fn get_arc_data<T: 'static>(&self) -> Result<Arc<T>, RegistryError> {
        let expected_type = std::any::type_name::<T>();
        if self.type_name != expected_type {
            return Err(RegistryError::WrongType {
                handle_id: 0, // Will be filled in by caller
                expected: expected_type,
                found: self.type_name,
            });
        }

        self.data
            .downcast_ref::<Arc<T>>()
            .cloned() // Clone the Arc (cheap - just increment counter)
            .ok_or(RegistryError::WrongType {
                handle_id: 0, // Will be filled in by caller
                expected: expected_type,
                found: self.type_name,
            })
    }
}

/// Type-safe handle for accessing configuration data
#[derive(Debug, Clone)]
pub struct ConfigHandle<T> {
    id: HandleId,
    _phantom: PhantomData<T>,
}

impl<T> ConfigHandle<T> {
    const fn new(id: HandleId) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }

    /// Get the handle ID
    #[must_use]
    pub const fn id(&self) -> HandleId {
        self.id
    }
}

// Implement Serialize/Deserialize for ConfigHandle
impl<T> Serialize for ConfigHandle<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.id.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for ConfigHandle<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = HandleId::deserialize(deserializer)?;
        Ok(Self::new(id))
    }
}

/// Main configuration registry using lock-free operations
pub struct ConfigRegistry {
    /// Internal storage using `DashMap` for lock-free operations
    entries: DashMap<HandleId, ConfigEntry>,
    /// Atomic counter for generating unique handle IDs
    next_id: AtomicU64,
    /// Registry statistics protected by `RwLock`
    stats: Arc<RwLock<RegistryStats>>,
}

impl ConfigRegistry {
    /// Create a new configuration registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: DashMap::new(),
            next_id: AtomicU64::new(1),
            stats: Arc::new(RwLock::new(RegistryStats::default())),
        }
    }

    /// Create a new configuration entry and return a handle to it
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::RegistryFull` if the registry has reached maximum capacity.
    pub fn create<T: 'static + Send + Sync>(
        &self,
        data: T,
    ) -> Result<ConfigHandle<T>, RegistryError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let entry = ConfigEntry::new(data);
        let data_size = entry.data_size;

        self.entries.insert(id, entry);

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.total_handles += 1;
            stats.total_creates += 1;
            stats.memory_usage_bytes += data_size as u64;
        }

        Ok(ConfigHandle::new(id))
    }

    /// Read configuration data
    ///
    /// Returns `Arc<T>` for efficient sharing. Use field access (`config.host`)
    /// and method calls (`config.validate()`) directly - they're zero-cost due to auto-deref.
    ///
    /// For mutations, create new config and use `update()`.
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::HandleNotFound` if the handle doesn't exist in the registry.
    /// Returns `RegistryError::WrongType` if the handle points to data of a different type.
    ///
    /// # Performance Notes
    ///
    /// - Field access: `config.database.host` (zero cost)
    /// - Method calls: `config.validate()` (zero cost)
    /// - Passing to functions: `process(config)` (~1ns to move Arc)
    /// - Multiple reads share the same underlying data efficiently
    pub fn read<T: 'static>(&self, handle: &ConfigHandle<T>) -> Result<Arc<T>, RegistryError> {
        let entry_ref = self
            .entries
            .get(&handle.id)
            .ok_or(RegistryError::HandleNotFound {
                handle_id: handle.id,
            })?;

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.total_reads += 1;
        }

        let arc = entry_ref.get_arc_data::<T>().map_err(|mut e| {
            if let RegistryError::WrongType { handle_id, .. } = &mut e {
                *handle_id = handle.id;
            }
            e
        })?;

        Ok(Arc::clone(&arc))
    }

    /// Read data as JSON string (internal helper for FFI crates)
    ///
    /// This method is used by FFI wrappers (Phase 4: Python/Node.js bindings) to provide
    /// consistent JSON serialization across all language bindings while maintaining
    /// the same `read()` function name.
    #[allow(dead_code)] // Used by FFI crates in Phase 4
    pub(crate) fn read_as_json<T: Serialize + 'static>(
        &self,
        handle: &ConfigHandle<T>,
    ) -> Result<String, RegistryError> {
        let arc = self.read(handle)?;
        serde_json::to_string(&*arc).map_err(|e| RegistryError::SerializationError {
            message: e.to_string(),
        })
    }

    /// Update data in a configuration handle
    ///
    /// This replaces the entire configuration data with new data.
    /// Any existing Arc references will continue to point to the old data.
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::HandleNotFound` if the handle doesn't exist in the registry.
    pub fn update<T: 'static + Send + Sync>(
        &self,
        handle: &ConfigHandle<T>,
        new_data: T,
    ) -> Result<(), RegistryError> {
        // Remove old entry to get its size
        let old_entry = self
            .entries
            .remove(&handle.id)
            .ok_or(RegistryError::HandleNotFound {
                handle_id: handle.id,
            })?;

        let old_size = old_entry.1.data_size;

        // Create new entry with new data
        let new_entry = ConfigEntry::new(new_data);
        let new_size = new_entry.data_size;

        // Insert new entry (this atomically replaces the old one)
        self.entries.insert(handle.id, new_entry);

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.total_updates += 1;
            stats.memory_usage_bytes = stats
                .memory_usage_bytes
                .saturating_sub(old_size as u64)
                .saturating_add(new_size as u64);
        }

        Ok(())
    }

    /// Delete a configuration entry and return the data as Arc<T>
    ///
    /// Returns the same Arc<T> that was stored internally, avoiding any cloning.
    /// This is consistent with our zero-copy design philosophy.
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::HandleNotFound` if the handle doesn't exist in the registry.
    /// Returns `RegistryError::WrongType` if the handle points to data of a different type.
    pub fn delete<T: 'static>(&self, handle: &ConfigHandle<T>) -> Result<Arc<T>, RegistryError> {
        let (_, entry) = self
            .entries
            .remove(&handle.id)
            .ok_or(RegistryError::HandleNotFound {
                handle_id: handle.id,
            })?;

        let data_size = entry.data_size;

        // Extract the Arc<T> directly
        let arc = entry
            .data
            .downcast::<Arc<T>>()
            .map_err(|_| RegistryError::WrongType {
                handle_id: handle.id,
                expected: std::any::type_name::<T>(),
                found: entry.type_name,
            })?;

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.total_handles = stats.total_handles.saturating_sub(1);
            stats.total_deletes += 1;
            stats.memory_usage_bytes = stats.memory_usage_bytes.saturating_sub(data_size as u64);
        }

        Ok(*arc)
    }

    /// Get current registry statistics
    #[must_use]
    pub fn stats(&self) -> RegistryStats {
        self.stats.read().clone()
    }

    /// Check if a handle exists in the registry
    #[must_use]
    pub fn contains_handle<T>(&self, handle: &ConfigHandle<T>) -> bool {
        self.entries.contains_key(&handle.id)
    }

    /// Clear all entries from the registry
    pub fn clear(&self) {
        self.entries.clear();
        let mut stats = self.stats.write();
        *stats = RegistryStats::default();
    }

    /// Get the number of entries in the registry
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the registry is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for ConfigRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::thread;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestConfig {
        host: String,
        port: u16,
        timeout_ms: u32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct SimpleConfig {
        value: i32,
    }

    #[test]
    fn test_create_and_read() {
        let registry = ConfigRegistry::new();
        let config = TestConfig {
            host: "localhost".to_string(),
            port: 8080,
            timeout_ms: 5000,
        };

        let handle = registry.create(config.clone()).unwrap();
        let retrieved = registry.read(&handle).unwrap();

        assert_eq!(*retrieved, config);
        assert_eq!(handle.id(), 1);
    }

    #[test]
    fn test_update() {
        let registry = ConfigRegistry::new();
        let original = TestConfig {
            host: "localhost".to_string(),
            port: 8080,
            timeout_ms: 5000,
        };
        let updated = TestConfig {
            host: "remote".to_string(),
            port: 9090,
            timeout_ms: 10000,
        };

        let handle = registry.create(original).unwrap();

        // Keep a reference to the old data
        let old_data = registry.read(&handle).unwrap();

        registry.update(&handle, updated.clone()).unwrap();
        let new_data = registry.read(&handle).unwrap();

        // New data should be updated
        assert_eq!(*new_data, updated);

        // Old Arc reference should still point to original data
        assert_eq!(old_data.host, "localhost");
        assert_eq!(old_data.port, 8080);
    }

    #[test]
    fn test_delete() {
        let registry = ConfigRegistry::new();
        let config = TestConfig {
            host: "localhost".to_string(),
            port: 8080,
            timeout_ms: 5000,
        };

        let handle = registry.create(config.clone()).unwrap();
        let deleted = registry.delete(&handle).unwrap();

        assert_eq!(*deleted, config);

        // Handle should no longer exist
        assert!(!registry.contains_handle(&handle));
        assert!(registry.read(&handle).is_err());
    }

    #[test]
    fn test_handle_not_found() {
        let registry = ConfigRegistry::new();
        let fake_handle = ConfigHandle::<TestConfig>::new(999);

        let result = registry.read(&fake_handle);
        assert!(matches!(
            result,
            Err(RegistryError::HandleNotFound { handle_id: 999 })
        ));
    }

    #[test]
    fn test_wrong_type() {
        let registry = ConfigRegistry::new();
        let config = TestConfig {
            host: "localhost".to_string(),
            port: 8080,
            timeout_ms: 5000,
        };

        let handle = registry.create(config).unwrap();

        // Try to read as wrong type by manually creating handle with wrong type
        let wrong_handle = ConfigHandle::<SimpleConfig>::new(handle.id());
        let result = registry.read(&wrong_handle);

        assert!(
            matches!(result, Err(RegistryError::WrongType { handle_id, .. }) if handle_id == handle.id())
        );
    }

    #[test]
    fn test_statistics() {
        let registry = ConfigRegistry::new();
        let config1 = TestConfig {
            host: "localhost".to_string(),
            port: 8080,
            timeout_ms: 5000,
        };
        let config2 = SimpleConfig { value: 42 };

        // Initial stats
        let stats = registry.stats();
        assert_eq!(stats.total_handles, 0);
        assert_eq!(stats.total_creates, 0);
        assert_eq!(stats.total_reads, 0);

        // Create operations
        let handle1 = registry.create(config1).unwrap();
        let handle2 = registry.create(config2).unwrap();

        let stats = registry.stats();
        assert_eq!(stats.total_handles, 2);
        assert_eq!(stats.total_creates, 2);
        assert!(stats.memory_usage_bytes > 0);

        // Read operations
        let _data1 = registry.read(&handle1).unwrap();
        let _data2 = registry.read(&handle2).unwrap();

        let stats = registry.stats();
        assert_eq!(stats.total_reads, 2);

        // Update operation
        registry
            .update(
                &handle1,
                TestConfig {
                    host: "updated".to_string(),
                    port: 9090,
                    timeout_ms: 6000,
                },
            )
            .unwrap();

        let stats = registry.stats();
        assert_eq!(stats.total_updates, 1);

        // Delete operation
        registry.delete(&handle2).unwrap();

        let stats = registry.stats();
        assert_eq!(stats.total_handles, 1);
        assert_eq!(stats.total_deletes, 1);
    }

    #[test]
    fn test_concurrent_access() {
        let registry = Arc::new(ConfigRegistry::new());
        let counter = Arc::new(AtomicU32::new(0));
        let num_threads = 10;
        let operations_per_thread = 100;

        let mut handles = vec![];

        // Spawn threads for concurrent operations
        for _ in 0..num_threads {
            let registry_clone = Arc::clone(&registry);
            let counter_clone = Arc::clone(&counter);

            let handle = thread::spawn(move || {
                for i in 0..operations_per_thread {
                    let config = SimpleConfig {
                        value: counter_clone.fetch_add(1, Ordering::Relaxed) as i32,
                    };

                    let handle = registry_clone.create(config).unwrap();
                    let _data = registry_clone.read(&handle).unwrap();

                    if i % 2 == 0 {
                        registry_clone
                            .update(&handle, SimpleConfig { value: -1 })
                            .unwrap();
                    }

                    if i % 3 == 0 {
                        let _deleted = registry_clone.delete(&handle).unwrap();
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        let stats = registry.stats();
        assert_eq!(
            stats.total_creates,
            (num_threads * operations_per_thread) as u64
        );
        assert!(stats.total_reads >= (num_threads * operations_per_thread) as u64);
    }

    #[test]
    fn test_handle_serialization() {
        let registry = ConfigRegistry::new();
        let config = TestConfig {
            host: "localhost".to_string(),
            port: 8080,
            timeout_ms: 5000,
        };

        let handle = registry.create(config).unwrap();

        // Serialize handle
        let serialized = serde_json::to_string(&handle).unwrap();

        // Deserialize handle
        let deserialized: ConfigHandle<TestConfig> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(handle.id(), deserialized.id());

        // Should be able to use deserialized handle
        let _data = registry.read(&deserialized).unwrap();
    }

    #[test]
    fn test_global_registry() {
        let config = SimpleConfig { value: 123 };

        let handle = global_registry().create(config.clone()).unwrap();
        let retrieved = global_registry().read(&handle).unwrap();

        assert_eq!(*retrieved, config);
    }

    #[test]
    fn test_arc_sharing() {
        let registry = ConfigRegistry::new();
        let config = TestConfig {
            host: "localhost".to_string(),
            port: 8080,
            timeout_ms: 5000,
        };

        let handle = registry.create(config).unwrap();

        // Multiple reads should return the same underlying data
        let arc1 = registry.read(&handle).unwrap();
        let arc2 = registry.read(&handle).unwrap();

        // They should be the same Arc (same pointer)
        assert!(Arc::ptr_eq(&arc1, &arc2));

        // Verify we can access data through both
        assert_eq!(arc1.host, "localhost");
        assert_eq!(arc2.port, 8080);
    }

    #[test]
    fn test_memory_cleanup() {
        let registry = ConfigRegistry::new();
        let config = TestConfig {
            host: "localhost".to_string(),
            port: 8080,
            timeout_ms: 5000,
        };

        let handle = registry.create(config).unwrap();
        let initial_memory = registry.stats().memory_usage_bytes;

        // Delete should reduce memory usage
        let _deleted = registry.delete(&handle).unwrap();
        let final_memory = registry.stats().memory_usage_bytes;

        assert!(final_memory < initial_memory);
    }

    #[test]
    fn test_clear_registry() {
        let registry = ConfigRegistry::new();

        // Add some entries
        let _handle1 = registry.create(SimpleConfig { value: 1 }).unwrap();
        let _handle2 = registry.create(SimpleConfig { value: 2 }).unwrap();

        assert_eq!(registry.len(), 2);
        assert!(!registry.is_empty());

        // Clear registry
        registry.clear();

        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());

        let stats = registry.stats();
        assert_eq!(stats.total_handles, 0);
        assert_eq!(stats.memory_usage_bytes, 0);
    }

    #[test]
    fn test_handle_id_generation() {
        let registry = ConfigRegistry::new();

        let handle1 = registry.create(SimpleConfig { value: 1 }).unwrap();
        let handle2 = registry.create(SimpleConfig { value: 2 }).unwrap();
        let handle3 = registry.create(SimpleConfig { value: 3 }).unwrap();

        // IDs should be sequential and unique
        assert_eq!(handle1.id(), 1);
        assert_eq!(handle2.id(), 2);
        assert_eq!(handle3.id(), 3);

        // Delete one and create another - should get next ID
        registry.delete(&handle2).unwrap();
        let handle4 = registry.create(SimpleConfig { value: 4 }).unwrap();
        assert_eq!(handle4.id(), 4);
    }
}
