# LogFFI Refactoring Plan: From Framework to Bridge

## Current Issues

1. **Auto-initialization complexity** - We try to initialize loggers automatically
2. **Backend management overhead** - We maintain backend structs that don't add value
3. **Misleading API** - `logger()` suggests we provide a logger, but we don't
4. **Comparison positioning** - We position as competing with log/tracing instead of complementing

## Proposed Changes

### 1. Remove Auto-initialization

**Current:**

```rust
impl LogBackendTrait for LogBackend {
    fn init(&self) {
        if log::max_level() == log::LevelFilter::Off {
            let _ = env_logger::try_init();  // This is presumptuous!
        }
    }
}
```

**Proposed:**

- Remove all `init()` methods
- User MUST initialize their chosen logger
- We just forward macro calls

### 2. Simplify Architecture

**Current:**

```rust
pub struct LogFFI {
    log_backend: LogBackend,
    tracing_backend: TracingBackend,
    // etc...
}
```

**Proposed:**

```rust
// No struct needed! Just feature-gated macro implementations
```

### 3. Macro Implementation with cfg-if

**Current:**

```rust
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        let _ = $crate::logger();  // Unnecessary!
        
        #[cfg(feature = "log")]
        ::log::info!($($arg)*);
        
        #[cfg(feature = "tracing")]
        ::tracing::info!($($arg)*);
    }
}
```

**Proposed with cfg-if:**

```rust
#[macro_export]
macro_rules! info {
    (target: $target:expr, $($arg:tt)*) => {
        $crate::cfg_if::cfg_if! {
            if #[cfg(feature = "log")] {
                ::log::info!(target: $target, $($arg)*);
            } else if #[cfg(feature = "tracing")] {
                ::tracing::info!(target: $target, $($arg)*);
            }
        }
        
        #[cfg(feature = "callback")]
        $crate::log_to_callback("info", $target, &format!($($arg)*));
    };
    ($($arg:tt)*) => {
        $crate::info!(target: module_path!(), $($arg)*)
    };
}
```

**Benefits of cfg-if:**

- Cleaner conditional compilation
- Ensures only one backend is used (with else-if chain)
- More readable than multiple #[cfg] blocks
- Can add compile error if no backend selected

### Handling Uninitialized Loggers

**How log/tracing handle it:**

- `log` crate returns a `NopLogger` when no logger is set
- `tracing` has a no-op subscriber system
- Both silently drop messages when uninitialized

**Our approach:**

- Do nothing! Just forward to log/tracing
- They already handle the uninitialized case properly
- No need for any special handling in LogFFI

### 4. Remove Unnecessary APIs

**Remove:**

- `logger()` function
- `LogFFI` struct
- `auto_init()` method
- `as_log()`, `as_tracing()`, etc.
- `available_backends()`
- Backend trait and implementations

**Keep:**

- Logging macros (error!, warn!, info!, debug!, trace!)
- `define_errors!` macro
- FFI callback functionality
- Error handling features

### 5. Simplified lib.rs Structure

```rust
//! LogFFI - Enhanced logging bridge for Rust

// Just re-export what users need
pub use crate::macros::{error, warn, info, debug, trace};
pub use crate::errors::{define_errors, LogLevel};

#[cfg(feature = "callback")]
pub use crate::callback::{set, Callback};

// That's it! No complex initialization, no backend management
```

## Benefits

1. **Simpler** - Less code to maintain
2. **Clearer** - Obviously a bridge, not a framework
3. **Honest** - We don't hide what we're doing
4. **Flexible** - Users have full control

## Migration Impact

This is a breaking change, but since we haven't released yet, it's the right time to make it.

## Implementation Steps

1. Remove backend module and all its implementations
2. Simplify macro implementations to remove initialization line only
3. Remove LogFFI struct and related functions
4. Update documentation
5. Update examples
6. Keep slog support (user must provide their own logger reference)
7. Keep callback at crate level (not namespaced into ffi module)
