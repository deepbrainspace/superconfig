# SuperConfig SuperFFI Architecture Plan

**Status**: Planning Phase  
**Priority**: High  
**Estimated Time**: 1-1.5 weeks with AI coding assistant (Claude Sonnet 4/Opus 4)  
**Dependencies**: Core SuperConfig stable API  

## Executive Summary

This plan outlines the implementation of a dual-layer architecture for SuperConfig multi-language support:
1. **Core `superconfig` crate**: Maintains native Rust API with zero FFI overhead
2. **New `superconfig-ffi` crate**: Provides JSON-based wrapper optimized for FFI
3. **New `superffi` macro crate**: Generates Python, Node.js, and WASM bindings automatically

This approach preserves Rust performance while optimizing FFI user experience and simplifying multi-language binding generation.

**Key Architecture Benefits:**
- **Single source of truth**: Write each method once, generates native APIs for all enabled languages
- **Feature flag flexibility**: Independently build Python-only, Node.js-only, or both
- **Native language ergonomics**: Users get clean, language-appropriate APIs (no JSON manipulation)
- **Zero performance regression**: Core Rust API remains unchanged
- **Incremental development**: Start with one language, add others progressively

## Architecture Overview

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   superconfig   │    │ superconfig-ffi  │    │    superffi     │
│                 │    │                  │    │                 │
│ Native Rust API │◄───│ JSON Wrapper API │◄───│ Macro Generator │
│ High Performance│    │ FFI Compatible   │    │ Py + Node + WASM│
│ Zero FFI Cost   │    │ serde_json::Value│    │ Auto Bindings   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Performance Benefits

| User Type | Current | With This Plan | Impact |
|-----------|---------|----------------|---------|
| **Rust** | Native types (optimal) | Native types (unchanged) | **No regression** |
| **Python** | Complex PyO3 marshaling | Simple JSON parsing | **Major improvement** |
| **Node.js** | Complex napi-rs marshaling | Simple JSON parsing | **Major improvement** |

## Project Structure (Moon Monorepo)

```
superconfig/                    # Moon workspace root
├── moon.yml                   # Workspace-level tasks and config
├── .moon/                     # Moon metadata (generated)
├── crates/
│   ├── superconfig/           # Core Rust API project (published to crates.io)
│   │   ├── moon.yml          # Project-specific tasks
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── superconfig-ffi/       # FFI wrapper + all language bindings
│   │   ├── moon.yml          # Build, package, publish ALL targets
│   │   ├── Cargo.toml        # Rust FFI code configuration
│   │   ├── src/              # Rust FFI implementation
│   │   ├── python/           # Python packaging (published to PyPI as "superconfig")
│   │   │   ├── setup.py      # Python package configuration
│   │   │   ├── pyproject.toml # Modern Python configuration
│   │   │   └── superconfig/  # Python module
│   │   │       └── __init__.py
│   │   ├── nodejs/           # Node.js server packaging (published to npm as "superconfig.js")
│   │   │   ├── package.json  # Node.js package configuration
│   │   │   ├── index.js      # Entry point
│   │   │   ├── src/          # Source files
│   │   │   └── lib/          # Build output
│   │   └── wasm/             # Browser/Universal JS packaging (published to npm as "superconfig-wasm")
│   │       ├── package.json  # WASM package configuration
│   │       ├── webpack.config.js # WASM bundling config
│   │       ├── src/          # TypeScript wrappers
│   │       └── pkg/          # wasm-pack output
│   └── superffi/             # Macro generator project
│       ├── moon.yml          # Macro development tasks
│       ├── Cargo.toml
│       └── src/
└── examples/
    ├── rust/                 # Using core superconfig
    ├── python/               # Using published Python package
    └── nodejs/               # Using published Node.js package
```

## Implementation Plan

### Phase 1: SuperFFI Macro Foundation (Days 1-2)

#### 1.1 Create `superffi` Crate with Feature Flags
```toml
# crates/superffi/Cargo.toml
[package]
name = "superffi"
version = "0.1.0"

[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1.0"
quote = "1.0" 
syn = "2.0"
pyo3 = { version = "0.20", optional = true }
napi = { version = "2.0", optional = true }
napi-derive = { version = "2.0", optional = true }
wasm-bindgen = { version = "0.2", optional = true }
js-sys = { version = "0.3", optional = true }
serde-wasm-bindgen = { version = "0.6", optional = true }

[features]
default = []
python = ["pyo3"]
nodejs = ["napi", "napi-derive"]
wasm = ["wasm-bindgen", "js-sys", "serde-wasm-bindgen"]
all = ["python", "nodejs", "wasm"]
```

#### 1.2 Implement Single Macro with Conditional Generation
```rust
// crates/superffi/src/lib.rs
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn superffi(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse which targets are enabled via feature flags
    let python_enabled = cfg!(feature = "python");
    let nodejs_enabled = cfg!(feature = "nodejs");
    let wasm_enabled = cfg!(feature = "wasm");
    
    // Generate core implementation + conditional FFI bindings
    generate_bindings(input, python_enabled, nodejs_enabled, wasm_enabled)
}

// Generates:
// 1. Core internal method (always)
// 2. Python binding with PyO3 (if python feature enabled)  
// 3. Node.js binding with napi-rs (if nodejs feature enabled)
// 4. WASM binding with wasm-bindgen (if wasm feature enabled)
```

#### 1.3 JSON Parameter Handling
```rust
// Automatic conversion from serde_json::Value to native types
pub fn handle_json_param(value: &serde_json::Value, expected_type: &str) -> TokenStream {
    match expected_type {
        "String" => quote! { value.as_str()?.to_string() },
        "PathBuf" => quote! { PathBuf::from(value.as_str()?) },
        "bool" => quote! { value.as_bool()? },
        "u32" => quote! { value.as_u64()? as u32 },
        // ... handle all common types
    }
}
```

### Phase 2: SuperConfig FFI Wrapper (Days 3-4)

#### 2.1 Create `superconfig-ffi` Crate with Feature Flags
```toml
# crates/superconfig-ffi/Cargo.toml
[package]
name = "superconfig-ffi"
version = "0.1.0"

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
crate-type = ["cdylib", "rlib"]  # cdylib needed for WASM
```

#### 2.2 Implement Core Wrapper Structure
```rust
// crates/superconfig-ffi/src/lib.rs
use superconfig::SuperConfig as CoreSuperConfig;
use superffi::superffi;
use serde_json::Value;

#[superffi]
pub struct SuperConfig {
    inner: CoreSuperConfig,
}

#[superffi]
impl SuperConfig {
    pub fn new() -> Self {
        Self { inner: CoreSuperConfig::new() }
    }
}
```

#### 2.3 Single Implementation with Native Language APIs (68% of API)
```rust
#[superffi]
impl SuperConfig {
    // Written once, generates native APIs for each language
    pub fn with_file(&self, path: String) -> Result<Self, String> {
        Ok(Self { inner: self.inner.with_file(path) })
    }
    
    pub fn with_env(&self, prefix: String) -> Result<Self, String> {
        Ok(Self { inner: self.inner.with_env(prefix) })
    }
    
    pub fn set_debug(&self, debug: bool) -> Result<Self, String> {
        Ok(Self { inner: self.inner.set_debug(debug) })
    }
}

// Macro generates:
// Python: def with_file(self, path: str) -> SuperConfig
// Node.js: withFile(path: string): SuperConfig  
// WASM: with_file(path: string): SuperConfig (via wasm-bindgen)
// Core: fn with_file_internal(&self, path: String) -> Result<Self, String>
```

#### 2.4 Complex Method Mappings with Multiple Parameters (21% of API)
```rust
#[superffi]
impl SuperConfig {
    // Multi-parameter methods generate native APIs
    pub fn with_hierarchical_config(
        &self, 
        base_name: String, 
        env_name: String, 
        auto_profiles: bool
    ) -> Result<Self, String> {
        Ok(Self { 
            inner: self.inner.with_hierarchical_config(&base_name, &env_name, auto_profiles)
        })
    }
    
    // Complex types use JSON internally but still native APIs
    #[multi_ffi(json_internal)]
    pub fn with_wildcard(&self, config: serde_json::Value) -> Result<Self, String> {
        let pattern = config["pattern"].as_str().unwrap_or("*");
        let search_strategy = create_search_strategy(&config["search"])?;
        let merge_order = create_merge_order(&config["merge_order"])?;
        
        let wildcard = Wildcard::from_pattern(pattern)
            .with_search_strategy(search_strategy)
            .with_merge_order(merge_order);
            
        Ok(Self { inner: self.inner.merge(wildcard) })
    }
}

// Macro generates:
// Python: def with_hierarchical_config(self, base_name: str, env_name: str, auto_profiles: bool) -> SuperConfig
// Node.js: withHierarchicalConfig(baseName: string, envName: string, autoProfiles: boolean): SuperConfig
// WASM: with_hierarchical_config(base_name: string, env_name: string, auto_profiles: boolean): SuperConfig

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
```

### Phase 3: Complex Type Handling (Days 5-6)

#### 3.1 Wildcard Provider JSON Schema
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

#### 3.2 Figment Method Exposure
```rust
#[superffi]
impl SuperConfig {
    // Expose key Figment methods through JSON interface
    pub fn extract_json(&self) -> Result<Value, String> {
        let figment_value = self.inner.extract::<figment::value::Value>()
            .map_err(|e| e.to_string())?;
        convert_figment_to_json(figment_value)
    }
    
    pub fn find(&self, path: Value) -> Result<Option<Value>, String> {
        let path_str = path.as_str()
            .ok_or("Expected string for path")?;
        
        match self.inner.find_ref(path_str) {
            Some(figment_value) => Ok(Some(convert_figment_to_json(figment_value)?)),
            None => Ok(None)
        }
    }
}
```

### Phase 4: Build & Publishing Integration with Feature Flags (Day 7)

#### 4.1 Unified Moon Task Configuration
```yaml
# Workspace-level moon.yml
tasks:
  build-all:
    command: 'moon run superconfig-ffi:package-all'
    
  publish-all:
    command: 'moon run superconfig-ffi:publish-all'

# crates/superffi/moon.yml
language: 'rust'
type: 'library'

tasks:
  build:
    command: 'cargo build'
    inputs: ['@globs(sources)', 'Cargo.toml']
    
  test:
    command: 'cargo test'
    inputs: ['@globs(sources)', '@globs(tests)']
    
  lint:
    command: 'cargo clippy -- -D warnings'
    inputs: ['@globs(sources)', '@globs(tests)']
    
  format:
    command: 'cargo fmt --check'
    inputs: ['@globs(sources)', '@globs(tests)']

# crates/superconfig-ffi/moon.yml
language: 'rust'
type: 'library'

tasks:
  # Rust FFI builds
  build-python:
    command: 'cargo build --features python'
    inputs: ['@globs(sources)', 'Cargo.toml']
    deps: ['superffi:build']
    
  build-nodejs:
    command: 'cargo build --features nodejs'
    inputs: ['@globs(sources)', 'Cargo.toml']
    deps: ['superffi:build']
    
  build-wasm:
    command: 'wasm-pack build --target bundler --features wasm'
    inputs: ['@globs(sources)', 'Cargo.toml']
    outputs: ['pkg/']
    deps: ['superffi:build']
    
  build-all:
    command: 'cargo build --features all'
    inputs: ['@globs(sources)', 'Cargo.toml']
    deps: ['superffi:build']
    
  # Python packaging (inside superconfig-ffi)
  package-python:
    command: 'cd python && python -m build'
    inputs: ['python/**/*', '../../target/release/*superconfig*']
    outputs: ['python/dist/']
    deps: ['build-python']
    
  # Node.js server packaging (inside superconfig-ffi)  
  package-nodejs:
    command: 'cd nodejs && npm run build'
    inputs: ['nodejs/**/*', '../../target/release/*superconfig*']
    outputs: ['nodejs/lib/']
    deps: ['build-nodejs']
    
  # WASM browser packaging (inside superconfig-ffi)
  package-wasm:
    command: 'cd wasm && npm run build'
    inputs: ['wasm/**/*', 'pkg/**/*']
    outputs: ['wasm/dist/']
    deps: ['build-wasm']
    
  # Publishing
  publish-python:
    command: 'cd python && python -m twine upload dist/*'
    deps: ['package-python']
    
  publish-nodejs:
    command: 'cd nodejs && npm publish'  # Published as "superconfig.js"
    deps: ['package-nodejs']
    
  publish-wasm:
    command: 'cd wasm && npm publish'    # Published as "superconfig-wasm"
    deps: ['package-wasm']
    
  # Testing
  test-python:
    command: 'cd python && python -m pytest'
    deps: ['package-python']
    
  test-nodejs:
    command: 'cd nodejs && npm test'
    deps: ['package-nodejs']
    
  test-wasm:
    command: 'cd wasm && npm test'
    deps: ['package-wasm']
    
  # Combined tasks
  package-all:
    deps: ['package-python', 'package-nodejs', 'package-wasm']
    
  test-all:
    deps: ['test-python', 'test-nodejs', 'test-wasm']
    
  publish-all:
    deps: ['publish-python', 'publish-nodejs', 'publish-wasm']
```

#### 4.2 Simplified Development Commands
```bash
# Build specific targets with Moon
moon run superconfig-ffi:build-python
moon run superconfig-ffi:build-nodejs  
moon run superconfig-ffi:build-all

# Development tasks for individual crates
moon run superffi:build
moon run superffi:test
moon run superffi:lint

moon run superconfig-ffi:test
moon run superconfig-ffi:lint

# Package and test specific languages
moon run python:build
moon run nodejs:build

# Test language bindings
moon run python:test
moon run nodejs:test

# Publish to package registries
moon run python:publish
moon run nodejs:publish

# Full release workflow
moon run release-all
```

#### 4.3 GitHub Actions with Moon Integration
```yaml
# .github/workflows/multi-ffi-release.yml
name: Multi-FFI Release

on:
  push:
    tags: ['v*']

jobs:
  build-python:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - name: Install Moon
        run: |
          curl -fsSL https://moonrepo.dev/install/moon.sh | bash
          echo "$HOME/.moon/bin" >> $GITHUB_PATH
      - name: Build and package Python bindings
        run: moon run python:build
      - name: Publish to PyPI
        uses: pypa/gh-action-pypi-publish@release/v1
        
  build-nodejs:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - name: Install Moon
        run: |
          curl -fsSL https://moonrepo.dev/install/moon.sh | bash
          echo "$HOME/.moon/bin" >> $GITHUB_PATH
      - name: Build and package Node.js bindings
        run: moon run nodejs:build
      - name: Publish to npm
        run: npm publish
        
  build-all:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - name: Install Moon
        run: |
          curl -fsSL https://moonrepo.dev/install/moon.sh | bash
          echo "$HOME/.moon/bin" >> $GITHUB_PATH
      - name: Build all targets
        run: moon run release-all
```

#### 4.2 Python Package Structure
```python
# bindings/python/setup.py (generated by multi-ffi)
from setuptools import setup
from pyo3_setuptools_rust import Pyo3RustExtension

setup(
    name="superconfig",
    version="1.0.0",
    rust_extensions=[
        Pyo3RustExtension(
            "superconfig._superconfig",
            path="../../crates/superconfig-ffi/Cargo.toml",
            features=["python"]
        )
    ],
    packages=["superconfig"],
    python_requires=">=3.8",
)
```

#### 4.3 Node.js Package Structure  
```json
{
  "name": "superconfig",
  "version": "1.0.0",
  "main": "lib/index.js",
  "scripts": {
    "build": "cargo build --manifest-path ../../crates/superconfig-ffi/Cargo.toml --features nodejs"
  },
  "engines": {
    "node": ">=16"
  }
}
```

## API Usage Examples

### Rust (Unchanged)
```rust
use superconfig::SuperConfig;

let config = SuperConfig::new()
    .with_file("config.toml")
    .with_env("APP_")
    .with_hierarchical_config("base", "prod", true);
```

### Python (Generated Native API)
```python
from superconfig import SuperConfig

config = SuperConfig.new() \
    .with_file("config.toml") \
    .with_env("APP_") \
    .with_hierarchical_config("base", "prod", True)
```

### Node.js (Generated Native API)
```javascript
const SuperConfig = require('superconfig');

const config = new SuperConfig()
    .withFile("config.toml")
    .withEnv("APP_")
    .withHierarchicalConfig("base", "prod", true);
```

**Key Improvement**: All languages get native, ergonomic APIs - no ugly JSON manipulation required!

## Testing Strategy

### Unit Tests
```rust
// crates/superconfig-ffi/tests/integration.rs
#[test]
fn test_json_parameter_conversion() {
    let config = SuperConfig::new();
    let result = config.with_file(json!("test.toml"));
    assert!(result.is_ok());
}

#[test] 
fn test_complex_wildcard_config() {
    let wildcard_config = json!({
        "pattern": "*.toml",
        "search": {"type": "recursive", "max_depth": 2},
        "merge_order": {"type": "alphabetical"}
    });
    
    let config = SuperConfig::new();
    let result = config.with_wildcard(wildcard_config);
    assert!(result.is_ok());
}
```

### Cross-Language Integration Tests
```python
# tests/python/test_integration.py
def test_superconfig_parity():
    """Test that Python FFI produces same results as Rust core"""
    # Create equivalent configs in both
    # Compare extracted values
```

## Migration Strategy

### Phase 1: Parallel Development
- Core `superconfig` continues unchanged
- New `superconfig-ffi` developed alongside
- No breaking changes to existing users

### Phase 2: FFI Release
- Release `superconfig-ffi` v1.0
- Publish Python and Node.js packages
- Documentation and examples

### Phase 3: Community Adoption
- Promote FFI packages to language communities
- Gather feedback and iterate
- Maintain both core and FFI versions

## Success Metrics

### Performance Targets
- **Rust**: No performance regression (0% overhead)
- **Python**: 50%+ improvement over complex PyO3 marshaling
- **Node.js**: 50%+ improvement over complex napi-rs marshaling

### Coverage Targets
- **Simple methods**: 100% coverage (68% of API)
- **Complex methods**: 90% coverage (21% of API) 
- **Debug/introspection**: 80% coverage (11% of API)

### Adoption Targets
- Python package: 1000+ downloads/month within 6 months
- Node.js package: 500+ downloads/month within 6 months
- Documentation: Complete API reference for all languages

## Risk Mitigation

### Technical Risks
- **Macro complexity**: Start simple, iterate based on usage
- **Type conversion errors**: Comprehensive error handling and testing
- **Platform compatibility**: Use GitHub Actions matrix builds

### Maintenance Risks  
- **API drift**: Automated tests to ensure FFI wrapper stays in sync
- **Documentation lag**: Generate docs from code where possible
- **Community support**: Clear contribution guidelines and responsive issue handling

## Implementation Timeline

### **Day-by-Day Breakdown**
- **Day 1-2**: SuperFFI macro crate (proc-macro infrastructure, PyO3/napi-rs/WASM bindings)
- **Day 3-4**: SuperConfig-ffi wrapper (all method mappings, JSON parameter handling)
- **Day 5-6**: Complex types + Figment integration (Wildcard provider, JSON schemas)
- **Day 7**: Build system, CI/CD, and documentation (GitHub Actions, package configs)

### **Why AI Makes This Faster**
- **Pattern recognition**: Converting 68% of simple methods mechanically
- **Boilerplate generation**: Proc-macros, build configs, package files  
- **Comprehensive coverage**: Handling all edge cases systematically
- **Error handling**: Generating robust error messages and validation
- **Documentation**: Auto-generating examples and API docs

### **Potential Time Extensions**
- **Testing edge cases**: Real-world integration testing (+1-2 days)
- **Platform-specific builds**: Cross-compilation quirks (+1 day)
- **Performance optimization**: Fine-tuning JSON overhead (+1 day)

## Next Steps

1. **Create SuperFFI crate foundation** (Days 1-2)
2. **Implement superconfig-ffi wrapper** (Days 3-4)  
3. **Complex type handling** (Days 5-6)
4. **Build system integration** (Day 7)
5. **Testing and iteration** (ongoing)

This architecture provides the optimal balance of performance, maintainability, and multi-language support while preserving the core SuperConfig API for Rust users.