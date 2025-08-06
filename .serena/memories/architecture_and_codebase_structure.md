# SuperConfig Architecture and Codebase Structure

## Workspace Architecture

### Monorepo Layout

```
superconfig/                         # Root workspace
├── crates/                          # All Rust crates
│   ├── superconfig/                 # 🏗️ Main configuration library
│   ├── logffi/                      # 📋 FFI logging with callback support
│   ├── multiffi/                    # 🔌 Multi-language FFI generator
│   ├── superconfig-macros/          # 🪄 Procedural macros
│   ├── superhashmap/               # ⚡ High-performance hashmap
│   ├── meta-rust/                  # 🦀 Rust metaprogramming utilities
│   └── hash-benchmark/             # 📊 Benchmarking utilities
├── .moon/                          # 🌙 Moon build system config
├── .github/                        # 🚀 CI/CD workflows
├── .claude/                        # 🤖 Claude Code configuration
├── marketing/                      # 📢 Marketing materials
└── crates-archive/                 # 📦 Archived crate versions
```

## Core Crate: SuperConfig

### Current Architecture (V2.1 Implementation)

```rust
// crates/superconfig/src/lib.rs structure:
superconfig/
├── types/                          # Core type system (implemented)
│   ├── mod.rs
│   └── [type definitions]
├── core/                           # Registry system (pending)
├── backend/                        # Backend implementations (pending)  
├── formats/                        # Multi-format support (pending)
├── sources/                        # Configuration sources (pending)
├── trees/                          # Tree management (pending)
└── api/                           # Public API layer (pending)
```

### Design Principles

- **Handle-based registry**: Sub-microsecond configuration lookup
- **Zero-copy access**: No serialization overhead for repeated access
- **Lock-free operations**: SCC HashMap-based concurrent registry
- **Multi-format support**: TOML, JSON, YAML with auto-detection
- **FFI compatibility**: Native integration with Python, Node.js, WASM

## Supporting Crates

### LogFFI (`crates/logffi/`)

- **Purpose**: Multi-backend logging with FFI callback support
- **Features**: Tracing integration, callback support, error macros
- **Architecture**: Conditional compilation for different backends
- **Status**: Stable (v0.1.0)

### MultiFI (`crates/multiffi/`)

- **Purpose**: Multi-language FFI binding generator
- **Targets**: Python, Node.js, WebAssembly
- **Features**: Automatic binding generation, memory management

### SuperConfig Macros (`crates/superconfig-macros/`)

- **Purpose**: Procedural macros for SuperConfig ecosystem
- **Features**: Fluent API error handling, FFI integration macros
- **Dependencies**: Built on meta-rust utilities

## Build System Architecture

### Moon Build System

- **Configuration**: `.moon/tasks.yml` defines shared tasks
- **Project discovery**: `projects: ['crates/*']`
- **Task inheritance**: All crates inherit common tasks
- **Affected detection**: Only builds changed crates

### Task Categories

1. **Build tasks**: `build`, `build-release`, `check`
2. **Quality tasks**: `format`, `lint`, `format-check`
3. **Testing tasks**: `test`, `coverage`, `bench`
4. **Security tasks**: `security-audit`, `deny`
5. **Documentation tasks**: `doc`, `doc-open`
6. **Publishing tasks**: `publish`, `publish-dry`

### Git Hooks (Lefthook)

```yaml
# Automated workflow:
pre-commit:
  - Format code (rust, python, js, markdown)
  - Run linting checks
  - Validate commit messages

pre-push:
  - Ensure clean repository
  - Run format checks
  - Security audits
```

## Dependency Architecture

### Core Dependencies

- **serde**: Serialization framework (v1.0+)
- **serde_json**: JSON handling with raw values
- **thiserror**: Error handling and custom errors
- **scc**: Lock-free concurrent data structures
- **logffi**: Internal logging (path dependency)

### Development Dependencies

- **criterion**: Benchmarking with HTML reports
- **serial_test**: Test serialization for resource conflicts
- **tempfile**: Temporary file handling in tests
- **tokio-test**: Async testing utilities

## Configuration Management

### Hierarchical Configuration Discovery

```
Priority (high to low):
1. CLI arguments                     # --config.key=value
2. Environment variables             # APP_CONFIG_KEY=value
3. Current directory                 # ./app.{toml,yaml,json}
4. Parent directories                # ../app.{toml,yaml,json}
5. User home                         # ~/.app/app.{toml,yaml,json}
6. System config                     # ~/.config/app/app.{toml,yaml,json}
7. Defaults                          # Built-in defaults
```

### Array Merging Strategy

```toml
# System config
features = ["auth", "logging"]

# User config
features_add = ["debug"] # Result: ["auth", "logging", "debug"]
features_remove = ["logging"] # Result: ["auth", "debug"]
```

## Performance Architecture

### Zero-Copy Design

- **Handle-based access**: Direct memory references
- **Lazy loading**: Parse only when accessed
- **Modification time caching**: Skip re-parsing unchanged files
- **SCC HashMap**: Lock-free concurrent data structures

### Memory Management

- **No unnecessary cloning**: Prefer borrowing
- **FFI boundaries**: Careful ownership transfer
- **String interning**: Reduce memory usage for repeated strings
- **Smart caching**: Cache parsed configurations

## FFI Architecture

### Multi-Language Support

```
Rust Core
    ↓
C FFI Layer (extern "C")
    ↓
Language Bindings:
├── Python (ctypes/cffi)
├── Node.js (N-API)
└── WebAssembly (wasm-bindgen)
```

### Error Handling Across Languages

- **Rust**: thiserror + Result types
- **C FFI**: Error codes + message buffers
- **Python**: Python exceptions
- **Node.js**: JavaScript errors
- **WASM**: JavaScript errors

## CI/CD Architecture

### GitHub Actions Workflow

```yaml
Workflow:
1. detect-affected          # Moon-based change detection
2. Parallel execution:
   ├── build               # Per-crate builds
   ├── test                # Per-crate testing  
   ├── quality             # Formatting/linting
   ├── security            # Security audits
   ├── compliance          # Dependency checks
   └── coverage            # Code coverage
```

### Caching Strategy

- **Build artifacts**: Cached by Cargo.lock hash
- **Dependencies**: Proto tool installations cached
- **Moon state**: Build system state cached

## Future Architecture (Planned)

### V2.1 Implementation Phases

1. **Phase 1**: Core registry system
2. **Phase 2**: Multi-format parsing system
3. **Phase 3**: Configuration sources
4. **Phase 4**: Tree management and merging
5. **Phase 5**: Public API layer

### Extensibility Points

- **Custom providers**: Plugin system for configuration sources
- **Format plugins**: Support for additional configuration formats
- **Validation plugins**: Schema validation extensions
- **Export formats**: Additional output format support
