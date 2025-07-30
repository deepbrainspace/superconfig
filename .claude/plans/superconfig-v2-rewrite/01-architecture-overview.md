# SuperConfig V2: Architecture Overview

## Vision

A complete ground-up rewrite of SuperConfig designed for maximum performance and multi-language compatibility from day one. Built around a handle-based registry system that provides zero-copy configuration access with native-level performance across Rust, Python, Node.js, and WebAssembly.

## Core Architecture Principles

### 1. **Handle-Based Registry System**

- **Central Registry**: Global configuration registry with lock-free operations
- **Handle-Based Access**: Each configuration gets a unique handle (ID) for efficient lookup
- **Zero-Copy Design**: Configurations stored once, accessed via lightweight handles
- **Thread-Safe**: Built on DashMap for concurrent access without locks

### 2. **Multi-Crate Design**

Clean separation of concerns across three focused crates:

```
superconfig/                   # Workspace root
├── crates/
│   ├── superconfig/          # Pure Rust core (ZERO FFI dependencies)
│   ├── superconfig-py/       # Python bindings via PyO3
│   └── superconfig-napi/     # Node.js bindings via NAPI-RS (234x faster than WASM)
```

### 3. **Crate Responsibility Architecture**

#### **superconfig (Core Business Logic)**

- **Purpose**: Contains ALL business logic, algorithms, and functionality
- **Dependencies**: Pure Rust performance libraries only (no FFI)
- **Examples**: Handle registry, file loading, array merging, hot reload, hierarchical discovery
- **Rule**: No FFI concerns - pure Rust implementation

#### **superconfig-py (Python Bindings)**

- **Purpose**: Direct Python bindings using PyO3
- **Dependencies**: Only `superconfig` + `pyo3`
- **Examples**: Python-specific type conversions, error handling, snake_case preservation
- **Rule**: Thin delegation layer - minimal business logic
- **Pattern**: Direct PyO3 annotations with delegation to core

#### **superconfig-napi (Node.js Bindings)**

- **Purpose**: High-performance Node.js bindings via NAPI-RS
- **Dependencies**: Only `superconfig` + `napi` + `napi-derive`
- **Examples**: Native Node.js types, camelCase conversion, zero-copy operations
- **Rule**: Maximum performance for server-side JavaScript applications
- **Pattern**: Direct NAPI annotations with delegation to core

## System Features

### Core Features (Always Available)

- **Lock-Free Handle Registry**: Sub-microsecond configuration lookup
- **Zero-Copy Data Access**: No serialization overhead for repeated access
- **SIMD-Optimized Loading**: Hardware-accelerated file parsing and format detection
- **Advanced Array Merging**: Intelligent `_add`/`_remove` pattern support
- **Smart Format Detection**: Content-based parsing with caching
- **Glob Pattern Discovery**: Powerful wildcard-based file matching using globset
- **Hierarchical Discovery**: Git-style configuration inheritance
- **Enhanced Warning System**: Non-fatal error collection with continued loading
- **Profile Support**: Environment-specific configurations (dev/prod/test profiles)
- **Join vs Merge Semantics**: Different composition strategies (replace vs fill-holes)
- **Rich Error Chains**: Detailed error context with source tracking

### Optional Features (Via Cargo Features)

- **Hot Reload**: Tokio-based file watching with automatic updates
- **Parallel Loading**: Multi-file loading parallelization using rayon (3+ files threshold)
- **Profiling**: Performance tracing and metrics collection
- **Extended Formats**: YAML, TOML support beyond core JSON
- **SIMD Acceleration**: Hardware SIMD for parsing and string operations

### Multi-Language Support

- **Python**: Direct bindings via PyO3 (snake_case preserved)
- **Node.js**: High-performance NAPI bindings (234x faster than WASM, camelCase conversion)
- **Browser/Edge**: WASM bindings available as optional future addition

## Technology Stack

### Core Rust Dependencies

- **DashMap**: Lock-free concurrent HashMap for registry
- **parking_lot**: High-performance synchronization primitives
- **serde_json**: Core JSON handling with optional SIMD acceleration
- **memmap2**: Memory-mapped file loading for large configurations
- **globset**: High-performance glob pattern matching for wildcard discovery
- **dirs**: Cross-platform directory discovery for hierarchical configs
- **thiserror**: Rich error type definitions

### Optional Performance Dependencies

- **rayon**: Data parallelism for multi-file loading (3-5x speedup for glob patterns)
- **simd-json**: SIMD-accelerated JSON parsing (30-50% faster)
- **tokio**: Async runtime for hot reload file watching
- **notify**: Cross-platform file system watching
- **tracing**: Performance profiling and debugging

### FFI Dependencies (Isolated to Binding Crates)

#### superconfig-py dependencies:

- **pyo3**: Python binding generation and interop
- **pyo3-build**: Build-time Python configuration

#### superconfig-napi dependencies:

- **napi + napi-derive**: Node.js N-API binding generation
- **napi-build**: Build-time Node.js configuration

## Performance Targets

### Expected Performance (vs Current)

- **Configuration Loading**: ~20-30μs (vs ~100μs current)
- **Multi-File Loading**: ~30-40μs for 5+ files (vs ~100-500μs current)
- **Handle Lookup**: ~0.1-0.5μs (sub-microsecond registry access)
- **NAPI FFI Overhead**: ~2μs (234x faster than WASM, vs ~100μs current)
- **Python FFI Overhead**: ~0.5-1μs (vs ~100μs current)
- **Array Merging**: ~5-10μs with SIMD optimizations
- **Hot Reload Update**: ~2-5μs configuration refresh

### FFI Performance Benchmarks (Node.js)

Based on comprehensive benchmarking, NAPI significantly outperforms WASM:

| Metric              | NAPI   | WASM   | NAPI Advantage   |
| ------------------- | ------ | ------ | ---------------- |
| **Startup Time**    | ~0.1ms | ~1.6ms | **15.4x faster** |
| **Operation Speed** | ~2μs   | ~578μs | **234x faster**  |
| **Bundle Size**     | ~607KB | ~50KB  | 12x larger       |

**Strategic Decision**: NAPI's 234x performance advantage outweighs WASM's smaller bundle size for our performance-focused use case.

### Memory Efficiency

- **Registry Overhead**: ~50KB base + ~100 bytes per handle
- **Zero-Copy Access**: No duplication for repeated extractions
- **Lazy Loading**: Files loaded only when needed, cached by mtime
- **Efficient Caching**: LRU cache for parsed configurations

## API Design Philosophy

### Rust Core API (Pure Business Logic)

```rust
// Core business logic API - lives in superconfig crate
use superconfig::core::{SuperConfigCore, ConfigBuilder};

let core = ConfigBuilder::new()
    .select_profile("production")
    .with_file("config.toml")?
    .with_glob_pattern("./config/*.toml")?
    .with_env("APP_")?
    .join_hierarchical("myapp")?
    .build()?;

let db_config: DatabaseConfig = core.extract()?;

if core.has_warnings() {
    for warning in core.warnings() {
        eprintln!("Config warning: {}", warning);
    }
}
```

### FFI Wrapper API (Thin Delegation Layer)

```rust
// Thin FFI wrapper - lives in superconfig-ffi crate
use multiffi::multiffi;
use superconfig::core::SuperConfigCore;

#[multiffi]
pub struct SuperConfig {
    inner: SuperConfigCore,
}

#[multiffi]
impl SuperConfig {
    pub fn new() -> Self {
        Self { inner: SuperConfigCore::new() }
    }
    
    pub fn with_file(&self, path: String) -> Result<SuperConfig, String> {
        // Pure delegation - no business logic
        self.inner.with_file(&path)
            .map(|inner| SuperConfig { inner })
            .map_err(|e| e.to_string())
    }
    
    pub fn extract_json(&self) -> Result<String, String> {
        // Pure delegation - no business logic  
        self.inner.extract_json()
            .map_err(|e| e.to_string())
    }
}
```

### Language Bindings (Build Process)

- **superconfig-ffi**: Contains thin wrapper code with `#[multiffi]` annotations
- **MultiFfi macro**: Translates `#[multiffi]` annotations into PyO3/NAPI/wasm-bindgen code at compile time
- **Build system**: Compiles the translated code to produce actual language packages
- **Result**: Python wheel, NPM package, WASM bundle generated in `superconfig-ffi/bindings/`
- **Thin delegation**: All business logic stays in Rust core, wrappers just translate types and delegate calls

## Key Architectural Benefits

1. **Performance**: Handle-based registry eliminates repeated parsing/serialization
2. **Scalability**: Lock-free design scales to thousands of concurrent accesses
3. **Maintainability**: Clean crate separation makes each component focused
4. **Extensibility**: Feature flags allow opt-in complexity without bloat
5. **Speed**: NAPI provides 234x faster operations than WASM for Node.js applications
6. **Developer Experience**: Fluent APIs with rich error messages and debugging

## Next Steps

This architecture overview establishes the foundation. Next documents will cover:

- **Implementation Plan**: Detailed development phases and timelines
- **Core Engine Design**: Handle registry and configuration storage details
- **FFI Integration**: MultiFfi macro usage and binding generation process
- **Performance Optimization**: SIMD, caching, and memory management strategies
- **Testing Strategy**: Unit tests, integration tests, and benchmarking approach
