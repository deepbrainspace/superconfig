# LogFFI Test Suite

This directory contains comprehensive tests for the LogFFI tracing-native implementation. All tests verify that the macro hygiene fix works correctly and that all feature combinations compile and execute properly.

## Test Structure

### Integration Tests (`tests/integration/`)

- **`mod.rs`** - Test module declarations and feature gates
- **`auto_initialization.rs`** - Tests automatic tracing subscriber initialization
- **`logging_macros.rs`** - Tests basic logging macros (error!, warn!, info!, etc.)
- **`callback_functionality.rs`** - Tests callback system integration
- **`define_errors_macro.rs`** - Comprehensive tests for the define_errors! macro

## Test Coverage Matrix

### define_errors! Macro Combinations

| Test Case                        | Level | Target | Source | Fields | Description                                |
| -------------------------------- | ----- | ------ | ------ | ------ | ------------------------------------------ |
| **basic_error_definition**       | ❌    | ❌     | ❌     | ✅     | Simple errors with/without fields          |
| **error_with_log_levels**        | ✅    | ❌     | ❌     | ❌     | Tests error, warn, info levels             |
| **error_with_targets**           | ✅    | ✅     | ❌     | ❌     | Custom targets with levels                 |
| **error_with_source_chain**      | ❌    | ❌     | ✅     | ✅     | Source chaining with thiserror             |
| **comprehensive_level_coverage** | ✅    | ❌     | ❌     | ❌     | All 5 levels (error/warn/info/debug/trace) |
| **source_with_different_levels** | ✅    | ❌     | ✅     | ✅     | Source chaining + all levels               |
| **source_with_custom_targets**   | ✅    | ✅     | ✅     | ✅     | Source chaining + custom targets           |
| **default_behavior_tests**       | ✅    | ✅     | ❌     | ❌     | Default pattern behaviors                  |

### Pattern Combinations Covered (8/8)

1. **Default behavior**: `#[error("msg")]` → error level + module_path!() target
2. **Level only**: `#[error("msg", level = debug)]` → debug level + module_path!()
3. **Target only**: `#[error("msg", target = "custom")]` → error level + custom target
4. **Level + Target**: `#[error("msg", level = warn, target = "net")]` → warn + custom
5. **Source only**: `#[error("msg")] + #[source]` → error + module_path!() + source chain
6. **Source + Level**: `#[error("msg", level = info)] + #[source]` → info + module_path!() + source
7. **Source + Target**: `#[error("msg", target = "db")] + #[source]` → error + custom + source
8. **All features**: `#[error("msg", level = warn, target = "net")] + #[source]` → all combined

## Macro Hygiene Testing

### The Problem

- User input `level = error` conflicted with `error!` macro name
- Caused "local ambiguity when calling macro `error`" compilation errors

### The Solution

- Extract `code` and `message` as separate variables before @do_log calls
- Use identifier matching (`level = error`) instead of string matching (`level = "error"`)
- Two-phase expansion avoids parsing conflicts

### Verification

All tests verify that `level = error` compiles without hygiene conflicts:

```rust
#[error("Critical error", level = error)]  // ✅ Works perfectly
```

## Target Resolution

### Default Targets

- **No target specified** → Uses `module_path!()` for contextual information
- **Example**: `lib::integration::define_errors_macro: [ErrorCode] Message`

### Custom Targets

- **Explicit target** → Uses provided literal for logical grouping
- **Example**: `app::database: [ErrorCode] Message`

## Source Chain Integration

### thiserror Compatibility

- Full `#[source]` support for error chaining
- Preserves `.source()` method access
- Works with any `Error + Send + Sync` type

### Logging Behavior

- **Top-level message** shown in logs for readability
- **Full chain preserved** for programmatic access via `.source()`
- **Format**: `[ErrorCode] Top-level message`

## Test Execution

```bash
# Run all tests
cargo test --tests

# Run specific test module
cargo test integration::define_errors_macro

# Run with log output visible
cargo test --tests -- --nocapture

# Run specific test case
cargo test error_with_source_chain -- --nocapture
```

## Test Output Examples

### Level Variations

```
ERROR lib::integration::define_errors_macro: [ErrorLevel] Error level message
WARN  lib::integration::define_errors_macro: [WarnLevel] Warn level message  
INFO  lib::integration::define_errors_macro: [InfoLevel] Info level message
```

### Target Variations

```
ERROR storage::db: [DatabaseIo] Database IO error
WARN  network::client: [NetworkIo] Network IO error
ERROR custom::module: [CustomTargetOnly] Custom target only
```

### Source Chain Examples

```
ERROR lib::integration::define_errors_macro: [CriticalIo] Critical IO error
WARN  lib::integration::define_errors_macro: [WarningIo] IO warning
INFO  lib::integration::define_errors_macro: [InfoIo] IO info
```

## Coverage Verification

To verify comprehensive coverage, check that:

1. **All 5 log levels work**: error, warn, info, debug, trace
2. **All target combinations work**: default (module_path!) vs custom literals
3. **All source combinations work**: with and without error chaining
4. **All pattern defaults work**: missing level/target use appropriate defaults
5. **Macro hygiene fixed**: `level = error` compiles without conflicts
6. **Field interpolation works**: Error messages with `{field}` syntax
7. **Logging integration works**: `.log()` method produces expected output
8. **Error trait compatibility**: `.source()`, `.to_string()`, `.code()` methods

## Status: ✅ Production Ready

All tests pass, proving the tracing-native LogFFI implementation is complete and robust.
