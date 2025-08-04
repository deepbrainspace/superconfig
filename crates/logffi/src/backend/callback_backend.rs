//! Callback backend implementation for FFI integration

use super::LogBackendTrait;
use std::sync::OnceLock;

/// FFI callback function type
pub type FfiCallback = Box<dyn Fn(&str, &str, &str) + Send + Sync>;

/// Global callback storage
static CALLBACK: OnceLock<FfiCallback> = OnceLock::new();

/// Callback backend for FFI integration
pub struct CallbackBackend;

impl CallbackBackend {
    /// Create a new callback backend
    pub fn new() -> Self {
        CallbackBackend
    }
}

impl Default for CallbackBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl LogBackendTrait for CallbackBackend {
    fn init(&self) {
        // Callback backend doesn't need initialization
        // Callbacks are set via set_callback()
    }
}

/// Set callback for bridging logs to other systems
pub fn set_callback(callback: FfiCallback) {
    CALLBACK.set(callback).ok();
}

/// Call callback if set
pub fn call_callback(level: &str, target: &str, message: &str) {
    if let Some(callback) = CALLBACK.get() {
        callback(level, target, message);
    }
}
