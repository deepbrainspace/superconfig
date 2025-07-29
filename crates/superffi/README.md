# SuperFFI - Multi-Language FFI Binding Generator

**SuperFFI** is a procedural macro that automatically generates FFI bindings for multiple target languages from your Rust code. Write once, run everywhere.

## Features

- **Python bindings** via PyO3
- **Node.js bindings** via NAPI-RS  
- **WebAssembly bindings** via wasm-bindgen (browser) and WASI (server-side)
- **Zero-cost abstractions** - only generates code for enabled features
- **Simple annotation** - just add `#[superffi]` to your items

## Installation

```toml
[dependencies]
superffi = { version = "0.1", features = ["python", "nodejs", "wasm"] }
```

**Features:**
- `python` - PyO3 bindings for Python
- `nodejs` - NAPI bindings for Node.js  
- `wasm` - wasm-bindgen bindings for WebAssembly (browser + WASI)
- `all` - All target languages

## Quick Start

```rust
use superffi::superffi;

#[superffi]
pub struct Calculator {
    pub value: f64,
}

#[superffi]
impl Calculator {
    pub fn new(initial_value: f64) -> Self {
        Self { value: initial_value }
    }
    
    pub fn add(&mut self, other: f64) {
        self.value += other;
    }
    
    pub fn get_value(&self) -> f64 {
        self.value
    }
}

#[superffi]
pub fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
```

## Usage

Apply `#[superffi]` to:
- **Structs** → generates class/object bindings
- **Impl blocks** → generates method bindings  
- **Functions** → generates standalone function bindings

## Language Usage Examples

### Python
```python
import your_library

calc = your_library.Calculator(10.0)
calc.add(5.0)
print(calc.get_value())  # 15.0
print(your_library.fibonacci(10))  # 55
```

### Node.js
```javascript
const lib = require('./target/release/your_library.node');

const calc = new lib.Calculator(10.0);
calc.add(5.0);
console.log(calc.getValue()); // 15.0
console.log(lib.fibonacci(10)); // 55
```

### WebAssembly (Browser + WASI)
```javascript
import init, { Calculator, fibonacci } from './pkg/your_library.js';

await init();
const calc = new Calculator(10.0);
calc.add(5.0);
console.log(calc.get_value()); // 15.0
console.log(fibonacci(10)); // 55
```

## 🎯 Language-Specific Usage

### Python

After building your Rust library with the `python` feature:

```python
import your_rust_library

# Create and use structs
calc = your_rust_library.Calculator(10.0)
calc.add(5.0)
calc.multiply(2.0)
print(calc.get_value())  # Output: 30.0

# Use standalone functions
result = your_rust_library.fibonacci(10)
print(result)  # Output: 55
```

### Node.js

After building your Rust library with the `nodejs` feature:

```javascript
const lib = require('./target/release/your_rust_library.node');

// Create and use structs
const calc = new lib.Calculator(10.0);
calc.add(5.0);
calc.multiply(2.0);
console.log(calc.getValue()); // Output: 30.0

// Use standalone functions
const result = lib.fibonacci(10);
console.log(result); // Output: 55
```

### WebAssembly

After building your Rust library with the `wasm` feature:

```javascript
import init, { Calculator, fibonacci } from './pkg/your_rust_library.js';

async function run() {
    await init();
    
    // Create and use structs
    const calc = new Calculator(10.0);
    calc.add(5.0);
    calc.multiply(2.0);
    console.log(calc.get_value()); // Output: 30.0
    
    // Use standalone functions
    const result = fibonacci(10);
    console.log(result); // Output: 55
}

run();
```

## 🏗️ Build Configuration

### For Python (PyO3)

Add to your `Cargo.toml`:

```toml
[lib]
name = "your_rust_library"
crate-type = ["cdylib"]

[dependencies]
superffi = { version = "0.1", features = ["python"] }
pyo3 = { version = "0.25", features = ["extension-module"] }
```

Build command:
```bash
maturin develop  # For development
maturin build --release  # For production
```

### For Node.js (NAPI)

Add to your `package.json`:

```json
{
  "napi": {
    "name": "your-rust-library",
    "triples": {
      "defaults": true
    }
  }
}
```

Build command:
```bash
napi build --platform --release
```

### For WebAssembly

Build command:
```bash
wasm-pack build --target web --out-dir pkg
```

## ⚠️ Limitations

- **Async functions**: Not currently supported across all target languages
- **Complex generics**: May not translate directly to all target languages  
- **Advanced lifetimes**: Rust-specific lifetime annotations may not be supported
- **Trait objects**: Not directly supported; use concrete types instead
- **Custom derives**: May conflict with generated bindings

## 🛠️ Supported Types

### Primitive Types
- `bool`, `i8`, `i16`, `i32`, `i64`, `isize`
- `u8`, `u16`, `u32`, `u64`, `usize`  
- `f32`, `f64`
- `char`

### Standard Library Types
- `String`
- `Vec<T>` (where T is supported)
- `Option<T>` (where T is supported)
- `HashMap<K, V>` (limited support)

### Custom Types
- Structs annotated with `#[superffi]`
- Enums (limited support, varies by target language)

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

1. Clone the repository
2. Install Rust nightly (required for proc-macro development)
3. Run tests: `cargo test`
4. Run examples: `cargo run --example basic`

### Testing

Run the test suite:
```bash
cargo test
cargo test --all-features
```

Test with specific features:
```bash
cargo test --features python
cargo test --features nodejs  
cargo test --features wasm
```

## 📄 License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

- [PyO3](https://github.com/PyO3/pyo3) for Python FFI
- [NAPI-RS](https://github.com/napi-rs/napi-rs) for Node.js FFI  
- [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) for WebAssembly FFI
- The Rust community for excellent procedural macro resources

---

**Happy coding! 🦀**