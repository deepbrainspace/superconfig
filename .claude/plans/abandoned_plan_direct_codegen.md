# Direct Language Package Generation (No XTP Middleware)

## Analysis: XTP vs Direct Generation

### What XTP Provides
- **Schema Validation**: OpenAPI-like IDL validation
- **Template Engine**: EJS-based code generation
- **Plugin Architecture**: Language-specific bindgen plugins
- **CLI Orchestration**: `xtp plugin init` workflow

### What We Actually Need
- **WASM Loading**: Load and call our SuperConfig WASM binary
- **Ergonomic APIs**: Builder pattern methods in each language
- **Type Mapping**: Rust types → language-specific types
- **Package Structure**: Native package layouts (npm, PyPI, etc.)

### XTP Overhead We Don't Need
- **Complex Schema Format**: We can use simpler metadata
- **Plugin Architecture**: We control the entire generation pipeline
- **External Dependencies**: Less moving parts = more reliable
- **Generic Use Cases**: We're optimized for builder patterns specifically

## Streamlined Direct Generation Architecture

```
Cargo Metadata + AST Analysis
    ↓
Builder Pattern Metadata (Simple JSON)
    ↓
Direct Language Code Generation
    ↓
Native Packages with WASM Binary Included
```

## Implementation: Direct Code Generation

### Simplified Metadata Format

Instead of XTP's complex schema, use builder-specific metadata:

```rust
#[derive(Debug, Serialize, Deserialize)]
struct BuilderMetadata {
    project_name: String,
    class_name: String,
    version: String,
    wasm_binary: String, // Path to WASM file
    methods: Vec<BuilderMethod>,
    dependencies: BuilderDependencies,
}

#[derive(Debug, Serialize, Deserialize)]
struct BuilderMethod {
    name: String,           // "withFile"
    snake_name: String,     // "with_file" (for Python)
    pascal_name: String,    // "WithFile" (for Go)
    parameter: MethodParam,
    field_name: String,     // "files"
    action: String,         // "push", "set", "merge"
    rust_signature: String, // Original Rust signature for reference
}

#[derive(Debug, Serialize, Deserialize)]
struct BuilderDependencies {
    wasm_runtime: String,   // "extism", "wasmtime", "wasmer"
    async_runtime: Option<String>, // "tokio", "async-std" for async languages
}
```

### Direct Template Generation

Create language-specific templates directly without XTP middleware:

```rust
// src/codegen/mod.rs
use std::collections::HashMap;
use handlebars::Handlebars;

pub struct DirectCodeGenerator {
    templates: HashMap<String, Handlebars<'static>>,
}

impl DirectCodeGenerator {
    pub fn new() -> Self {
        let mut generator = Self {
            templates: HashMap::new(),
        };
        
        // Register templates for each language
        generator.register_language_templates();
        generator
    }
    
    fn register_language_templates(&mut self) {
        // TypeScript
        let mut ts_hb = Handlebars::new();
        ts_hb.register_template_string("main", include_str!("templates/typescript/index.ts.hbs")).unwrap();
        ts_hb.register_template_string("package", include_str!("templates/typescript/package.json.hbs")).unwrap();
        ts_hb.register_template_string("readme", include_str!("templates/typescript/README.md.hbs")).unwrap();
        self.templates.insert("typescript".to_string(), ts_hb);
        
        // Python
        let mut py_hb = Handlebars::new();
        py_hb.register_template_string("main", include_str!("templates/python/__init__.py.hbs")).unwrap();
        py_hb.register_template_string("setup", include_str!("templates/python/setup.py.hbs")).unwrap();
        py_hb.register_template_string("readme", include_str!("templates/python/README.md.hbs")).unwrap();
        self.templates.insert("python".to_string(), py_hb);
        
        // Add more languages...
    }
    
    pub fn generate_language_package(
        &self,
        language: &str,
        metadata: &BuilderMetadata,
        wasm_binary: &[u8],
        output_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match language {
            "typescript" => self.generate_typescript_package(metadata, wasm_binary, output_dir),
            "python" => self.generate_python_package(metadata, wasm_binary, output_dir),
            "go" => self.generate_go_package(metadata, wasm_binary, output_dir),
            "rust" => self.generate_rust_package(metadata, wasm_binary, output_dir),
            _ => Err(format!("Unsupported language: {}", language).into()),
        }
    }
}
```

### TypeScript Template Example

```typescript
// templates/typescript/index.ts.hbs
import { Plugin } from '@extism/extism';
import * as fs from 'fs';
import * as path from 'path';

export class {{class_name}} {
    private plugin: Plugin | null = null;
    private state: any = {};
    private static wasmBinary: ArrayBuffer | null = null;

    private constructor() {}

    static async new(): Promise<{{class_name}}> {
        const instance = new {{class_name}}();
        await instance.initializePlugin();
        return instance;
    }

    private async initializePlugin(): Promise<void> {
        if (!{{class_name}}.wasmBinary) {
            const wasmPath = path.join(__dirname, 'wasm', '{{wasm_binary}}');
            {{class_name}}.wasmBinary = fs.readFileSync(wasmPath);
        }
        
        this.plugin = await Plugin.create({
            wasm: [{ data: {{class_name}}.wasmBinary }],
        });
    }

    {{#each methods}}
    {{name}}({{parameter.name}}: {{parameter.typescript_type}}): {{../class_name}} {
        {{#if (eq action "push")}}
        if (!this.state.{{field_name}}) {
            this.state.{{field_name}} = [];
        }
        this.state.{{field_name}}.push({{parameter.name}});
        {{else}}
        this.state.{{field_name}} = {{parameter.name}};
        {{/if}}
        return this;
    }

    {{/each}}

    async extract<T = any>(): Promise<T> {
        if (!this.plugin) {
            throw new Error('Plugin not initialized');
        }

        const result = await this.plugin.call('process_{{snake_case project_name}}', JSON.stringify(this.state));
        const response = JSON.parse(result);
        
        if (!response.success) {
            throw new Error(response.error || 'Unknown error');
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

// Convenience function
export async function load{{class_name}}<T = any>(
    builderFn: (config: {{class_name}}) => {{class_name}}
): Promise<T> {
    const config = await {{class_name}}.new();
    try {
        const configured = builderFn(config);
        return await configured.extract<T>();
    } finally {
        await config.dispose();
    }
}
```

### Python Template Example

```python
# templates/python/__init__.py.hbs
import json
import asyncio
from typing import Any, Dict, Optional, TypeVar, Callable, Union
from pathlib import Path
import pkg_resources
from extism import Plugin

T = TypeVar('T')

class {{class_name}}:
    _wasm_binary: Optional[bytes] = None
    
    def __init__(self):
        self._plugin: Optional[Plugin] = None
        self._state: Dict[str, Any] = {}

    @classmethod
    async def new(cls) -> '{{class_name}}':
        instance = cls()
        await instance._initialize_plugin()
        return instance

    async def _initialize_plugin(self) -> None:
        if {{class_name}}._wasm_binary is None:
            wasm_path = pkg_resources.resource_filename(__name__, 'wasm/{{wasm_binary}}')
            with open(wasm_path, 'rb') as f:
                {{class_name}}._wasm_binary = f.read()
        
        self._plugin = Plugin(
            manifest={"wasm": [{"data": {{class_name}}._wasm_binary}]},
            wasi=True
        )

    {{#each methods}}
    def {{snake_name}}(self, {{parameter.name}}: {{parameter.python_type}}) -> '{{../class_name}}':
        {{#if (eq action "push")}}
        if '{{field_name}}' not in self._state:
            self._state['{{field_name}}'] = []
        self._state['{{field_name}}'].append({{parameter.name}})
        {{else}}
        self._state['{{field_name}}'] = {{parameter.name}}
        {{/if}}
        return self

    {{/each}}

    async def extract(self) -> Any:
        if not self._plugin:
            raise RuntimeError('Plugin not initialized')

        result = self._plugin.call('process_{{snake_case project_name}}', json.dumps(self._state))
        response = json.loads(result)
        
        if not response.get('success', False):
            raise Exception(response.get('error', 'Unknown error'))
        
        return response.get('data')

    async def __aenter__(self):
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self._plugin:
            self._plugin.close()

# Convenience function
async def load_{{snake_case class_name}}(builder_fn: Callable[['{{class_name}}'], '{{class_name}}']) -> Any:
    async with await {{class_name}}.new() as config:
        configured = builder_fn(config)
        return await configured.extract()
```

## Benefits of Direct Generation

### Simplified Pipeline
1. **Cargo Extension** analyzes Rust code → **Builder Metadata**
2. **Direct Templates** generate native packages with embedded WASM
3. **No XTP Dependencies** or complex schema validation

### Better Performance
- **Faster Generation**: No XTP plugin loading/execution
- **Smaller Output**: Only generate what we need
- **Direct Control**: Optimize for builder patterns specifically

### Easier Maintenance
- **Fewer Dependencies**: Just Handlebars for templating
- **Simpler Debugging**: Direct template → code mapping
- **Custom Optimizations**: Builder-pattern specific optimizations

## Updated Cargo Extension

```rust
fn exec_analyze(gctx: &mut GlobalContext, args: &ArgMatches) -> CliResult {
    let ws = args.workspace(gctx)?;
    
    // Auto-detect all builder patterns in workspace
    let all_builders = auto_detect_builders(&ws)?;
    
    if all_builders.is_empty() {
        println!("ℹ️  No builder patterns detected in workspace");
        return Ok(());
    }
    
    // List detected builders
    println!("🔍 Detected builder patterns:");
    for builder in &all_builders {
        println!("  📋 {} in {} ({})", 
            builder.class_name, 
            builder.package_name,
            builder.source_file
        );
    }
    
    // Generate for all detected builders or specific one
    let target_builders = if let Some(target) = args.get_one::<String>("target-struct") {
        all_builders.into_iter()
            .filter(|b| b.class_name == target)
            .collect()
    } else {
        all_builders
    };
    
    // Generate language packages directly
    let generator = DirectCodeGenerator::new();
    let languages = ["typescript", "python", "go", "rust", "java", "csharp"];
    
    for builder in target_builders {
        println!("🚀 Generating packages for: {}", builder.class_name);
        
        // Build WASM binary first
        let wasm_binary = build_wasm_binary(&builder, &ws)?;
        
        for language in languages {
            let output_dir = ws.target_dir()
                .join("universal-bindings")
                .join(&builder.class_name.to_lowercase())
                .join(language);
            
            generator.generate_language_package(
                language,
                &builder,
                &wasm_binary,
                &output_dir,
            )?;
            
            println!("  ✅ {} package: {}", language, output_dir.display());
        }
    }
    
    Ok(())
}

fn auto_detect_builders(ws: &Workspace) -> CargoResult<Vec<BuilderMetadata>> {
    let mut builders = Vec::new();
    
    for package in ws.members() {
        for target in package.targets() {
            if target.is_lib() {
                // Analyze the library target for builder patterns
                let detected = detect_builder_patterns(target, package)?;
                builders.extend(detected);
            }
        }
    }
    
    Ok(builders)
}

fn detect_builder_patterns(target: &Target, package: &Package) -> CargoResult<Vec<BuilderMetadata>> {
    // Heuristics for detecting builder patterns:
    // 1. Struct with new() method
    // 2. Multiple with_* methods returning Self
    // 3. Terminal method (extract, build, finish, etc.)
    
    // This would contain the AST analysis logic
    // but focused on automatic detection rather than targeting specific structs
    
    todo!("Implement builder pattern detection heuristics")
}
```

## Usage Becomes Much Simpler

```bash
# Auto-detect and generate all builders
cargo builder-analyzer analyze

# Output for SuperConfig:
🔍 Detected builder patterns:
  📋 SuperConfig in superconfig (src/lib.rs)
🚀 Generating packages for: SuperConfig
  ✅ typescript package: target/universal-bindings/superconfig/typescript
  ✅ python package: target/universal-bindings/superconfig/python  
  ✅ go package: target/universal-bindings/superconfig/go
  ✅ rust package: target/universal-bindings/superconfig/rust
  ✅ java package: target/universal-bindings/superconfig/java
  ✅ csharp package: target/universal-bindings/superconfig/csharp
```

This approach is much more streamlined and gives us complete control over the generation process while eliminating XTP complexity!