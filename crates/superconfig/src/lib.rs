//! # `SuperConfig` V2
//!
//! High-performance configuration management with handle-based registry, zero-copy access,
//! and multi-language support.
//!
//! ## Features
//!
//! - **Handle-Based Registry**: Sub-microsecond configuration lookup via handles
//! - **Zero-Copy Access**: No serialization overhead for repeated access
//! - **Lock-Free Operations**: DashMap-based concurrent registry
//! - **SIMD Optimizations**: Hardware-accelerated parsing (optional)
//! - **Multi-Language Support**: FFI bindings for Python, Node.js, and WebAssembly
//!
//! ## Performance Targets
//!
//! - Configuration loading: ≤30μs (vs current ~100μs)
//! - Handle operations: ≤0.5μs (sub-microsecond registry access)
//! - FFI overhead: ≤2μs for Node.js, ≤1μs for Python

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::result_large_err)] // Maintain rich error context

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Module exports will be added as we implement each phase
// Phase 1: Core registry system
pub mod config_flags;
pub mod core;

// Future phases (commented out until implemented)
// pub mod providers;   // Phase 2: Configuration engine
// pub mod parsing;     // Phase 2: Format parsers
// pub mod merging;     // Phase 2: Configuration composition
// pub mod builder;     // Phase 3: Public API
// pub mod features;    // Phase 5: Advanced features

// Re-exports for convenience
pub use config_flags::*;
pub use core::*;
