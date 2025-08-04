// LogFFI Macro System
//
// This module provides:
// - Universal logging macros (error!, warn!, info!, debug!, trace!) generated with meta-rust
// - Enhanced error handling with define_errors! macro
// - LogLevel enum for client error definitions
// - Complete thiserror::Error integration with automatic LogFFI logging

use meta_rust::for_each;

/// Log level enumeration for error macro integration  
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

// Generate all logging macros using meta-rust for_each! macro
for_each!([error, warn, info, debug, trace], |level| {
    #[macro_export]
    macro_rules! %{level} {
        (target: $target:expr, $($arg:tt)*) => {
            {
                let has_callback = $crate::CALLBACK.get().is_some();
                let force_native = $crate::FORCE_NATIVE_BACKENDS.load(std::sync::atomic::Ordering::Relaxed);

                // Always call callback if it exists (FFI, remote, custom routing)
                if has_callback {
                    let message = format!($($arg)*);
                    $crate::call_callback(stringify!(%{level:upper}), $target, &message);
                }

                // Call native backends if: no callback OR force_native is enabled (dual-mode)
                if !has_callback || force_native {
                    // Initialize logger to ensure backend is set up
                    let _ = $crate::logger();

                    match $crate::current_backend() {
                        $crate::Backend::Tracing => {
                            // Full tracing macro with all functionality
                            tracing::%{level}!(target: $target, $($arg)*);
                        }
                        $crate::Backend::Log => {
                            // Full log macro with all functionality
                            log::%{level}!(target: $target, $($arg)*);
                        }
                        $crate::Backend::Slog => {
                            // For slog, we need to access the logger instance
                            if let Some(slog_backend) = $crate::logger().as_slog() {
                                let message = format!($($arg)*);
                                slog::%{level}!(slog_backend, "{}", message; "target" => $target);
                            }
                        }
                    }
                }
            }
        };
        ($($arg:tt)*) => {
            $crate::%{level}!(target: module_path!(), $($arg)*)
        };
    }
});


/// Enhanced `define_errors!` macro with LogFFI integration, field support, log levels, targets, and source error chaining
#[macro_export]
macro_rules! define_errors {
    // Enhanced pattern with log level, target support, and source error chaining
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident {
            $(
                #[error($msg:literal $(, level = $level:ident)? $(, target = $target:literal)? $(, code = $code:literal)? $(, source)?)]
                $variant:ident $({
                    $(
                        $(#[$field_meta:meta])*
                        $field_name:ident: $field_type:ty,
                    )*
                })?,
            )*
        }
    ) => {
        // Generate thiserror Error enum with field support and source chaining
        #[derive(thiserror::Error, Debug)]
        $(#[$enum_meta])*
        $vis enum $name {
            $(
                #[error($msg)]
                $variant $({
                    $(
                        $(#[$field_meta])*
                        $field_name: $field_type,
                    )*
                })?,
            )*
        }

        impl $name {
            /// Get error code for API stability and FFI mapping
            pub fn code(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant { .. } => define_errors!(@code $(code = $code,)? variant = $variant),
                    )*
                }
            }

            /// Get error type identifier for FFI mapping
            pub fn kind(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant { .. } => stringify!($variant),
                    )*
                }
            }

            /// Get full error message including source chain (for FFI)
            pub fn full_message_chain(&self) -> String {
                self.to_string()
            }

            /// Automatically log this error using LogFFI with appropriate level and target
            pub fn log(&self) {
                match self {
                    $(
                        Self::$variant { $($($field_name,)*)? .. } => {
                            let code = self.code();
                            let message = self.full_message_chain();
                            
                            // Generate logging call with compile-time level and target resolution
                            define_errors!(@do_log $(level = $level,)? $(target = $target,)? code, message);
                        },
                    )*
                }
            }
            
            // Generate constructor methods for each variant
            $(
                paste::paste! {
                    /// Create and log a new error instance
                    pub fn [<new_ $variant:snake>]($($($field_name: $field_type),*)?) -> Self {
                        let error = $name::$variant $({
                            $($field_name,)*
                        })?;
                        error.log();
                        error
                    }
                }
            )*
        }
    };
    
    // Helper macro for error code resolution
    (@code code = $code:literal, variant = $variant:ident) => { $code };
    (@code variant = $variant:ident) => { stringify!($variant) };
    
    // Helper macro for log level resolution with default
    (@level level = error,) => { $crate::LogLevel::Error };
    (@level level = warn,) => { $crate::LogLevel::Warn };
    (@level level = info,) => { $crate::LogLevel::Info };
    (@level level = debug,) => { $crate::LogLevel::Debug };
    (@level level = trace,) => { $crate::LogLevel::Trace };
    (@level) => { $crate::LogLevel::Error }; // Default level
    
    // Helper macro for target resolution with default
    (@target target = $target:literal,) => { $target };
    (@target) => { "app" }; // Default target
    
    // Helper macro for generating the actual logging calls with all combinations
    (@do_log level = error, target = $target:literal, $code:expr, $message:expr) => {
        $crate::error!(target: $target, "[{}] {}", $code, $message);
    };
    (@do_log level = warn, target = $target:literal, $code:expr, $message:expr) => {
        $crate::warn!(target: $target, "[{}] {}", $code, $message);
    };
    (@do_log level = info, target = $target:literal, $code:expr, $message:expr) => {
        $crate::info!(target: $target, "[{}] {}", $code, $message);
    };
    (@do_log level = debug, target = $target:literal, $code:expr, $message:expr) => {
        $crate::debug!(target: $target, "[{}] {}", $code, $message);
    };
    (@do_log level = trace, target = $target:literal, $code:expr, $message:expr) => {
        $crate::trace!(target: $target, "[{}] {}", $code, $message);
    };
    // Level only (default target)
    (@do_log level = error, $code:expr, $message:expr) => {
        $crate::error!(target: "app", "[{}] {}", $code, $message);
    };
    (@do_log level = warn, $code:expr, $message:expr) => {
        $crate::warn!(target: "app", "[{}] {}", $code, $message);
    };
    (@do_log level = info, $code:expr, $message:expr) => {
        $crate::info!(target: "app", "[{}] {}", $code, $message);
    };
    (@do_log level = debug, $code:expr, $message:expr) => {
        $crate::debug!(target: "app", "[{}] {}", $code, $message);
    };
    (@do_log level = trace, $code:expr, $message:expr) => {
        $crate::trace!(target: "app", "[{}] {}", $code, $message);
    };
    // Target only (default level = error)
    (@do_log target = $target:literal, $code:expr, $message:expr) => {
        $crate::error!(target: $target, "[{}] {}", $code, $message);
    };
    // Neither (both defaults)
    (@do_log $code:expr, $message:expr) => {
        $crate::error!(target: "app", "[{}] {}", $code, $message);
    };
}