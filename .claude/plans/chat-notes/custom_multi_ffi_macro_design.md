# Custom Multi-FFI Macro Design

**Created**: July 28, 2025  
**Status**: Architecture Design  
**Goal**: Eliminate signature duplication across PyO3, napi-rs, and UniFFI

## 🎯 Problem Statement

Current multi-FFI approaches require duplicating method signatures:

```rust
// Core API
impl SuperConfig {
    pub fn with_file(self, path: String) -> Self { /* ... */ }
}

// Node.js wrapper - DUPLICATE signature
#[napi] pub fn with_file_js(self, path: String) -> Self { self.with_file(path) }

// Python wrapper - DUPLICATE signature  
fn with_file_py(self, path: String) -> Self { self.with_file(path) }

// UniFFI wrapper - DUPLICATE signature
pub fn with_file_uniffi(self: Arc<Self>, path: String) -> Arc<Self> { /* ... */ }
```

**Problem**: Every new method = 4 signature definitions to maintain.

## 🔧 Proposed Solution: `#[multi_ffi]` Macro

### Usage API

```rust
use multi_ffi::multi_ffi;

#[multi_ffi(nodejs, python, uniffi)]
pub struct SuperConfig {
    builder: figment::Figment,
}

#[multi_ffi(nodejs, python, uniffi)]
impl SuperConfig {
    #[constructor]
    pub fn new() -> Self {
        Self {
            builder: figment::Figment::new(),
        }
    }
    
    pub fn with_file(self, path: String) -> Self {
        Self {
            builder: self.builder.merge(figment::providers::Json::file(path)),
        }
    }
    
    pub fn with_env(self, prefix: String) -> Self {
        Self {
            builder: self.builder.merge(figment::providers::Env::prefixed(&prefix)),
        }
    }
    
    pub fn extract_json(&self) -> Result<String, String> {
        self.builder.extract::<serde_json::Value>()
            .map(|v| v.to_string())
            .map_err(|e| e.to_string())
    }
}
```

### Generated Output

The macro expands to generate all FFI wrappers automatically:

```rust
// Original implementation (unchanged)
pub struct SuperConfig {
    builder: figment::Figment,
}

impl SuperConfig {
    pub fn new() -> Self { /* user's implementation */ }
    pub fn with_file(self, path: String) -> Self { /* user's implementation */ }
    pub fn with_env(self, prefix: String) -> Self { /* user's implementation */ }
    pub fn extract_json(&self) -> Result<String, String> { /* user's implementation */ }
}

// GENERATED: Node.js bindings
#[cfg(feature = "nodejs")]
#[napi]
impl SuperConfig {
    #[napi(constructor)]
    pub fn nodejs_new() -> Self {
        Self::new()
    }
    
    #[napi]
    pub fn nodejs_with_file(self, path: String) -> Self {
        self.with_file(path)
    }
    
    #[napi]
    pub fn nodejs_with_env(self, prefix: String) -> Self {
        self.with_env(prefix)
    }
    
    #[napi]
    pub fn nodejs_extract_json(&self) -> napi::Result<String> {
        self.extract_json().map_err(|e| napi::Error::from_reason(e))
    }
}

// GENERATED: Python bindings
#[cfg(feature = "python")]
#[pyo3::pymethods]
impl SuperConfig {
    #[new]
    fn python_new() -> Self {
        Self::new()
    }
    
    fn python_with_file(self, path: String) -> Self {
        self.with_file(path)
    }
    
    fn python_with_env(self, prefix: String) -> Self {
        self.with_env(prefix)
    }
    
    fn python_extract_json(&self) -> PyResult<String> {
        self.extract_json().map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))
    }
}

// GENERATED: UniFFI bindings
#[cfg(feature = "uniffi")]
#[uniffi::export]
impl SuperConfig {
    #[uniffi::constructor]
    pub fn uniffi_new() -> Arc<Self> {
        Arc::new(Self::new())
    }
    
    pub fn uniffi_with_file(self: Arc<Self>, path: String) -> Arc<Self> {
        Arc::new(Arc::try_unwrap(self).unwrap().with_file(path))
    }
    
    pub fn uniffi_with_env(self: Arc<Self>, prefix: String) -> Arc<Self> {
        Arc::new(Arc::try_unwrap(self).unwrap().with_env(prefix))
    }
    
    pub fn uniffi_extract_json(&self) -> Result<String, String> {
        self.extract_json()
    }
}
```

## 🏗️ Macro Implementation Strategy

### Phase 1: Parse Attributes
```rust
// Parse the target languages
#[multi_ffi(nodejs, python, uniffi)]
//          ^^^^^^^^^^^^^^^^^^^^^^^ - Extract these
```

### Phase 2: Generate FFI Wrappers
For each target language, generate appropriate wrapper:

```rust
match target {
    "nodejs" => generate_napi_wrapper(impl_block),
    "python" => generate_pyo3_wrapper(impl_block), 
    "uniffi" => generate_uniffi_wrapper(impl_block),
    _ => panic!("Unsupported FFI target: {}", target),
}
```

### Phase 3: Handle Type Conversions
The macro needs to handle different type requirements:

```rust
// Original method
pub fn with_file(self, path: String) -> Self

// Node.js: Direct mapping (String -> String)
#[napi] pub fn nodejs_with_file(self, path: String) -> Self

// Python: Direct mapping (String -> String)  
fn python_with_file(self, path: String) -> Self

// UniFFI: Arc wrapping required
pub fn uniffi_with_file(self: Arc<Self>, path: String) -> Arc<Self>
```

### Phase 4: Error Handling Conversion
```rust
// Original return type
Result<String, String>

// Node.js conversion
napi::Result<String> // Convert String error to napi::Error

// Python conversion  
PyResult<String> // Convert String error to PyErr

// UniFFI conversion
Result<String, String> // Direct mapping
```

## 📋 Implementation Roadmap

### Step 1: Basic Proc Macro (2-3 hours)
- Parse `#[multi_ffi(targets)]` attribute
- Generate basic wrapper functions
- Handle simple types (String, i32, bool)

### Step 2: Constructor Support (1 hour)
- Handle `#[constructor]` attribute
- Generate appropriate constructor wrappers per language

### Step 3: Error Handling (1-2 hours)
- Convert Result types to target-specific error types
- Handle Option types appropriately

### Step 4: Complex Types (2-3 hours)
- Support custom structs and enums
- Handle Arc wrapping for UniFFI
- Support generic parameters

### Step 5: Testing & Documentation (1 hour)
- Create comprehensive test suite
- Write usage documentation
- Performance benchmarks

## 🎯 Benefits

1. **Zero Signature Duplication**: Write interface once
2. **Maximum Performance**: napi-rs for Node.js, PyO3 for Python
3. **Broad Coverage**: UniFFI for remaining languages
4. **Type Safety**: Compile-time validation of all bindings
5. **Maintainability**: Add new methods in one place

## 🔧 Usage in SuperConfig

```toml
[dependencies]
multi-ffi = "0.1"  # Our custom macro crate
pyo3 = { version = "0.20", optional = true }
napi = { version = "2", optional = true }
napi-derive = { version = "2", optional = true }
uniffi = { version = "0.25", optional = true }

[features]
nodejs = ["napi", "napi-derive", "multi-ffi/nodejs"]
python = ["pyo3", "multi-ffi/python"]  
uniffi = ["dep:uniffi", "multi-ffi/uniffi"]
```

## 🚀 Expected Outcome

- **Single source of truth**: One interface definition
- **No performance compromise**: Native speed for Node.js and Python
- **Comprehensive coverage**: 95%+ of configuration management market
- **Maintenance efficiency**: Add features in one place
- **Type safety**: Compile-time validation across all languages

This approach gives us the best of all worlds: the performance of direct FFI bindings with the convenience of single interface definition.