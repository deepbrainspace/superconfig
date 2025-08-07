# SuperConfig Project Overview

## Project Purpose

SuperConfig is a comprehensive multi-language configuration management ecosystem designed for modern applications. It provides:

- **Core SuperConfig Library**: Advanced configuration management with hierarchical cascading, intelligent array merging, and smart format detection
- **LogFusion**: Drop-in replacement for Rust's log crate with FFI callback support for multi-language integration
- **MultiFI**: Multi-language FFI binding generator for Python, Node.js, and WebAssembly
- **SuperConfig Macros**: Procedural macros for fluent API error handling and FFI integration

## Tech Stack

- **Primary Language**: Rust (edition 2024, version 1.88.0)
- **Build System**: Moon (moonrepo.dev) for workspace management
- **Git Hooks**: Lefthook for pre-commit/pre-push automation
- **CI/CD**: GitHub Actions with affected crate detection
- **Testing**: Standard Rust testing + cargo-llvm-cov for coverage
- **Documentation**: Rustdoc with custom Ayu theme
- **Formatting**: cargo fmt + dprint for multi-format files
- **Linting**: Clippy with pedantic/nursery lints
- **Security**: cargo-audit + cargo-deny for dependency scanning
- **Package Manager**: Cargo with workspace configuration

## Key Features

- High-performance configuration management with zero-copy access
- Handle-based registry for sub-microsecond lookups
- Multi-format support (TOML, JSON, YAML) with auto-detection
- Hierarchical configuration cascading (system → user → project)
- Advanced array merging with `_add`/`_remove` patterns
- Environment variable parsing with JSON support
- FFI bindings for Python, Node.js, and WebAssembly
- Production-ready with performance optimizations

## Repository Structure

```
superconfig/
├── crates/                 # All Rust crates
│   ├── superconfig/        # Main configuration library
│   ├── logfusion/             # FFI logging library
│   ├── multiffi/           # Multi-language FFI generator
│   ├── superconfig-macros/ # Procedural macros
│   ├── superhashmap/       # High-performance hashmap
│   ├── rusttoolkit/          # Rust metaprogramming utilities
│   └── hash-benchmark/     # Benchmarking utilities
├── .moon/                  # Moon build system configuration
├── .github/                # GitHub Actions workflows
├── .claude/                # Claude Code configuration and plans
├── marketing/              # Marketing and documentation materials
└── crates-archive/         # Archived versions of crates
```

## Current Development Status

The project is actively being developed with SuperConfig v2.1 currently in implementation phase. The main crate structure is defined but core functionality is being rewritten for enhanced performance and FFI support.
