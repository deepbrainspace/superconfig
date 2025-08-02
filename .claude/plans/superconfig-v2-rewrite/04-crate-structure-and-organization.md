# SuperConfig V2: Crate Structure and Organization

## Overview

This document details the multi-crate workspace layout, internal module organization, and dependency architecture for SuperConfig V2. The structure follows a clean separation principle where the core contains all business logic and FFI crates provide thin wrapper layers with identical module organization for easy maintenance.

## Workspace Root Structure

```
superconfig/                    # Workspace root
├── Cargo.toml                 # Workspace configuration with shared dependencies
├── README.md                  # Multi-language usage guide
├── LICENSE                    # License file
├── .github/                   # CI/CD workflows
│   └── workflows/
│       ├── rust.yml           # Core Rust testing
│       ├── python.yml         # Python binding tests
│       └── nodejs.yml         # Node.js binding tests
├── crates/
│   ├── superconfig/           # Pure Rust core (ZERO FFI dependencies)
│   ├── superconfig-py/        # Python bindings via PyO3
│   └── superconfig-napi/      # Node.js bindings via NAPI-RS
├── benchmarks/                # Cross-language performance validation
│   ├── rust_bench.rs         # Native Rust benchmarks
│   ├── python_bench.py       # Python FFI benchmarks
│   └── nodejs_bench.js       # Node.js FFI benchmarks
├── examples/                  # Usage examples for all languages
│   ├── rust/                 # Native Rust examples
│   ├── python/               # Python usage examples
│   └── nodejs/               # Node.js usage examples
├── docs/                     # Documentation and guides
│   ├── architecture.md      # High-level architecture guide
│   ├── performance.md       # Performance characteristics
│   └── migration.md         # V1 to V2 migration guide
└── scripts/                  # Build and release automation
    ├── build-all.sh         # Cross-platform build script
    └── release.sh           # Multi-language release automation
```

## Core Design Principles

### 1. **Clean Separation Architecture**

- **superconfig**: Contains ALL business logic, algorithms, and functionality
- **FFI crates**: Provide thin wrapper layers with identical module structure
- **Zero duplication**: Business logic exists only in the core crate

### 2. **Mirror Module Organization**

- FFI crates mirror the exact folder/file structure of the core crate
- Each core module has a corresponding wrapper module in FFI crates
- Wrapper functions follow 1:1 mapping with core functions where possible

### 3. **Dependency Isolation**

- Core crate has zero FFI dependencies
- Each FFI crate depends only on core + its specific FFI library
- No cross-dependencies between FFI crates

## Detailed Crate Structure

### superconfig/ (Pure Rust Core)

```
crates/superconfig/
├── moon.yml                   # Moon project configuration
├── Cargo.toml                 # Pure Rust dependencies only
├── src/
│   ├── lib.rs                 # Public API exports and re-exports
│   ├── core.rs                # Registry, handles, config data, error types (~500-700 lines)
│   ├── providers.rs           # File, env, hierarchical, glob loading (~600-800 lines)
│   ├── parsing.rs             # JSON, TOML, YAML, env format parsers (~500-700 lines)
│   ├── merging.rs             # Merge engine, array patterns, strategies (~400-600 lines)
│   ├── builder.rs             # Fluent API, profiles, validation (~500-700 lines)
│   └── features.rs            # Hot reload, SIMD, profiling, verbosity (~500-700 lines)
├── tests/                     # Unit and integration tests
│   ├── core_tests.rs         # Registry and handle tests
│   ├── provider_tests.rs     # File loading and env parsing tests
│   ├── parser_tests.rs       # Format parser validation tests
│   ├── merge_tests.rs        # Merge algorithm tests
│   ├── builder_tests.rs      # API builder tests
│   └── integration_tests.rs  # End-to-end integration tests
└── benches/                   # Performance benchmarks
    ├── registry_bench.rs     # Handle registry benchmarks
    ├── parsing_bench.rs      # Parser performance tests
    └── merging_bench.rs      # Merge algorithm benchmarks
```

### superconfig-py/ (Python Bindings)

```
crates/superconfig-py/
├── moon.yml                   # Moon project configuration
├── Cargo.toml                 # Dependencies: superconfig + pyo3
├── pyproject.toml             # Python package configuration
├── src/
│   └── lib.rs                 # All Python wrappers (~800-1200 lines)
│                             # Sections: core, providers, parsing, merging, builder, features
├── python/                    # Generated Python package structure
│   └── superconfig/
│       ├── __init__.py       # Python module initialization
│       ├── py.typed          # Type hint marker file
│       └── superconfig.pyi   # Python type stubs
├── tests/                     # Python-specific tests
│   ├── test_core.py          # Core functionality tests
│   ├── test_providers.py     # Provider wrapper tests
│   ├── test_parsers.py       # Parser wrapper tests
│   ├── test_builders.py      # Builder API tests
│   └── test_integration.py   # End-to-end Python tests
└── examples/                  # Python usage examples
    ├── basic_usage.py        # Simple configuration example
    ├── advanced_features.py  # Hot reload and profiles
    └── performance_demo.py   # Performance comparison demo
```

### superconfig-napi/ (Node.js Bindings)

```
crates/superconfig-napi/
├── moon.yml                   # Moon project configuration
├── Cargo.toml                 # Dependencies: superconfig + napi + napi-derive
├── package.json               # NPM package configuration
├── src/
│   └── lib.rs                 # All Node.js wrappers (~800-1200 lines)
│                             # Sections: core, providers, parsing, merging, builder, features
├── bindings/                  # Generated Node.js package structure
│   ├── package.json          # Generated NPM package
│   ├── index.js              # Main entry point
│   ├── index.d.ts            # TypeScript definitions
│   └── superconfig.node      # Compiled native module
├── tests/                     # Node.js-specific tests
│   ├── core.test.js          # Core functionality tests
│   ├── providers.test.js     # Provider wrapper tests
│   ├── parsers.test.js       # Parser wrapper tests
│   ├── builders.test.js      # Builder API tests
│   └── integration.test.js   # End-to-end Node.js tests
└── examples/                  # Node.js usage examples
    ├── basic-usage.js        # Simple configuration example
    ├── advanced-features.js  # Hot reload and profiles
    └── performance-demo.js   # Performance comparison demo
```

## Flattened File Organization

### Core File Structure (~3000-4200 lines total)

Each file contains related functionality grouped logically:

#### `core.rs` - Foundation (~500-700 lines)

- `ConfigRegistry` - Handle registry system with DashMap
- `ConfigHandle<T>` - Type-safe handle operations
- `ConfigData` - Configuration data structures with caching
- `ConfigError` - Error types with source chains
- Global registry management and handle lifecycle

#### `providers.rs` - Data Sources (~600-800 lines)

- File loading with memory mapping and caching
- Environment variable processing with nesting
- Hierarchical discovery (system → user → project)
- Glob pattern matching for multi-file loading
- Smart format detection and provider chaining

#### `parsing.rs` - Format Parsers (~500-700 lines)

- JSON parsing with optional SIMD acceleration
- TOML parsing with detailed error reporting
- YAML parsing with safe loading
- Environment file format parsing
- Content-based format detection algorithms

#### `merging.rs` - Configuration Composition (~400-600 lines)

- Deep merge algorithm for nested objects
- Array merge patterns (`_add`/`_remove` support)
- Merge vs join strategies
- Conflict resolution policies
- Performance-optimized merging for large configs

#### `builder.rs` - Public API (~500-700 lines)

- Fluent builder API with method chaining
- Profile selection and environment-specific configs
- Configuration validation and type-safe extraction
- Hierarchical discovery coordination
- Warning collection and verbosity management

#### `features.rs` - Optional Features (~500-700 lines)

- Hot reload with Tokio file watching
- SIMD optimization utilities
- Performance profiling and metrics collection
- Feature flag organization
- Advanced debugging and introspection

### FFI File Structure (~800-1200 lines each)

Single `lib.rs` file per language containing all wrappers:

#### Organized by sections matching core files:

```rust
// superconfig-py/src/lib.rs structure
mod core_wrappers;      // Python wrappers for core.rs functionality
mod provider_wrappers;  // Python wrappers for providers.rs functionality  
mod parsing_wrappers;   // Python wrappers for parsing.rs functionality
mod merging_wrappers;   // Python wrappers for merging.rs functionality
mod builder_wrappers;   // Python wrappers for builder.rs functionality (main user API)
mod feature_wrappers;   // Python wrappers for features.rs functionality

// PyO3 module exports
#[pymodule]
fn superconfig(_py: Python, m: &PyModule) -> PyResult<()> {
    // Export all wrapper classes and functions
}
```

## Wrapper Organization Principles

### 1. **Logical Section Mapping**

Each section in FFI files corresponds to a core file:

- `core_wrappers` ↔ `core.rs`
- `provider_wrappers` ↔ `providers.rs`
- `builder_wrappers` ↔ `builder.rs` (main user-facing API)

### 2. **Consistent Wrapper Patterns**

```rust
// Core implementation (superconfig/src/builder.rs)
impl ConfigBuilder {
    pub fn with_file(&self, path: &str) -> Result<ConfigBuilder, ConfigError> { /* ... */ }
}

// Python wrapper (superconfig-py/src/lib.rs)
#[pyclass]
struct PyConfigBuilder { inner: ConfigBuilder }

#[pymethods]  
impl PyConfigBuilder {
    fn with_file(&self, path: &str) -> PyResult<PyConfigBuilder> {
        // Delegate to core, convert errors
    }
}

// Node.js wrapper (superconfig-napi/src/lib.rs)
#[napi]
struct JsConfigBuilder { inner: ConfigBuilder }

#[napi]
impl JsConfigBuilder {
    #[napi]
    fn with_file(&self, path: String) -> Result<JsConfigBuilder> {
        // Delegate to core, convert errors  
    }
}
```

### 3. **Dependency Flow**

- **Core crate**: Pure Rust dependencies only (no FFI libraries)
- **FFI crates**: Depend only on core crate + their specific FFI library
- **No circular dependencies**: FFI crates never depend on each other

## Moon Integration and Build Organization

### Moon Project Configuration

#### superconfig/moon.yml

```yaml
type: 'rust'
language: 'rust'
platform: 'rust'

tasks:
  build:
    command: 'cargo build --release'
    inputs: ['src/**/*', 'Cargo.toml']
    outputs: ['target/release/']
  
  test:
    command: 'cargo test'
    inputs: ['src/**/*', 'tests/**/*', 'Cargo.toml']
    
  bench:
    command: 'cargo bench'
    inputs: ['src/**/*', 'benches/**/*', 'Cargo.toml']
    deps: ['build']
```

#### superconfig-py/moon.yml

```yaml
type: 'rust'
language: 'python'
platform: 'rust'

dependsOn: ['superconfig']

tasks:
  build:
    command: 'maturin build --release'
    inputs: ['src/**/*', 'Cargo.toml', 'pyproject.toml']
    outputs: ['target/wheels/']
    deps: ['superconfig:build']
    
  test:
    command: 'python -m pytest tests/'
    inputs: ['tests/**/*', 'examples/**/*']
    deps: ['build']
    
  develop:
    command: 'maturin develop'
    inputs: ['src/**/*', 'Cargo.toml']
    deps: ['superconfig:build']
```

#### superconfig-napi/moon.yml

```yaml
type: 'rust'
language: 'javascript'
platform: 'node'

dependsOn: ['superconfig']

tasks:
  build:
    command: 'napi build --platform --release'
    inputs: ['src/**/*', 'Cargo.toml', 'package.json']
    outputs: ['bindings/']
    deps: ['superconfig:build']
    
  test:
    command: 'npm test'
    inputs: ['tests/**/*', 'examples/**/*']
    deps: ['build']
    
  develop:
    command: 'napi build --platform'
    inputs: ['src/**/*', 'Cargo.toml']
    deps: ['superconfig:build']
```

### Generated Package Structure

#### Python Package (in superconfig-py/python/)

```
python/superconfig/
├── __init__.py              # Main Python module exports
├── superconfig.so           # Compiled extension (Linux)
├── superconfig.pyd          # Compiled extension (Windows)  
├── superconfig.dylib        # Compiled extension (macOS)
├── py.typed                 # Type hints marker
└── superconfig.pyi          # Generated type stubs
```

#### Node.js Package (in superconfig-napi/bindings/)

```
bindings/
├── package.json             # NPM package metadata
├── index.js                 # Main entry point with camelCase exports
├── index.d.ts               # TypeScript definitions
└── superconfig.node         # Compiled native module
```

### File Size and Complexity Summary

**Total Files**: 8 core files + 2 FFI wrapper files = **10 files** (vs 75+ in complex structure)

**Core Files** (~3000-4200 lines total):

- `lib.rs` (~50-100 lines) - exports only
- `core.rs` (~500-700 lines) - foundation
- `providers.rs` (~600-800 lines) - data sources
- `parsing.rs` (~500-700 lines) - format parsers
- `merging.rs` (~400-600 lines) - composition logic
- `builder.rs` (~500-700 lines) - public API
- `features.rs` (~500-700 lines) - optional features

**FFI Files** (~1600-2400 lines total):

- `superconfig-py/src/lib.rs` (~800-1200 lines) - all Python wrappers
- `superconfig-napi/src/lib.rs` (~800-1200 lines) - all Node.js wrappers

### Maintenance Benefits

This flattened structure provides:

1. **Reduced Complexity**: 10 files instead of 75+ files (7.5x reduction)
2. **AI-Friendly**: Related code co-located, better context for AI development
3. **Easy Navigation**: Clear functional separation without excessive nesting
4. **Simple Duplication**: Only 2 wrapper files need to track core changes
5. **Moon Integration**: Clean project boundaries with clear dependencies
6. **Efficient Builds**: Moon can optimize builds with proper dependency tracking

### Development Workflow

1. **Core Development**: Work primarily in 6 focused core files
2. **FFI Updates**: Update corresponding sections in 2 wrapper files
3. **Testing**: Moon coordinates testing across all languages
4. **Builds**: Moon handles cross-language build dependencies automatically
5. **CI/CD**: Each language has independent build/test cycles with proper sequencing
