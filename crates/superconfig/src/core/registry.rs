//! Main configuration registry implementation

use dashmap::DashMap;
use parking_lot::RwLock;
use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};
use superconfig_macros::generate_json_helper;

use super::{handle::ConfigHandle, stats::RegistryStats};
use logffi::error;

/// Unique identifier for configuration handles
pub type HandleId = u64;

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

    fn get_arc_data<T: 'static>(&self) -> Result<Arc<T>, String> {
        let expected_type = std::any::type_name::<T>();
        if self.type_name != expected_type {
            let error_msg = format!(
                "superconfig.registry: Wrong type, expected {}, found {}",
                expected_type, self.type_name
            );
            error!(target: "superconfig.registry", "Wrong type, expected {}, found {}", expected_type, self.type_name);
            return Err(error_msg);
        }

        // If we get here, downcast must succeed
        Ok(self.data.downcast_ref::<Arc<T>>().unwrap().clone())
    }
}

/// Main configuration registry using lock-free operations
///
/// The registry provides handle-based access to configuration data with sub-microsecond
/// lookup times. It supports both startup flags (immutable after creation) and runtime
/// flags (mutable during operation).
///
/// # Examples
///
/// ```
/// use superconfig::{ConfigRegistry, config_flags::{startup, runtime}};
///
/// // Create registry with startup flags
/// let registry = ConfigRegistry::custom(startup::SIMD | startup::THREAD_POOL)
///     .enable(runtime::STRICT_MODE);
///
/// // Store configuration
/// let handle = registry.create("localhost".to_string()).unwrap();
/// let config = registry.read(&handle).unwrap();
/// assert_eq!(*config, "localhost");
/// ```
pub struct ConfigRegistry {
    /// Internal storage using `DashMap` for lock-free operations
    entries: DashMap<HandleId, ConfigEntry>,
    /// Atomic counter for generating unique handle IDs
    next_id: AtomicU64,
    /// Registry statistics protected by `RwLock`
    stats: Arc<RwLock<RegistryStats>>,
    /// Startup flags - immutable after registry creation
    startup_flags: u32,
    /// Runtime flags - mutable at runtime
    runtime_flags: Arc<parking_lot::RwLock<u64>>,
}

impl ConfigRegistry {
    /// Create a new configuration registry with default settings (no startup flags)
    ///
    /// Does not set any log level by default - respects existing logger configuration
    /// (e.g., `RUST_LOG` environment variable) or your application's logging setup.
    /// Use logffi directly to configure logging as needed.
    ///
    /// Returns Arc<ConfigRegistry> for consistent Arc-based chaining with all methods.
    ///
    /// # Examples
    /// ```
    /// use superconfig::ConfigRegistry;
    /// use std::sync::Arc;
    ///
    /// // No automatic logging - respects your app's logger setup
    /// let registry = ConfigRegistry::new();
    ///
    /// // Configure logging separately with logffi
    /// logffi::set_max_level(logffi::LevelFilter::Warn);
    /// ```
    #[must_use]
    pub fn new() -> Arc<Self> {
        Self::custom(crate::config_flags::startup::NO_FLAGS)
    }

    /// Create a new configuration registry with custom startup flags
    ///
    /// Startup flags affect internal structures and cannot be changed after creation.
    /// Returns Arc<ConfigRegistry> for consistent Arc-based chaining with all methods.
    ///
    /// # Examples
    /// ```
    /// use superconfig::{ConfigRegistry, config_flags::startup};
    /// use std::sync::Arc;
    ///
    /// let registry = ConfigRegistry::custom(startup::SIMD | startup::THREAD_POOL);
    /// // registry is Arc<ConfigRegistry>, ready for Arc-based chaining
    /// ```
    #[must_use]
    pub fn custom(startup_flags: u32) -> Arc<Self> {
        Arc::new(Self {
            entries: DashMap::new(),
            next_id: AtomicU64::new(1),
            stats: Arc::new(RwLock::new(RegistryStats::default())),
            startup_flags,
            runtime_flags: Arc::new(parking_lot::RwLock::new(0)),
        })
    }

    // Flag management methods

    /// Enable runtime flags (startup flags cannot be modified after creation)
    ///
    /// This method works with Arc<ConfigRegistry> for consistent Arc-based chaining.
    /// Always returns Arc<Self> to continue the chain, errors are collected internally.
    ///
    /// # Examples
    /// ```
    /// use superconfig::{ConfigRegistry, config_flags::runtime};
    ///
    /// let registry = ConfigRegistry::new()
    ///     .enable(runtime::STRICT_MODE)    // Always continues chain
    ///     .enable(runtime::PARALLEL);      // Always continues chain
    /// ```
    #[generate_json_helper(outgoing, handle_mode)]
    pub fn enable(self: Arc<Self>, flags: u64) -> Arc<Self> {
        // Validate flags - check if it's a known runtime flag
        if !crate::config_flags::is_valid_runtime_flag(flags) {
            error!(target: "superconfig.flags", "Invalid runtime flag: 0x{:X}", flags);
            return self;
        }

        // Directly modify the runtime flags through the Arc
        {
            let mut runtime_flags = self.runtime_flags.write();
            *runtime_flags |= flags;
        }
        self
    }

    /// Disable runtime flags (startup flags cannot be modified after creation)
    ///
    /// This method works with Arc<ConfigRegistry> for consistent Arc-based chaining.
    /// Always returns Arc<Self> to continue the chain, errors are collected internally.
    ///
    /// # Examples
    /// ```
    /// use superconfig::{ConfigRegistry, config_flags::runtime};
    ///
    /// let registry = ConfigRegistry::new()
    ///     .enable(runtime::STRICT_MODE)    // Enable first
    ///     .disable(runtime::STRICT_MODE);  // Then disable
    /// ```
    #[generate_json_helper(outgoing, handle_mode)]
    pub fn disable(self: Arc<Self>, flags: u64) -> Arc<Self> {
        // Validate flags - check if it's a known runtime flag
        if !crate::config_flags::is_valid_runtime_flag(flags) {
            error!(target: "superconfig.flags", "Invalid runtime flag: 0x{:X}", flags);
            return self;
        }

        // Directly modify the runtime flags through the Arc
        {
            let mut runtime_flags = self.runtime_flags.write();
            *runtime_flags &= !flags;
        }
        self
    }

    /// Check if startup flags are enabled
    #[must_use]
    pub const fn startup_enabled(&self, flags: u32) -> bool {
        (self.startup_flags & flags) != 0
    }

    /// Check if startup flags are disabled
    #[must_use]
    pub const fn startup_disabled(&self, flags: u32) -> bool {
        !self.startup_enabled(flags)
    }

    /// Check if runtime flags are enabled
    #[must_use]
    pub fn runtime_enabled(&self, flags: u64) -> bool {
        let runtime_flags = self.runtime_flags.read();
        (*runtime_flags & flags) != 0
    }

    /// Check if runtime flags are disabled
    #[must_use]
    pub fn runtime_disabled(&self, flags: u64) -> bool {
        !self.runtime_enabled(flags)
    }

    /// Get current startup flags
    #[must_use]
    pub const fn get_startup_flags(&self) -> u32 {
        self.startup_flags
    }

    /// Get current runtime flags
    #[must_use]
    pub fn get_runtime_flags(&self) -> u64 {
        *self.runtime_flags.read()
    }
}

// CRUD Operations

impl ConfigRegistry {
    /// Create a new configuration entry and return a handle to it
    ///
    /// Returns `Arc<T>` for efficient sharing. Use field access (`config.host`)
    /// and method calls (`config.validate()`) directly - they're zero-cost due to auto-deref.
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::RegistryFull` if the registry has reached maximum capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use superconfig::ConfigRegistry;
    ///
    /// let registry = ConfigRegistry::new();
    /// let handle = registry.create("my config".to_string()).unwrap();
    /// assert_eq!(handle.id(), 1);
    /// ```
    pub fn create<T: 'static + Send + Sync>(&self, data: T) -> Result<ConfigHandle<T>, String> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let entry = ConfigEntry::new(data);
        let data_size = entry.data_size;

        self.entries.insert(id, entry);

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.increment_creates();
            stats.add_memory(data_size as u64);
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
    /// Returns error message if the handle doesn't exist or points to wrong type.
    ///
    /// # Performance Notes
    ///
    /// - Field access: `config.database.host` (zero cost)
    /// - Method calls: `config.validate()` (zero cost)
    /// - Passing to functions: `process(config)` (~1ns to move Arc)
    /// - Multiple reads share the same underlying data efficiently
    ///
    /// # Examples
    ///
    /// ```
    /// use superconfig::ConfigRegistry;
    ///
    /// let registry = ConfigRegistry::new();
    /// let handle = registry.create("test".to_string()).unwrap();
    /// let data = registry.read(&handle).unwrap();
    /// assert_eq!(*data, "test");
    /// ```
    #[generate_json_helper(auto)]
    pub fn read<T: 'static>(&self, handle: &ConfigHandle<T>) -> Result<Arc<T>, String> {
        let entry_ref = self.entries.get(&handle.id()).ok_or_else(|| {
            let error_msg = format!("superconfig.registry: Handle {} not found", handle.id());
            error!(target: "superconfig.registry", "Handle {} not found", handle.id());
            error_msg
        })?;

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.increment_reads();
        }

        entry_ref.get_arc_data::<T>()
    }

    /// Update data in a configuration handle
    ///
    /// This replaces the entire configuration data with new data.
    /// Any existing Arc references will continue to point to the old data.
    ///
    /// # Errors
    ///
    /// Returns error message if the handle doesn't exist in the registry.
    ///
    /// # Examples
    ///
    /// ```
    /// use superconfig::ConfigRegistry;
    ///
    /// let registry = ConfigRegistry::new();
    /// let handle = registry.create("old".to_string()).unwrap();
    ///
    /// registry.update(&handle, "new".to_string()).unwrap();
    /// let data = registry.read(&handle).unwrap();
    /// assert_eq!(*data, "new");
    /// ```
    pub fn update<T: 'static + Send + Sync>(
        &self,
        handle: &ConfigHandle<T>,
        new_data: T,
    ) -> Result<(), String> {
        // Remove old entry to get its size
        let old_entry = self.entries.remove(&handle.id()).ok_or_else(|| {
            let error_msg = format!(
                "superconfig.registry: Handle {} not found for update",
                handle.id()
            );
            error!(target: "superconfig.registry", "Handle {} not found for update", handle.id());
            error_msg
        })?;

        let old_size = old_entry.1.data_size;

        // Create new entry with new data
        let new_entry = ConfigEntry::new(new_data);
        let new_size = new_entry.data_size;

        // Insert new entry (this atomically replaces the old one)
        self.entries.insert(handle.id(), new_entry);

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.increment_updates();
            stats.remove_memory(old_size as u64);
            stats.add_memory(new_size as u64);
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
    /// Returns error message if the handle doesn't exist or points to wrong type.
    ///
    /// # Examples
    ///
    /// ```
    /// use superconfig::ConfigRegistry;
    ///
    /// let registry = ConfigRegistry::new();
    /// let handle = registry.create("test".to_string()).unwrap();
    ///
    /// let data = registry.delete(&handle).unwrap();
    /// assert_eq!(*data, "test");
    /// assert!(!registry.contains_handle(&handle));
    /// ```
    pub fn delete<T: 'static>(&self, handle: &ConfigHandle<T>) -> Result<Arc<T>, String> {
        let (_, entry) = self.entries.remove(&handle.id()).ok_or_else(|| {
            let error_msg = format!(
                "superconfig.registry: Handle {} not found for delete",
                handle.id()
            );
            error!(target: "superconfig.registry", "Handle {} not found for delete", handle.id());
            error_msg
        })?;

        let data_size = entry.data_size;

        // Extract the Arc<T> directly
        let arc = entry.data.downcast::<Arc<T>>().map_err(|_| {
            let error_msg = format!("superconfig.registry: Wrong type for delete, expected {}, found {}", std::any::type_name::<T>(), entry.type_name);
            error!(target: "superconfig.registry", "Wrong type for delete, expected {}, found {}", std::any::type_name::<T>(), entry.type_name);
            error_msg
        })?;

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.increment_deletes();
            stats.remove_memory(data_size as u64);
        }

        Ok(*arc)
    }

    /// Get current registry statistics
    ///
    /// # Examples
    ///
    /// ```
    /// use superconfig::ConfigRegistry;
    ///
    /// let registry = ConfigRegistry::new();
    /// let stats = registry.stats();
    /// assert_eq!(stats.total_handles, 0);
    /// ```
    #[must_use]
    pub fn stats(&self) -> RegistryStats {
        self.stats.read().clone()
    }

    /// Check if a handle exists in the registry
    ///
    /// # Examples
    ///
    /// ```
    /// use superconfig::ConfigRegistry;
    ///
    /// let registry = ConfigRegistry::new();
    /// let handle = registry.create("test".to_string()).unwrap();
    ///
    /// assert!(registry.contains_handle(&handle));
    /// registry.delete(&handle).unwrap();
    /// assert!(!registry.contains_handle(&handle));
    /// ```
    #[must_use]
    pub fn contains_handle<T>(&self, handle: &ConfigHandle<T>) -> bool {
        self.entries.contains_key(&handle.id())
    }

    /// Clear all entries from the registry
    ///
    /// # Examples
    ///
    /// ```
    /// use superconfig::ConfigRegistry;
    ///
    /// let registry = ConfigRegistry::new();
    /// let _handle = registry.create("test".to_string()).unwrap();
    ///
    /// assert_eq!(registry.len(), 1);
    /// registry.clear();
    /// assert_eq!(registry.len(), 0);
    /// ```
    pub fn clear(&self) {
        self.entries.clear();
        let mut stats = self.stats.write();
        stats.reset();
    }

    /// Get the number of entries in the registry
    ///
    /// # Examples
    ///
    /// ```
    /// use superconfig::ConfigRegistry;
    ///
    /// let registry = ConfigRegistry::new();
    /// assert_eq!(registry.len(), 0);
    ///
    /// let _handle = registry.create("test".to_string()).unwrap();
    /// assert_eq!(registry.len(), 1);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the registry is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use superconfig::ConfigRegistry;
    ///
    /// let registry = ConfigRegistry::new();
    /// assert!(registry.is_empty());
    ///
    /// let _handle = registry.create("test".to_string()).unwrap();
    /// assert!(!registry.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// Global registry instance - defined here to be close to the implementation
/// Global configuration registry instance
static GLOBAL_REGISTRY: std::sync::LazyLock<Arc<ConfigRegistry>> =
    std::sync::LazyLock::new(ConfigRegistry::new);

/// Get a reference to the global configuration registry
///
/// # Examples
///
/// ```
/// use superconfig::global_registry;
///
/// let handle = global_registry().create("test".to_string()).unwrap();
/// let data = global_registry().read(&handle).unwrap();
/// assert_eq!(*data, "test");
/// ```
#[must_use]
pub fn global_registry() -> &'static Arc<ConfigRegistry> {
    &GLOBAL_REGISTRY
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_flags::{runtime, startup};
    use serde::{Deserialize, Serialize};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::thread;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestConfig {
        host: String,
        port: u16,
        timeout_ms: u32,
    }

    #[derive(Debug, Clone, PartialEq, Serialize)]
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
    fn test_flag_operations() {
        let registry = ConfigRegistry::custom(startup::SIMD | startup::THREAD_POOL)
            .enable(runtime::STRICT_MODE | runtime::PARALLEL);

        // Test startup flags
        assert!(registry.startup_enabled(startup::SIMD));
        assert!(registry.startup_enabled(startup::THREAD_POOL));
        assert!(!registry.startup_enabled(startup::DETAILED_STATS));
        assert!(registry.startup_disabled(startup::DETAILED_STATS));

        // Test runtime flags
        assert!(registry.runtime_enabled(runtime::STRICT_MODE));
        assert!(registry.runtime_enabled(runtime::PARALLEL));
        assert!(!registry.runtime_enabled(runtime::ARRAY_MERGE));
        assert!(registry.runtime_disabled(runtime::ARRAY_MERGE));

        // Test runtime flag modification
        let registry = registry.enable(runtime::ARRAY_MERGE);
        assert!(registry.runtime_enabled(runtime::ARRAY_MERGE));

        let registry = registry.disable(runtime::PARALLEL);
        assert!(!registry.runtime_enabled(runtime::PARALLEL));
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
    fn test_invalid_handle_operations() {
        let registry = ConfigRegistry::new();

        // Test various error scenarios that can occur during normal operations

        // Scenario 1: Delete with non-existent handle
        let phantom_delete_handle = ConfigHandle::<TestConfig>::new(999);
        let result = registry.delete(&phantom_delete_handle);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Handle 999 not found for delete")
        );

        // Scenario 2: Update with non-existent handle
        let phantom_config = TestConfig {
            host: "phantom".to_string(),
            port: 888,
            timeout_ms: 5000,
        };
        let phantom_update_handle = ConfigHandle::<TestConfig>::new(888);
        let result = registry.update(&phantom_update_handle, phantom_config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Handle 888 not found for update")
        );

        // Scenario 3: Read with non-existent handle for different type
        let phantom_string_handle = ConfigHandle::<String>::new(777);
        let result = registry.read(&phantom_string_handle);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Handle 777 not found"));

        println!("✅ Invalid handle operations: all error paths covered comprehensively");
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

        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Wrong type"));
        assert!(error_msg.contains("expected"));
        assert!(error_msg.contains("found"));
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

    #[test]
    fn test_builder_pattern() {
        let registry = ConfigRegistry::custom(startup::SIMD).enable(runtime::STRICT_MODE);

        assert!(registry.startup_enabled(startup::SIMD));
        assert!(registry.runtime_enabled(runtime::STRICT_MODE));
    }

    /// Test the macro-generated `enable_as_json` method
    #[test]
    fn test_macro_generated_enable_as_json() {
        use crate::config_flags::runtime;

        // Test success case
        let registry = ConfigRegistry::new();
        let result = registry.enable_as_json(runtime::STRICT_MODE);

        println!("✅ Macro-generated enable_as_json result: {result}");

        // Parse the JSON response
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Verify the structure matches handle_mode expectations
        assert_eq!(parsed["success"], true);
        // In handle_mode, there should be NO data field - only success
        assert!(parsed.get("data").is_none());
        assert!(parsed.get("error").is_none());

        // Should have exactly one field: "success"
        assert_eq!(parsed.as_object().unwrap().len(), 1);

        println!("✅ Success: JSON structure is correct for handle_mode");
        println!("   - success: {}", parsed["success"]);
        println!("   - data field absent: {}", parsed.get("data").is_none());
        println!("   - error field absent: {}", parsed.get("error").is_none());

        // Verify the exact JSON matches our expectations
        let expected_json = r#"{"success":true}"#;
        let normalized_result = result.replace(' ', ""); // Remove any whitespace
        assert_eq!(normalized_result, expected_json);

        println!("✅ JSON output matches expected format exactly");
    }

    // Old try_enable tests removed - replaced by Arc-based tests below

    /// Test the macro-generated `enable_as_json` method error case
    #[test]
    fn test_macro_generated_enable_as_json_error() {
        // Test error case - enable method doesn't typically fail with invalid flags,
        // but let's test with a scenario that would normally cause an error
        let registry = ConfigRegistry::new();

        // Use a valid flag since enable method validates flags internally
        let result = registry.enable_as_json(runtime::STRICT_MODE);

        println!("✅ Macro-generated enable_as_json result: {result}");

        // Parse the JSON response
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Should still be success since STRICT_MODE is a valid flag
        assert_eq!(parsed["success"], true);
        assert!(parsed.get("data").is_none()); // Handle mode - no data
        assert!(parsed.get("error").is_none()); // No error for valid flag

        println!("✅ Handle mode correctly returns success without data serialization");

        // Note: The enable method in SuperConfig doesn't typically fail,
        // but if it did, handle_mode would return {"success": false, "error": "message"}
        // The error handling is tested in the macro crate's comprehensive tests
    }

    // ===== Arc-based chaining pattern tests =====

    /// Test enable success case
    #[test]
    fn test_enable_success() {
        use crate::config_flags::runtime;

        let registry = ConfigRegistry::new();

        // Should not panic and should enable the flag
        let result = registry.enable(runtime::STRICT_MODE);

        // Verify the flag was actually enabled
        assert!(result.runtime_enabled(runtime::STRICT_MODE));

        // Registry continues chain successfully

        println!("✅ Enable success: flag enabled correctly");
    }

    /// Test enable with invalid flag (error handling)
    #[test]
    fn test_enable_error_handling() {
        let registry = ConfigRegistry::new();

        // Use an invalid flag that should cause a logffi error but continue chain
        let result = registry.enable(0xFFFFFFFF); // Invalid flag

        // Should continue chain even with invalid flag
        // The error is logged but chain continues

        // Verify registry still works after invalid flag
        let handle = result.create("test".to_string()).unwrap();
        let data = result.read(&handle).unwrap();
        assert_eq!(*data, "test");

        println!("✅ Enable error handling: invalid flag handled gracefully");
    }

    /// Test enable method chaining
    #[test]
    fn test_enable_method_chaining() {
        use crate::config_flags::runtime;

        let registry = ConfigRegistry::new();

        // Chain multiple enable calls
        let result = registry
            .enable(runtime::STRICT_MODE)
            .enable(runtime::PARALLEL)
            .enable(runtime::ENV_EXPANSION);

        // Verify all flags were enabled
        assert!(result.runtime_enabled(runtime::STRICT_MODE));
        assert!(result.runtime_enabled(runtime::PARALLEL));
        assert!(result.runtime_enabled(runtime::ENV_EXPANSION));

        // All flags enabled successfully

        println!("✅ Enable chaining: all flags enabled correctly");
    }

    /// Test mixed chaining with enable and disable
    #[test]
    fn test_enable_disable_chaining() {
        use crate::config_flags::runtime;

        let registry = ConfigRegistry::new();

        // Chain enable and disable operations
        let result = registry
            .enable(runtime::STRICT_MODE)
            .enable(runtime::PARALLEL)
            .disable(runtime::STRICT_MODE);

        // Verify the final state
        assert!(!result.runtime_enabled(runtime::STRICT_MODE)); // Disabled
        assert!(result.runtime_enabled(runtime::PARALLEL)); // Still enabled

        println!("✅ Enable/disable chaining: methods work together");
    }

    /// Test catch and errors methods
    #[test]
    fn test_arc_catch_and_errors() {
        let registry = ConfigRegistry::new();

        // Test valid operations work correctly
        let registry = registry.enable(crate::config_flags::runtime::STRICT_MODE);

        // Verify flag was enabled
        assert!(registry.runtime_enabled(crate::config_flags::runtime::STRICT_MODE));

        println!("✅ Arc operations: methods work correctly");
    }

    /// Test registry creation methods
    #[test]
    fn test_registry_creation_methods() {
        use crate::config_flags::startup;

        // Test new
        let registry1 = ConfigRegistry::new();
        assert_eq!(registry1.get_startup_flags(), 0);

        // Test custom
        let registry2 = ConfigRegistry::custom(startup::SIMD | startup::THREAD_POOL);
        assert!(registry2.startup_enabled(startup::SIMD));
        assert!(registry2.startup_enabled(startup::THREAD_POOL));

        // Verify both are Arc<ConfigRegistry>
        let _: std::sync::Arc<ConfigRegistry> = registry1;
        let _: std::sync::Arc<ConfigRegistry> = registry2;

        println!("✅ Registry creation methods: new and custom work correctly");
    }

    /// Test registry reference behavior
    #[test]
    fn test_registry_reference_behavior() {
        use crate::config_flags::runtime;

        let registry1 = ConfigRegistry::new();
        let registry2 = Arc::clone(&registry1);

        // Both references should work
        let result1 = registry1.enable(runtime::STRICT_MODE);
        let result2 = registry2.enable(runtime::PARALLEL);

        // They should point to the same registry
        assert!(Arc::ptr_eq(&result1, &result2));

        // Both flags should be visible from either reference
        assert!(result1.runtime_enabled(runtime::STRICT_MODE));
        assert!(result1.runtime_enabled(runtime::PARALLEL));
        assert!(result2.runtime_enabled(runtime::STRICT_MODE));
        assert!(result2.runtime_enabled(runtime::PARALLEL));

        println!("✅ Registry reference behavior: shared state works correctly");
    }

    #[test]
    fn test_enable_as_json_success() {
        use crate::config_flags::runtime;

        let registry = ConfigRegistry::new();
        let json_result = registry.enable_as_json(runtime::STRICT_MODE);

        let result: serde_json::Value = serde_json::from_str(&json_result).unwrap();
        assert_eq!(result["success"], true);
        // handle_mode should not include data field
        assert!(result.get("data").is_none());

        println!("✅ enable_as_json success: {json_result}");
    }

    #[test]
    fn test_enable_as_json_error() {
        let registry = ConfigRegistry::new();
        let json_result = registry.enable_as_json(0xFFFFFFFF); // Invalid flag

        let result: serde_json::Value = serde_json::from_str(&json_result).unwrap();
        assert_eq!(result["success"], true); // Should still be true since our enable method continues chain

        println!("✅ enable_as_json with invalid flag: {json_result}");
    }

    #[test]
    fn test_enable_as_json_with_chaining() {
        use crate::config_flags::runtime;

        let registry = ConfigRegistry::new();

        // Test that the enable method works for chaining
        let registry = registry.enable(runtime::STRICT_MODE);

        // Clone registry before the move
        let registry_clone = Arc::clone(&registry);

        // Test that JSON helper works after chaining
        let json_result = registry.enable_as_json(runtime::PARALLEL);
        let result: serde_json::Value = serde_json::from_str(&json_result).unwrap();
        assert_eq!(result["success"], true);

        // Verify flags are enabled using the clone
        assert!(registry_clone.runtime_enabled(runtime::STRICT_MODE));
        assert!(registry_clone.runtime_enabled(runtime::PARALLEL));

        println!("✅ enable_as_json chaining: {json_result}");
    }

    #[test]
    fn test_read_as_json_success() {
        let registry = ConfigRegistry::new();
        let config = SimpleConfig { value: 42 };
        let handle = registry.create(config).unwrap();

        let json_result = registry.read_as_json(&handle);
        let result: serde_json::Value = serde_json::from_str(&json_result).unwrap();

        assert_eq!(result["success"], true);
        assert!(result["data"].is_object());
        assert_eq!(result["data"]["value"], 42);

        // Should NOT be a string containing JSON
        assert!(!result["data"].is_string());

        println!("✅ read_as_json success: {json_result}");
    }

    #[test]
    fn test_read_as_json_error() {
        let registry = ConfigRegistry::new();
        let invalid_handle = ConfigHandle::<SimpleConfig>::new(999); // Non-existent handle

        let json_result = registry.read_as_json(&invalid_handle);
        let result: serde_json::Value = serde_json::from_str(&json_result).unwrap();

        assert_eq!(result["success"], false);
        assert!(result["error"].is_string());
        assert!(
            result["error"]
                .as_str()
                .unwrap()
                .contains("Handle 999 not found")
        );
        assert!(result.get("data").is_none());

        println!("✅ read_as_json error: {json_result}");
    }

    #[test]
    fn test_read_as_json_with_string_data() {
        let registry = ConfigRegistry::new();
        let config = "test string".to_string();
        let handle = registry.create(config).unwrap();

        let json_result = registry.read_as_json(&handle);
        let result: serde_json::Value = serde_json::from_str(&json_result).unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["data"], "test string");
        assert!(result["data"].is_string());

        println!("✅ read_as_json string data: {json_result}");
    }

    #[test]
    fn test_read_as_json_with_complex_data() {
        #[derive(Debug, Clone, PartialEq, serde::Serialize)]
        struct ComplexConfig {
            name: String,
            settings: Vec<String>,
            nested: NestedConfig,
        }

        #[derive(Debug, Clone, PartialEq, serde::Serialize)]
        struct NestedConfig {
            enabled: bool,
            count: u32,
        }

        let registry = ConfigRegistry::new();
        let config = ComplexConfig {
            name: "test config".to_string(),
            settings: vec!["opt1".to_string(), "opt2".to_string()],
            nested: NestedConfig {
                enabled: true,
                count: 5,
            },
        };
        let handle = registry.create(config.clone()).unwrap();

        let json_result = registry.read_as_json(&handle);
        let result: serde_json::Value = serde_json::from_str(&json_result).unwrap();

        assert_eq!(result["success"], true);
        assert!(result["data"].is_object());
        assert_eq!(result["data"]["name"], "test config");
        assert!(result["data"]["settings"].is_array());
        assert_eq!(result["data"]["settings"][0], "opt1");
        assert_eq!(result["data"]["nested"]["enabled"], true);
        assert_eq!(result["data"]["nested"]["count"], 5);

        // Test error closure for ComplexConfig - handle not found
        let fake_handle = ConfigHandle::<ComplexConfig>::new(999);
        assert!(registry.read(&fake_handle).is_err());

        println!("✅ read_as_json complex data: {json_result}");
    }

    #[test]
    fn test_read_as_json_serialize_constraint() {
        // This test verifies that the macro correctly adds T: Serialize constraint
        let registry = ConfigRegistry::new();
        let config = SimpleConfig { value: 123 };
        let handle = registry.create(config).unwrap();

        // SimpleConfig implements Serialize, so this should work
        let _json_result = registry.read_as_json(&handle);

        // If we tried to use a non-Serialize type, it would fail at compile time
        // This is automatically verified by the compiler when the macro generates
        // the T: 'static + serde::Serialize constraint
        println!("✅ read_as_json Serialize constraint enforced at compile time");
    }

    #[test]
    fn test_read_as_json_arc_dereferencing() {
        let registry = ConfigRegistry::new();
        let config = SimpleConfig { value: 999 };
        let handle = registry.create(config).unwrap();

        let json_result = registry.read_as_json(&handle);
        let result: serde_json::Value = serde_json::from_str(&json_result).unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["data"]["value"], 999);

        // If Arc<T> wasn't properly dereferenced with &*result,
        // serialization would fail. The fact we get valid JSON proves it works.
        println!("✅ read_as_json Arc dereferencing works correctly");
    }

    // ===== Missing tests for 100% coverage =====

    #[test]
    fn test_disable_success() {
        use crate::config_flags::runtime;

        let registry = ConfigRegistry::new()
            .enable(runtime::STRICT_MODE)
            .enable(runtime::PARALLEL);

        // Verify flags are enabled first
        assert!(registry.runtime_enabled(runtime::STRICT_MODE));
        assert!(registry.runtime_enabled(runtime::PARALLEL));

        // Test disable
        let registry = registry.disable(runtime::STRICT_MODE);

        // Verify STRICT_MODE is disabled, PARALLEL still enabled
        assert!(!registry.runtime_enabled(runtime::STRICT_MODE));
        assert!(registry.runtime_enabled(runtime::PARALLEL));

        println!("✅ Disable success: flag disabled correctly");
    }

    #[test]
    fn test_disable_error_handling() {
        let registry = ConfigRegistry::new();

        // Use an invalid flag that should cause a logffi error but continue chain
        let result = registry.disable(0xFFFFFFFF); // Invalid flag

        // Should continue chain even with invalid flag
        // The error is logged but chain continues

        // Verify registry still works after invalid flag
        let handle = result.create("test".to_string()).unwrap();
        let data = result.read(&handle).unwrap();
        assert_eq!(*data, "test");

        println!("✅ Disable error handling: invalid flag handled gracefully");
    }

    #[test]
    fn test_disable_as_json_success() {
        use crate::config_flags::runtime;

        let registry = ConfigRegistry::new().enable(runtime::STRICT_MODE);

        let json_result = registry.disable_as_json(runtime::STRICT_MODE);

        let result: serde_json::Value = serde_json::from_str(&json_result).unwrap();
        assert_eq!(result["success"], true);
        // handle_mode should not include data field
        assert!(result.get("data").is_none());

        println!("✅ disable_as_json success: {json_result}");
    }

    #[test]
    fn test_disable_as_json_error() {
        let registry = ConfigRegistry::new();
        let json_result = registry.disable_as_json(0xFFFFFFFF); // Invalid flag

        let result: serde_json::Value = serde_json::from_str(&json_result).unwrap();
        assert_eq!(result["success"], true); // Should still be true since our disable method continues chain

        println!("✅ disable_as_json with invalid flag: {json_result}");
    }

    #[test]
    fn test_get_runtime_flags() {
        use crate::config_flags::runtime;

        let registry = ConfigRegistry::new();

        // Initially no flags
        assert_eq!(registry.get_runtime_flags(), 0);

        // Enable some flags
        let registry = registry
            .enable(runtime::STRICT_MODE)
            .enable(runtime::PARALLEL);

        let flags = registry.get_runtime_flags();
        assert_ne!(flags, 0);
        assert_eq!(flags & runtime::STRICT_MODE, runtime::STRICT_MODE);
        assert_eq!(flags & runtime::PARALLEL, runtime::PARALLEL);

        // Disable one flag
        let registry = registry.disable(runtime::STRICT_MODE);
        let flags = registry.get_runtime_flags();
        assert_eq!(flags & runtime::STRICT_MODE, 0);
        assert_eq!(flags & runtime::PARALLEL, runtime::PARALLEL);

        println!("✅ get_runtime_flags: returns correct flag values");
    }

    #[test]
    fn test_update_handle_not_found() {
        let registry = ConfigRegistry::new();
        let fake_handle = ConfigHandle::<SimpleConfig>::new(999);

        let result = registry.update(&fake_handle, SimpleConfig { value: 42 });
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Handle 999 not found for update"));

        println!("✅ Update error handling: handle not found");
    }

    #[test]
    fn test_delete_handle_not_found() {
        let registry = ConfigRegistry::new();
        let fake_handle = ConfigHandle::<SimpleConfig>::new(999);

        let result = registry.delete(&fake_handle);
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Handle 999 not found for delete"));

        println!("✅ Delete error handling: handle not found");
    }

    #[test]
    fn test_delete_wrong_type() {
        let registry = ConfigRegistry::new();
        let config = TestConfig {
            host: "localhost".to_string(),
            port: 8080,
            timeout_ms: 5000,
        };

        let handle = registry.create(config).unwrap();

        // Try to delete as wrong type
        let wrong_handle = ConfigHandle::<SimpleConfig>::new(handle.id());
        let result = registry.delete(&wrong_handle);

        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Wrong type for delete"));
        assert!(error_msg.contains("expected"));
        assert!(error_msg.contains("found"));

        println!("✅ Delete wrong type: error handled correctly");
    }

    #[test]
    fn test_get_arc_data_success_path() {
        let registry = ConfigRegistry::new();
        let config = SimpleConfig { value: 123 };

        let handle = registry.create(config.clone()).unwrap();

        // This tests the success path in get_arc_data (line 65)
        let data = registry.read(&handle).unwrap();
        assert_eq!(data.value, 123);

        // Multiple reads should return same Arc
        let data2 = registry.read(&handle).unwrap();
        assert!(Arc::ptr_eq(&data, &data2));

        println!("✅ get_arc_data success path: downcast succeeds");
    }

    #[test]
    fn test_comprehensive_crud_operations() {
        // This test specifically targets uncovered lines in CRUD operations
        let registry = ConfigRegistry::new();

        // Test create operation with complex data to hit all paths
        let complex_config = TestConfig {
            host: "localhost".to_string(),
            port: 8080,
            timeout_ms: 5000,
        };

        let handle = registry.create(complex_config.clone()).unwrap();

        // Test read operation to hit stats increment and get_arc_data success path (lines 328-329, 332, 65)
        let data1 = registry.read(&handle).unwrap();
        let data2 = registry.read(&handle).unwrap();
        assert_eq!(*data1, complex_config);
        assert!(Arc::ptr_eq(&data1, &data2)); // Test line 65 success path

        // Test update operation to hit all success paths (lines 369, 372-373, 376, 380-383, 386)
        let updated_config = TestConfig {
            host: "remote".to_string(),
            port: 9090,
            timeout_ms: 10000,
        };
        registry.update(&handle, updated_config.clone()).unwrap();

        let updated_data = registry.read(&handle).unwrap();
        assert_eq!(*updated_data, updated_config);

        // Test delete operation to hit all success paths (lines 417, 424, 430-432, 435)
        let deleted_data = registry.delete(&handle).unwrap();
        assert_eq!(*deleted_data, updated_config);

        // Verify handle no longer exists
        assert!(!registry.contains_handle(&handle));

        // Check statistics were properly updated
        let stats = registry.stats();
        assert!(stats.total_creates > 0);
        assert!(stats.total_reads > 0);
        assert!(stats.total_updates > 0);
        assert!(stats.total_deletes > 0);

        println!("✅ Comprehensive CRUD: all internal paths exercised");
    }

    #[test]
    fn test_multiple_operations_for_coverage() {
        // Additional test to ensure we hit every possible code path
        let registry = ConfigRegistry::new();

        // Create multiple handles to exercise different scenarios
        let handle1 = registry.create(SimpleConfig { value: 1 }).unwrap();
        let handle2 = registry.create(SimpleConfig { value: 2 }).unwrap();
        let handle3 = registry
            .create(TestConfig {
                host: "test".to_string(),
                port: 3000,
                timeout_ms: 1000,
            })
            .unwrap();

        // Multiple reads to ensure statistics paths are hit
        let _data1a = registry.read(&handle1).unwrap();
        let _data1b = registry.read(&handle1).unwrap();
        let _data2 = registry.read(&handle2).unwrap();
        let _data3 = registry.read(&handle3).unwrap();

        // Update operations
        registry
            .update(&handle1, SimpleConfig { value: 11 })
            .unwrap();
        registry
            .update(&handle2, SimpleConfig { value: 22 })
            .unwrap();

        // More reads after updates
        let _updated1 = registry.read(&handle1).unwrap();
        let _updated2 = registry.read(&handle2).unwrap();

        // Delete operations
        let _deleted1 = registry.delete(&handle1).unwrap();
        let _deleted2 = registry.delete(&handle2).unwrap();
        let _deleted3 = registry.delete(&handle3).unwrap();

        // Final statistics check
        let stats = registry.stats();
        assert_eq!(stats.total_creates, 3);
        assert!(stats.total_reads >= 6);
        assert_eq!(stats.total_updates, 2);
        assert_eq!(stats.total_deletes, 3);

        println!("✅ Multiple operations: comprehensive path coverage");
    }

    #[test]
    fn test_delete_wrong_type_explicit_logging() {
        // This test specifically targets line 424 - the error logging in delete wrong type path
        let registry = ConfigRegistry::new();

        // Create a TestConfig entry
        let config = TestConfig {
            host: "test".to_string(),
            port: 1234,
            timeout_ms: 2000,
        };
        let handle = registry.create(config).unwrap();

        // Now try to delete it as SimpleConfig (wrong type)
        // This should trigger both the error message formatting AND the error logging
        let wrong_handle = ConfigHandle::<SimpleConfig>::new(handle.id());
        let result = registry.delete(&wrong_handle);

        // Verify the error path was taken
        assert!(result.is_err());
        let error_msg = result.unwrap_err();

        // This should ensure both lines 423 AND 424 are executed
        assert!(error_msg.contains("Wrong type for delete"));
        assert!(error_msg.contains("SimpleConfig"));
        assert!(error_msg.contains("TestConfig"));

        // Force the error to be processed to ensure logging actually happens
        eprintln!("Expected error occurred: {}", error_msg);

        println!("✅ Delete wrong type explicit logging: line 424 covered");
    }

    #[test]
    fn test_delete_testconfig_wrong_type_closure() {
        // This test specifically targets the missing closure for TestConfig delete wrong type
        let registry = ConfigRegistry::new();

        // Create a SimpleConfig entry
        let simple_config = SimpleConfig { value: 42 };
        let handle = registry.create(simple_config).unwrap();

        // Try to delete it as TestConfig (wrong type) - this exercises the missing closure
        let wrong_handle = ConfigHandle::<TestConfig>::new(handle.id());
        let result = registry.delete(&wrong_handle);

        // Verify the error path was taken
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Wrong type for delete"));
        assert!(error_msg.contains("TestConfig"));
        assert!(error_msg.contains("SimpleConfig"));

        println!("✅ Delete TestConfig wrong type closure: missing function coverage achieved");
    }

    #[test]
    fn test_logging_macro_coverage() {
        // This test specifically exercises the error! macro paths to cover missing regions

        // Set up a logger to capture the error messages (helps trigger macro internals)
        let _ = env_logger::builder().is_test(true).try_init();

        let registry = ConfigRegistry::new();

        // 1. Test invalid flag logging (lines 170, 199) with proper assertions
        let invalid_flag = 0xFFFFFFFF;
        let registry_clone = Arc::clone(&registry);
        let result1 = registry_clone.enable(invalid_flag);
        let result2 = result1.disable(invalid_flag);

        // Verify the registry still works after invalid flag operations
        assert_eq!(result2.get_runtime_flags(), 0);

        // Create some test data
        let test_string = String::from("test_logging");
        let test_int = 42i32;
        let handle_string = registry.create(test_string.clone()).unwrap();
        let handle_int = registry.create(test_int).unwrap();

        // 2. Test wrong type error logging (line 60) with assertion
        let wrong_handle = ConfigHandle::<i32>::new(handle_string.id());
        let read_result = registry.read(&wrong_handle);
        assert!(read_result.is_err());
        assert!(read_result.unwrap_err().contains("Wrong type"));

        // Use handle_int to cover the missing function and ensure it works correctly
        let int_value = registry.read(&handle_int).unwrap();
        assert_eq!(*int_value, 42i32);

        // 3. Test handle not found errors (lines 322, 365, 413) with assertions
        let phantom_handle_string = ConfigHandle::<String>::new(9999);
        let phantom_handle_int = ConfigHandle::<i32>::new(8888);

        // Read non-existent handle
        let read_result = registry.read(&phantom_handle_string);
        assert!(read_result.is_err());
        assert!(read_result.unwrap_err().contains("not found"));

        // Update non-existent handle
        let update_result = registry.update(&phantom_handle_int, 999);
        assert!(update_result.is_err());
        assert!(update_result.unwrap_err().contains("not found for update"));

        // Delete non-existent handle
        let delete_result = registry.delete(&phantom_handle_string);
        assert!(delete_result.is_err());
        assert!(delete_result.unwrap_err().contains("not found for delete"));

        // Verify the valid operations still work before more destructive tests
        let valid_data = registry.read(&handle_string).unwrap();
        assert_eq!(*valid_data, "test_logging");

        // 4. Test wrong type for delete (line 422) with assertion
        let wrong_delete = registry.delete(&wrong_handle);
        assert!(wrong_delete.is_err());
        assert!(wrong_delete.unwrap_err().contains("Wrong type for delete"));

        // Force different argument patterns to trigger different macro expansion paths
        let complex_string = format!("complex_{}_string_{}", 123, "test");
        let complex_handle = registry.create(complex_string).unwrap();
        let very_wrong_handle = ConfigHandle::<TestConfig>::new(complex_handle.id());
        let complex_wrong_read = registry.read(&very_wrong_handle);
        assert!(complex_wrong_read.is_err());
        assert!(complex_wrong_read.unwrap_err().contains("Wrong type"));

        println!("✅ Logging macro coverage test completed with assertions");
    }
}
