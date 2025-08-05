use cfg_if::cfg_if;
use meta_rust::for_each;

cfg_if! {
    if #[cfg(feature = "tracing")] {
        use std::sync::Once;

        static INIT: Once = Once::new();

        /// Check if tracing subscriber is already set
        fn has_active_subscriber() -> bool {
            std::panic::catch_unwind(|| {
                tracing::subscriber::with_default(
                    tracing::subscriber::NoSubscriber::default(),
                    || {}
                )
            }).is_err() // If panics, subscriber is already set
        }

        /// Auto-initialization with smart defaults
        pub fn ensure_logging_initialized() {
            INIT.call_once(|| {
                if !has_active_subscriber() {
                    let env_filter = std::env::var("RUST_LOG")
                        .unwrap_or_else(|_| "info".to_string());

                    let _ = tracing_subscriber::fmt()
                        .with_env_filter(env_filter)
                        .try_init();
                }
            });
        }

        // Event macros with auto-init AND callback routing
        for_each!([
            trace, debug, info, warn, error, event
        ], |macro_name| {
            #[macro_export]
            macro_rules! %{macro_name} {
                // Pattern for structured logging: fields followed by message
                (target: $target:expr, $($field:ident = $value:expr,)* $fmt:literal $(, $($arg:expr),*)?) => {
                    {
                        $crate::ensure_logging_initialized();

                        // Call tracing macro with structured fields
                        ::tracing::%{macro_name}!(target: $target, $($field = $value,)* $fmt $(, $($arg),*)?);

                        // Call FFI callback only if feature enabled (zero-overhead by default)
                        #[cfg(feature = "callback")]
                        {
                            let message = format!($fmt $(, $($arg),*)?);
                            $crate::callback::call(stringify!(%{macro_name}), $target, &message);
                        }
                    }
                };
                // Pattern for simple message (backwards compatibility)
                (target: $target:expr, $fmt:literal $(, $($arg:expr),*)?) => {
                    {
                        $crate::ensure_logging_initialized();

                        // Call tracing macro
                        ::tracing::%{macro_name}!(target: $target, $fmt $(, $($arg),*)?);

                        // Call FFI callback only if feature enabled (zero-overhead by default)
                        #[cfg(feature = "callback")]
                        {
                            let message = format!($fmt $(, $($arg),*)?);
                            $crate::callback::call(stringify!(%{macro_name}), $target, &message);
                        }
                    }
                };
                // Pattern for any other syntax - pass through (fallback for complex cases)
                (target: $target:expr, $($arg:tt)*) => {
                    {
                        $crate::ensure_logging_initialized();

                        // Call tracing macro
                        ::tracing::%{macro_name}!(target: $target, $($arg)*);

                        // Call FFI callback only if feature enabled (zero-overhead by default)
                        #[cfg(feature = "callback")]
                        {
                            // For complex syntax, just use a generic message
                            let message = concat!("Complex log: ", stringify!($($arg)*));
                            $crate::callback::call(stringify!(%{macro_name}), $target, message);
                        }
                    }
                };
                // Delegate to target version with module_path!
                ($($arg:tt)*) => {
                    $crate::%{macro_name}!(target: module_path!(), $($arg)*)
                };
            }
        });

        // Span macros (return Span objects, no callback routing needed)
        for_each!([
            trace_span, debug_span, info_span, warn_span, error_span, span
        ], |macro_name| {
            #[macro_export]
            macro_rules! %{macro_name} {
                // Pattern for structured span creation: fields with name
                (target: $target:expr, $name:expr, $($field:ident = $value:expr),* $(,)?) => {
                    {
                        $crate::ensure_logging_initialized();
                        ::tracing::%{macro_name}!(target: $target, $name, $($field = $value),*)
                    }
                };
                // Pattern for simple span creation: just name
                (target: $target:expr, $name:expr) => {
                    {
                        $crate::ensure_logging_initialized();
                        ::tracing::%{macro_name}!(target: $target, $name)
                    }
                };
                // Pattern for span with level (for generic span! macro)
                (target: $target:expr, $level:expr, $name:expr, $($field:ident = $value:expr),* $(,)?) => {
                    {
                        $crate::ensure_logging_initialized();
                        ::tracing::%{macro_name}!(target: $target, $level, $name, $($field = $value),*)
                    }
                };
                (target: $target:expr, $level:expr, $name:expr) => {
                    {
                        $crate::ensure_logging_initialized();
                        ::tracing::%{macro_name}!(target: $target, $level, $name)
                    }
                };
                // Delegate to target version with module_path!
                ($($arg:tt)*) => {
                    $crate::%{macro_name}!(target: module_path!(), $($arg)*)
                };
            }
        });

        // Direct re-exports (no wrapping needed)
        pub use tracing::{
            // Types
            Level, Event, Span, Id, Metadata, Dispatch,
            // Traits
            Subscriber,
            // Functions
            subscriber,
            // Attribute macros
            instrument
        };

        // Full tracing-subscriber re-export for complete replacement
        pub use tracing_subscriber::{
            fmt, filter, layer, registry, reload, util,
            EnvFilter, Registry
        };
    }
}
