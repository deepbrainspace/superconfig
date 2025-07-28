# FFI Pivot: From WASM to Native Bindings

**Created**: July 27, 2025  
**Author**: Claude Opus 4  
**Status**: Technical Strategy Pivot - CRITICAL  
**Decision**: Use FFI (Foreign Function Interface) instead of WASM for multi-language support

## 🚨 Major Discovery: FFI > WASM

The biome analysis revealed crucial insights:

### Performance Comparison
- **FFI**: ~99% of native Rust performance ✅
- **WASM**: 90-95% of native performance ❌
- **CLI subprocess**: 30-60% performance ❌

### Industry Standard
- **PyO3**: Rust→Python bindings (used by Ruff, UV, etc.)
- **napi-rs**: Rust→Node.js bindings (used by swc, rspack, etc.)
- **Direct memory access**: No serialization overhead
- **Native packaging**: npm, pip, gem native distribution

## 🎯 New Technical Strategy

### What Changes
1. **No WASM compilation** - Direct native bindings instead
2. **Better performance** - 99% vs 90-95%
3. **Simpler debugging** - Native stack traces
4. **Smaller packages** - No WASM runtime overhead

### What Stays the Same
1. **Single Rust codebase** - Still one source of truth
2. **Multi-language support** - Node.js, Python, Ruby, etc.
3. **"WasmBridge" name** - Keep it, but reframe purpose

## 📦 Implementation Architecture

### Single-Crate Multi-FFI Approach
```toml
# crates/superconfig/Cargo.toml
[package]
name = "superconfig"
version = "0.1.0"

[lib]
crate-type = ["cdylib", "rlib"]

[features]
default = []
python = ["pyo3"]
nodejs = ["napi", "napi-derive"]

[dependencies]
figment = "0.10"
serde = "1.0"

# FFI dependencies
pyo3 = { version = "0.20", optional = true }
napi = { version = "2", optional = true }
napi-derive = { version = "2", optional = true }
```

### Build Scripts
```bash
# Build for Python
cargo build --features python
maturin build

# Build for Node.js  
cargo build --features nodejs
napi build --platform
```

## 🔄 Rebranding "WasmBridge"

### Old Positioning
"WasmBridge: Rust→WASM→Multi-language tool"

### New Positioning
"WasmBridge: Universal language bindings for Rust libraries"

### Why Keep the Name?
1. **Already decided on it** - No need to rebrand
2. **Bridge is still accurate** - Bridging Rust to other languages
3. **WASM can be one option** - Among FFI, WASM, etc.
4. **Marketing flexibility** - "Multiple bridge technologies"

## 💻 Code Examples

### Python (PyO3)
```python
import superconfig

# Native performance!
config = superconfig.Config()
config.with_file("config.toml")
config.with_env("APP_")
result = config.extract()
```

### Node.js (napi-rs)
```javascript
const { SuperConfig } = require('superconfig');

// Direct memory access!
const config = new SuperConfig();
config.withFile('config.json');
config.withEnv('APP_');
const result = config.extract();
```

### Ruby (magnus)
```ruby
require 'superconfig'

# Native Ruby extension!
config = SuperConfig.new
config.with_file('config.yml')
result = config.extract
```

## 📊 Why This Is Better

### Performance
- **No serialization overhead** - Direct memory access
- **Native async support** - Real threads, not WASM promises
- **Zero-copy operations** - Where possible

### Developer Experience
- **Native debugging** - Regular debuggers work
- **Familiar errors** - Python exceptions, JS errors
- **IDE support** - Type hints, autocomplete

### Distribution
- **Standard channels** - npm, pip, gem
- **Platform wheels** - Pre-built binaries
- **No WASM runtime** - Smaller, faster

## 🚀 Implementation Timeline

### Week 1: Python Bindings
- Set up PyO3 in superconfig crate
- Create Python wrapper API
- Build with maturin
- Publish to PyPI

### Week 2: Node.js Bindings
- Add napi-rs to superconfig
- Create JS/TS wrapper
- Build for multiple platforms
- Publish to npm

### Week 3: Documentation & Launch
- Update all docs for FFI approach
- Create binding examples
- Performance benchmarks
- Launch announcement

## 💡 Marketing the Pivot

### Don't Say
"We were wrong about WASM"

### Do Say
"We chose the fastest approach: native bindings deliver 99% Rust performance"

### The Story
> "While exploring WASM, we discovered native FFI bindings provide better performance and developer experience. SuperConfig uses the same technology as Ruff, UV, and swc for maximum speed."

## 📝 Action Items

1. **Update all strategic docs** - Remove WASM-specific language
2. **Reframe WasmBridge** - "Universal bindings" not "WASM tool"
3. **Start with PyO3** - Easiest and most mature
4. **Benchmark everything** - Prove the 99% claim
5. **Update website messaging** - Performance-first narrative

## 🎯 The Bottom Line

**FFI is the way forward**. It's:
- Faster (99% vs 90%)
- Simpler (no WASM complexity)
- Proven (Ruff, UV, swc use it)
- Standard (npm, pip native packages)

This isn't a setback - it's finding the optimal solution before we build the wrong thing.

---

**Note**: Keep "WasmBridge" name but pivot the technology. The bridge metaphor still works perfectly.