# SuperConfig Code Style and Conventions

## Rust Code Style

### Edition and Toolchain

- **Rust Edition**: 2024
- **Rust Version**: 1.88.0
- **Components**: rustfmt, clippy, llvm-tools-preview

### Formatting

- **Tool**: `cargo fmt` (enforced by git hooks)
- **Configuration**: Default rustfmt settings
- **Line Width**: Default (100 characters)
- **Staged Fixes**: Automatically staged by lefthook

### Linting Configuration

```toml
[lints.clippy]
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
result_large_err = "allow" # Allow large error types for rich context
```

### Naming Conventions

- **Crates**: kebab-case (`superconfig`, `logffi`, `superconfig-macros`)
- **Modules**: snake_case
- **Functions**: snake_case
- **Types/Structs**: PascalCase
- **Constants**: SCREAMING_SNAKE_CASE
- **File names**: snake_case.rs

### Documentation Standards

- **Required**: `#![warn(missing_docs)]` for public APIs
- **Doc comments**: Use `///` for public items, `//!` for module docs
- **Examples**: Include code examples in doc comments
- **Rustdoc theme**: Ayu (specified in Cargo.toml metadata)

### Error Handling

- **Library**: `thiserror` for custom error types
- **Pattern**: Rich error context preferred over simple errors
- **FFI Errors**: Use `logffi` macros for FFI-compatible error handling

### Dependencies

- **Serde**: Version 1.0+ with derive feature
- **SCC**: Lock-free data structures (v2.3+)
- **LogFFI**: Internal logging with FFI support
- **Performance**: Prefer zero-allocation where possible

### Code Organization

- **Lib.rs**: Module declarations and re-exports only
- **Modules**: One concept per module
- **Features**: Use feature flags for optional functionality
- **Internal APIs**: Use `pub(crate)` for internal visibility

## Multi-Format File Conventions

### Dprint Configuration

- **Markdown**: 100 character line width
- **JSON/TOML**: 2-space indentation
- **Formats**: `.md`, `.json`, `.yaml`, `.yml`, `.toml`
- **Excludes**: `node_modules`, `.git`, `target`, `*.min.json`

### Configuration Files

- **Moon**: `.moon/` directory for build system config
- **Rust**: Standard `Cargo.toml` with rich metadata
- **Security**: `deny.toml` for dependency policies
- **Git**: `lefthook.yml` for git hooks

## Commit and Git Conventions

### Conventional Commits (Enforced)

```
type(scope): description

Types: feat, fix, docs, style, refactor, test, chore
Scopes: crate names (superconfig, logffi, multiffi)
Examples:
- feat(superconfig): add hierarchical configuration provider  
- fix(logffi): resolve callback memory leak
- chore(ci): update GitHub Actions workflow
```

### Branch Strategy

- **Main branch**: `main`
- **Feature branches**: `feat/feature-name`
- **Fix branches**: `fix/issue-description`
- **Merge strategy**: Regular merge (preserve commit history)

### Git Hooks (Lefthook)

- **Pre-commit**: Format, lint, conventional commit validation
- **Pre-push**: Clean repo check, format validation, security audit
- **Parallel execution**: Enabled for performance
- **Staged fixes**: Automatically stage formatted files

## Performance Conventions

### Optimization Principles

- **Zero-copy**: Prefer borrowing over cloning
- **Lazy loading**: Load resources on-demand
- **Caching**: Cache expensive operations (file parsing, etc.)
- **Lock-free**: Use SCC for concurrent data structures
- **LLVM optimization**: Use cargo-llvm-cov for coverage

### Benchmarking

- **Tool**: Criterion.rs
- **HTML reports**: Enabled for detailed analysis
- **Baselines**: Save benchmarks for comparison
- **CI**: Optional benchmark runs (commented out if slow)

## Testing Conventions

### Test Organization

- **Unit tests**: In same file with `#[cfg(test)]`
- **Integration tests**: In `tests/` directory
- **Examples**: Runnable examples in `examples/`
- **Coverage**: Target high coverage with meaningful tests

### Test Tools

- **Serial tests**: Use `serial_test` for resource conflicts
- **Async tests**: `tokio-test` for async functionality
- **Temp files**: `tempfile` for file system tests
- **Environment**: `env_logger` for test logging

## FFI Conventions

### Multi-language Support

- **Target languages**: Python, Node.js, WebAssembly
- **Logging**: Use `logffi` for consistent logging across languages
- **Error handling**: FFI-compatible error types
- **Memory management**: Careful ownership in FFI boundaries

### Naming Conversions

- **C exports**: `extern "C"` with clear naming
- **Python**: Snake_case following Python conventions
- **JavaScript**: camelCase following JavaScript conventions
- **WASM**: Consider JavaScript conventions
