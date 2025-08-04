//! Log crate backend implementation

use super::LogBackendTrait;

/// Log backend wrapper
pub struct LogBackend;

impl LogBackend {
    /// Create a new log backend
    pub fn new() -> Self {
        LogBackend
    }
}

impl Default for LogBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl LogBackendTrait for LogBackend {
    fn init(&self) {
        // Check if logger is already initialized by checking max level
        if log::max_level() == log::LevelFilter::Off {
            // Try to initialize with env_logger
            let _ = env_logger::try_init();
        }
    }
}
