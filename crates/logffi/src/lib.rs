//! Drop-in replacement for log crate with FFI callback support
//!
//! This crate provides identical macros to the `log` crate (`warn!`, `debug!`, etc.)
//! but adds FFI callback support for bridging Rust logging to Python, Node.js, and
//! other languages.
//!
//! # Usage
//!
//! Replace `log` with `logffi` in your dependencies:
//!
//! ```toml
//! [dependencies]
//! logffi = "0.1"
//! ```
//!
//! Use identical syntax to the `log` crate:
//!
//! ```rust
//! use logffi::{warn, debug, info, error, trace};
//!
//! let value = 42;
//! warn!("This is a warning: {}", value);
//! debug!(target: "my_target", "Debug message with target");
//! ```
//!
//! For FFI integration, set a callback:
//!
//! ```rust
//! use logffi::set_ffi_callback;
//!
//! set_ffi_callback(Box::new(|level, target, message| {
//!     // Bridge to Python logging.getLogger(target).warning(message)
//!     // or Node.js winston.log(level, message, { target })
//! }));
//! ```

use std::sync::OnceLock;

// Re-export everything from log crate for compatibility
pub use log::*;

/// FFI callback function type
/// Parameters: (level, target, message)
pub type FfiCallback = Box<dyn Fn(&str, &str, &str) + Send + Sync>;

/// Global FFI callback storage
static FFI_CALLBACK: OnceLock<FfiCallback> = OnceLock::new();

/// Set FFI callback for bridging Rust logs to other languages
///
/// This callback will be called for every log message that passes
/// both the log crate's filtering and any additional filtering.
///
/// # Examples
///
/// ```rust
/// use logffi::set_ffi_callback;
///
/// set_ffi_callback(Box::new(|level, target, message| {
///     println!("FFI: [{}] {}: {}", level, target, message);
/// }));
/// ```
pub fn set_ffi_callback(callback: FfiCallback) {
    FFI_CALLBACK.set(callback).ok();
}

/// Internal function to call FFI callback if set
#[doc(hidden)]
pub fn call_ffi_callback(level: &str, target: &str, message: &str) {
    if let Some(callback) = FFI_CALLBACK.get() {
        callback(level, target, message);
    }
}

/// Enhanced log macro that includes FFI callback
#[macro_export]
macro_rules! log_with_ffi {
    (target: $target:expr, $level:expr, $($arg:tt)*) => {
        {
            let level_str = match $level {
                $crate::Level::Error => "ERROR",
                $crate::Level::Warn => "WARN",
                $crate::Level::Info => "INFO",
                $crate::Level::Debug => "DEBUG",
                $crate::Level::Trace => "TRACE",
            };

            // Use log crate's standard logging (respects all filtering)
            $crate::log!(target: $target, $level, $($arg)*);

            // Call FFI callback if log would be enabled
            if $crate::log_enabled!(target: $target, $level) {
                let message = format!($($arg)*);
                $crate::call_ffi_callback(level_str, $target, &message);
            }
        }
    };
    ($level:expr, $($arg:tt)*) => {
        $crate::log_with_ffi!(target: module_path!(), $level, $($arg)*)
    };
}

/// Log an error message with FFI callback support
///
/// Identical API to `log::error!` with added FFI bridging.
#[macro_export]
macro_rules! error {
    (target: $target:expr, $($arg:tt)*) => {
        $crate::log_with_ffi!(target: $target, $crate::Level::Error, $($arg)*)
    };
    ($($arg:tt)*) => {
        $crate::log_with_ffi!($crate::Level::Error, $($arg)*)
    };
}

/// Log a warning message with FFI callback support
///
/// Identical API to `log::warn!` with added FFI bridging.
#[macro_export]
macro_rules! warn {
    (target: $target:expr, $($arg:tt)*) => {
        $crate::log_with_ffi!(target: $target, $crate::Level::Warn, $($arg)*)
    };
    ($($arg:tt)*) => {
        $crate::log_with_ffi!($crate::Level::Warn, $($arg)*)
    };
}

/// Log an info message with FFI callback support
///
/// Identical API to `log::info!` with added FFI bridging.
#[macro_export]
macro_rules! info {
    (target: $target:expr, $($arg:tt)*) => {
        $crate::log_with_ffi!(target: $target, $crate::Level::Info, $($arg)*)
    };
    ($($arg:tt)*) => {
        $crate::log_with_ffi!($crate::Level::Info, $($arg)*)
    };
}

/// Log a debug message with FFI callback support
///
/// Identical API to `log::debug!` with added FFI bridging.
#[macro_export]
macro_rules! debug {
    (target: $target:expr, $($arg:tt)*) => {
        $crate::log_with_ffi!(target: $target, $crate::Level::Debug, $($arg)*)
    };
    ($($arg:tt)*) => {
        $crate::log_with_ffi!($crate::Level::Debug, $($arg)*)
    };
}

/// Log a trace message with FFI callback support
///
/// Identical API to `log::trace!` with added FFI bridging.
#[macro_export]
macro_rules! trace {
    (target: $target:expr, $($arg:tt)*) => {
        $crate::log_with_ffi!(target: $target, $crate::Level::Trace, $($arg)*)
    };
    ($($arg:tt)*) => {
        $crate::log_with_ffi!($crate::Level::Trace, $($arg)*)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_ffi_callback_integration() {
        let captured_logs = Arc::new(Mutex::new(Vec::new()));
        let captured_logs_clone = Arc::clone(&captured_logs);

        set_ffi_callback(Box::new(move |level, target, message| {
            captured_logs_clone.lock().unwrap().push((
                level.to_string(),
                target.to_string(),
                message.to_string(),
            ));
        }));

        // Initialize env_logger to enable logging
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .try_init();

        // Test macros
        warn!("Test warning message");
        debug!(target: "test_target", "Debug with target: {}", 42);

        // Check captured logs
        let logs = captured_logs.lock().unwrap();
        assert!(!logs.is_empty());

        // Find our test messages
        let warning_found = logs
            .iter()
            .any(|(level, _, msg)| level == "WARN" && msg.contains("Test warning message"));
        let debug_found = logs.iter().any(|(level, target, msg)| {
            level == "DEBUG" && target == "test_target" && msg.contains("Debug with target: 42")
        });

        assert!(warning_found, "Warning message not captured");
        assert!(debug_found, "Debug message not captured");
    }

    #[test]
    fn test_macro_api_compatibility() {
        // These should compile without errors, proving API compatibility
        error!("Error message");
        warn!("Warning message");
        info!("Info message");
        debug!("Debug message");
        trace!("Trace message");

        error!(target: "custom", "Error with target");
        warn!(target: "custom", "Warning with target");
        info!(target: "custom", "Info with target");
        debug!(target: "custom", "Debug with target");
        trace!(target: "custom", "Trace with target");

        // Test formatting
        let val = 42;
        warn!("Formatted message: {}", val);
        debug!(target: "fmt", "Multiple args: {} and {}", "hello", "world");
    }
}
