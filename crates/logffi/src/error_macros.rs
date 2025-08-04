//! Enhanced error handling with automatic LogFFI integration
//!
//! This module provides the `define_errors!` macro that generates:
//! - Complete thiserror::Error implementation
//! - Automatic LogFFI integration with appropriate levels and targets
//! - Source error chaining with std::error::Error
//! - FFI-friendly error mapping for cross-language consistency
//! - Constructor methods for each variant

/// Log level enumeration for error macro integration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// `define_errors!` macro with LogFFI integration
#[macro_export]
macro_rules! define_errors {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                #[error($msg:literal $(, code = $code:literal)? $(, level = $level:ident)? $(, target = $target:literal)? $(, source = $source:ty)?)]
                $variant:ident $({ $($field:ident : $field_type:ty),* $(,)? })?,
            )*
        }
    ) => {
        // Generate thiserror Error enum with automatic derives
        #[derive(thiserror::Error, Debug)]
        $(#[$meta])*
        $vis enum $name {
            $(
                #[error($msg)]
                $variant $({
                    $($field : $field_type,)*
                    $(source: Option<Box<$source>>,)?
                })?,
            )*
        }

        impl $name {
            /// Get error code for API stability and FFI mapping
            pub fn code(&self) -> &'static str {
                match self {
                    $(
                        $name::$variant $({ .. })? => {
                            $crate::define_errors!(@code $($code)?, $variant)
                        }
                    )*
                }
            }

            /// Get error type identifier for FFI mapping
            pub fn kind(&self) -> &'static str {
                match self {
                    $(
                        $name::$variant $({ .. })? => stringify!($variant),
                    )*
                }
            }

            /// Automatically log this error using LogFFI with full backend support
            pub fn log(&self) {
                match self {
                    $(
                        $name::$variant $({ $($field,)* source, .. })? => {
                            let level = $crate::define_errors!(@level $($level)?);
                            let target = $crate::define_errors!(@target $($target)?);
                            let code = self.code();

                            // Use our universal logging macros
                            match level {
                                $crate::LogLevel::Error => {
                                    $crate::error!(
                                        target: target,
                                        "Error [{}:{}]: {} $(- $({}: {:?})*)?",
                                        code,
                                        stringify!($variant),
                                        $msg
                                        $(, $($field)*)?
                                    );
                                }
                                $crate::LogLevel::Warn => {
                                    $crate::warn!(
                                        target: target,
                                        "Warning [{}:{}]: {} $(- $({}: {:?})*)?",
                                        code,
                                        stringify!($variant),
                                        $msg
                                        $(, $($field)*)?
                                    );
                                }
                                $crate::LogLevel::Info => {
                                    $crate::info!(
                                        target: target,
                                        "Info [{}:{}]: {} $(- $({}: {:?})*)?",
                                        code,
                                        stringify!($variant),
                                        $msg
                                        $(, $($field)*)?
                                    );
                                }
                                $crate::LogLevel::Debug => {
                                    $crate::debug!(
                                        target: target,
                                        "Debug [{}:{}]: {} $(- $({}: {:?})*)?",
                                        code,
                                        stringify!($variant),
                                        $msg
                                        $(, $($field)*)?
                                    );
                                }
                                $crate::LogLevel::Trace => {
                                    $crate::trace!(
                                        target: target,
                                        "Trace [{}:{}]: {} $(- $({}: {:?})*)?",
                                        code,
                                        stringify!($variant),
                                        $msg
                                        $(, $($field)*)?
                                    );
                                }
                            }

                            // Log source errors at debug level
                            $(
                                if let Some(ref src) = source {
                                    $crate::debug!(target: target, "Source error: {}", src);
                                }
                            )?
                        }
                    )*
                }
            }

            /// Create and immediately log error variants
            $(
                paste::paste! {
                    pub fn [<new_ $variant:snake>]($($($field: $field_type,)* $(source: $source,)?)?) -> Self {
                        let error = $name::$variant $({
                            $($field,)*
                            $(source: Some(Box::new(source)),)?
                        })?;
                        error.log();
                        error
                    }
                }
            )*
        }

        // Implement std::error::Error with source chaining
        impl std::error::Error for $name {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match self {
                    $(
                        $name::$variant $({ $($field,)* source, .. })? => {
                            $(source.as_ref().map(|s| s.as_ref() as &(dyn std::error::Error + 'static)))?
                            #[allow(unreachable_code)]
                            None
                        }
                    )*
                }
            }
        }
    };

    // Helper macros for defaults and code generation
    (@code $code:literal, $variant:ident) => { $code };
    (@code $variant:ident) => {
        // Auto-generate code from variant name: KeyNotFound -> "KEY_NOT_FOUND"
        stringify!($variant)
    };
    (@level) => { $crate::LogLevel::Error };
    (@level error) => { $crate::LogLevel::Error };
    (@level warn) => { $crate::LogLevel::Warn };
    (@level info) => { $crate::LogLevel::Info };
    (@level debug) => { $crate::LogLevel::Debug };
    (@level trace) => { $crate::LogLevel::Trace };
    (@target) => { "app" };
    (@target $target:literal) => { $target };
}

// Note: Example usage of define_errors! macro is provided in crate documentation
