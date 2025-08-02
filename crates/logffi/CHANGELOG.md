# Changelog

All notable changes to the `logffi` crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-02

### Added

#### Core Features

- **100% API compatibility** with the standard `log` crate
- **FFI callback support** for bridging Rust logs to Python, Node.js, and other languages
- **Thread-safe callback management** using `OnceLock` for global state
- **Zero overhead** when FFI callbacks are not used
- **Respects log filtering** - callbacks only called for enabled log levels

#### Public API

- `set_ffi_callback(callback)` - Set global FFI callback function
- `call_ffi_callback(level, target, message)` - Manually invoke FFI callback
- `FfiCallback` type alias for callback function signatures
- Complete re-export of `log` crate for drop-in compatibility

#### Logging Macros

All standard logging macros with identical API to `log` crate plus FFI bridging:

- `error!()` - Log error messages with FFI callback support
- `warn!()` - Log warning messages with FFI callback support
- `info!()` - Log info messages with FFI callback support
- `debug!()` - Log debug messages with FFI callback support
- `trace!()` - Log trace messages with FFI callback support

Each macro supports both simple and targeted logging:

```rust
error!("Simple error message");
error!(target: "database", "Database connection failed: {}", err);
```

#### Internal Implementation

- `log_with_ffi!()` macro for enhanced logging with FFI callback integration
- Level conversion from `log::Level` to string representation
- Conditional FFI callback execution based on `log_enabled!` checks
- Format string handling and message construction

### Testing & Quality

#### Comprehensive Test Coverage

- **94.33% region coverage** (22 missing out of 388 total regions)
- **100% function coverage** (13 functions fully covered)
- **100% line coverage** (182 lines fully covered)

#### Test Categories

- **Functional testing**: All public APIs and code paths exercised
- **Edge case coverage**: Empty strings, special characters, complex formatting
- **Error path testing**: OnceLock conflicts, callback failures, logging state changes
- **Macro expansion coverage**: Multiple invocation patterns for different generated code paths
- **API compatibility testing**: Verification of drop-in replacement capability
- **Thread safety testing**: Serial test execution to verify OnceLock behavior

#### Coverage Analysis

- Detailed coverage analysis documented in `COVERAGE_ANALYSIS.md`
- Missing regions identified as either defensive assertions or architectural limitations
- Coverage represents practical maximum given logging framework constraints

### Documentation

#### Comprehensive Documentation

- **Complete API documentation** with examples for all public functions
- **Usage examples** for Python, Node.js, and WebAssembly integration
- **Performance characteristics** and zero-overhead design explanation
- **Architecture documentation** explaining FFI callback integration
- **Coverage analysis** with detailed explanation of testing strategy

#### Use Case Examples

- Python extensions bridging to Python's `logging` module
- Node.js addons forwarding to Winston or other Node.js loggers
- WebAssembly modules sending logs to JavaScript console
- Mobile apps bridging to platform-specific logging (iOS/Android)
- Microservices with centralized logging across polyglot architectures

### Performance

#### Zero-Overhead Design

- When no FFI callback is set, performance identical to standard `log` crate
- FFI callback stored in `OnceLock` for minimal runtime overhead
- Lazy message formatting - only formats when logging is enabled
- Respects all standard log filtering to avoid unnecessary work

#### Thread Safety

- Global FFI callback management via `OnceLock`
- Thread-safe callback execution
- No blocking or synchronization overhead during logging

### Integration

#### SuperConfig Ecosystem

- Designed as core logging component for SuperConfig toolkit
- Integration with `superconfig` crate for configuration-driven logging
- Foundation for multilingual logging across Python, Node.js, WebAssembly

#### Drop-in Compatibility

- Complete API compatibility with `log` crate
- Simple migration: change `use log::*` to `use logffi::*`
- No breaking changes to existing logging code

### Build Configuration

#### Dependencies

- `log = "0.4.27"` - Core logging functionality
- `env_logger = "0.11.8"` (dev) - Testing infrastructure
- `serial_test = "3.2.0"` (dev) - Thread-safe test execution

#### Rust Requirements

- **Rust version**: 1.88 or higher
- **Edition**: 2024
- **License**: MIT

### Initial Release Notes

This is the initial release of `logffi`, providing a drop-in replacement for the `log` crate with FFI callback support. The crate has been designed from the ground up for:

- **Maximum performance** with zero-overhead abstractions
- **Enterprise-grade reliability** with comprehensive testing
- **Seamless integration** with existing Rust logging ecosystems
- **Multilingual support** for modern polyglot applications

The 94.33% region coverage represents the practical maximum achievable given the architectural constraints of the Rust logging ecosystem, ensuring thorough testing while respecting design limitations.

---

**LogFFI** - Bridging Rust logs to the world. 🌍

[0.1.0]: https://crates.io/crates/logffi
