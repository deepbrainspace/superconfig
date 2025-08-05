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
                            let code = self.code();
                            let message = self.to_string();
                            
                            // Pass separate code and message like the working version
                            define_errors!(@do_log $(level = $level,)? $(target = $target,)? code, message);
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
    
    // Call the main logging macros directly - with target and level
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
    
    // Call the main logging macros directly - level only (default target)
    (@do_log level = error, $code:expr, $message:expr) => {
        $crate::error!(target: module_path!(), "[{}] {}", $code, $message);
    };
    (@do_log level = warn, $code:expr, $message:expr) => {
        $crate::warn!(target: module_path!(), "[{}] {}", $code, $message);
    };
    (@do_log level = info, $code:expr, $message:expr) => {
        $crate::info!(target: module_path!(), "[{}] {}", $code, $message);
    };
    (@do_log level = debug, $code:expr, $message:expr) => {
        $crate::debug!(target: module_path!(), "[{}] {}", $code, $message);
    };
    (@do_log level = trace, $code:expr, $message:expr) => {
        $crate::trace!(target: module_path!(), "[{}] {}", $code, $message);
    };
    
    // Target only (default level = error)  
    (@do_log target = $target:literal, $code:expr, $message:expr) => {
        $crate::error!(target: $target, "[{}] {}", $code, $message);
    };
    
    // Neither level nor target (both defaults)
    (@do_log $code:expr, $message:expr) => {
        $crate::error!(target: module_path!(), "[{}] {}", $code, $message);
    };
}