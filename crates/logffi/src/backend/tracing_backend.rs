//! Tracing crate backend implementation

use super::LogBackendTrait;

/// Tracing backend wrapper
pub struct TracingBackend;

impl TracingBackend {
    /// Create a new tracing backend
    pub fn new() -> Self {
        TracingBackend
    }
}

impl Default for TracingBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl LogBackendTrait for TracingBackend {
    fn init(&self) {
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
}
