// Enhanced define_errors! macro with structured tracing integration
// This module ONLY contains the define_errors! macro - all logging macros are in tracing.rs

/// Enhanced `define_errors!` macro with structured tracing integration
#[macro_export]
macro_rules! define_errors {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident {
            $(
                #[error($msg:literal $(, level = $level:ident)? $(, target = $target:literal)? $(, source)?)]
                $variant:ident $({
                    $(
                        $(#[$field_meta:meta])*
                        $field_name:ident: $field_type:ty,
                    )*
                })?,
            )*
        }
    ) => {
        // Generate thiserror Error enum with source chain support
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
            /// Automatically log this error with structured tracing (preserves source chain)
            pub fn log(&self) {
                match self {
                    $(
                        Self::$variant { .. } => {
                            // Pass the full error object to tracing for structured logging
                            define_errors!(@do_log $(level = $level,)? $(target = $target,)? self);
                        },
                    )*
                }
            }
            
            /// Get error code for API stability
            pub fn code(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant { .. } => stringify!($variant),
                    )*
                }
            }
        }
    };
    
    // Call the main logging macros directly - with target
    (@do_log level = error, target = $target:literal, $error:expr) => {
        $crate::error!(
            target: $target,
            error = $error as &dyn std::error::Error,
            error.code = $error.code(),
            "Error: {}", $error
        );
    };
    (@do_log level = warn, target = $target:literal, $error:expr) => {
        $crate::warn!(
            target: $target,
            error = $error as &dyn std::error::Error,
            error.code = $error.code(),
            "Warn: {}", $error
        );
    };
    (@do_log level = info, target = $target:literal, $error:expr) => {
        $crate::info!(
            target: $target,
            error = $error as &dyn std::error::Error,
            error.code = $error.code(),
            "Info: {}", $error
        );
    };
    (@do_log level = debug, target = $target:literal, $error:expr) => {
        $crate::debug!(
            target: $target,
            error = $error as &dyn std::error::Error,
            error.code = $error.code(),
            "Debug: {}", $error
        );
    };
    (@do_log level = trace, target = $target:literal, $error:expr) => {
        $crate::trace!(
            target: $target,
            error = $error as &dyn std::error::Error,
            error.code = $error.code(),
            "Trace: {}", $error
        );
    };
    
    // Call the main logging macros directly - without target (default target)
    (@do_log level = error, $error:expr) => {
        $crate::error!(
            target: "app",
            error = $error as &dyn std::error::Error,
            error.code = $error.code(),
            "Error: {}", $error
        );
    };
    (@do_log level = warn, $error:expr) => {
        $crate::warn!(
            target: "app",
            error = $error as &dyn std::error::Error,
            error.code = $error.code(),
            "Warn: {}", $error
        );
    };
    (@do_log level = info, $error:expr) => {
        $crate::info!(
            target: "app",
            error = $error as &dyn std::error::Error,
            error.code = $error.code(),
            "Info: {}", $error
        );
    };
    (@do_log level = debug, $error:expr) => {
        $crate::debug!(
            target: "app",
            error = $error as &dyn std::error::Error,
            error.code = $error.code(),
            "Debug: {}", $error
        );
    };
    (@do_log level = trace, $error:expr) => {
        $crate::trace!(
            target: "app",
            error = $error as &dyn std::error::Error,
            error.code = $error.code(),
            "Trace: {}", $error
        );
    };
    
    // Default patterns (error level, default target)
    (@do_log target = $target:literal, $error:expr) => {
        $crate::error!(
            target: $target,
            error = $error as &dyn std::error::Error,
            error.code = $error.code(),
            "Error: {}", $error
        );
    };
    (@do_log $error:expr) => {
        $crate::error!(
            error = $error as &dyn std::error::Error,
            error.code = $error.code(),
            "Error: {}", $error
        );
    };
}