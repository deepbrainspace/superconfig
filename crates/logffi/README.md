# LogFFI

[![Crates.io](https://img.shields.io/crates/v/logffi.svg)](https://crates.io/crates/logffi)
[![Documentation](https://docs.rs/logffi/badge.svg)](https://docs.rs/logffi)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/github/actions/workflow/status/deepbrain/superconfig/ci.yml?branch=main)](https://github.com/deepbrain/superconfig/actions)
[![Coverage](https://img.shields.io/badge/coverage-100%25%20lines-brightgreen.svg)](https://github.com/deepbrain/superconfig/tree/main/crates/logffi/COVERAGE_ANALYSIS.md)
[![Region Coverage](https://img.shields.io/badge/region%20coverage-94.33%25-green.svg)](https://github.com/deepbrain/superconfig/tree/main/crates/logffi/COVERAGE_ANALYSIS.md)

Drop-in replacement for the `log` crate with FFI callback support for bridging Rust logs to Python, Node.js, and other languages.

## Features

- **100% API compatibility** with the standard `log` crate
- **FFI callback support** for bridging logs to other languages
- **Zero overhead** when FFI callbacks are not used
- **Thread-safe** callback management with `OnceLock`
- **Respects log filtering** - callbacks only called for enabled log levels

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
logffi = "0.1"
```

Use identical syntax to the `log` crate:

```rust
use logffi::{warn, debug, info, error, trace};

fn main() {
    let value = 42;
    warn!("This is a warning: {}", value);
    debug!(target: "my_target", "Debug message with target");
}
```

## FFI Integration

Set a callback to bridge Rust logs to other languages:

```rust
use logffi::set_ffi_callback;

// Bridge to Python logging
set_ffi_callback(Box::new(|level, target, message| {
    // Call Python: logging.getLogger(target).log(level, message)
    python_log_bridge(level, target, message);
}));

// Bridge to Node.js winston
set_ffi_callback(Box::new(|level, target, message| {
    // Call Node.js: winston.log(level, message, { target })
    nodejs_log_bridge(level, target, message);
}));
```

## API Reference

### Logging Macros

All standard `log` crate macros are supported:

- `error!(...)` - Log error messages
- `warn!(...)` - Log warning messages
- `info!(...)` - Log info messages
- `debug!(...)` - Log debug messages
- `trace!(...)` - Log trace messages

Each macro supports both simple and targeted logging:

```rust
error!("Simple error message");
error!(target: "database", "Database connection failed: {}", err);
```

### FFI Functions

- `set_ffi_callback(callback)` - Set the global FFI callback function
- `call_ffi_callback(level, target, message)` - Manually call the FFI callback

### Callback Signature

```rust
pub type FfiCallback = Box<dyn Fn(&str, &str, &str) + Send + Sync>;
//                              level  target  message
```

## How It Works

1. **Standard Logging**: All macros first call the standard `log!` macro, respecting all filtering and configuration
2. **FFI Check**: If `log_enabled!` returns true for the target/level, the FFI callback is invoked
3. **Thread Safety**: FFI callback is stored in a `OnceLock` for thread-safe access
4. **Zero Overhead**: When no FFI callback is set, performance is identical to the standard `log` crate

## Use Cases

- **Python Extensions**: Bridge Rust logs to Python's `logging` module
- **Node.js Addons**: Forward Rust logs to Winston or other Node.js loggers
- **WebAssembly**: Send logs from WASM modules to JavaScript console
- **Mobile Apps**: Bridge Rust logs to platform-specific logging (iOS/Android)
- **Microservices**: Centralized logging across polyglot service architectures

## Coverage and Testing

LogFFI maintains **94.33% region coverage** with comprehensive testing:

- ✅ All public APIs tested
- ✅ Edge cases and error conditions covered
- ✅ Thread safety verified
- ✅ Macro expansion testing
- ✅ FFI callback behavior validated

See [COVERAGE_ANALYSIS.md](COVERAGE_ANALYSIS.md) for detailed coverage analysis.

## License

MIT License - see [LICENSE](../../LICENSE) for details.

## Contributing

Contributions welcome! This crate is part of the [SuperConfig](https://github.com/deepbrain/superconfig) monorepo.
