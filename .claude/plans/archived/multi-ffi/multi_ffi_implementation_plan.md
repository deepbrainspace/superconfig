# Multi-FFI Macro Implementation Plan

**Created**: July 28, 2025  
**Updated**: July 28, 2025  
**Status**: Ready for Implementation  
**Priority**: High  
**Scope**: Python + Node.js (WASM consideration for future)

## 🎯 Decision: Focused Dual-Language Strategy

**Crate Name**: `multi-ffi`  
**Target Languages**: Python (PyO3) + Node.js (napi-rs)  
**Future Consideration**: WASM via wasm-bindgen  

**Why This Scope:**
- ✅ **75% market coverage** with Python (49.3%) + Node.js (62.3%)
- ✅ **Maximum performance** for both targets 
- ✅ **Manageable complexity** - 2 FFI systems vs 3+
- ✅ **Faster delivery** - 6-8 hours vs 10-12 hours

## 🏗️ Architecture Decision: Separate Crate

**Why separate crate:**
✅ **Reusability** - Other Rust projects can use it  
✅ **Clean separation** - SuperConfig focuses on config logic  
✅ **Independent versioning** - Macro can evolve separately  
✅ **Open source potential** - Becomes ecosystem contribution  
✅ **Testing isolation** - Easier to test macro independently  

**Repository Structure:**
```
superconfig/
├── crates/
│   ├── superconfig/           # Main library
│   └── multi-ffi/            # Our custom macro
├── examples/
└── README.md
```

## 📦 Crate Setup

### multi-ffi/Cargo.toml
```toml
[package]
name = "multi-ffi"
version = "0.1.0"
edition = "2021"
description = "Procedural macro for generating multi-language FFI bindings"
license = "MIT OR Apache-2.0"
repository = "https://github.com/deepbrainspace/superconfig"

[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1.0"
syn = { version = "2.0", features = ["full", "extra-traits"] }
quote = "1.0"

# FFI dependencies (encapsulated in this crate)
pyo3 = { version = "0.25", optional = true }
napi = { version = "3", optional = true }
napi-derive = { version = "3", optional = true }

[features]
default = []
python = ["pyo3"]
nodejs = ["napi", "napi-derive"]

[dev-dependencies]
trybuild = "1.0"  # For testing macro expansion
```

### superconfig/Cargo.toml
```toml
[package]
name = "superconfig" 
version = "0.1.0"
edition = "2021"

[dependencies]
multi-ffi = { path = "../multi-ffi" }
figment = "0.10"
serde = { version = "1.0", features = ["derive"] }

# No FFI dependencies here - they're encapsulated in multi-ffi
[features]
default = []
python = ["multi-ffi/python"]
nodejs = ["multi-ffi/nodejs"]
```

## 🎯 Implementation Phases

### Phase 1: MVP Macro (4-6 hours)
- [x] Basic proc macro setup
- [ ] Parse `#[multi_ffi(nodejs, python)]` attributes
- [ ] Generate simple method wrappers
- [ ] Handle basic types (String, i32, bool)
- [ ] Feature flag generation

### Phase 2: SuperConfig Integration (2-3 hours)  
- [ ] Apply macro to SuperConfig impl
- [ ] Test Node.js bindings work
- [ ] Test Python bindings work
- [ ] Verify performance benchmarks

### Phase 3: Advanced Features (2-3 hours)
- [ ] Constructor support (`#[constructor]`)
- [ ] Error handling (`Result<T, E>`)
- [ ] Complex type support (Vec, HashMap)
- [ ] Generic method support

### Phase 4: Polish & Documentation (1-2 hours)
- [ ] Comprehensive tests
- [ ] Usage documentation
- [ ] Error messages
- [ ] Examples

## 📋 Implementation Checklist

### Multi-FFI Crate Tasks
- [ ] Create `crates/multi-ffi/` directory
- [ ] Set up proc macro boilerplate
- [ ] Implement attribute parsing
- [ ] Create code generation engine
- [ ] Add feature flag support
- [ ] Write test suite

### SuperConfig Integration Tasks  
- [ ] Add multi-ffi dependency
- [ ] Apply `#[multi_ffi(nodejs, python)]` to impl
- [ ] Update build scripts for each target
- [ ] Create language-specific examples
- [ ] Performance benchmarking

### Build System Tasks
- [ ] GitHub Actions for multi-platform builds
- [ ] Python wheel generation (maturin)
- [ ] Node.js native module builds (napi-rs)
- [ ] Documentation generation
- [ ] Evaluate WASM integration (future consideration)

## 🚀 Usage Preview

```rust
// crates/superconfig/src/lib.rs
use multi_ffi::multi_ffi;

#[multi_ffi(nodejs, python)]
impl SuperConfig {
    #[constructor]
    pub fn new() -> Self {
        Self { builder: Figment::new() }
    }
    
    pub fn with_file(self, path: String) -> Self {
        Self {
            builder: self.builder.merge(Json::file(path)),
        }
    }
    
    pub fn extract_json(&self) -> Result<String, String> {
        self.builder.extract::<serde_json::Value>()
            .map(|v| v.to_string())
            .map_err(|e| e.to_string())
    }
}
```

**Generated output:**
- Node.js: `nodejs_new()`, `nodejs_with_file()`, `nodejs_extract_json()`
- Python: `python_new()`, `python_with_file()`, `python_extract_json()`

## ⚡ Success Metrics

- [ ] **Zero signature duplication** - Write interface once
- [ ] **Native performance** - Node.js gets napi-rs (~700ns/call), Python gets PyO3 (~700ns/call)
- [ ] **Focused coverage** - Python + Node.js = 75% market coverage  
- [ ] **Easy maintenance** - Add methods in one place
- [ ] **Reusable tool** - Other projects can adopt multi-ffi

## 🎯 Next Steps

1. **Create multi-ffi crate** - Set up the separate crate structure
2. **Implement MVP macro** - Basic method wrapper generation
3. **Test with SuperConfig** - Validate it works end-to-end
4. **Add advanced features** - Constructors, error handling, etc.
5. **Open source release** - Publish to crates.io

## 📊 SuperConfig Functionality Coverage

| Method/Feature | FFI Support | Work Required | Notes |
|----------------|-------------|---------------|--------|
| **Constructor** |
| `SuperConfig::new()` | ✅ Perfect | None | `#[constructor]` attribute |
| **Fluent Builder Methods** |
| `with_verbosity(VerbosityLevel)` | ✅ Perfect | None | Enum serializes easily |
| `with_file(path)` | ✅ Perfect | None | String path parameter |
| `with_env(prefix)` | ✅ Perfect | None | String prefix parameter |
| `with_hierarchical_config(name)` | ✅ Perfect | None | String name parameter |
| `with_defaults_string(content)` | ✅ Perfect | None | String content parameter |
| `with_file_opt(Option<path>)` | ✅ Perfect | None | Handle Option in macro |
| `with_env_ignore_empty(prefix)` | ✅ Perfect | None | String prefix parameter |
| **Generic Builder Methods** |
| `with_defaults<T: Serialize>(defaults)` | ⚠️ Needs Work | JSON conversion | Convert T to JSON string input |
| `with_cli_opt<T: Serialize>(cli_opt)` | ⚠️ Needs Work | JSON conversion | Convert T to JSON string input |
| **Extraction Methods** |
| `extract<T: Deserialize>()` | ⚠️ Needs Work | JSON extraction | Provide `extract_json() -> String` |
| `as_json()` | ✅ Perfect | None | Already returns String |
| `as_yaml()` | ✅ Perfect | None | Already returns String |
| `as_toml()` | ✅ Perfect | None | Already returns String |
| **Simple Accessors** |
| `get_string(key)` | ✅ Perfect | None | String in/out |
| `has_key(key)` | ✅ Perfect | None | String -> bool |
| `keys()` | ✅ Perfect | None | Returns Vec<String> |
| **Generic Accessors** |
| `get_array<T>(key)` | ⚠️ Needs Work | JSON arrays | Return JSON array string |
| **Complex Debug Methods** |
| `debug_messages()` | ⚠️ Needs Work | JSON serialization | Serialize Vec<DebugMessage> to JSON |
| `debug_sources()` | ⚠️ Needs Work | JSON serialization | Serialize Vec<Metadata> to JSON |
| `debug_config()` | ✅ Perfect | None | Already returns String |
| **Simple Debug Methods** |
| `verbosity()` | ✅ Perfect | None | Enum -> string |
| `print_debug_messages()` | ✅ Perfect | None | No return value |
| `clear_debug_messages()` | ✅ Perfect | None | No return value |
| **Warning System** |
| `warnings()` | ✅ Perfect | None | Returns Vec<String> |
| `has_warnings()` | ✅ Perfect | None | Returns bool |
| `print_warnings()` | ✅ Perfect | None | No return value |
| **Complex Merge Methods** |
| `merge<P: Provider>(provider)` | ❌ Complex | Provider abstraction | Pre-instantiate common providers |
| `merge_validated<P>(provider)` | ❌ Complex | Provider abstraction | Pre-instantiate common providers |
| `merge_opt<P>(provider)` | ❌ Complex | Provider abstraction | Pre-instantiate common providers |

### Coverage Summary
- ✅ **Perfect Support**: 19/28 methods (68%)
- ⚠️ **Needs Work**: 6/28 methods (21%) 
- ❌ **Complex**: 3/28 methods (11%)

### Work Categories

#### 1. JSON Conversion (Easy - 2 hours)
- `with_defaults()` → `with_defaults_json(json_string)`
- `with_cli_opt()` → `with_cli_json(json_string)`  
- `get_array<T>()` → `get_array_json(key) -> String`

#### 2. JSON Serialization (Easy - 1 hour)
- `debug_messages()` → `debug_messages_json() -> String`
- `debug_sources()` → `debug_sources_json() -> String`

#### 3. Provider Abstraction (Medium - 3 hours)
Replace generic `merge()` with specific methods:
- `merge_json_file(path)`
- `merge_toml_file(path)`
- `merge_yaml_file(path)`
- `merge_env_nested(prefix)`

**Total SuperConfig Coverage Time**: ~6 hours to achieve 100% functionality coverage

**Estimated Total Time: 6-8 hours** (reduced scope: Python + Node.js only)

This focused approach delivers maximum performance for the two most important languages while maintaining clean architecture and reusability.