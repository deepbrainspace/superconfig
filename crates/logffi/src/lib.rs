//! # LogFFI - Universal Rust Logging System
//!
//! LogFFI provides runtime backend switching, full API access, and universal FFI bridging capabilities.
//!
//! ## Core Features
//!
//! - **Runtime Backend Switching** - Choose log/tracing/slog at runtime, not compile time
//! - **Full Backend Access** - Complete API access via Deref, no functionality lost  
//! - **Universal FFI Bridge** - Python/Node.js integration through callback system
//! - **Enhanced Error Macros** - `define_errors!` with error codes, source chaining + FFI mapping
//! - **Zero Configuration** - Auto-initialization with environment detection
//! - **Callback Mode** - Custom routing with dual-mode support (callback + native)
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
//! ## Runtime Backend Configuration
//!
//! ```bash
//! # Environment variable control
//! export LOGFFI_BACKEND=tracing  # or "log" or "slog"
//! export LOGFFI_FORMAT=json      # or "text" or "compact"
//! export LOGFFI_FORCE_NATIVE=true # dual-mode: callback + native
//! ```
//!
//! ## Universal FFI Bridge
//!
//! ```rust
//! use logffi::set_callback;
//!
//! // Set up custom callback for external systems
//! set_callback(Box::new(|level, target, message| {
//!     println!("FFI Bridge: [{}] {} - {}", level, target, message);
//! }));
//! ```
//!
//! ## Full Backend Access
//!
//! ```rust
//! use logffi::logger;
//!
//! // Check which backend is active and access its features
//! let logger_instance = logger();
//!
//! if let Some(_tracing) = logger_instance.as_tracing() {
//!     // Tracing backend is active - can access tracing-specific features
//!     println!("Using tracing backend");
//! }
//!
//! if let Some(_slog) = logger_instance.as_slog() {
//!     // Slog backend is active - can access slog-specific features  
//!     println!("Using slog backend");
//! }
//! ```

use std::ops::Deref;
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

// Re-export log crate for compatibility (but not the macros since we provide our own)
pub use log::{Level, LevelFilter, Log, Metadata, Record, max_level, set_logger, set_max_level};

/// Backend selection for runtime switching
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Backend {
    Log = 0,
    #[default]
    Tracing = 1,
    Slog = 2,
}

/// Global backend selection (atomic for thread safety)
static CURRENT_BACKEND: AtomicU8 = AtomicU8::new(Backend::Tracing as u8);

/// Logger instance (initialized once)
static LOGGER_INSTANCE: OnceLock<LogFFI> = OnceLock::new();

/// Force native backends flag (for dual-mode: callback + native)
pub static FORCE_NATIVE_BACKENDS: AtomicBool = AtomicBool::new(false);

/// FFI callback function type
pub type FfiCallback = Box<dyn Fn(&str, &str, &str) + Send + Sync>;

/// Global callback storage (renamed from FFI_CALLBACK for universal naming)
pub static CALLBACK: OnceLock<FfiCallback> = OnceLock::new();

/// Universal LogFFI instance with backend abstraction
pub struct LogFFI {
    backend_impl: BackendImpl,
}

/// Backend implementation variants
enum BackendImpl {
    Log(LogBackend),
    Tracing(TracingBackend),
    Slog(SlogBackend),
}

/// Log backend wrapper
pub struct LogBackend;

/// Tracing backend wrapper with full API access
pub struct TracingBackend;

/// Slog backend wrapper with full API access
pub struct SlogBackend {
    root_logger: slog::Logger,
}

impl LogFFI {
    /// Auto-initialization with environment variable support
    pub fn auto_init() -> Self {
        let backend = std::env::var("LOGFFI_BACKEND")
            .unwrap_or_else(|_| "tracing".to_string())
            .to_lowercase();

        // Check for FORCE_NATIVE environment variable
        if let Ok(force_native) = std::env::var("LOGFFI_FORCE_NATIVE") {
            if force_native.to_lowercase() == "true" {
                FORCE_NATIVE_BACKENDS.store(true, Ordering::Relaxed);
            }
        }

        let backend_impl = match backend.as_str() {
            "log" => {
                set_backend(Backend::Log);
                Self::init_log_backend();
                BackendImpl::Log(LogBackend::new())
            }
            "tracing" => {
                set_backend(Backend::Tracing);
                Self::init_tracing_backend();
                BackendImpl::Tracing(TracingBackend::new())
            }
            "slog" => {
                set_backend(Backend::Slog);
                Self::init_slog_backend();
                BackendImpl::Slog(SlogBackend::new())
            }
            _ => {
                eprintln!("Warning: Unknown LOGFFI_BACKEND '{backend}', defaulting to tracing");
                set_backend(Backend::Tracing);
                Self::init_tracing_backend();
                BackendImpl::Tracing(TracingBackend::new())
            }
        };

        LogFFI { backend_impl }
    }

    /// Direct access to tracing with ALL functionality
    pub fn as_tracing(&self) -> Option<&TracingBackend> {
        match &self.backend_impl {
            BackendImpl::Tracing(t) => Some(t),
            _ => None,
        }
    }

    /// Direct access to slog with ALL functionality
    pub fn as_slog(&self) -> Option<&SlogBackend> {
        match &self.backend_impl {
            BackendImpl::Slog(s) => Some(s),
            _ => None,
        }
    }

    /// Direct access to log backend
    pub fn as_log(&self) -> Option<&LogBackend> {
        match &self.backend_impl {
            BackendImpl::Log(l) => Some(l),
            _ => None,
        }
    }

    /// Initialize tracing backend with environment configuration
    fn init_tracing_backend() {
        use tracing_subscriber::{EnvFilter, fmt};

        let format = std::env::var("LOGFFI_FORMAT").unwrap_or_else(|_| "text".to_string());

        let result = match format.as_str() {
            "json" => {
                let subscriber = fmt()
                    .json()
                    .with_env_filter(EnvFilter::from_default_env())
                    .finish();
                tracing::subscriber::set_global_default(subscriber)
            }
            "compact" => {
                let subscriber = fmt()
                    .compact()
                    .with_env_filter(EnvFilter::from_default_env())
                    .finish();
                tracing::subscriber::set_global_default(subscriber)
            }
            _ => {
                let subscriber = fmt()
                    .with_env_filter(EnvFilter::from_default_env())
                    .finish();
                tracing::subscriber::set_global_default(subscriber)
            }
        };

        if result.is_err() {
            // Already set, that's fine
        }
    }

    /// Initialize log backend
    fn init_log_backend() {
        // Check if logger is already initialized by checking max level
        if log::max_level() == log::LevelFilter::Off {
            // Try to initialize with env_logger (it's in dev-dependencies, so use it conditionally)
            #[cfg(test)]
            {
                let _ = env_logger::try_init();
            }
            #[cfg(not(test))]
            {
                // For non-test builds, just ensure max level is set
                log::set_max_level(log::LevelFilter::Info);
            }
        }
    }

    /// Initialize slog backend
    fn init_slog_backend() {
        // slog initialization happens in SlogBackend::new()
    }
}

impl LogBackend {
    fn new() -> Self {
        LogBackend
    }
}

impl TracingBackend {
    fn new() -> Self {
        TracingBackend
    }
}

impl SlogBackend {
    fn new() -> Self {
        use slog::{Drain, o};

        let format = std::env::var("LOGFFI_FORMAT").unwrap_or_else(|_| "text".to_string());

        let drain = match format.as_str() {
            "json" => {
                let drain = slog_json::Json::default(std::io::stderr()).fuse();
                slog_async::Async::new(drain).build().fuse()
            }
            _ => {
                let decorator = slog_term::TermDecorator::new().build();
                let drain = slog_term::FullFormat::new(decorator).build().fuse();
                slog_async::Async::new(drain).build().fuse()
            }
        };

        let root_logger = slog::Logger::root(drain, o!());

        SlogBackend { root_logger }
    }
}

// Deref implementations for full API access
impl Deref for SlogBackend {
    type Target = slog::Logger;

    fn deref(&self) -> &Self::Target {
        &self.root_logger
    }
}

/// Clear API - get the logger instance (renamed from "global")
pub fn logger() -> &'static LogFFI {
    LOGGER_INSTANCE.get_or_init(LogFFI::auto_init)
}

/// Runtime backend configuration
pub fn set_backend(backend: Backend) {
    CURRENT_BACKEND.store(backend as u8, Ordering::Relaxed);
}

/// Get current backend
pub fn current_backend() -> Backend {
    match CURRENT_BACKEND.load(Ordering::Relaxed) {
        0 => Backend::Log,
        1 => Backend::Tracing,
        2 => Backend::Slog,
        _ => Backend::Tracing, // safe default
    }
}

/// Set callback for bridging logs to other systems (renamed from set_ffi_callback)
pub fn set_callback(callback: FfiCallback) {
    CALLBACK.set(callback).ok();
}

/// Call callback if set (renamed from call_ffi_callback)
pub fn call_callback(level: &str, target: &str, message: &str) {
    if let Some(callback) = CALLBACK.get() {
        callback(level, target, message);
    }
}

// Legacy compatibility exports
pub use call_callback as call_ffi_callback;
pub use set_callback as set_ffi_callback;

// Include generated macros at crate root level (required for procedural macros)
include!("generated_macros.rs");

// Import error macros module
mod error_macros;
pub use error_macros::LogLevel;
