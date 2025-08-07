//! Handle ID type definition and generation for `SuperConfig` v2.1
//!
//! This module provides the enhanced `HandleID` system that's compatible with the existing
//! handle system while supporting the new multi-format architecture.
//!
//! ## Functions
//!
//! - [`generate_handle_id`] - Generate new unique handle IDs
//! - [`reset_handle_counter`] - Reset counter for testing
//! - [`get_current_handle_count`] - Get current counter value
//! - [`is_valid_handle_id`] - Validate handle ID values

use std::sync::atomic::{AtomicU64, Ordering};

/// Unique identifier for configuration data in the `DataMap`
///
/// Compatible with existing handle system - maintains u64 type for FFI compatibility.
/// This type is shared between handles and the `DataMap` backend for efficient lookups.
pub type HandleID = u64;

/// Global counter for generating unique handle IDs
///
/// Uses atomic operations for thread-safety without locks.
/// Starts at 0 so that `fetch_add` returns 0, 1, 2, etc. and we add 1 to get 1, 2, 3, etc.
/// This reserves 0 for special cases (e.g., null/invalid handles).
static NEXT_HANDLE_ID: AtomicU64 = AtomicU64::new(0);

/// Generate a new unique handle ID
///
/// This function is thread-safe and generates monotonically increasing IDs.
/// The IDs start at 1 and increment for each call.
///
/// # Examples
///
/// ```
/// use superconfig::types::generate_handle_id;
///
/// let id1 = generate_handle_id();
/// let id2 = generate_handle_id();
///
/// assert!(id2 > id1);
/// assert_eq!(id2, id1 + 1);
/// ```
#[must_use]
pub fn generate_handle_id() -> HandleID {
    // fetch_add returns the previous value, then atomically increments
    // We add 1 to skip 0 (reserved for invalid/null handles)
    let id = NEXT_HANDLE_ID.fetch_add(1, Ordering::SeqCst);
    id + 1
}

/// Reset the handle ID counter (primarily for testing)
///
/// # Safety
///
/// This function should only be used in tests or during registry reset operations.
/// Using this in production with active handles can lead to ID collisions.
///
/// # Examples
///
/// ```
/// use superconfig::types::{generate_handle_id, reset_handle_counter};
///
/// // In tests only
/// reset_handle_counter();
/// let id = generate_handle_id();
/// assert_eq!(id, 1);
/// ```
pub fn reset_handle_counter() {
    NEXT_HANDLE_ID.store(0, Ordering::SeqCst);
}

/// Get the current handle ID counter value without incrementing
///
/// This is useful for statistics and debugging.
///
/// # Examples
///
/// ```
/// use superconfig::types::{generate_handle_id, get_current_handle_count};
///
/// let initial_count = get_current_handle_count();
/// generate_handle_id();
/// let new_count = get_current_handle_count();
///
/// assert_eq!(new_count, initial_count + 1);
/// ```
#[must_use]
pub fn get_current_handle_count() -> HandleID {
    NEXT_HANDLE_ID.load(Ordering::Relaxed) + 1
}

/// Check if a handle ID is valid (non-zero)
///
/// Handle IDs start at 1, so 0 represents an invalid/null handle.
/// This function provides a convenient way to validate handle IDs.
///
/// # Examples
///
/// ```
/// use superconfig::types::{generate_handle_id, is_valid_handle_id};
///
/// let valid_id = generate_handle_id();
/// assert!(is_valid_handle_id(valid_id));
/// assert!(!is_valid_handle_id(0));
/// ```
#[must_use]
pub const fn is_valid_handle_id(handle_id: HandleID) -> bool {
    handle_id != 0
}
