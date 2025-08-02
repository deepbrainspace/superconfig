# Native Logging Integration with Reusable log-ffi Crate

## Overview

Create a reusable `log-ffi` crate that extends the standard `log` crate with FFI callback support, then integrate it into SuperConfig. This provides transparent logging that works seamlessly across Rust, Python, and Node.js ecosystems with zero learning curve and maximum reusability.

## Architecture Goals

### Core Principles

- **Reusable Design**: Create `log-ffi` crate that any Rust library can use for FFI logging
- **Transparent Integration**: Drop-in replacement for `log` crate with identical macro names
- **Zero Learning Curve**: Uses familiar `warn!()`, `debug!()` macros - no new API to learn
- **Enterprise Ready**: Works with ELK, EFK, Datadog, Splunk, and all major logging infrastructure
- **Minimal Code**: ~100 lines for log-ffi crate, ~20 lines integration per project
- **Performance Optimized**: Macros compile away when disabled, FFI overhead only on actual log events

### Performance Targets

- **Rust native**: 10-50ns per log call (0ns when disabled)
- **Python FFI**: 1-5μs per log call (acceptable for errors)
- **Node.js FFI**: 0.5-2μs per log call (acceptable for errors)
- **Memory overhead**: Zero (no custom storage)

## Detailed Design

### 1. Core Superconfig Changes

**Replace println! with structured logging:**

```rust
// Before (current)
pub fn enable(self: Arc<Self>, flags: u64) -> Arc<Self> {
    if !is_valid_runtime_flag(flags) {
        println!("Error: Invalid runtime flag: 0x{flags:X}");
        return self;
    }
    self
}

// After (new approach with ergonomic methods)
pub fn enable(self: Arc<Self>, flags: u64) -> Arc<Self> {
    if !is_valid_runtime_flag(flags) {
        self.warn("superconfig.flags", &format!("Invalid runtime flag: 0x{flags:X}"));
        return self;
    }
    
    self.debug("superconfig.flags", &format!("Enabled runtime flags: 0x{flags:X}"));
    self
}

// Ergonomic logging methods (internal use)
impl ConfigRegistry {
    fn warn(&self, target: &str, message: &str) {
        self.log_message(LogLevel::Warn, target, message);
    }
    
    fn debug(&self, target: &str, message: &str) {
        self.log_message(LogLevel::Debug, target, message);
    }
    
    // ... other levels (error, info, trace)
    
    fn log_message(&self, level: LogLevel, target: &str, message: &str) {
        // Check our log level setting first
        if level as u8 <= self.get_log_level() as u8 {
            // Native Rust logging
            match level {
                LogLevel::Warn => log::warn!(target: target, "{}", message),
                LogLevel::Debug => log::debug!(target: target, "{}", message),
                // ... other levels
            }
            
            // FFI logging
            #[cfg(feature = "ffi")]
            if let Some(logger) = FFI_LOGGER.get() {
                logger(level.as_str(), target, message);
            }
        }
    }
}
```

### 2. FFI Integration Architecture

**Rust Core FFI Support:**

```rust
#[cfg(feature = "ffi")]
use std::sync::OnceLock;

#[cfg(feature = "ffi")]
type FfiLogger = Box<dyn Fn(&str, &str, &str) + Send + Sync>;
// Parameters: (level, target, message)

#[cfg(feature = "ffi")]
static FFI_LOGGER: OnceLock<FfiLogger> = OnceLock::new();

#[cfg(feature = "ffi")]
pub fn set_ffi_logger(logger: FfiLogger) {
    FFI_LOGGER.set(logger).ok();
}
```

**Python Integration (in superconfig-python-ffi):**

```rust
use pyo3::prelude::*;

#[pymodule]
fn superconfig(_py: Python, m: &PyModule) -> PyResult<()> {
    // Auto-setup Python logging bridge on module import
    setup_python_logging_bridge()?;
    
    // Register all the existing PyO3 classes/functions
    m.add_class::<ConfigRegistry>()?;
    Ok(())
}

fn setup_python_logging_bridge() -> PyResult<()> {
    let callback = |level: &str, target: &str, message: &str| {
        Python::with_gil(|py| {
            if let Ok(logging) = py.import("logging") {
                if let Ok(logger) = logging.call_method1("getLogger", (target,)) {
                    let _ = match level {
                        "WARN" => logger.call_method1("warning", (message,)),
                        "DEBUG" => logger.call_method1("debug", (message,)),
                        "INFO" => logger.call_method1("info", (message,)),
                        "TRACE" => logger.call_method1("debug", (message,)), // Python doesn't have TRACE
                        _ => logger.call_method1("info", (message,)),
                    };
                }
            }
        });
    };
    
    superconfig_core::set_ffi_logger(Box::new(callback));
    Ok(())
}
```

**Node.js Integration (in superconfig-node-ffi):**

```rust
use neon::prelude::*;

pub fn setup_node_logging_bridge(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // Get the Node.js callback function
    let js_callback = cx.argument::<JsFunction>(0)?;
    let callback_ref = js_callback.root(&mut cx);
    
    let callback = move |level: &str, target: &str, message: &str| {
        // Emit to Node.js event system
        // Implementation details for calling JS from Rust thread
    };
    
    superconfig_core::set_ffi_logger(Box::new(callback));
    Ok(cx.undefined())
}
```

### 3. Client Usage Patterns

**Rust Client:**

```rust
// Standard Rust logging setup
use log::LevelFilter;
use env_logger::Builder;

Builder::new()
    .filter_level(LevelFilter::Warn)
    .filter_module("superconfig", LevelFilter::Debug)
    .init();

// Or use tracing for structured logging
use tracing_subscriber::{fmt, EnvFilter};
fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .json()
    .init();

// SuperConfig automatically uses client's logging setup
let registry = ConfigRegistry::new().enable(invalid_flag);
```

**Python Client:**

```python
import logging
import superconfig

# Standard Python logging - works with any handler/formatter
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

# Or enterprise ELK integration
from pythonjsonlogger import jsonlogger
handler = logging.StreamHandler()
handler.setFormatter(jsonlogger.JsonFormatter())
logging.getLogger("superconfig").addHandler(handler)

# SuperConfig errors flow through Python's logging system
registry = superconfig.ConfigRegistry()
registry.enable(invalid_flag)  # -> logging.getLogger("superconfig.flags").warning(...)
```

**Node.js Client:**

```javascript
const winston = require('winston');
const superconfig = require('superconfig');

// Standard Winston setup
const logger = winston.createLogger({
    level: 'debug',
    format: winston.format.combine(
        winston.format.timestamp(),
        winston.format.json()
    ),
    transports: [
        new winston.transports.Console(),
        new winston.transports.File({ filename: 'app.log' })
    ]
});

// Bridge SuperConfig to Winston
superconfig.setLogHandler((level, target, message) => {
    logger.log(level.toLowerCase(), message, { target });
});

// SuperConfig errors flow through Winston
const registry = new superconfig.ConfigRegistry();
registry.enable(invalidFlag);  // -> winston logger handles it
```

## Implementation Plan

### Phase 0: Create Reusable log-ffi Crate (1-2 hours)

1. **Create log-ffi crate structure**
   - New independent crate: `/crates/log-ffi/`
   - Drop-in replacement for `log` crate with FFI support
   - Re-export all `log` crate functionality
   - Override macros (`warn!`, `debug!`, etc.) with FFI-enhanced versions

2. **Implement FFI callback system**
   - Thread-safe global callback storage
   - `set_ffi_callback()` function for FFI setup
   - Automatic FFI routing when callback is set
   - Respect `log` crate filtering (only call FFI if log would be enabled)

3. **Add comprehensive tests**
   - Test macro functionality matches `log` crate
   - Test FFI callback integration
   - Performance benchmarks

### Phase 1: Setup SuperConfig FFI Crates (1 hour)

1. **Create superconfig-python-ffi crate**
   - PyO3 bindings structure with proper Cargo.toml
   - Basic ConfigRegistry wrapper class
   - log-ffi bridge to Python's `logging` module

2. **Create superconfig-node-ffi crate**
   - Neon bindings structure with proper Cargo.toml
   - Basic ConfigRegistry wrapper class
   - log-ffi bridge to Node.js logging libraries

### Phase 2: SuperConfig Integration (1-2 hours)

1. **Replace log crate with log-ffi**
   - Update superconfig/Cargo.toml: `log-ffi = { path = "../log-ffi" }`
   - Replace all `println!` with `warn!()`, `debug!()`, etc.
   - Use identical macro syntax to `log` crate - zero learning curve

2. **Rename verbosity to log_level**
   - Change `verbosity()` method to `log_level()`
   - Integrate with `log` crate's filtering system
   - Update documentation and examples

3. **Add structured logging**
   - Use targets: "superconfig.flags", "superconfig.registry", etc.
   - Add contextual information with macro fields
   - Ensure proper log level usage throughout codebase

### Phase 3: Python FFI Integration (1 hour)

1. **Implement Python logging bridge**
   - Auto-setup in superconfig-python-ffi module initialization
   - Call `log_ffi::set_ffi_callback()` with Python bridge function
   - Route to Python's `logging.getLogger().warning()`, etc.

### Phase 4: Node.js FFI Integration (1 hour)

1. **Implement Node.js logging bridge**
   - Event-based bridge in superconfig-node-ffi
   - Call `log_ffi::set_ffi_callback()` with Node.js bridge function
   - Integration with popular Node.js logging libraries

### Phase 5: Testing & Documentation (1 hour)

1. **Integration tests**
   - Rust with different log implementations
   - Python with different logging setups
   - Node.js with popular logging libraries

2. **Update documentation**
   - Logging setup examples for each language
   - Enterprise integration patterns
   - Migration guide from current println! approach

## Logging Targets & Levels

### Target Hierarchy

- `superconfig` - General library messages
- `superconfig::flags` - Flag validation and operations
- `superconfig::registry` - Registry operations (create, read, update, delete)
- `superconfig::config` - Configuration loading and validation

### Log Levels Usage (Standard Hierarchy with Stacking)

- **ERROR** (1): Critical failures that prevent operation - always shown
- **WARN** (2): Invalid inputs, deprecated usage, recoverable errors - shows ERROR + WARN
- **INFO** (3): Major operations (registry creation, flag changes) - shows ERROR + WARN + INFO
- **DEBUG** (4): Detailed operation flow, performance metrics - shows ERROR + WARN + INFO + DEBUG
- **TRACE** (5): Verbose internal state (development only) - shows all levels

**Level Stacking**: Each level includes all lower levels (standard logging behavior)

## Dependencies

### Core superconfig/Cargo.toml additions:

```toml
[dependencies]
log = "0.4"

[features]
ffi = [] # Existing feature, no new deps needed

[dev-dependencies]
env_logger = "0.11" # For examples and tests
```

### No new dependencies for FFI crates

- Python: Uses existing PyO3 + Python's built-in `logging`
- Node.js: Uses existing Neon + Node's built-in event system

## Benefits Over ErrorRegistry Approach

### Advantages

1. **Ecosystem Integration**: Works with all existing logging infrastructure
2. **Zero Learning Curve**: Developers use familiar patterns
3. **Enterprise Ready**: ELK, Splunk, Datadog integration out of box
4. **Performance**: No memory overhead, logs compile away when disabled
5. **Minimal Code**: ~100 lines vs 1000+ for custom registry
6. **Industry Standard**: Follows established best practices

### Client Flexibility Examples

- **Development**: Console logging with colors and formatting
- **Production**: JSON logs to files with rotation
- **Monitoring**: Direct integration with ELK stack, Datadog, New Relic
- **Debugging**: Structured logs with correlation IDs, spans
- **Testing**: Capture logs for assertions

## Success Criteria

1. **Rust Integration**: Works with env_logger, tracing, and custom implementations
2. **Python Integration**: Works with logging, structlog, loguru
3. **Node.js Integration**: Works with winston, pino, bunyan
4. **Performance**: No measurable overhead when logging disabled
5. **Enterprise**: Successful ELK stack integration example
6. **Migration**: All existing println! statements converted

## Timeline

- **Total Estimate**: 6-8 hours
- **Phase 0**: 1-2 hours (Create reusable log-ffi crate)
- **Phase 1**: 1 hour (Setup SuperConfig FFI crates)
- **Phase 2**: 1-2 hours (SuperConfig integration)
- **Phase 3**: 1 hour (Python FFI logging)
- **Phase 4**: 1 hour (Node.js FFI logging)
- **Phase 5**: 1 hour (Testing & docs)

This approach provides professional, enterprise-ready logging that integrates seamlessly with existing infrastructure while requiring minimal implementation effort.
