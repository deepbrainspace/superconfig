//! Slog crate backend implementation

use super::LogBackendTrait;
use slog::{Drain, Logger, o};

/// Slog backend wrapper
pub struct SlogBackend {
    pub root_logger: Logger,
}

impl SlogBackend {
    /// Create a new slog backend
    pub fn new() -> Self {
        let format = std::env::var("LOGFFI_FORMAT").unwrap_or_else(|_| "text".to_string());

        let root_logger = match format.as_str() {
            "json" => {
                let drain = slog_json::Json::default(std::io::stderr()).fuse();
                let drain = slog_async::Async::new(drain).build().fuse();
                Logger::root(drain, o!())
            }
            _ => {
                let decorator = slog_term::TermDecorator::new().build();
                let drain = slog_term::FullFormat::new(decorator).build().fuse();
                let drain = slog_async::Async::new(drain).build().fuse();
                Logger::root(drain, o!())
            }
        };

        SlogBackend { root_logger }
    }

    /// Get access to the root logger
    pub fn logger(&self) -> &Logger {
        &self.root_logger
    }
}

impl Default for SlogBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl LogBackendTrait for SlogBackend {
    fn init(&self) {
        // Slog doesn't need global initialization
        // The logger is created in new()
    }
}
