//! # LogFFI - Universal Rust Logging System
//!
//! [![Crates.io](https://img.shields.io/crates/v/logffi.svg)](https://crates.io/crates/logffi)
//! [![Documentation](https://docs.rs/logffi/badge.svg)](https://docs.rs/logffi)
//! [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
//!
//! Universal logging for Rust with compile-time backend selection, FFI support, and advanced error handling.
//!
//! ## ✨ Features
//!
//! - 🔧 **Feature-Based Backends** - Choose `log`, `tracing`, `slog`, or `callback` via Cargo features
//! - 🌉 **FFI Support** - Bridge Rust logs to Python, Node.js, C/C++, and more
//! - 🎯 **Complete Error Handling** - All `thiserror` features + `define_errors!` macro with automatic logging
//! - 🔗 **Error Chaining** - Full support for source errors with `#[source]` attribute
//! - 🚀 **Zero Overhead** - Only compile what you use, no runtime switching cost
//! - 🛡️ **Type Safe** - Leverage Rust's type system for error handling
//! - 📊 **Multi-Backend Support** - Use multiple backends simultaneously when needed
//! - 🌍 **Cross-Language** - Automatic error mapping for FFI (Python, Node.js, WASM)
//!
//! ## 🆚 Why Choose LogFFI?
//!
//! LogFFI is the **only logging solution** that provides:
//!
//! | Feature | log | tracing | slog | **LogFFI** |
//! |---------|-----|---------|------|------------|
//! | Backend Selection | ❌ | ❌ | ❌ | ✅ Feature-based |
//! | Multi-Backend Support | ❌ | ❌ | ❌ | ✅ Simultaneous |
//! | Zero Overhead | ✅ | ❌ | ❌ | ✅ Compile-time |
//! | Error Integration | ❌ | ❌ | ❌ | ✅ define_errors! |
//! | FFI Bridge | ❌ | ❌ | ❌ | ✅ Callback system |
//! | Supported Destinations | ~200 | ~50 | ~30 | **ALL** |
//!
//! **🎯 Complete Flexibility**: Choose any backend, use multiple simultaneously,
//! get advanced error handling, and integrate with any system via callbacks.
//!
//! ## 🚀 Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! logffi = "0.2"
//! ```
//!
//! ### Basic Logging
//!
//! ```rust
//! use logffi::{error, warn, info, debug, trace};
//!
//! fn main() {
//!     // Works immediately with auto-initialization (tracing backend by default)
//!     info!("Starting application");
//!     debug!("Configuration loaded");
//!     
//!     if let Err(e) = dangerous_operation() {
//!         error!("Operation failed: {}", e);
//!     }
//! }
//!
//! fn dangerous_operation() -> Result<(), &'static str> {
//!     Err("Something went wrong")
//! }
//! ```
//!
//! ## 🔧 Backend Configuration
//!
//! Choose your logging backend via Cargo features:
//!
//! ```toml
//! # Default: tracing backend (recommended)
//! logffi = "0.2"
//!
//! # Specific backend for libraries
//! logffi = { version = "0.2", default-features = false, features = ["log"] }
//!
//! # Multiple backends simultaneously
//! logffi = { version = "0.2", default-features = false, features = ["log", "tracing"] }
//!
//! # All backends (maximum flexibility)
//! logffi = { version = "0.2", features = ["all"] }
//!
//! # FFI applications (Python/Node.js bindings)
//! logffi = { version = "0.2", default-features = false, features = ["callback"] }
//! ```
//!
//! ### Available Backends
//!
//! - **`tracing`** (default) - Modern structured logging with async support
//! - **`log`** - Standard Rust logging facade, lightweight and compatible  
//! - **`slog`** - Highly structured logging with composition capabilities
//! - **`callback`** - Custom routing for FFI integration
//!
//! ### Environment Configuration
//!
//! ```bash
//! # Control output format
//! export LOGFFI_FORMAT=json    # json, text, compact
//! export RUST_LOG=debug        # Standard log level control
//! ```
//!
//! ## 🎯 Advanced Features
//!
//! ### Backend Access
//!
//! Get direct access to backend-specific functionality:
//!
//! ```rust
//! use logffi::logger;
//!
//! let logger = logger();
//!
//! // Access tracing backend for spans and structured logging
//! if let Some(_tracing) = logger.as_tracing() {
//!     use tracing::{span, Level};
//!     let span = span!(Level::INFO, "database_operation");
//!     let _enter = span.enter();
//!     
//!     info!("Inside database span");
//! }
//!
//! // Access slog for hierarchical logging
//! if let Some(slog_backend) = logger.as_slog() {
//!     use slog::{info as slog_info, o};
//!     let child = slog_backend.logger().new(o!("component" => "auth"));
//!     slog_info!(child, "User authentication successful");
//! }
//!
//! // Check available backends
//! let available = logger.available_backends();
//! println!("Available backends: {:?}", available);
//! ```
//!
//! ### Error Handling with Automatic Logging
//!
//! LogFFI provides **all thiserror features** plus automatic logging:
//!
//! ```rust
//! use logffi::define_errors;
//!
//! define_errors! {
//!     pub enum AppError {
//!         #[error("Configuration not found: {path}", level = error)]
//!         ConfigNotFound {
//!             path: String,
//!         },
//!         
//!         #[error("Database connection failed", level = error, target = "db")]
//!         DatabaseConnection,
//!         
//!         #[error("Invalid input: {field}", level = warn)]
//!         ValidationError {
//!             field: String,
//!         },
//!     }
//! }
//!
//! // ✅ Gets ALL thiserror features: Display, Error, Debug, From conversions
//! // ✅ PLUS automatic logging integration
//! // ✅ PLUS constructor methods with auto-logging
//! // ✅ PLUS FFI-friendly error mapping
//!
//! fn load_config(path: &str) -> Result<(), AppError> {
//!     if !std::path::Path::new(path).exists() {
//!         // Creates error + logs automatically + works across languages
//!         return Err(AppError::new_config_not_found(path.to_string()));
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ### Source Error Chaining
//!
//! ```rust
//! use logffi::define_errors;
//! use std::io;
//!
//! define_errors! {
//!     pub enum DataError {
//!         #[error("Failed to read file: {path}")]
//!         ReadError {
//!             path: String,
//!             #[source]
//!             source: io::Error,  // Proper error chaining!
//!         },
//!     }
//! }
//!
//! fn read_data(path: &str) -> Result<String, DataError> {
//!     std::fs::read_to_string(path)
//!         .map_err(|source| DataError::ReadError {
//!             path: path.to_string(),
//!             source,  // Original IO error is preserved
//!         })
//! }
//! ```
//!
//! ### FFI Integration
//!
//! Bridge Rust logs to any language:
//!
//! ```rust
//! use logffi::set_callback;
//!
//! // Bridge to Python logging
//! set_callback(Box::new(|level, target, message| {
//!     // This would call into Python's logging system
//!     println!("Python Logger: [{}] {}: {}", level, target, message);
//! }));
//!
//! // Now all Rust logs appear in Python!
//! info!("This log will be bridged to Python");
//! ```
//!
//! ## 🎛️ Use Cases
//!
//! ### For Library Authors
//!
//! ```toml
//! # Provide maximum compatibility
//! logffi = { version = "0.2", default-features = false, features = ["log"] }
//! ```
//!
//! ### For Applications
//!
//! ```toml
//! # Use modern structured logging
//! logffi = "0.2"  # defaults to tracing
//! ```
//!
//! ### For FFI/Bindings
//!
//! ```toml
//! # Callback-only for Python/Node.js integration
//! logffi = { version = "0.2", default-features = false, features = ["callback"] }
//! ```
//!
//! ### For Complex Applications
//!
//! ```toml
//! # Use multiple backends for different purposes
//! logffi = { version = "0.2", features = ["tracing", "callback"] }
//! ```
//!
//! ```rust
//! use logffi::{info, set_callback};
//!
//! // Set up metrics callback
//! set_callback(Box::new(|level, target, message| {
//!     if target.starts_with("metrics") {
//!         send_to_prometheus(level, target, message);
//!     }
//! }));
//!
//! // Regular app logging (goes to tracing)
//! info!("Application started");
//!
//! // Metrics logging (goes to both tracing and Prometheus)
//! info!(target: "metrics::requests", "Request count: {}", 100);
//!
//! fn send_to_prometheus(_level: &str, _target: &str, _message: &str) {
//!     // Send metrics to Prometheus
//! }
//! ```
//!
//! ## 📚 Learn More
//!
//! - **[GitHub Repository](https://github.com/deepbrain/superconfig/tree/main/crates/logffi)** - Source code and examples
//! - **[Cookbook](https://github.com/deepbrain/superconfig/tree/main/crates/logffi/cookbook)** - Real-world usage patterns
//! - **[Examples](https://github.com/deepbrain/superconfig/tree/main/crates/logffi/examples)** - Runnable example code
//!
//! ## 🤝 Contributing
//!
//! Contributions are welcome! This crate is part of the [SuperConfig](https://github.com/deepbrain/superconfig) project.
//!
//! ## 📄 License
//!
//! This project is licensed under the MIT License.

use std::sync::OnceLock;

// Backend module with all implementations
pub mod backend;

// Re-exports for convenience
pub use backend::{Backend, BackendImpl};

#[cfg(feature = "callback")]
pub use backend::callback_backend::{FfiCallback, call_callback, set_callback};

// Re-export log types when available
#[cfg(feature = "log")]
pub use log::{Level, LevelFilter};

/// Universal LogFFI instance
pub struct LogFFI {
    #[cfg(feature = "log")]
    log_backend: backend::LogBackend,
    #[cfg(feature = "tracing")]
    tracing_backend: backend::TracingBackend,
    #[cfg(feature = "slog")]
    slog_backend: backend::SlogBackend,
    #[cfg(feature = "callback")]
    callback_backend: backend::CallbackBackend,
}

/// Logger instance (initialized once)
static LOGGER_INSTANCE: OnceLock<LogFFI> = OnceLock::new();

impl LogFFI {
    /// Auto-initialization with all enabled backends
    pub fn auto_init() -> Self {
        #[cfg(feature = "log")]
        let log_backend = {
            let backend = backend::LogBackend::new();
            backend::LogBackendTrait::init(&backend);
            backend
        };

        #[cfg(feature = "tracing")]
        let tracing_backend = {
            let backend = backend::TracingBackend::new();
            backend::LogBackendTrait::init(&backend);
            backend
        };

        #[cfg(feature = "slog")]
        let slog_backend = {
            let backend = backend::SlogBackend::new();
            backend::LogBackendTrait::init(&backend);
            backend
        };

        #[cfg(feature = "callback")]
        let callback_backend = {
            let backend = backend::CallbackBackend::new();
            backend::LogBackendTrait::init(&backend);
            backend
        };

        LogFFI {
            #[cfg(feature = "log")]
            log_backend,
            #[cfg(feature = "tracing")]
            tracing_backend,
            #[cfg(feature = "slog")]
            slog_backend,
            #[cfg(feature = "callback")]
            callback_backend,
        }
    }

    /// Get log backend if enabled
    #[cfg(feature = "log")]
    pub fn as_log(&self) -> Option<&backend::LogBackend> {
        Some(&self.log_backend)
    }

    /// Get tracing backend if enabled
    #[cfg(feature = "tracing")]
    pub fn as_tracing(&self) -> Option<&backend::TracingBackend> {
        Some(&self.tracing_backend)
    }

    /// Get slog backend if enabled
    #[cfg(feature = "slog")]
    pub fn as_slog(&self) -> Option<&backend::SlogBackend> {
        Some(&self.slog_backend)
    }

    /// Get callback backend if enabled
    #[cfg(feature = "callback")]
    pub fn as_callback(&self) -> Option<&backend::CallbackBackend> {
        Some(&self.callback_backend)
    }

    /// Check which backends are available
    #[allow(clippy::vec_init_then_push)]
    pub fn available_backends(&self) -> Vec<Backend> {
        let mut backends = Vec::new();

        #[cfg(feature = "log")]
        backends.push(Backend::Log);

        #[cfg(feature = "tracing")]
        backends.push(Backend::Tracing);

        #[cfg(feature = "slog")]
        backends.push(Backend::Slog);

        #[cfg(feature = "callback")]
        backends.push(Backend::Callback);

        backends
    }
}

/// Get the logger instance
pub fn logger() -> &'static LogFFI {
    LOGGER_INSTANCE.get_or_init(LogFFI::auto_init)
}

// Include combined macros at crate root level
include!("macros.rs");
