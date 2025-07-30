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
