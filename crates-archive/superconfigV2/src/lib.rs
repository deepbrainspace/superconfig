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
//! - **Native Logging**: Built-in structured logging with FFI bridges to Python/Node.js
//! - **SIMD Optimizations**: Hardware-accelerated parsing (optional)
//! - **Multi-Language Support**: FFI bindings for Python, Node.js, and WebAssembly
//!
//! ## Performance Targets
//!
//! - Configuration loading: ≤30μs (vs current ~100μs)
//! - Handle operations: ≤0.5μs (sub-microsecond registry access)
//! - FFI overhead: ≤2μs for Node.js, ≤1μs for Python
//!
//! ## Basic Usage
//!
//! ### Basic Rust Usage
//!
//! ```rust
//! use superconfig::ConfigRegistry;
//!
//! // Initialize logging (using env_logger as example)
//! // env_logger::init(); // or your preferred logger
//!
//! // Create registry
//! let registry = ConfigRegistry::new();
//!
//! // SuperConfig uses logffi which respects your logger configuration
//! let handle = registry.create("my-config".to_string()).unwrap();
//! ```
//!
//! ### Log Levels
//!
//! - **`Off`**: No logging output
//! - **`Error`**: Critical errors that prevent operation  
//! - **`Warn`**: Invalid inputs, recoverable errors (default)
//! - **`Info`**: Major operations (registry create, update, delete)
//! - **`Debug`**: Detailed operation flow, flag changes
//! - **`Trace`**: Verbose internal state (development only)
//!
//! ### Integration with Popular Logging Crates
//!
//! #### With `env_logger`
//! ```rust
//! use superconfig::ConfigRegistry;
//!
//! // Initialize env_logger (uncomment to use)
//! // env_logger::Builder::from_default_env()
//! //     .filter_level(log::LevelFilter::Warn)
//! //     .filter_module("superconfig", log::LevelFilter::Debug)
//! //     .init();
//!
//! let registry = ConfigRegistry::new();
//! ```
//!
//! #### With `tracing`
//! ```rust
//! use tracing_subscriber::{fmt, EnvFilter};
//! use superconfig::{ConfigRegistry, logging};
//!
//! // Initialize tracing
//! fmt()
//!     .with_env_filter(
//!         EnvFilter::from_default_env()
//!             .add_directive("superconfig=debug".parse().unwrap())
//!     )
//!     .init();
//!
//! let registry = ConfigRegistry::new();
//! logging::set_max_level(logging::LevelFilter::Debug);
//! ```
//!
//! ### Environment Variable Control
//!
//! You can control logging via environment variables:
//!
//! ```bash
//! # Set overall log level
//! export RUST_LOG=warn
//!
//! # Set SuperConfig-specific level
//! export RUST_LOG=warn,superconfig=debug
//!
//! # See specific targets
//! export RUST_LOG=superconfig::flags=trace,superconfig::registry=debug
//! ```
//!
//! ### FFI Logging (Python/Node.js)
//!
//! When using `SuperConfig` from Python or Node.js, logging is automatically bridged
//! to the native logging systems:
//!
//! - **Python**: Messages appear in Python's `logging` module
//! - **Node.js**: Messages integrate with Winston, Pino, or console logging
//!
//! See the FFI crate documentation for setup details.

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

// Re-export logffi under a logging namespace for better API organization
/// Logging functionality provided by the logffi crate
///
/// This module provides structured logging with FFI callback support.
/// Use this for all logging needs within `SuperConfig` applications.
///
/// # Examples
/// ```
/// use superconfig::logging::{warn, set_max_level, LevelFilter};
///
/// set_max_level(LevelFilter::Debug);
/// warn!("This is a warning message");
/// ```
pub mod logging {
    pub use logffi::*;
}
