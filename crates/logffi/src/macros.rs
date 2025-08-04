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


/// Enhanced `define_errors!` macro with LogFFI integration and field support
#[macro_export]
macro_rules! define_errors {
    // Handle both simple variants and variants with fields
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident $({
                    $(
                        $field_name:ident: $field_type:ty,
                    )*
                })?,
            )*
        }
    ) => {
        // Generate thiserror Error enum with field support
        #[derive(thiserror::Error, Debug)]
        $(#[$enum_meta])*
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant $({
                    $(
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
                        Self::$variant { .. } => stringify!($variant),
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

            /// Automatically log this error using LogFFI with full backend support
            pub fn log(&self) {
                $crate::error!("Error [{}]: {}", self.code(), self.full_message_chain());
            }
        }
    };
}