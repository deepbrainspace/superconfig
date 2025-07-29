# SuperConfig Extism with Ergonomic API Implementation Plan

## Executive Summary

This plan outlines how to implement SuperConfig using Extism with **ergonomic, native-feeling APIs** that completely hide the `plugin.call()` complexity. The solution involves extending XTP Bindgen (the official Extism code generation tool) with custom templates that generate fluent builder APIs for 15+ languages automatically.

## Understanding XTP and Extism Relationship

### What is XTP?

- **XTP** (eXtensible Typescript Plugins) is a **platform and toolchain** built on top of Extism
- **XTP Bindgen** is the code generation framework that creates language bindings from schemas
- **XTP CLI** is the command-line tool that orchestrates the binding generation process
- XTP is **the official way to create production Extism plugins with type safety**

### Architecture Overview

```
SuperConfig Schema (YAML)
    ↓
XTP Bindgen + Custom Templates
    ↓
Generated Language Wrappers (15+ languages)
    ↓
Ergonomic APIs (no plugin.call visible)
    ↓
Single WASM Binary (universal)
```

## Technical Architecture Analysis

### Current XTP Bindgen Architecture

Based on analysis of `/home/nsm/code/forks/xtp-bindgen/`:

1. **Schema Parser**: Validates and normalizes XTP schema (OpenAPI-like IDL)
2. **Template Engine**: Uses EJS templates to generate code
3. **Plugin System**: Each language has its own bindgen plugin (WASM-based)
4. **CLI Integration**: `xtp plugin init` orchestrates the generation

### How XTP Bindgen Currently Works

```bash
# Current XTP workflow
xtp plugin init --schema-file ./schema.yaml
  > 1. TypeScript    # Downloads @dylibso/xtp-typescript-bindgen
  > 2. Go           # Downloads @dylibso/xtp-go-bindgen  
  > 3. Python       # Downloads @dylibso/xtp-python-bindgen
  # etc...
```

Each bindgen is a **zip bundle** containing:

- `plugin.wasm` - Extism plugin that processes templates
- `config.yaml` - Generator configuration
- `template/` - EJS template files for the target language

### Existing Template Structure (TypeScript Example)

```
typescript-bindgen-bundle/
├── config.yaml
├── plugin.wasm
└── template/
    ├── package.json.ejs
    ├── src/
    │   ├── index.ts.ejs      # Main entry point
    │   ├── main.ts.ejs       # Plugin implementation  
    │   └── pdk.ts.ejs        # PDK bindings
    └── tsconfig.json
```

**Problem**: Current templates generate **plugin-side code** (PDK), not **host-side wrapper libraries**.

## Solution: Custom XTP Templates for Host-Side Wrappers

### Approach 1: Custom XTP Templates (Recommended)

Create custom XTP bindgen templates that generate **host-side wrapper libraries** instead of plugin code.

#### Implementation Strategy

1. **Fork existing bindgens** or create new ones
2. **Replace template contents** with host-side wrapper templates
3. **Modify config.yaml** to indicate host-side generation
4. **Publish custom bindgens** with ergonomic API templates

#### Custom Template Structure for TypeScript

```
superconfig-typescript-bindgen/
├── config.yaml
├── plugin.wasm
└── template/
    ├── package.json.ejs
    ├── src/
    │   ├── index.ts.ejs           # Ergonomic SuperConfig class
    │   ├── loader.ts.ejs          # Hidden WASM/Extism complexity
    │   └── types.ts.ejs           # Generated TypeScript types
    ├── wasm/
    │   └── .gitkeep               # Placeholder for WASM binary
    └── tsconfig.json
```

#### Enhanced SuperConfig Schema

```yaml
# superconfig-schema.yaml
version: v1-draft

# Add custom metadata for ergonomic API generation
x-superconfig:
  builder-pattern: true
  fluent-methods:
    - name: "withFile"
      parameter: 
        name: "path"
        type: "string"
      action: "addToArray"
      field: "files"
    - name: "withEnvIgnoreEmpty"
      parameter:
        name: "prefix" 
        type: "string"
      action: "setValue"
      field: "env_prefix"
    - name: "withDefaults"
      parameter:
        name: "defaults"
        type: "object"
      action: "setValue"
      field: "defaults"

exports:
  ProcessConfig:
    input:
      $ref: "#/components/schemas/ConfigRequest"
      contentType: application/json
    output:
      $ref: "#/components/schemas/ConfigResponse"
      contentType: application/json

components:
  schemas:
    ConfigRequest:
      properties:
        files:
          type: array
          items:
            type: string
        env_prefix:
          type: string
        defaults:
          type: object
        hierarchical_base:
          type: string

    ConfigResponse:
      properties:
        success:
          type: boolean
        data:
          type: object
        error:
          type: string
          nullable: true
```

#### Custom Template: TypeScript Wrapper

```typescript
// template/src/index.ts.ejs
import { Plugin } from '@extism/extism';
import fs from 'fs';

export class SuperConfig {
  private plugin: Plugin | null = null;
  private configState: any = {};
  private static wasmCache: ArrayBuffer | null = null;

  private constructor() {}

  static async new(): Promise<SuperConfig> {
    const instance = new SuperConfig();
    await instance.initializePlugin();
    return instance;
  }

  private async initializePlugin(): Promise<void> {
    if (!SuperConfig.wasmCache) {
      SuperConfig.wasmCache = fs.readFileSync('./wasm/superconfig.wasm');
    }
    
    this.plugin = await Plugin.create({
      wasm: [{ data: SuperConfig.wasmCache }],
      allowedHosts: [],
      config: {}
    });
  }

  <% for (const method of schema['x-superconfig']['fluent-methods']) { %>
  <%= method.name %>(<%= method.parameter.name %>: <%= method.parameter.type %>): SuperConfig {
    <% if (method.action === 'addToArray') { %>
    this.configState.<%= method.field %> = this.configState.<%= method.field %> || [];
    this.configState.<%= method.field %>.push(<%= method.parameter.name %>);
    <% } else if (method.action === 'setValue') { %>
    this.configState.<%= method.field %> = <%= method.parameter.name %>;
    <% } %>
    return this;
  }
  <% } %>

  async extract<T = any>(): Promise<T> {
    if (!this.plugin) {
      throw new Error('Plugin not initialized');
    }

    const result = await this.plugin.call('ProcessConfig', JSON.stringify(this.configState));
    const response = JSON.parse(result);
    
    if (!response.success) {
      throw new Error(response.error);
    }
    
    return response.data as T;
  }

  async dispose(): Promise<void> {
    if (this.plugin) {
      await this.plugin.close();
      this.plugin = null;
    }
  }
}

// Helper function
export async function loadConfig<T = any>(
  builderFn: (config: SuperConfig) => SuperConfig
): Promise<T> {
  const config = await SuperConfig.new();
  const configured = builderFn(config);
  const result = await configured.extract<T>();
  await configured.dispose();
  return result;
}
```

#### Custom Template: Python Wrapper

```python
# template/src/__init__.py.ejs
import json
import asyncio
from typing import Any, Dict, Optional, TypeVar, Callable
from extism import Plugin

T = TypeVar('T')

class SuperConfig:
    _wasm_cache: Optional[bytes] = None
    
    def __init__(self):
        self._plugin: Optional[Plugin] = None
        self._config_state: Dict[str, Any] = {}

    @classmethod
    async def new(cls) -> 'SuperConfig':
        instance = cls()
        await instance._initialize_plugin()
        return instance

    async def _initialize_plugin(self) -> None:
        if SuperConfig._wasm_cache is None:
            with open('./wasm/superconfig.wasm', 'rb') as f:
                SuperConfig._wasm_cache = f.read()
        
        self._plugin = Plugin(
            manifest={"wasm": [{"data": SuperConfig._wasm_cache}]},
            wasi=True
        )

    <% for (const method of schema['x-superconfig']['fluent-methods']) { %>
    def <%= method.name.replace(/([A-Z])/g, '_$1').toLowerCase() %>(self, <%= method.parameter.name %>: <%= method.parameter.type %>) -> 'SuperConfig':
        <% if (method.action === 'addToArray') { %>
        if '<%= method.field %>' not in self._config_state:
            self._config_state['<%= method.field %>'] = []
        self._config_state['<%= method.field %>'].append(<%= method.parameter.name %>)
        <% } else if (method.action === 'setValue') { %>
        self._config_state['<%= method.field %>'] = <%= method.parameter.name %>
        <% } %>
        return self
    <% } %>

    async def extract(self) -> Any:
        if not self._plugin:
            raise RuntimeError('Plugin not initialized')

        result = self._plugin.call('ProcessConfig', json.dumps(self._config_state))
        response = json.loads(result)
        
        if not response['success']:
            raise Exception(response['error'])
        
        return response['data']

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        if self._plugin:
            self._plugin.close()

# Helper function
async def load_config(builder_fn: Callable[['SuperConfig'], 'SuperConfig']) -> Any:
    async with SuperConfig.new() as config:
        configured = builder_fn(config)
        return await configured.extract()
```

### Approach 2: Custom XTP Plugin (Alternative)

Create a completely custom XTP bindgen plugin that generates the ergonomic wrappers.

#### Custom Plugin Structure

```rust
// custom-bindgen-plugin/src/lib.rs
use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct GenerateRequest {
    schema: serde_json::Value,
    language: String,
    template_dir: String,
}

#[derive(Serialize)]  
struct GenerateResponse {
    files: Vec<GeneratedFile>,
}

#[derive(Serialize)]
struct GeneratedFile {
    path: String,
    content: String,
}

#[plugin_fn]
pub fn generate(input: String) -> FnResult<String> {
    let request: GenerateRequest = serde_json::from_str(&input)?;
    
    match request.language.as_str() {
        "typescript" => generate_typescript(&request.schema),
        "python" => generate_python(&request.schema),
        "go" => generate_go(&request.schema),
        // ... etc for all languages
        _ => Err(WithReturnCode::new(Error::msg("Unsupported language"), 1))
    }
}

fn generate_typescript(schema: &serde_json::Value) -> FnResult<String> {
    // Extract fluent methods from schema
    let fluent_methods = schema["x-superconfig"]["fluent-methods"].as_array()
        .ok_or_else(|| Error::msg("Missing fluent methods"))?;
    
    // Generate TypeScript class with fluent methods
    let mut class_content = String::from("export class SuperConfig {\n");
    
    for method in fluent_methods {
        let method_name = method["name"].as_str().unwrap();
        let param_name = method["parameter"]["name"].as_str().unwrap(); 
        let param_type = method["parameter"]["type"].as_str().unwrap();
        
        class_content.push_str(&format!(
            "  {}({}: {}): SuperConfig {{\n    // Implementation\n    return this;\n  }}\n",
            method_name, param_name, param_type
        ));
    }
    
    class_content.push_str("}\n");
    
    let response = GenerateResponse {
        files: vec![GeneratedFile {
            path: "src/index.ts".to_string(),
            content: class_content,
        }],
    };
    
    Ok(serde_json::to_string(&response)?)
}
```

## Implementation Plan

### Phase 1: Custom Template Development (Week 1-2)

1. **Fork XTP TypeScript Bindgen**
   ```bash
   git clone https://github.com/dylibso/xtp-typescript-bindgen
   cd xtp-typescript-bindgen
   # Modify templates for host-side wrapper generation
   ```

2. **Create Custom Templates**
   - Replace PDK templates with host-side wrapper templates
   - Add ergonomic API generation logic
   - Include WASM loading and management code

3. **Test with SuperConfig Schema**
   ```bash
   xtp plugin init --schema-file ./superconfig-schema.yaml \
       --template ./custom-typescript-bindgen \
       --path ./test-wrapper
   ```

### Phase 2: Multi-Language Support (Week 3-4)

1. **Fork Additional Bindgens**
   - Python: https://github.com/dylibso/xtp-python-bindgen
   - Go: https://github.com/dylibso/xtp-go-bindgen
   - Rust: https://github.com/dylibso/xtp-rust-bindgen
   - C#: https://github.com/dylibso/xtp-csharp-bindgen

2. **Create Consistent APIs Across Languages**
   - Ensure identical method names (adjusted for language conventions)
   - Consistent error handling patterns
   - Similar helper functions

3. **Automate Build Process**
   ```bash
   # build-all-wrappers.sh
   for lang in typescript python go rust csharp java zig cpp; do
     xtp plugin init --schema-file ./superconfig-schema.yaml \
         --template ./custom-${lang}-bindgen \
         --path ./packages/${lang}
   done
   ```

### Phase 3: Distribution (Week 5-6)

1. **Publish Custom Bindgens**
   - Host on GitHub or npm registry
   - Create bundles compatible with XTP CLI
   - Document usage for each language

2. **Package Distribution**
   - NPM: `@superconfig/typescript`
   - PyPI: `superconfig-extism`
   - Cargo: `superconfig-extism`
   - Go Module: `github.com/superconfig/extism-go`

3. **WASM Binary Distribution**
   - Include `superconfig.wasm` in each package
   - Optimize binary size with `wasm-opt`
   - Version management across languages

## Automation Analysis

### Template Generation: Manual vs AI-Assisted

#### Manual Template Creation (Recommended)

**Pros:**

- Full control over generated code quality
- Optimized for each language's idioms
- Predictable and debuggable output
- Maintainable long-term

**Cons:**

- Initial setup time (1-2 weeks per language)
- Need to understand each language's conventions
- Manual updates when schema changes

#### AI-Assisted Template Creation

**Pros:**

- Faster initial creation
- Can generate multiple languages quickly
- Consistent patterns across languages

**Cons:**

- Generated code may not be idiomatic
- Requires manual review and cleanup
- Less predictable output quality
- Harder to maintain and debug

**Recommendation**: Use **manual template creation** for production quality. AI can help with initial boilerplate, but manual refinement is essential for ergonomic APIs.

### Automatic Generation Process

Once templates are created, the generation process is **fully automated**:

```bash
# Single command generates all language wrappers
./build-superconfig-wrappers.sh

# Output: 15+ language packages with identical APIs
packages/
├── typescript/     # npm package ready
├── python/         # PyPI package ready  
├── go/             # Go module ready
├── rust/           # Cargo crate ready
├── csharp/         # NuGet package ready
└── ...
```

## Extending XTP vs Creating Custom Solution

### Option A: Extend XTP Bindgen (Recommended)

**Approach**: Fork existing XTP bindgens and modify templates

**Pros:**

- Leverages mature, tested framework
- Inherits XTP's schema validation and tooling
- Compatible with existing XTP CLI workflow
- Community support and documentation
- Future XTP improvements benefit us automatically

**Cons:**

- Dependent on XTP's roadmap and decisions
- Need to maintain forks of multiple repositories
- May have limitations we can't easily change

### Option B: Custom Wrapper Generator

**Approach**: Build completely custom code generation tool

**Pros:**

- Full control over generation process
- Can optimize specifically for SuperConfig needs
- No dependency on external tools
- Custom schema extensions without limitations

**Cons:**

- Need to build schema parsing, validation, CLI tooling from scratch
- Miss out on XTP ecosystem improvements
- More maintenance burden
- Need to create documentation and tooling

**Recommendation**: **Extend XTP Bindgen** - it's specifically designed for this use case and provides a solid foundation.

## User Experience Comparison

### Current Approach (Basic Extism)

```javascript
// Visible complexity
const result = await plugin.call('load_config', JSON.stringify({
  files: ['config.toml'],
  env_prefix: 'APP_'
}));
```

### Our Enhanced Approach (Custom XTP Templates)

```javascript
// Clean, ergonomic API - identical to plan_wasm.md
const config = await SuperConfig.new()
  .withFile('config.toml')
  .withEnvIgnoreEmpty('APP_');
const result = await config.extract();
```

### Generated Across All Languages

**Python:**

```python
config = await SuperConfig.new() \
    .with_file('config.toml') \
    .with_env_ignore_empty('APP_')
result = await config.extract()
```

**Go:**

```go
config, _ := superconfig.New(ctx)
result, _ := config.WithFile("config.toml").
    WithEnvIgnoreEmpty("APP_").
    Extract()
```

**Rust:**

```rust
let config = SuperConfig::new()
    .with_file("config.toml")
    .with_env_ignore_empty("APP_");
let result = config.extract()?;
```

## Key Benefits of This Approach

1. **Perfect Ergonomics**: No `plugin.call()` visible - native feeling APIs
2. **Write Once, Deploy Everywhere**: Single schema → 15+ languages automatically
3. **Type Safety**: Generated types for every language from the schema
4. **Maintenance Efficiency**: Update schema, regenerate all languages
5. **Performance**: Same optimized WASM binary across all languages
6. **Future-Proof**: New XTP language support benefits us automatically
7. **Professional**: Built on production-grade tooling (XTP/Extism)

## Technical Requirements

### Development Tools Needed

- XTP CLI installation
- Node.js (for template development)
- Language-specific toolchains for testing
- WASM toolchain (wasm-pack, wasm-opt)

### Infrastructure

- GitHub repositories for custom bindgens
- Package registries (npm, PyPI, Cargo, etc.)
- CI/CD for automated testing and publishing
- Documentation hosting

### Skills Required

- Understanding of XTP schema format
- Template engine knowledge (EJS)
- Language-specific packaging (npm, pip, cargo, etc.)
- WASM compilation and optimization

## Success Metrics

- [ ] Schema-driven generation works for TypeScript
- [ ] Generated TypeScript API matches plan_wasm.md ergonomics
- [ ] 5+ languages have working generated wrappers
- [ ] All generated packages install and run correctly
- [ ] Performance within 10% of native implementation
- [ ] Complete API coverage through templates
- [ ] Documentation generated automatically

## Conclusion

**The custom XTP template approach provides the perfect solution**: we get the ergonomic APIs from plan_wasm.md AND the universal deployment benefits of Extism, with the engineering efficiency of writing once and deploying everywhere.

By extending XTP Bindgen with custom templates, we can automatically generate beautiful, idiomatic wrapper libraries for 15+ languages that completely hide the Extism complexity while providing the exact same fluent APIs you designed.

This is the best of all worlds: **write once, generate everywhere, perfect ergonomics**.
