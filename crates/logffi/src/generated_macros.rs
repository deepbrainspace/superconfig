// Generated logging macros using meta-rust for_each! macro
//
// This file contains the macro generation code that gets included at the crate root level.
// We use the include! pattern to keep macro generation in a separate file while ensuring
// it executes at the crate root where procedural macros are allowed.

use meta_rust::for_each;

// Generate all logging macros using meta-rust for_each! macro - super clean!
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