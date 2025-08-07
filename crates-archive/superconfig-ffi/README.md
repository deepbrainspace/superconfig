# SuperConfig FFI

Multi-language bindings for SuperConfig - enabling use from Python, Node.js, and WebAssembly environments.

## 🚀 Quick Start

### Build Commands

```bash
# Python bindings (fastest performance)
cargo build --features python --release

# Node.js bindings  
cargo build --features nodejs --release

# WebAssembly bindings (smallest size)
wasm-pack build --features wasm --target web

# WASI bindings (filesystem access)
cargo build --target wasm32-wasip1 --features wasm --release
```

### Usage Examples

```python
# Python
import superconfig_ffi
config = superconfig_ffi.SuperConfig()
print(config.get_verbosity())
```

```javascript
// Node.js
const { SuperConfig } = require('./superconfig_ffi.node');
const config = new SuperConfig();
console.log(config.getVerbosity());
```

```javascript
// WebAssembly
import { SuperConfig } from './pkg/superconfig_ffi.js';
const config = new SuperConfig();
console.log(config.get_verbosity());
```

## 📊 Performance Summary

| Technology | Speed    | File Size | Best For                       |
| ---------- | -------- | --------- | ------------------------------ |
| **Python** | 0.411 μs | 681KB     | **Fastest operations**         |
| **NAPI**   | 1.206 μs | 607KB     | Node.js integration            |
| **WASI**   | 396.0 μs | 50KB      | **Smallest size + filesystem** |

## 📁 Project Structure

- `src/lib.rs` - Main FFI implementation
- `pkg/` - Generated WebAssembly bindings
- `reference/` - Benchmarks, tests, and detailed documentation
- `target/` - Compiled binaries

## 🔧 Features

- `python` - Enable Python bindings via PyO3
- `nodejs` - Enable Node.js bindings via NAPI-RS
- `wasm` - Enable WebAssembly bindings via wasm-bindgen

**Note**: Only one feature can be enabled at a time to prevent conflicts.

## 📚 Documentation

See `reference/` folder for:

- Performance benchmarks and analysis
- Next.js integration examples
- Complete test suites
- Detailed comparison documents
