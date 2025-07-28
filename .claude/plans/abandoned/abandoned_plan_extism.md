# SuperConfig Extism Plugin Implementation Plan

## Overview

This plan outlines implementing SuperConfig as an Extism WebAssembly plugin to provide universal configuration management across multiple programming languages through a single WASM binary and schema-driven type-safe bindings.

## Architecture

### Core Components

1. **SuperConfig Extism Plugin (Rust)**: WASM plugin built with Extism PDK
2. **XTP Schema**: API definition for type-safe binding generation
3. **Generated Language Bindings**: Auto-generated clients for each language
4. **Distribution Package**: Single .wasm file + language-specific packages

## Implementation Details

### 1. XTP Schema Definition

**File: `superconfig-schema.yaml`**

```yaml
version: v1-draft
exports:
  LoadConfig:
    input:
      $ref: "#/components/schemas/ConfigRequest"
      contentType: application/json
    output:
      $ref: "#/components/schemas/ConfigResponse"
      contentType: application/json
  
  ValidateConfig:
    input:
      $ref: "#/components/schemas/ValidationRequest"
      contentType: application/json
    output:
      $ref: "#/components/schemas/ValidationResponse"
      contentType: application/json

components:
  schemas:
    ConfigRequest:
      properties:
        files:
          type: array
          items:
            type: string
          description: "Configuration file paths"
        env_prefix:
          type: string
          description: "Environment variable prefix"
        defaults:
          type: object
          description: "Default configuration values"
        hierarchical_base:
          type: string
          description: "Base name for hierarchical config search"
      required: ["files"]

    ConfigResponse:
      properties:
        success:
          type: boolean
        data:
          type: object
          description: "Merged configuration data"
        error:
          type: string
          nullable: true
        metadata:
          $ref: "#/components/schemas/ConfigMetadata"

    ConfigMetadata:
      properties:
        sources:
          type: array
          items:
            type: string
          description: "Configuration sources loaded"
        format_detected:
          type: string
          description: "Detected configuration format"

    ValidationRequest:
      properties:
        config_data:
          type: object
        schema:
          type: object
          description: "JSON schema for validation"

    ValidationResponse:
      properties:
        valid:
          type: boolean
        errors:
          type: array
          items:
            type: string
```

### 2. SuperConfig Plugin Implementation

**File: `src/lib.rs`**

```rust
use extism_pdk::*;
use serde::{Deserialize, Serialize};
use superconfig::SuperConfig;

#[derive(Deserialize)]
struct ConfigRequest {
    files: Vec<String>,
    env_prefix: Option<String>,
    defaults: Option<serde_json::Value>,
    hierarchical_base: Option<String>,
}

#[derive(Serialize)]
struct ConfigResponse {
    success: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
    metadata: ConfigMetadata,
}

#[derive(Serialize)]
struct ConfigMetadata {
    sources: Vec<String>,
    format_detected: String,
}

#[derive(Deserialize)]
struct ValidationRequest {
    config_data: serde_json::Value,
    schema: serde_json::Value,
}

#[derive(Serialize)]
struct ValidationResponse {
    valid: bool,
    errors: Vec<String>,
}

#[plugin_fn]
pub fn load_config(input: String) -> FnResult<String> {
    let request: ConfigRequest = serde_json::from_str(&input)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("Invalid input: {}", e)), 1))?;

    // Build SuperConfig instance
    let mut config = SuperConfig::new();

    // Add defaults if provided
    if let Some(defaults) = request.defaults {
        config = config.with_defaults(defaults);
    }

    // Add files
    for file in request.files {
        config = config.with_file(&file);
    }

    // Add environment variables if prefix provided
    if let Some(prefix) = request.env_prefix {
        config = config.with_env_ignore_empty(&prefix);
    }

    // Add hierarchical config if requested
    if let Some(base) = request.hierarchical_base {
        config = config.with_hierarchical_config(&base);
    }

    // Extract configuration
    match config.extract::<serde_json::Value>() {
        Ok(data) => {
            let response = ConfigResponse {
                success: true,
                data: Some(data),
                error: None,
                metadata: ConfigMetadata {
                    sources: request.files, // In real implementation, track actual sources
                    format_detected: "auto".to_string(),
                },
            };
            Ok(serde_json::to_string(&response)?)
        }
        Err(e) => {
            let response = ConfigResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
                metadata: ConfigMetadata {
                    sources: vec![],
                    format_detected: "unknown".to_string(),
                },
            };
            Ok(serde_json::to_string(&response)?)
        }
    }
}

#[plugin_fn]
pub fn validate_config(input: String) -> FnResult<String> {
    let request: ValidationRequest = serde_json::from_str(&input)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("Invalid input: {}", e)), 1))?;

    // Implement JSON Schema validation using jsonschema crate
    // (This is simplified - you'd use a proper JSON schema validator)
    let response = ValidationResponse {
        valid: true, // Placeholder
        errors: vec![],
    };

    Ok(serde_json::to_string(&response)?)
}
```

**Cargo.toml:**

```toml
[package]
name = "superconfig-extism-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
extism-pdk = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
superconfig = { path = "../superconfig" }

[profile.release]
opt-level = "s"
lto = true
```

### 3. Client Usage Examples

#### Rust Client

```rust
use extism::*;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the SuperConfig plugin
    let url = Wasm::file("./superconfig-extism-plugin.wasm");
    let manifest = Manifest::new([url]);
    let mut plugin = Plugin::new(&manifest, [], true)?;

    // Configure what we want to load
    let request = json!({
        "files": ["config.toml", "config.yaml"],
        "env_prefix": "MYAPP_",
        "defaults": {
            "host": "localhost",
            "port": 8080
        }
    });

    // Load configuration
    let result = plugin.call::<String, String>("load_config", request.to_string())?;
    let response: serde_json::Value = serde_json::from_str(&result)?;
    
    if response["success"].as_bool().unwrap() {
        println!("Config loaded: {}", response["data"]);
    } else {
        eprintln!("Error: {}", response["error"]);
    }

    Ok(())
}
```

#### JavaScript/Node.js Client

```javascript
import { Extism } from '@extism/extism';
import fs from 'fs';

async function main() {
    // Load the SuperConfig plugin
    const wasmData = fs.readFileSync('./superconfig-extism-plugin.wasm');
    const plugin = await Extism.create({
        wasm: [{ data: wasmData }],
        allowedHosts: [],
        config: {}
    });

    // Configure what we want to load
    const request = {
        files: ['config.json', 'config.toml'],
        env_prefix: 'MYAPP_',
        defaults: {
            host: 'localhost',
            port: 8080
        }
    };

    try {
        // Load configuration
        const result = await plugin.call('load_config', JSON.stringify(request));
        const response = JSON.parse(result);
        
        if (response.success) {
            console.log('Config loaded:', response.data);
        } else {
            console.error('Error:', response.error);
        }
    } catch (error) {
        console.error('Plugin error:', error);
    }
}

main().catch(console.error);
```

#### Python Client

```python
import extism
import json

def main():
    # Load the SuperConfig plugin
    with open('./superconfig-extism-plugin.wasm', 'rb') as f:
        wasm_data = f.read()
    
    plugin = extism.Plugin(
        manifest={"wasm": [{"data": wasm_data}]},
        wasi=True
    )

    # Configure what we want to load
    request = {
        "files": ["config.yaml", "config.json"],
        "env_prefix": "MYAPP_",
        "defaults": {
            "host": "localhost",
            "port": 8080
        }
    }

    try:
        # Load configuration
        result = plugin.call('load_config', json.dumps(request))
        response = json.loads(result)
        
        if response['success']:
            print(f"Config loaded: {response['data']}")
        else:
            print(f"Error: {response['error']}")
            
    except Exception as e:
        print(f"Plugin error: {e}")

if __name__ == "__main__":
    main()
```

#### Go Client

```go
package main

import (
    "context"
    "encoding/json"
    "fmt"
    "log"
    "os"

    "github.com/extism/go-sdk"
)

type ConfigRequest struct {
    Files           []string               `json:"files"`
    EnvPrefix       *string                `json:"env_prefix,omitempty"`
    Defaults        map[string]interface{} `json:"defaults,omitempty"`
}

type ConfigResponse struct {
    Success bool                   `json:"success"`
    Data    map[string]interface{} `json:"data,omitempty"`
    Error   *string                `json:"error,omitempty"`
}

func main() {
    ctx := context.Background()
    
    // Load the SuperConfig plugin
    wasmData, err := os.ReadFile("./superconfig-extism-plugin.wasm")
    if err != nil {
        log.Fatal(err)
    }

    manifest := extism.Manifest{
        Wasm: []extism.WasmData{
            {Data: wasmData},
        },
    }

    plugin, err := extism.NewPlugin(ctx, manifest, extism.PluginConfig{}, []extism.HostFunction{})
    if err != nil {
        log.Fatal(err)
    }
    defer plugin.Close()

    // Configure what we want to load
    request := ConfigRequest{
        Files:     []string{"config.toml", "config.yaml"},
        EnvPrefix: stringPtr("MYAPP_"),
        Defaults: map[string]interface{}{
            "host": "localhost",
            "port": 8080,
        },
    }

    requestJSON, _ := json.Marshal(request)

    // Load configuration
    exit, output, err := plugin.Call("load_config", requestJSON)
    if err != nil {
        log.Fatal(err)
    }
    if exit != 0 {
        log.Fatalf("Plugin exited with code: %d", exit)
    }

    var response ConfigResponse
    if err := json.Unmarshal(output, &response); err != nil {
        log.Fatal(err)
    }

    if response.Success {
        fmt.Printf("Config loaded: %+v\n", response.Data)
    } else {
        fmt.Printf("Error: %s\n", *response.Error)
    }
}

func stringPtr(s string) *string {
    return &s
}
```

## Build & Distribution Process

### Build Commands

```bash
# Generate type-safe bindings for all languages
xtp plugin init --schema-file ./superconfig-schema.yaml

# Build the WASM plugin
cargo build --target wasm32-unknown-unknown --release

# Optimize the WASM (optional but recommended)
wasm-opt -Oz -o superconfig.wasm target/wasm32-unknown-unknown/release/superconfig_extism_plugin.wasm
```

### Package Structure

```
superconfig-release/
├── superconfig.wasm              # The plugin binary
├── superconfig-schema.yaml       # API schema
├── bindings/
│   ├── rust/                    # Generated Rust bindings
│   ├── typescript/              # Generated TypeScript bindings
│   ├── python/                  # Generated Python bindings
│   ├── go/                      # Generated Go bindings
│   └── ...
└── examples/                    # Usage examples for each language
```

### Distribution

Each language gets its own package:

- **NPM**: `@superconfig/extism` 
- **Cargo**: `superconfig-extism`
- **PyPI**: `superconfig-extism`
- **Go Module**: `github.com/superconfig/extism-go`

## Advantages

1. **Write Once, Use Everywhere**: Single Rust codebase supports all languages
2. **Type Safety**: Schema-generated bindings ensure type safety across languages
3. **Universal Distribution**: One `.wasm` file works everywhere
4. **Performance**: Near-native speed with WASM
5. **Security**: Sandboxed execution environment
6. **Easy Updates**: Update the `.wasm` file to upgrade all clients
7. **Mature Ecosystem**: Leverages Extism's proven multi-language support

## Trade-offs

1. **Plugin Call Interface**: Uses `plugin.call('function_name', data)` instead of native method calls
2. **JSON Serialization**: All data must be JSON-serializable
3. **Framework Dependency**: Tied to Extism ecosystem
4. **Learning Curve**: Teams need to understand Extism concepts

## Implementation Timeline

1. **Week 1-2**: Implement core plugin with basic functionality
2. **Week 3**: Define comprehensive XTP schema
3. **Week 4**: Generate and test bindings for primary languages (Rust, JS, Python, Go)
4. **Week 5**: Create distribution packages and documentation
5. **Week 6**: Testing and optimization

## Success Metrics

- [ ] Plugin compiles to optimized WASM under 1MB
- [ ] All primary languages (Rust, JS, Python, Go) have working bindings
- [ ] Performance within 10% of native SuperConfig
- [ ] Complete API coverage through schema
- [ ] Comprehensive test suite across all languages