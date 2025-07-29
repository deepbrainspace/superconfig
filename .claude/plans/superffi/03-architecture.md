# SuperFFI Architecture Design

## Executive Summary

This document outlines the three-layer architecture for SuperConfig multi-language support that preserves Rust performance while providing ergonomic FFI bindings.

### Complete Build & Distribution Flow

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   superconfig   │    │ superconfig-ffi  │    │    superffi     │
│                 │    │                  │    │                 │
│ Native Rust API │◄───│ JSON Wrapper API │◄───│ Macro Generator │
│ High Performance│    │ FFI Compatible   │    │ Py + Node + WASM│
│ Zero FFI Cost   │    │ serde_json::Value│    │ Auto Bindings   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │
                                │ Build Tools Compile This Rust Code Into:
                                ▼
                       ┌─────────────────────────────────┐
                       │         NATIVE BINARIES         │
                       │                                 │
                       │ maturin → superconfig.so        │ Python extension
                       │ napi    → superconfig.node      │ Node.js addon
                       │ wasm-pack → superconfig.wasm    │ WebAssembly module
                       └─────────────────────────────────┘
                                │
                                │ Package Into Distribution Format:
                                ▼
                       ┌─────────────────────────────────┐
                       │      DISTRIBUTION PACKAGES      │
                       │                                 │
                       │ Python: .whl file → PyPI        │
                       │ Node.js: .tgz file → npm        │
                       │ WASM: .tgz file → npm           │
                       └─────────────────────────────────┘
```

### Key Architecture Benefits

- **Single source of truth**: Write each method once, generates native APIs for all enabled languages
- **Feature flag flexibility**: Independently build Python-only, Node.js-only, or both
- **Native language ergonomics**: Users get clean, language-appropriate APIs (no JSON manipulation)
- **Zero performance regression**: Core Rust API remains unchanged
- **Incremental development**: Start with one language, add others progressively

## Performance Benefits

| User Type | Current | With This Plan | Impact |
|-----------|---------|----------------|---------|
| **Rust** | Native types (optimal) | Native types (unchanged) | **No regression** |
| **Python** | Complex PyO3 marshaling | Simple JSON parsing | **Major improvement** |
| **Node.js** | Complex napi-rs marshaling | Simple JSON parsing | **major improvement** |

## Layer Details

### Layer 1: Core SuperConfig (Unchanged)

**Purpose**: Maintain high-performance native Rust API  
**Changes**: None - preserves existing API contract  
**Users**: Rust developers who need maximum performance

```rust
// Existing SuperConfig API remains unchanged
use superconfig::SuperConfig;

let config = SuperConfig::new()
    .with_file("config.toml")
    .with_env("APP_")
    .with_hierarchical_config("base", "prod", true);
```

### Layer 2: SuperConfig FFI Wrapper

**Purpose**: FFI-compatible interface using JSON for complex types  
**Location**: `crates/superconfig-ffi/`  
**Approach**: Wraps core SuperConfig with JSON parameter handling

**Design Pattern**:
```rust
#[superffi]
pub struct SuperConfig {
    inner: CoreSuperConfig,  // Wraps the real implementation
}

#[superffi]
impl SuperConfig {
    // Simple methods: direct parameter mapping
    pub fn with_file(&self, path: String) -> Result<Self, String> {
        Ok(Self { inner: self.inner.with_file(path) })
    }
    
    // Complex methods: JSON parameter handling
    pub fn with_wildcard(&self, config: serde_json::Value) -> Result<Self, String> {
        // Convert JSON to native types, then call core API
    }
}
```

### Layer 3: SuperFFI Macro Generator

**Purpose**: Generates language-specific bindings from annotated Rust code  
**Location**: `crates/superffi/`  
**Status**: ✅ COMPLETED in Phase 1

**Generation Strategy**:
```rust
#[superffi]  // Applied to struct/impl/fn
// Generates based on enabled features:
// - #[pyo3::pyclass] + #[pyo3::pymethods] (Python)
// - #[napi::napi] annotations (Node.js)  
// - #[wasm_bindgen] annotations (WebAssembly)
```

## SuperFFI Generic Naming Strategy

### JavaScript API Consistency Requirement

SuperFFI ensures **identical function signatures** across all JavaScript environments (Node.js and WebAssembly) by implementing a generic naming conversion strategy:

**Problem**: Raw bindings produce inconsistent APIs:
- Node.js (NAPI): `withFile()` (automatically camelCase)
- WebAssembly (wasm-bindgen): `with_file()` (preserves snake_case)

**Solution**: SuperFFI applies generic naming conversion:

```rust
// SuperFFI automatically converts any Rust function name:
// snake_case → camelCase for JavaScript environments
// snake_case → snake_case for Python (preserved)

#[superffi]
impl SuperConfig {
    pub fn with_file(&self, path: String) -> Result<Self, String> { ... }
    pub fn with_wildcard(&self, config: serde_json::Value) -> Result<Self, String> { ... }
    pub fn set_debug(&self, debug: bool) -> Result<Self, String> { ... }
}

// Generated APIs:
// Python: with_file(), with_wildcard(), set_debug()  (snake_case preserved)
// Node.js: withFile(), withWildcard(), setDebug()    (converted to camelCase)
// WASM:    withFile(), withWildcard(), setDebug()    (converted to camelCase)
```

### Naming Conversion Rules

1. **JavaScript Environments** (Node.js, WebAssembly):
   - Convert `snake_case` → `camelCase`
   - `with_file` → `withFile`
   - `set_debug` → `setDebug`
   - `extract_json` → `extractJson`

2. **Python Environment**:
   - Preserve `snake_case` (Python convention)
   - `with_file` → `with_file`
   - `set_debug` → `set_debug`

3. **Generic Implementation**:
   - SuperFFI applies conversion regardless of original function name
   - Works with any Rust function using snake_case convention
   - No hardcoded function names in SuperFFI macro

### Testing Requirements

SuperFFI includes comprehensive tests for naming conversion:

```rust
#[test]
fn test_generic_naming_conversion() {
    // Test generic snake_case to camelCase conversion
    assert_eq!(convert_to_camel_case("with_file"), "withFile");
    assert_eq!(convert_to_camel_case("set_debug"), "setDebug");
    assert_eq!(convert_to_camel_case("extract_json"), "extractJson");
    assert_eq!(convert_to_camel_case("with_wildcard"), "withWildcard");
    
    // Test edge cases
    assert_eq!(convert_to_camel_case("single"), "single");
    assert_eq!(convert_to_camel_case("already_camelCase"), "alreadyCamelCase");
}

#[test]
fn test_javascript_api_consistency() {
    // Verify Node.js and WASM generate identical function signatures
    // This ensures users get the same API regardless of JavaScript environment
}
```

## Required Build Tools

### Python (PyO3) Distribution
- **Tool**: `maturin` (Python packaging tool for Rust extensions)
- **Install**: `pip install maturin`
- **What it does**: Compiles `superconfig-ffi` Rust code → `superconfig.so` → packages into `.whl`
- **Command**: `maturin build --release`

### Node.js (NAPI) Distribution  
- **Tool**: `@napi-rs/cli` (Node.js native addon build tool)
- **Install**: `npm install -g @napi-rs/cli`
- **What it does**: Compiles `superconfig-ffi` Rust code → `superconfig.node` → packages into `.tgz`
- **Command**: `napi build --platform --release`

### WebAssembly Distribution
- **Tool**: `wasm-pack` (WebAssembly build tool)
- **Install**: `cargo install wasm-pack`
- **What it does**: Compiles `superconfig-ffi` Rust code → `superconfig.wasm` + JS bindings → packages into `.tgz`
- **Command**: `wasm-pack build --target web` (browser) or `--target nodejs` (WASI)

## API Coverage Strategy

### Simple Methods (68% of API)
**Characteristics**: Direct parameter mapping, no complex types  
**Implementation**: One-to-one wrapper around core SuperConfig  
**Languages**: All get native APIs with proper naming conventions

**Examples**:
- `with_file(path: String)` → `with_file(path: str)` (Python) / `withFile(path: string)` (Node.js)
- `with_env(prefix: String)` → Native language equivalents
- `set_debug(debug: bool)` → Native boolean handling

### Complex Methods (21% of API) 
**Characteristics**: Multiple parameters, complex types (Wildcard, SearchStrategy)  
**Implementation**: JSON schema interface with internal conversion  
**Languages**: Accept JSON objects, return native types

**Examples**:
- `with_hierarchical_config(base: String, env: String, auto: bool)` → Multiple native parameters
- `with_wildcard(config: JsonValue)` → JSON object with schema validation

### Debug/Introspection (11% of API)
**Characteristics**: Figment method exposure, debugging utilities  
**Implementation**: JSON serialization of internal state  
**Languages**: Return JSON objects for inspection

## Error Handling Strategy

### Rust Layer
- Maintains existing Result<T, Error> patterns
- No breaking changes to error types

### FFI Layer  
- All errors converted to `Result<T, String>` for language compatibility
- Detailed error messages preserved
- Error context maintained across language boundaries

### Language-Specific
- **Python**: Raises appropriate Python exceptions
- **Node.js**: Returns Error objects with message property
- **WebAssembly**: Returns Result-like objects

## Memory Management

### Rust Objects
- SuperConfig instances managed by Rust ownership system
- No changes to existing memory patterns

### FFI Boundaries
- **Python**: PyO3 handles reference counting automatically
- **Node.js**: NAPI manages garbage collection integration  
- **WebAssembly**: wasm-bindgen handles JS object lifecycle

### JSON Parameters
- Temporary JSON objects created for parameter conversion
- Immediately converted to native types
- No long-term JSON storage

## Security Considerations

### Input Validation
- All JSON parameters validated before conversion
- Schema validation for complex types
- Bounds checking for numeric types

### Memory Safety
- No unsafe code in FFI layer
- All conversions use safe Rust patterns
- Language-specific safety guaranteed by underlying frameworks

### Error Boundaries
- All FFI functions return Results
- No panics propagated across language boundaries
- Graceful degradation for malformed inputs

---
*See [`project-structure.md`](./project-structure.md) for implementation layout*