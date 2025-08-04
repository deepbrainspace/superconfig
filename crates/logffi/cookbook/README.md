# LogFFI Cookbook

Welcome to the LogFFI cookbook! This collection of guides shows you how to use LogFFI's features effectively in real-world scenarios.

## 📚 Guides

### [1. Basic Logging](01-basic-logging.md)

Learn the fundamentals of logging with LogFFI, including:

- Simple logging statements
- Logging with variables and formatting
- Target-based logging for module organization
- Performance considerations
- Integration with different backends

### [2. Error Handling](02-error-handling.md)

Master the `define_errors!` macro for sophisticated error handling:

- Creating error enums with automatic logging
- Using constructor methods for cleaner code
- Setting error levels and targets
- Custom error codes for monitoring
- FFI-friendly error handling

### [3. Source Error Chaining](03-source-error-chaining.md)

Implement proper error chaining for better debugging:

- Using thiserror's `#[source]` attribute
- Chaining multiple error types
- Working with dynamic errors
- Integration with anyhow and other error libraries
- Best practices for error context

### [4. FFI Integration](04-ffi-integration.md)

Bridge Rust logs to other languages:

- Python integration with PyO3
- Node.js integration with Neon
- C/C++ integration
- WebAssembly support
- Advanced FFI patterns

### [5. Backend Configuration](05-backend-configuration.md)

Configure and optimize logging backends:

- Switching between log, tracing, and slog
- Runtime backend selection
- Backend-specific features
- Performance optimization
- Testing strategies

## 🚀 Quick Start

If you're new to LogFFI, start with:

1. [Basic Logging](01-basic-logging.md) - Learn the fundamentals
2. [Error Handling](02-error-handling.md) - See the power of `define_errors!`
3. [FFI Integration](04-ffi-integration.md) - If you need cross-language logging

## 🎯 Common Use Cases

### "I want structured error handling with automatic logging"

→ See [Error Handling](02-error-handling.md) and use the `define_errors!` macro

### "I need to bridge Rust logs to Python/Node.js"

→ Check [FFI Integration](04-ffi-integration.md) for language-specific examples

### "I want proper error context and chaining"

→ Read [Source Error Chaining](03-source-error-chaining.md)

### "I need JSON structured logging in production"

→ See [Backend Configuration](05-backend-configuration.md) for JSON output setup

### "I want to organize logs by module/component"

→ Check target-based logging in [Basic Logging](01-basic-logging.md)

## 💡 Tips

- Always use the constructor methods (`new_variant_name()`) for errors - they automatically log!
- Use target-based logging to organize your logs by component
- Consider using source error chaining for better debugging context
- Choose the right backend based on your deployment environment
- Test your error handling paths - LogFFI makes it easy

## 📝 Contributing

Found a great pattern or use case? We welcome contributions to the cookbook! Please submit a PR to add your examples.
