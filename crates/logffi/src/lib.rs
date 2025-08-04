//! # LogFFI - Universal Rust Logging System
//!
//! LogFFI provides compile-time backend selection with feature flags and full API access.
//!
//! ## Core Features
//!
//! - **Feature-based Backend Selection** - Choose log/tracing/slog/callback via features
//! - **Full Backend Access** - Direct access to backend-specific functionality  
//! - **Universal FFI Bridge** - Python/Node.js integration through callback system
//! - **Enhanced Error Macros** - `define_errors!` with error codes, source chaining + FFI mapping
//! - **Zero Configuration** - Auto-initialization with environment detection
//! - **Multi-Backend Support** - Use multiple backends simultaneously
//!
//! ## Quick Start
//!
//! ```rust
//! use logffi::{error, warn, info, debug, trace};
//!
//! // Works immediately with auto-initialization
//! error!("Database connection failed");
//! warn!("High memory usage detected");
//! info!("Server started on port 8080");
//! ```
//!
//! ## Backend Configuration
//!
//! Enable backends via Cargo features:
//!
//! ```toml
//! # Default: tracing backend
//! logffi = "0.2"
//!
//! # Specific backend
//! logffi = { version = "0.2", default-features = false, features = ["log"] }
//!
//! # Multiple backends
//! logffi = { version = "0.2", default-features = false, features = ["log", "tracing"] }
//!
//! # All backends
//! logffi = { version = "0.2", features = ["all"] }
//! ```
//!
//! ## Backend Access
//!
//! ```rust
//! use logffi::logger;
//!
//! // Access specific backends when enabled
//! let logger = logger();
//!
//! if let Some(tracing) = logger.as_tracing() {
//!     // Use tracing-specific features directly
//!     use tracing::span;
//!     let span = span!(tracing::Level::INFO, "operation");
//! }
//!
//! if let Some(slog) = logger.as_slog() {
//!     // Access slog logger directly
//!     use slog::o;
//!     let child = slog.logger().new(o!("component" => "database"));
//! }
//! ```

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
