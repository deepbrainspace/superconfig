//! Type system for `SuperConfig` v2.1
//!
//! This module provides the foundational type system for multi-format configuration
//! management with profile support and dynamic type handling.
//!
//! ## Current Components (Phase 1)
//!
//! - [`HandleID`] - Unique identifiers for configuration data in the `DataMap`
//!
//! Additional components will be added in subsequent implementation phases.

pub mod handle_id;

// Re-export key types
pub use handle_id::*;
