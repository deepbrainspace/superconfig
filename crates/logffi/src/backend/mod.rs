//! Backend implementations for LogFFI
//!
//! This module contains all backend-specific implementations and types.
//! Each backend is conditionally compiled based on feature flags.

#[cfg(feature = "callback")]
pub mod callback_backend;
#[cfg(feature = "log")]
pub mod log_backend;
#[cfg(feature = "slog")]
pub mod slog_backend;
#[cfg(feature = "tracing")]
pub mod tracing_backend;

#[cfg(feature = "callback")]
pub use callback_backend::CallbackBackend;
#[cfg(feature = "log")]
pub use log_backend::LogBackend;
#[cfg(feature = "slog")]
pub use slog_backend::SlogBackend;
#[cfg(feature = "tracing")]
pub use tracing_backend::TracingBackend;

/// Backend selection for runtime switching
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Backend {
    #[cfg(feature = "log")]
    Log = 0,
    #[cfg(feature = "tracing")]
    Tracing = 1,
    #[cfg(feature = "slog")]
    Slog = 2,
    #[cfg(feature = "callback")]
    Callback = 3,
}

impl Backend {
    /// Get the name of the backend as a string
    pub fn name(&self) -> &'static str {
        match self {
            #[cfg(feature = "log")]
            Backend::Log => "log",
            #[cfg(feature = "tracing")]
            Backend::Tracing => "tracing",
            #[cfg(feature = "slog")]
            Backend::Slog => "slog",
            #[cfg(feature = "callback")]
            Backend::Callback => "callback",
        }
    }
}

impl Default for Backend {
    fn default() -> Self {
        #[cfg(feature = "tracing")]
        return Backend::Tracing;

        #[cfg(all(not(feature = "tracing"), feature = "log"))]
        return Backend::Log;

        #[cfg(all(not(feature = "tracing"), not(feature = "log"), feature = "slog"))]
        return Backend::Slog;

        #[cfg(all(
            not(feature = "tracing"),
            not(feature = "log"),
            not(feature = "slog"),
            feature = "callback"
        ))]
        return Backend::Callback;

        #[cfg(not(any(
            feature = "log",
            feature = "tracing",
            feature = "slog",
            feature = "callback"
        )))]
        compile_error!("At least one backend must be enabled");
    }
}

// No concept of "current" backend - all enabled backends are always available

/// Backend implementation trait
pub trait LogBackendTrait: Send + Sync {
    /// Initialize the backend
    fn init(&self);
}

/// Backend implementation variants
pub enum BackendImpl {
    #[cfg(feature = "log")]
    Log(LogBackend),
    #[cfg(feature = "tracing")]
    Tracing(TracingBackend),
    #[cfg(feature = "slog")]
    Slog(SlogBackend),
    #[cfg(feature = "callback")]
    Callback(CallbackBackend),
}
