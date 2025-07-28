# Native Binary Implementation Example

## How It Works (No Rewriting Required!)

Your existing Rust library becomes the **core engine**. You create small wrapper binaries that expose this functionality through different interfaces.

## 1. CLI Binary Example

Create `crates/superconfig-cli/Cargo.toml`:
```toml
[package]
name = "superconfig-cli"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "superconfig"
path = "src/main.rs"

[dependencies]
superconfig = { path = "../superconfig" }  # Your existing library
clap = { version = "4", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
anyhow = "1.0"
```

Create `crates/superconfig-cli/src/main.rs`:
```rust
use clap::{Parser, Subcommand};
use superconfig::SuperConfig;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "superconfig")]
#[command(about = "Advanced configuration management")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate configuration
    Validate {
        /// Configuration file
        #[arg(short, long)]
        config: PathBuf,
    },
    /// Export configuration as JSON
    Export {
        /// Configuration file
        #[arg(short, long)]
        config: PathBuf,
        /// Output format
        #[arg(short, long, default_value = "json")]
        format: String,
    },
    /// Merge multiple configuration files
    Merge {
        /// Input files
        files: Vec<PathBuf>,
        /// Output file
        #[arg(short, long)]
        output: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    let verbosity = superconfig::VerbosityLevel::from_cli_args(cli.verbose);
    
    match cli.command {
        Commands::Validate { config } => {
            let result = SuperConfig::new()
                .with_verbosity(verbosity)
                .with_file(config)
                .extract::<serde_json::Value>();
                
            match result {
                Ok(_) => {
                    println!("✅ Configuration is valid");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Configuration is invalid: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Export { config, format } => {
            let super_config = SuperConfig::new()
                .with_verbosity(verbosity)
                .with_file(config);
                
            let output = match format.as_str() {
                "json" => super_config.as_json()?,
                "yaml" => super_config.as_yaml()?,
                "toml" => super_config.as_toml()?,
                _ => return Err(anyhow::anyhow!("Unsupported format: {}", format)),
            };
            
            println!("{}", output);
            Ok(())
        }
        Commands::Merge { files, output } => {
            let mut config = SuperConfig::new().with_verbosity(verbosity);
            
            for file in files {
                config = config.with_file(file);
            }
            
            let merged = config.as_json()?;
            std::fs::write(output, merged)?;
            println!("✅ Configurations merged successfully");
            Ok(())
        }
    }
}
```

## 2. Distribution Strategy

### A. Native Binaries via GitHub Releases
```yaml
# .github/workflows/release.yml
- name: Build binaries
  run: |
    cargo build --release --bin superconfig
    
    # Cross-compile for different platforms
    cargo build --release --target x86_64-unknown-linux-gnu --bin superconfig
    cargo build --release --target x86_64-pc-windows-msvc --bin superconfig
    cargo build --release --target x86_64-apple-darwin --bin superconfig
    cargo build --release --target aarch64-apple-darwin --bin superconfig
```

### B. Language-Specific Package Managers

#### npm (JavaScript/TypeScript)
Create `packages/superconfig-js/package.json`:
```json
{
  "name": "superconfig",
  "version": "0.1.0",
  "bin": {
    "superconfig": "bin/superconfig"
  },
  "optionalDependencies": {
    "superconfig-cli-linux-x64": "0.1.0",
    "superconfig-cli-darwin-x64": "0.1.0", 
    "superconfig-cli-darwin-arm64": "0.1.0",
    "superconfig-cli-win32-x64": "0.1.0"
  },
  "scripts": {
    "install": "node install.js"
  }
}
```

Create `packages/superconfig-js/install.js`:
```javascript
// Detects platform and symlinks correct binary
const os = require('os');
const path = require('path');
const fs = require('fs');

const platform = os.platform();
const arch = os.arch();

const binaryName = `superconfig-cli-${platform}-${arch}`;
const binaryPath = path.join(__dirname, 'node_modules', binaryName, 'bin', 'superconfig');
const targetPath = path.join(__dirname, 'bin', 'superconfig');

if (fs.existsSync(binaryPath)) {
    fs.symlinkSync(binaryPath, targetPath);
} else {
    console.error(`Binary not found for ${platform}-${arch}`);
}
```

#### pip (Python)
Create `packages/superconfig-py/setup.py`:
```python
from setuptools import setup, find_packages
import platform
import subprocess
import os

class BinaryInstaller:
    def __init__(self):
        self.platform = platform.system().lower()
        self.arch = platform.machine().lower()
        
    def download_binary(self):
        # Download appropriate binary from GitHub releases
        binary_name = f"superconfig-{self.platform}-{self.arch}"
        # Implementation to download and install binary
        pass

setup(
    name="superconfig",
    version="0.1.0",
    packages=find_packages(),
    include_package_data=True,
    entry_points={
        'console_scripts': [
            'superconfig=superconfig.cli:main',
        ],
    },
    install_requires=[],
)
```

Create `packages/superconfig-py/superconfig/cli.py`:
```python
import subprocess
import sys
import os

def main():
    # Find the binary
    binary_path = os.path.join(os.path.dirname(__file__), 'bin', 'superconfig')
    
    if not os.path.exists(binary_path):
        print("Error: superconfig binary not found", file=sys.stderr)
        sys.exit(1)
    
    # Execute the binary with all arguments
    result = subprocess.run([binary_path] + sys.argv[1:])
    sys.exit(result.returncode)

if __name__ == "__main__":
    main()
```

## 3. WASM Bindings (Fallback/Web)

Create `crates/superconfig-wasm/Cargo.toml`:
```toml
[package]
name = "superconfig-wasm"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
superconfig = { path = "../superconfig" }
wasm-bindgen = "0.2"
serde-wasm-bindgen = "0.6"
js-sys = "0.3"

[dependencies.web-sys]
version = "0.3"
features = [
  "console",
]
```

Create `crates/superconfig-wasm/src/lib.rs`:
```rust
use wasm_bindgen::prelude::*;
use superconfig::SuperConfig;

#[wasm_bindgen]
pub struct WasmSuperConfig {
    inner: SuperConfig,
}

#[wasm_bindgen]
impl WasmSuperConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmSuperConfig {
        WasmSuperConfig {
            inner: SuperConfig::new(),
        }
    }
    
    #[wasm_bindgen]
    pub fn with_file(&mut self, path: &str) -> Result<(), JsValue> {
        // Implementation using your existing library
        Ok(())
    }
    
    #[wasm_bindgen]
    pub fn extract_json(&self) -> Result<String, JsValue> {
        self.inner.as_json()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
```

## 4. Build Pipeline Example

Create `scripts/build-all.sh`:
```bash
#!/bin/bash
set -e

echo "Building native binaries..."
cargo build --release --bin superconfig

echo "Building for different targets..."
cargo build --release --target x86_64-unknown-linux-gnu --bin superconfig
cargo build --release --target x86_64-pc-windows-msvc --bin superconfig
cargo build --release --target x86_64-apple-darwin --bin superconfig

echo "Building WASM..."
cd crates/superconfig-wasm
wasm-pack build --target nodejs --scope superconfig
wasm-pack build --target web --scope superconfig
cd ../..

echo "Building JavaScript packages..."
cd packages/superconfig-js
npm run build
cd ../..

echo "Building Python packages..."
cd packages/superconfig-py
python setup.py bdist_wheel
cd ../..

echo "All builds complete!"
```

## Key Benefits

1. **No Rewriting**: Your Rust code stays exactly the same
2. **Performance**: Native binaries are fastest possible
3. **Compatibility**: Each language gets idiomatic package management
4. **Fallback**: WASM provides universal compatibility
5. **Maintenance**: Single Rust codebase powers everything

## Usage Examples

```bash
# JavaScript/TypeScript
npm install superconfig
npx superconfig validate config.json

# Python
pip install superconfig
superconfig export config.toml --format json

# Direct binary
./superconfig merge config1.toml config2.yaml -o merged.json
```

This approach gives you the best of all worlds: **native performance + universal compatibility + no code duplication**.