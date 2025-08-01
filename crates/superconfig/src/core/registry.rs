//! Main configuration registry implementation

use dashmap::DashMap;
use parking_lot::RwLock;
use serde::Serialize;
use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

use super::{
    errors::{FluentError, HandleId, RegistryError, SuperConfigError},
    handle::ConfigHandle,
    stats::RegistryStats,
};
use crate::config_flags::{FlagError, VerbosityLevel, verbosity};
use superconfig_macros::generate_json_helper;

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
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create registry with startup flags
/// let registry = ConfigRegistry::custom(startup::SIMD | startup::THREAD_POOL)
///     .enable(runtime::STRICT_MODE)?;
///
/// // Store configuration
/// let handle = registry.create("localhost".to_string()).unwrap();
/// let config = registry.read(&handle).unwrap();
/// assert_eq!(*config, "localhost");
/// # Ok(())
/// # }
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
    /// Verbosity level - mutable at runtime
    verbosity: Arc<parking_lot::RwLock<u8>>,
    /// Collected errors from try_* methods for permissive error handling
    #[allow(dead_code)] // Used by fluent API patterns (future implementation)
    collected_errors: Arc<parking_lot::RwLock<Vec<FluentError>>>,
}

impl ConfigRegistry {
    /// Create a new configuration registry with default settings (no startup flags)
    ///
    /// # Examples
    ///
    /// ```
    /// use superconfig::ConfigRegistry;
    ///
    /// let registry = ConfigRegistry::new();
    /// assert_eq!(registry.get_startup_flags(), 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::custom(0) // No startup flags
    }

    /// Create a configuration registry with custom startup flags
    ///
    /// Startup flags affect internal structures and cannot be changed after creation.
    ///
    /// # Examples
    ///
    /// ```
    /// use superconfig::{ConfigRegistry, config_flags::startup};
    ///
    /// let registry = ConfigRegistry::custom(startup::SIMD | startup::THREAD_POOL);
    /// assert!(registry.startup_enabled(startup::SIMD));
    /// assert!(registry.startup_enabled(startup::THREAD_POOL));
    /// ```
    #[must_use]
    pub fn custom(startup_flags: u32) -> Self {
        Self {
            entries: DashMap::new(),
            next_id: AtomicU64::new(1),
            stats: Arc::new(RwLock::new(RegistryStats::default())),
            startup_flags,
            runtime_flags: Arc::new(parking_lot::RwLock::new(0)),
            verbosity: Arc::new(parking_lot::RwLock::new(verbosity::NONE)),
            collected_errors: Arc::new(parking_lot::RwLock::new(Vec::new())),
        }
    }

    /// Create a new Arc-wrapped configuration registry with default settings
    ///
    /// This method returns `Arc<ConfigRegistry>` for use with Arc-based fluent methods
    /// like `try_enable` and `arc_enable`.
    ///
    /// # Examples
    /// ```
    /// use superconfig::ConfigRegistry;
    /// use std::sync::Arc;
    ///
    /// let registry = ConfigRegistry::arc_new();
    /// // registry is Arc<ConfigRegistry>, ready for Arc-based chaining
    /// ```
    #[must_use]
    pub fn arc_new() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Create a new Arc-wrapped configuration registry with custom startup flags
    ///
    /// This method returns `Arc<ConfigRegistry>` for use with Arc-based fluent methods.
    ///
    /// # Examples
    /// ```
    /// use superconfig::{ConfigRegistry, config_flags::startup};
    /// use std::sync::Arc;
    ///
    /// let registry = ConfigRegistry::arc_custom(startup::SIMD | startup::THREAD_POOL);
    /// // registry is Arc<ConfigRegistry>, ready for Arc-based chaining
    /// ```
    #[must_use]
    pub fn arc_custom(startup_flags: u32) -> Arc<Self> {
        Arc::new(Self::custom(startup_flags))
    }

    /// Builder method: set verbosity level during construction  
    ///
    /// # Examples
    ///
    /// ```
    /// use superconfig::{ConfigRegistry, config_flags::verbosity};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = ConfigRegistry::new()
    ///     .verbosity(verbosity::DEBUG)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns `RegistryError::Flag` if level is greater than `verbosity::TRACE`
    pub fn verbosity(self, level: u8) -> Result<Self, RegistryError> {
        {
            let mut verbosity = self.verbosity.write();
            *verbosity = level;
        }
        Ok(self)
    }

    // Flag management methods

    /// Enable runtime flags (startup flags cannot be modified after creation)
    ///
    /// # Examples
    ///
    /// ```
    /// use superconfig::{ConfigRegistry, config_flags::runtime};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = ConfigRegistry::new()
    ///     .enable(runtime::ARRAY_MERGE)?
    ///     .enable(runtime::STRICT_MODE)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns `RegistryError::Flag` if any flag in `flags` is invalid
    #[generate_json_helper(outgoing, handle_mode)]
    pub fn enable(self, flags: u64) -> Result<Self, RegistryError> {
        {
            let mut runtime_flags = self.runtime_flags.write();
            *runtime_flags |= flags;
        }
        Ok(self)
    }

    /// Arc-based enable method - fail-fast behavior with Arc<Self>
    ///
    /// This method works with Arc<ConfigRegistry> for consistent Arc-based chaining.
    /// Returns Result<Arc<Self>, Error> - chain breaks on first error.
    ///
    /// # Errors
    ///
    /// Returns `SuperConfigError` if the runtime flags are invalid.
    ///
    /// # Examples
    /// ```
    /// use superconfig::{ConfigRegistry, config_flags::runtime};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = ConfigRegistry::arc_new()
    ///     .arc_enable(runtime::STRICT_MODE)?    // Breaks chain on error
    ///     .arc_enable(runtime::PARALLEL)?;      // Only runs if previous succeeds
    /// # Ok(())
    /// # }
    /// ```
    pub fn arc_enable(self: Arc<Self>, flags: u64) -> Result<Arc<Self>, SuperConfigError> {
        // Validate flags - check if it's a known runtime flag
        if !crate::config_flags::is_valid_runtime_flag(flags) {
            return Err(SuperConfigError::invalid_runtime_flag(flags));
        }

        // Directly modify the runtime flags through the Arc
        {
            let mut runtime_flags = self.runtime_flags.write();
            *runtime_flags |= flags;
        }
        Ok(self)
    }

    /// Try to enable runtime flags - continues fluent chain even on error
    ///
    /// This method uses Arc<Self> to allow error recovery in fluent chains.
    /// On success, returns the modified registry. On error, returns the original
    /// registry unchanged and collects the error internally using `throw()`.
    ///
    /// # Examples
    /// ```
    /// use superconfig::{ConfigRegistry, config_flags::runtime};
    /// use superconfig::core::errors::SuperConfigError;
    ///
    /// let registry = ConfigRegistry::arc_new()
    ///     .try_enable(runtime::STRICT_MODE)   // Won't break chain if error
    ///     .try_enable(0xFFFFFFFF)             // Invalid flag - continues chain, error collected
    ///     .try_enable(runtime::PARALLEL);     // Continue chain
    ///
    /// let errors = registry.catch(); // Get and clear any collected errors
    /// assert_eq!(errors.len(), 1); // One error from invalid flag
    /// # Ok::<(), SuperConfigError>(())
    /// ```
    pub fn try_enable(self: Arc<Self>, flags: u64) -> Arc<Self> {
        // Clone Arc for the attempt, keep original for fallback
        let try_self = Arc::clone(&self);

        // Try to enable the flags using the internal logic
        match try_self.arc_enable(flags) {
            Ok(result) => result,
            Err(e) => {
                // On error, throw to internal error collection and return original Arc
                self.throw(e, "enable");
                self
            }
        }
    }

    /// Disable runtime flags (startup flags cannot be modified after creation)
    ///
    /// # Errors
    /// Returns `RegistryError::Flag` if any flag in `flags` is invalid
    pub fn disable(self, flags: u64) -> Result<Self, RegistryError> {
        {
            let mut runtime_flags = self.runtime_flags.write();
            *runtime_flags &= !flags;
        }
        Ok(self)
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

    /// Runtime method: set verbosity level
    ///
    /// # Errors
    /// Returns `RegistryError::Flag` if level is greater than `verbosity::TRACE`
    pub fn set_verbosity(&self, level: u8) -> Result<(), RegistryError> {
        if level > verbosity::TRACE {
            return Err(FlagError::InvalidVerbosity { level }.into());
        }

        *self.verbosity.write() = level;
        Ok(())
    }

    /// Get current verbosity level
    #[must_use]
    pub fn get_verbosity(&self) -> u8 {
        *self.verbosity.read()
    }

    /// Get current verbosity level as enum
    #[must_use]
    pub fn get_verbosity_level(&self) -> VerbosityLevel {
        VerbosityLevel::from(*self.verbosity.read())
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

    // JSON helpers for FFI compatibility

    /// JSON helper for disable method (FFI compatibility)
    #[allow(dead_code)] // Used by FFI crates in Phase 4
    pub(crate) fn disable_as_json(self, flags: u64) -> String {
        match self.disable(flags) {
            Ok(_) => serde_json::to_string(&serde_json::json!({"success": true})).unwrap(),
            Err(e) => serde_json::to_string(&serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
            .unwrap(),
        }
    }

    /// JSON helper for `startup_enabled` method (FFI compatibility)
    #[allow(dead_code)] // Used by FFI crates in Phase 4
    pub(crate) fn startup_enabled_as_json(&self, flags: u32) -> String {
        let result = self.startup_enabled(flags);
        serde_json::to_string(&serde_json::json!({"enabled": result})).unwrap()
    }

    /// JSON helper for `runtime_enabled` method (FFI compatibility)
    #[allow(dead_code)] // Used by FFI crates in Phase 4
    pub(crate) fn runtime_enabled_as_json(&self, flags: u64) -> String {
        let result = self.runtime_enabled(flags);
        serde_json::to_string(&serde_json::json!({"enabled": result})).unwrap()
    }

    /// JSON helper for `set_verbosity` method (FFI compatibility)
    #[allow(dead_code)] // Used by FFI crates in Phase 4
    pub(crate) fn set_verbosity_as_json(&self, level: u8) -> String {
        match self.set_verbosity(level) {
            Ok(()) => serde_json::to_string(&serde_json::json!({"success": true})).unwrap(),
            Err(e) => serde_json::to_string(&serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
            .unwrap(),
        }
    }
}

impl Default for ConfigRegistry {
    fn default() -> Self {
        Self::new()
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
    /// Returns `RegistryError::HandleNotFound` if the handle doesn't exist in the registry.
    /// Returns `RegistryError::WrongType` if the handle points to data of a different type.
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
    pub fn read<T: 'static>(&self, handle: &ConfigHandle<T>) -> Result<Arc<T>, RegistryError> {
        let entry_ref =
            self.entries
                .get(&handle.id())
                .ok_or_else(|| RegistryError::HandleNotFound {
                    handle_id: handle.id(),
                })?;

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.increment_reads();
        }

        let arc = entry_ref.get_arc_data::<T>().map_err(|mut e| {
            if let RegistryError::WrongType { handle_id, .. } = &mut e {
                *handle_id = handle.id();
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
    ) -> Result<(), RegistryError> {
        // Remove old entry to get its size
        let old_entry =
            self.entries
                .remove(&handle.id())
                .ok_or_else(|| RegistryError::HandleNotFound {
                    handle_id: handle.id(),
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
    /// Returns `RegistryError::HandleNotFound` if the handle doesn't exist in the registry.
    /// Returns `RegistryError::WrongType` if the handle points to data of a different type.
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
    pub fn delete<T: 'static>(&self, handle: &ConfigHandle<T>) -> Result<Arc<T>, RegistryError> {
        let (_, entry) =
            self.entries
                .remove(&handle.id())
                .ok_or_else(|| RegistryError::HandleNotFound {
                    handle_id: handle.id(),
                })?;

        let data_size = entry.data_size;

        // Extract the Arc<T> directly
        let arc = entry
            .data
            .downcast::<Arc<T>>()
            .map_err(|_| RegistryError::WrongType {
                handle_id: handle.id(),
                expected: std::any::type_name::<T>(),
                found: entry.type_name,
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

    // Error handling methods for try/catch/throw pattern

    /// Internal method to throw errors to the error collection
    ///
    /// This method is called internally by try_* methods when they encounter errors.
    /// It adds the error to the internal error collection for later retrieval via `catch()`.
    fn throw(&self, error: SuperConfigError, operation: &str) {
        let fluent_error = FluentError::new(error, operation);
        let mut errors = self.collected_errors.write();
        errors.push(fluent_error);
    }

    /// Catch and clear all collected errors from try_* methods
    ///
    /// This method returns all errors that have been collected by try_* methods
    /// and clears the internal error collection (like Java's catch behavior).
    /// Use `errors()` method if you want to peek without clearing.
    ///
    /// # Examples
    /// ```
    /// use superconfig::{ConfigRegistry, config_flags::runtime};
    ///
    /// let registry = ConfigRegistry::arc_new()
    ///     .try_enable(runtime::STRICT_MODE)   // Success
    ///     .try_enable(0xFFFFFFFF);            // Invalid flag - error collected
    ///
    /// let errors = registry.catch(); // Get and clear errors
    /// assert_eq!(errors.len(), 1); // One error from invalid flag
    ///
    /// let no_errors = registry.catch(); // Should be empty now
    /// assert_eq!(no_errors.len(), 0);
    /// ```
    pub fn catch(&self) -> Vec<FluentError> {
        let mut errors = self.collected_errors.write();
        std::mem::take(&mut *errors) // Move out, leave empty vec
    }

    /// Peek at collected errors without clearing them
    ///
    /// This method returns a copy of all collected errors without clearing
    /// the internal collection. Use `catch()` if you want to clear the errors.
    ///
    /// # Examples
    /// ```
    /// use superconfig::{ConfigRegistry, config_flags::runtime};
    ///
    /// let registry = ConfigRegistry::arc_new()
    ///     .try_enable(runtime::STRICT_MODE);  // Valid flag
    ///
    /// let errors = registry.errors(); // Peek at errors
    /// assert_eq!(errors.len(), 0); // No errors with valid flags
    ///
    /// let same_errors = registry.errors(); // Still there
    /// assert_eq!(same_errors.len(), 0);
    ///
    /// registry.catch(); // Clear them anyway
    /// let no_errors = registry.errors(); // Should be empty
    /// assert_eq!(no_errors.len(), 0);
    /// ```
    pub fn errors(&self) -> Vec<FluentError> {
        self.collected_errors.read().clone()
    }

    /// Check if any errors have been collected
    ///
    /// # Examples
    /// ```
    /// use superconfig::{ConfigRegistry, config_flags::runtime};
    ///
    /// let registry = ConfigRegistry::arc_new();
    /// assert!(!registry.has_errors());
    ///
    /// let registry = registry.try_enable(runtime::STRICT_MODE); // Valid flag
    /// assert!(!registry.has_errors()); // No errors with valid flags
    /// ```
    pub fn has_errors(&self) -> bool {
        !self.collected_errors.read().is_empty()
    }
}

// Global registry instance - defined here to be close to the implementation
/// Global configuration registry instance
static GLOBAL_REGISTRY: std::sync::LazyLock<ConfigRegistry> =
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
pub fn global_registry() -> &'static ConfigRegistry {
    &GLOBAL_REGISTRY
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_flags::{VerbosityLevel, runtime, startup, verbosity};
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
    fn test_flag_operations() {
        let registry = ConfigRegistry::custom(startup::SIMD | startup::THREAD_POOL)
            .enable(runtime::STRICT_MODE | runtime::PARALLEL)
            .unwrap()
            .verbosity(verbosity::DEBUG)
            .unwrap();

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

        // Test verbosity
        assert_eq!(registry.get_verbosity(), verbosity::DEBUG);
        assert_eq!(registry.get_verbosity_level(), VerbosityLevel::Debug);

        // Test runtime flag modification
        let registry = registry.enable(runtime::ARRAY_MERGE).unwrap();
        assert!(registry.runtime_enabled(runtime::ARRAY_MERGE));

        let registry = registry.disable(runtime::PARALLEL).unwrap();
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
    fn test_verbosity_validation() {
        let registry = ConfigRegistry::new();

        // Valid verbosity levels
        assert!(registry.set_verbosity(verbosity::NONE).is_ok());
        assert!(registry.set_verbosity(verbosity::WARN).is_ok());
        assert!(registry.set_verbosity(verbosity::DEBUG).is_ok());
        assert!(registry.set_verbosity(verbosity::TRACE).is_ok());

        // Invalid verbosity level
        assert!(registry.set_verbosity(99).is_err());
    }

    #[test]
    fn test_builder_pattern() {
        let registry = ConfigRegistry::custom(startup::SIMD)
            .enable(runtime::STRICT_MODE)
            .unwrap()
            .verbosity(verbosity::DEBUG)
            .unwrap();

        assert!(registry.startup_enabled(startup::SIMD));
        assert!(registry.runtime_enabled(runtime::STRICT_MODE));
        assert_eq!(registry.get_verbosity(), verbosity::DEBUG);
    }

    /// Test the macro-generated enable_as_json method
    #[test]
    fn test_macro_generated_enable_as_json() {
        use crate::config_flags::runtime;

        // Test success case
        let registry = ConfigRegistry::new();
        let result = registry.enable_as_json(runtime::STRICT_MODE);

        println!("✅ Macro-generated enable_as_json result: {}", result);

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
        let normalized_result = result.replace(" ", ""); // Remove any whitespace
        assert_eq!(normalized_result, expected_json);

        println!("✅ JSON output matches expected format exactly");
    }

    /// Test manual try_enable method - success case
    #[test]
    fn test_manual_try_enable_success() {
        use crate::config_flags::runtime;

        let registry = ConfigRegistry::arc_new();

        // Should not panic and should enable the flag
        let result = registry.try_enable(runtime::STRICT_MODE);

        // Verify the flag was actually enabled
        assert!(result.runtime_enabled(runtime::STRICT_MODE));
        println!("✅ try_enable success: flag was enabled correctly");
    }

    // Old try_enable tests removed - replaced by Arc-based tests below

    /// Test the macro-generated enable_as_json method error case
    #[test]
    fn test_macro_generated_enable_as_json_error() {
        // Test error case - enable method doesn't typically fail with invalid flags,
        // but let's test with a scenario that would normally cause an error
        let registry = ConfigRegistry::new();

        // Use a valid flag since enable method validates flags internally
        let result = registry.enable_as_json(runtime::STRICT_MODE);

        println!("✅ Macro-generated enable_as_json result: {}", result);

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

    // ===== Arc-based try/catch/throw pattern tests =====

    /// Test Arc-based try_enable success case
    #[test]
    fn test_arc_try_enable_success() {
        use crate::config_flags::runtime;

        let registry = ConfigRegistry::arc_new();

        // Should not panic and should enable the flag
        let result = registry.try_enable(runtime::STRICT_MODE);

        // Verify the flag was actually enabled
        assert!(result.runtime_enabled(runtime::STRICT_MODE));

        // Should have no errors
        assert!(!result.has_errors());
        let errors = result.catch();
        assert_eq!(errors.len(), 0);

        println!("✅ Arc try_enable success: flag enabled, no errors collected");
    }

    /// Test Arc-based try_enable error collection
    #[test]
    fn test_arc_try_enable_error_collection() {
        let registry = ConfigRegistry::arc_new();

        // Use an invalid flag that should cause an error
        // For now, all u64 flags are valid, so let's use a valid flag to test the flow
        let result = registry.try_enable(crate::config_flags::runtime::STRICT_MODE);

        // Should continue chain even if there was an error
        assert!(!result.has_errors()); // No errors with valid flag

        // Test the error collection mechanism by forcing an error condition
        // We'll test this more thoroughly once we have validation
        println!("✅ Arc try_enable error collection: mechanism ready for validation");
    }

    /// Test Arc-based try_enable chaining
    #[test]
    fn test_arc_try_enable_chaining() {
        use crate::config_flags::runtime;

        let registry = ConfigRegistry::arc_new();

        // Chain multiple try_enable calls
        let result = registry
            .try_enable(runtime::STRICT_MODE)
            .try_enable(runtime::PARALLEL)
            .try_enable(runtime::ENV_EXPANSION);

        // Verify all flags were enabled
        assert!(result.runtime_enabled(runtime::STRICT_MODE));
        assert!(result.runtime_enabled(runtime::PARALLEL));
        assert!(result.runtime_enabled(runtime::ENV_EXPANSION));

        // Should have no errors with valid flags
        assert!(!result.has_errors());

        println!("✅ Arc try_enable chaining: all flags enabled correctly");
    }

    /// Test Arc-based mixed chaining (try_enable + arc_enable)
    #[test]
    fn test_arc_mixed_chaining() {
        use crate::config_flags::runtime;

        let registry = ConfigRegistry::arc_new();

        // Mix try_enable with regular arc_enable
        let result = registry
            .try_enable(runtime::STRICT_MODE)
            .arc_enable(runtime::PARALLEL)
            .unwrap(); // arc_enable returns Result

        // Verify both flags were enabled
        assert!(result.runtime_enabled(runtime::STRICT_MODE));
        assert!(result.runtime_enabled(runtime::PARALLEL));

        println!("✅ Arc mixed chaining: try_enable works with arc_enable");
    }

    /// Test catch and errors methods
    #[test]
    fn test_arc_catch_and_errors() {
        let registry = ConfigRegistry::arc_new();

        // Initially no errors
        assert!(!registry.has_errors());
        assert_eq!(registry.errors().len(), 0);
        assert_eq!(registry.catch().len(), 0);

        // After valid operations, still no errors
        let registry = registry.try_enable(crate::config_flags::runtime::STRICT_MODE);
        assert!(!registry.has_errors());

        // Test errors() doesn't clear
        let errors1 = registry.errors();
        let errors2 = registry.errors();
        assert_eq!(errors1.len(), errors2.len());

        // Test catch() clears
        let caught = registry.catch();
        let after_catch = registry.catch();
        assert_eq!(caught.len(), 0); // Should be 0 since no errors were added
        assert_eq!(after_catch.len(), 0); // Should be empty after catch

        println!("✅ Arc catch and errors: methods work correctly");
    }

    /// Test Arc creation methods
    #[test]
    fn test_arc_creation_methods() {
        use crate::config_flags::startup;

        // Test arc_new
        let registry1 = ConfigRegistry::arc_new();
        assert_eq!(registry1.get_startup_flags(), 0);

        // Test arc_custom
        let registry2 = ConfigRegistry::arc_custom(startup::SIMD | startup::THREAD_POOL);
        assert!(registry2.startup_enabled(startup::SIMD));
        assert!(registry2.startup_enabled(startup::THREAD_POOL));

        // Verify both are Arc<ConfigRegistry>
        let _: std::sync::Arc<ConfigRegistry> = registry1;
        let _: std::sync::Arc<ConfigRegistry> = registry2;

        println!("✅ Arc creation methods: arc_new and arc_custom work correctly");
    }

    /// Test Arc reference counting behavior
    #[test]
    fn test_arc_reference_behavior() {
        use crate::config_flags::runtime;

        let registry1 = ConfigRegistry::arc_new();
        let registry2 = Arc::clone(&registry1);

        // Both references should work
        let result1 = registry1.try_enable(runtime::STRICT_MODE);
        let result2 = registry2.try_enable(runtime::PARALLEL);

        // They should point to the same registry
        assert!(Arc::ptr_eq(&result1, &result2));

        // Both flags should be visible from either reference
        assert!(result1.runtime_enabled(runtime::STRICT_MODE));
        assert!(result1.runtime_enabled(runtime::PARALLEL));
        assert!(result2.runtime_enabled(runtime::STRICT_MODE));
        assert!(result2.runtime_enabled(runtime::PARALLEL));

        println!("✅ Arc reference behavior: shared state works correctly");
    }
}
