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

        // Single for_each! for ALL tracing macros with auto-init AND callback routing
        for_each!([
            // Event macros
            trace, debug, info, warn, error,
            // Span macros  
            trace_span, debug_span, info_span, warn_span, error_span,
            // Generic macros
            span, event
        ], |macro_name| {
            #[macro_export]
            macro_rules! %{macro_name} {
                (target: $target:expr, $($arg:tt)*) => {
                    {
                        $crate::ensure_logging_initialized();
                        
                        // Call tracing macro
                        ::tracing::%{macro_name}!(target: $target, $($arg)*);
                        
                        // Call FFI callback only if feature enabled (zero-overhead by default)
                        #[cfg(feature = "callback")]
                        {
                            let message = format!($($arg)*);
                            $crate::callback::call(stringify!(%{macro_name}), $target, &message);
                        }
                    }
                };
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
            Subscriber, Collect,
            // Functions
            collect, dispatch, subscriber,
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