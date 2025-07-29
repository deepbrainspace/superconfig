# Phase 2: SuperConfig FFI Wrapper

**Status**: 🔄 READY FOR IMPLEMENTATION\
**Estimated Duration**: 4-6 hours\
**Dependencies**: Phase 1 Complete ✅

## Overview

Phase 2 involves creating the `superconfig-ffi` crate that wraps the core SuperConfig API with FFI-compatible interfaces using the SuperFFI macro from Phase 1.

## Deliverables

### 🎯 **Core Objectives**

1. **Create `superconfig-ffi` crate** with feature flags matching SuperFFI
2. **Implement wrapper struct** that contains core SuperConfig instance
3. **Map simple methods** (68% of API) with direct parameter translation
4. **Map complex methods** (21% of API) with JSON parameter handling
5. **Implement error handling** across all FFI boundaries

### 📋 **Implementation Tasks**

#### Task 1: Crate Setup (1 hour)

**Create `crates/superconfig-ffi/Cargo.toml`**:

```toml
[package]
name = "superconfig-ffi"
version = "0.1.0"
edition = "2024"

[dependencies]
superconfig = { path = "../superconfig" }
superffi = { path = "../superffi" }
serde_json = "1.0"
serde = { version = "1.0", features = ["derive"] }

[features]
default = []
python = ["superffi/python"]
nodejs = ["superffi/nodejs"]
wasm = ["superffi/wasm"]
all = ["python", "nodejs", "wasm"]

[lib]
crate-type = ["cdylib", "rlib"] # cdylib needed for WASM
```

**Create `crates/superconfig-ffi/moon.yml`**:

```yaml
language: 'rust'
type: 'library'

tasks:
  test:
    command: 'cargo test'
    inputs: ['src/**/*', 'Cargo.toml']
    
  lint:
    command: 'cargo clippy -- -D warnings'
    inputs: ['src/**/*', 'Cargo.toml']
    
  format:
    command: 'cargo fmt --check'
    inputs: ['src/**/*', 'Cargo.toml']
```

#### Task 2: Core Wrapper Structure (1 hour)

**Create `crates/superconfig-ffi/src/lib.rs`** with basic structure:

```rust
use superconfig::SuperConfig as CoreSuperConfig;
use superffi::superffi;
use serde_json::Value;

/// FFI-compatible wrapper around SuperConfig
#[superffi]
pub struct SuperConfig {
    inner: CoreSuperConfig,
}

#[superffi]
impl SuperConfig {
    /// Create a new SuperConfig instance
    pub fn new() -> Self {
        Self { 
            inner: CoreSuperConfig::new() 
        }
    }
}
```

#### Task 3: Simple Method Mappings (2-3 hours)

**Target Methods (68% of SuperConfig API)**:

```rust
#[superffi]
impl SuperConfig {
    // File-based configuration
    pub fn with_file(&self, path: String) -> Result<Self, String> {
        self.inner.with_file(&path)
            .map(|inner| Self { inner })
            .map_err(|e| e.to_string())
    }
    
    // Environment variable configuration  
    pub fn with_env(&self, prefix: String) -> Result<Self, String> {
        self.inner.with_env(&prefix)
            .map(|inner| Self { inner })
            .map_err(|e| e.to_string())
    }
    
    // Debug mode configuration
    pub fn set_debug(&self, debug: bool) -> Result<Self, String> {
        self.inner.set_debug(debug)
            .map(|inner| Self { inner })
            .map_err(|e| e.to_string())
    }
    
    // Multi-parameter method example
    pub fn with_hierarchical_config(
        &self, 
        base_name: String, 
        env_name: String, 
        auto_profiles: bool
    ) -> Result<Self, String> {
        self.inner.with_hierarchical_config(&base_name, &env_name, auto_profiles)
            .map(|inner| Self { inner })
            .map_err(|e| e.to_string())
    }
    
    // Value extraction
    pub fn extract_json(&self) -> Result<Value, String> {
        let figment_value = self.inner.extract::<figment::value::Value>()
            .map_err(|e| e.to_string())?;
        convert_figment_to_json(figment_value)
    }
}
```

**Generated APIs** (SuperFFI macro produces with naming strategy):

- **Python**: `def with_file(self, path: str) -> SuperConfig` (snake_case preserved)
- **Node.js**: `withFile(path: string): SuperConfig` (converted to camelCase)
- **WASM**: `withFile(path: string): SuperConfig` (converted to camelCase - **identical to Node.js**)

#### Task 4: Complex Method Mappings (1-2 hours)

**JSON Parameter Handling for Complex Types**:

```rust
#[superffi]
impl SuperConfig {
    /// Configure wildcard file discovery via JSON schema
    pub fn with_wildcard(&self, config: Value) -> Result<Self, String> {
        let pattern = config["pattern"].as_str()
            .unwrap_or("*")
            .to_string();
            
        let search_strategy = create_search_strategy(&config["search"])?;
        let merge_order = create_merge_order(&config["merge_order"])?;
        
        let wildcard = Wildcard::from_pattern(&pattern)
            .with_search_strategy(search_strategy)
            .with_merge_order(merge_order);
            
        self.inner.merge(wildcard)
            .map(|inner| Self { inner })
            .map_err(|e| e.to_string())
    }
    
    /// Find configuration value by path
    pub fn find(&self, path: String) -> Result<Option<Value>, String> {
        match self.inner.find_ref(&path) {
            Some(figment_value) => Ok(Some(convert_figment_to_json(figment_value)?)),
            None => Ok(None)
        }
    }
}

// Helper functions for JSON conversion
fn create_search_strategy(config: &Value) -> Result<SearchStrategy, String> {
    match config["type"].as_str().unwrap_or("current") {
        "current" => Ok(SearchStrategy::Current),
        "recursive" => Ok(SearchStrategy::Recursive {
            roots: vec![PathBuf::from(config["root"].as_str().unwrap_or("."))],
            max_depth: config["max_depth"].as_u64().map(|d| d as usize),
        }),
        "directories" => {
            let dirs: Result<Vec<_>, _> = config["directories"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|v| v.as_str().map(PathBuf::from))
                .collect::<Option<Vec<_>>>()
                .ok_or("Invalid directories array")?;
            Ok(SearchStrategy::Directories(dirs))
        },
        invalid => Err(format!("Invalid search strategy: {}", invalid))
    }
}

fn convert_figment_to_json(figment_value: figment::value::Value) -> Result<Value, String> {
    // Implementation for converting Figment values to serde_json::Value
    // Handle all Figment value types: Bool, Num, String, Dict, Array, etc.
}
```

**JSON Schema Example**:

```json
{
  "pattern": "*.toml",
  "search": {
    "type": "recursive",
    "root": "./config",
    "max_depth": 3
  },
  "merge_order": {
    "type": "custom",
    "patterns": ["base.*", "env-*.toml", "local.*"]
  }
}
```

## API Coverage Strategy

### **Simple Methods (68% of API)**

**Characteristics**: Direct parameter mapping, primitive types only
**Implementation**: One-to-one wrapper with error string conversion
**Languages**: Native APIs with proper type conversion

**Examples**:

- Configuration sources: `with_file`, `with_env`, `with_directory`
- Behavior settings: `set_debug`, `set_strict_mode`
- Hierarchical configs: `with_hierarchical_config`
- Profile management: `with_profile`, `with_auto_profiles`

### **Complex Methods (21% of API)**

**Characteristics**: Complex types requiring JSON schemas
**Implementation**: JSON parameter interface with internal conversion
**Languages**: Accept JSON objects, return native types

**Examples**:

- Wildcard configuration: `with_wildcard(json_config)`
- Custom providers: `with_provider(json_spec)`
- Advanced merging: `with_merge_strategy(json_strategy)`

### **Debug/Introspection (11% of API)**

**Characteristics**: Figment method exposure for debugging
**Implementation**: JSON serialization of internal state
**Languages**: Return JSON objects for inspection

**Examples**:

- Value extraction: `extract_json() -> JSON`
- Path finding: `find(path) -> Option<JSON>`
- Configuration inspection: `debug_info() -> JSON`

## Error Handling Strategy

### **Rust Error Types → String Conversion**

```rust
// All FFI methods return Result<T, String>
pub fn with_file(&self, path: String) -> Result<Self, String> {
    self.inner.with_file(&path)
        .map(|inner| Self { inner })
        .map_err(|e| format!("Failed to load file '{}': {}", path, e))
}
```

### **Language-Specific Error Handling**

- **Python**: SuperFFI generates `PyResult<T>` with appropriate Python exceptions
- **Node.js**: SuperFFI generates NAPI error handling with Error objects
- **WebAssembly**: SuperFFI generates `Result<T, JsValue>` for JS error compatibility

### **Error Context Preservation**

- File paths included in file loading errors
- JSON schema validation errors with path information
- Type conversion errors with expected vs actual type information

## Testing Strategy

### **Unit Tests** (`crates/superconfig-ffi/tests/`)

```rust
#[test]
fn test_basic_wrapper_functionality() {
    let config = SuperConfig::new();
    let result = config.with_file("test.toml".to_string());
    assert!(result.is_ok());
}

#[test]
fn test_json_parameter_conversion() {
    let config = SuperConfig::new();
    let wildcard_config = json!({
        "pattern": "*.toml",
        "search": {"type": "current"}
    });
    let result = config.with_wildcard(wildcard_config);
    assert!(result.is_ok());
}

#[test]
fn test_error_handling() {
    let config = SuperConfig::new();
    let result = config.with_file("nonexistent.toml".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to load file"));
}
```

### **Feature Flag Testing**

```rust
#[cfg(feature = "python")]
#[test]
fn test_python_feature_compilation() {
    // Verify Python bindings compile correctly
}

#[cfg(feature = "nodejs")]  
#[test]
fn test_nodejs_feature_compilation() {
    // Verify Node.js bindings compile correctly
}
```

## Success Metrics

### **Completion Criteria**

- [ ] All simple methods (68% of API) mapped and tested
- [ ] Complex methods (21% of API) accept JSON parameters
- [ ] Error handling works across all language boundaries
- [ ] Feature flags compile correctly for all targets
- [ ] Unit tests pass for all implemented methods

### **Quality Targets**

- [ ] Zero unsafe code in FFI layer
- [ ] All public methods documented with examples
- [ ] Comprehensive error messages with context
- [ ] Memory safety guaranteed by underlying frameworks

### **Integration Readiness**

- [ ] SuperFFI macro annotations working correctly
- [ ] JSON parameter validation implemented
- [ ] Ready for Phase 3 complex type integration
- [ ] Build system integration points identified

## Next Phase Preparation

**Phase 3 Dependencies**:

- JSON schema validation utilities implemented
- Complex type conversion helpers created
- Figment-to-JSON conversion working
- Error handling patterns established

---

_Ready for implementation. See [`build-system.md`](./build-system.md) for Moon task configuration._
