# LogFFI v0.2.0: Tracing Superset with Error Handling & Optional FFI Bridge

## Overview

Transform LogFFI from a multi-backend logging framework into a **tracing superset crate** that provides:

1. **Auto-initialization** - logs "just work" out of the box
2. **Complete tracing replacement** - users never need to import tracing directly
3. **Enhanced error handling** - define_errors! macro with error source chain preservation
4. **Optional FFI callback support** - zero-overhead by default, opt-in for FFI bridges

## Current Problems

1. **Multi-backend complexity** - managing log/tracing/callback backends
2. **Silent by default** - users get no output without manual initialization
3. **Feature flag confusion** - which backend to choose?
4. **Incomplete tracing coverage** - missing span macros and advanced features

## Proposed Architecture

### New Structure

```
src/
├── lib.rs           # Main exports and auto-init logic
├── tracing.rs       # Complete tracing re-exports and wrapped macros
├── callback.rs      # FFI callback functionality  
├── macros.rs        # define_errors! macro only
└── (remove backend/ folder entirely)
```

### Core Principles

1. **Tracing superset** - complete tracing API with enhancements
2. **Auto-initialization** - sensible defaults, override-friendly
3. **Zero-overhead by default** - callback support is opt-in via feature flag
4. **Error source chain preservation** - maintain full error context in tracing
5. **Complete API coverage** - all tracing macros, types, functions

## Implementation Plan

### Phase 1: Remove Backend System

- [ ] Delete entire `src/backend/` folder
- [ ] Remove backend-related code from lib.rs
- [ ] Remove feature flags from Cargo.toml (keep only tracing, tracing-subscriber)

### Phase 2: Create New Module Structure

#### 2.1 `src/tracing.rs` - Complete tracing wrapper:

```rust
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
```

#### 2.2 `src/callback.rs` - Optional FFI callback system:

```rust
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "callback")] {
        use std::sync::Mutex;
        
        /// Callback function type for FFI bridges
        pub type Callback = Box<dyn Fn(&str, &str, &str) + Send + Sync>;
        
        /// Global callback storage
        static CALLBACK: Mutex<Option<Callback>> = Mutex::new(None);
        
        /// Set callback for bridging logs to other systems (Python, Node.js, etc.)
        pub fn set(callback: Callback) {
            let mut guard = CALLBACK.lock().unwrap();
            *guard = Some(callback);
        }
        
        /// Internal function to call the callback if set
        pub(crate) fn call(level: &str, target: &str, message: &str) {
            if let Ok(guard) = CALLBACK.lock() {
                if let Some(callback) = guard.as_ref() {
                    callback(level, target, message);
                }
            }
        }
    }
}
```

#### 2.3 `src/macros.rs` - Enhanced define_errors! macro:

```rust
// REMOVE: LogLevel enum - use tracing::Level directly
// REMOVE: for_each! logging macro generation 
// KEEP: Only define_errors! implementation
// ENHANCE: Pass full error object to tracing (preserve source chain)

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
    
    // Generate @do_log helper patterns using for_each! with transforms - no paste! needed
    for_each!([error, warn, info, debug, trace], |level| {
        // Pattern with both level and target
        (@do_log level = %{level}, target = $target:literal, $error:expr) => {
            $crate::%{level}!(
                target: $target,
                error = $error as &dyn std::error::Error,
                error.code = $error.code(),
                "%{level:title}: {}", $error
            );
        };
        
        // Pattern with level only (default target)
        (@do_log level = %{level}, $error:expr) => {
            $crate::%{level}!(
                target: "app",
                error = $error as &dyn std::error::Error,
                error.code = $error.code(),
                "%{level:title}: {}", $error
            );
        };
    });
    
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
```

#### 2.4 `src/lib.rs` - Main exports:

```rust
//! # LogFFI - Enhanced Logging Bridge for Rust
//! 
//! A tracing-native bridge that provides auto-initialization, complete tracing API coverage,
//! enhanced error handling, and FFI callback support.

mod tracing;
mod callback;
mod macros;

// Re-export everything for complete tracing replacement
pub use crate::tracing::*;       // All tracing functionality with auto-init
pub use crate::callback::{set as set_callback, Callback};
pub use crate::macros::define_errors;

// Make ensure_logging_initialized available internally
pub(crate) use crate::tracing::ensure_logging_initialized;
```

### Phase 3: Cargo.toml Simplification

```toml
[features]
default = ["tracing"]
tracing = ["dep:tracing", "dep:tracing-subscriber"]
callback = [] # Optional FFI callback support

[dependencies]
# Core tracing (optional via features)
tracing = { version = "0.1.41", optional = true }
tracing-subscriber = { version = "0.3.19", features = ["env-filter", "json"], optional = true }

# For macro generation, error handling, and conditional compilation
meta-rust = { path = "../meta-rust" }
thiserror = "2.0.12"
cfg-if = "1.0"

# Remove: log, env_logger, paste (not needed)
```

### Phase 4: Testing & Validation

#### 4.1 Build Testing

- [ ] `cargo build` - basic compilation
- [ ] `moon logffi:build` - integration with build system

#### 4.2 Functionality Testing

- [ ] Auto-initialization works (logs appear by default)
- [ ] Override works (custom subscriber takes precedence)
- [ ] All macros work (all 11+ tracing macros)
- [ ] Attribute macros work (#[instrument])
- [ ] Callback system works
- [ ] define_errors! macro works with tracing::Level

#### 4.3 User Experience Testing

```rust
// Test 1: Zero-config usage
use logffi::{info, error};
fn main() {
    info!("Should appear immediately");
}

// Test 2: Complete tracing replacement  
use logffi::{info, span, Level, fmt, instrument};
#[instrument]
fn test() { info!("Works like tracing"); }

// Test 3: Enhanced errors with structured logging
use logffi::{define_errors, Level};
define_errors! {
    pub enum TestError {
        #[error("Database connection failed", level = error, source)]
        DatabaseError { 
            #[source]
            cause: std::io::Error 
        },
        
        #[error("Config not found: {path}", level = warn, target = "config")]
        ConfigNotFound { path: String },
    }
}

fn test_structured_errors() {
    let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused");
    let db_err = TestError::DatabaseError { cause: io_err };
    
    db_err.log(); // Should log with structured fields: error, error.code, and full source chain
    
    // Expected tracing output:
    // ERROR TestError: Error: Database connection failed
    //   error=DatabaseError error.code=DatabaseError
    //   source: Connection refused (kind: ConnectionRefused)
}

// Test 4: FFI callback integration
use logffi::{info, set_callback};
fn test_callback() {
    // Set up callback (always available, no feature flag)
    set_callback(Box::new(|level, target, message| {
        println!("CALLBACK: [{level}] {target}: {message}");
    }));
    
    info!("This goes to both tracing AND callback");
    
    // Expected output:
    // 2024-01-01T12:00:00Z INFO test: This goes to both tracing AND callback
    // CALLBACK: [info] test: This goes to both tracing AND callback
}
```

## Benefits

1. **🎯 Just Works** - logs appear immediately with auto-initialization
2. **🔄 Complete Tracing Superset** - full tracing API plus enhancements through logffi
3. **🗑️ No Redundancy** - use tracing::Level, not custom LogLevel
4. **⚡ Zero Overhead by Default** - callback support is opt-in via feature flag
5. **📦 Modular** - choose tracing-only, callback-only, or both via features
6. **🔗 Error Source Chain Preservation** - maintains full error context in tracing
7. **📊 Rich Error Data** - error objects, codes, and source chains available to tracing backends
8. **🧹 Clean Dependencies** - cfg-if replaces scattered conditional compilation

## Success Criteria

1. **✅ Zero-config works**: Immediate log output with tracing feature
2. **✅ Complete tracing superset**: All tracing functionality available plus enhancements
3. **✅ Use tracing types**: No custom LogLevel enum
4. **✅ Smart auto-init**: Proper subscriber detection
5. **✅ Zero-overhead by default**: Callback support via opt-in feature
6. **✅ Modular features**: Works with tracing-only, callback-only, or both
7. **✅ Error source chain preservation**: Full error objects with source chains in tracing
8. **✅ Clean conditional compilation**: cfg-if instead of scattered #[cfg] attributes

## Implementation Order

1. **Remove backend folder**
2. **Create tracing.rs** - Smart auto-init + all wrapped macros
3. **Extract callback.rs** - Clean FFI system
4. **Update macros.rs** - define_errors! only, use tracing::Level
5. **Update lib.rs** - Clean exports
6. **Update Cargo.toml** - No features
7. **Test and validate**
8. **Update docs**
