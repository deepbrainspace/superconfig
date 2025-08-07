//! # `SuperConfig` V2.1
//!
//! High-performance multi-format configuration management with handle-based registry,
//! zero-copy access, and multi-language support.
//!
//! ## Features
//!
//! - **Multi-Format Support**: TOML, JSON, YAML, INI with auto-detection
//! - **Handle-Based Registry**: Sub-microsecond configuration lookup via handles
//! - **Key/Value Access**: Nested key support (`storage.db.host`)
//! - **Profile Management**: Environment-specific configurations (`default`, `staging`, `prod`)
//! - **Environment Variables**: `APP_DB_HOST` → `db.host` automatic conversion
//! - **CLI Arguments**: `--db.host=value` parsing and integration
//! - **Zero-Copy Access**: No serialization overhead for repeated access
//! - **Lock-Free Operations**: SCC HashMap-based concurrent registry
//! - **Native Logging**: Built-in structured logging with FFI bridges to Python/Node.js
//! - **Multi-Language Support**: FFI bindings for Python, Node.js, and WebAssembly

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::result_large_err)] // Maintain rich error context

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Core type system - implemented first
pub mod types;

// Module exports will be added as we implement each phase
// Phase 1: Core registry system (pending implementation)
// pub mod core;
// pub mod backend;

// Phase 2: Multi-format system (pending implementation)
// pub mod formats;

// Phase 3: Sources system (pending implementation)
// pub mod sources;

// Phase 4: Tree management (pending implementation)
// pub mod trees;

// Phase 5: Public API (pending implementation)
// pub mod api;

// Re-exports for current types
pub use types::*;

/// Re-export logfusion under a logging namespace for better API organization
/// Logging functionality provided by the logfusion crate
///
/// This module provides structured logging with FFI callback support.
/// Use this for all logging needs within `SuperConfig` applications.
pub mod logging {
    pub use logfusion::*;
}
