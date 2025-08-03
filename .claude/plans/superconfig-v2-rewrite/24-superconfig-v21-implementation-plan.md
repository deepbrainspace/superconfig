# SuperConfig v2.1 Implementation Plan

## Grok3 Multi-Format Design Implementation with Full Key/Value Support

**Plan Document**: 24-superconfig-v21-implementation-plan.md\
**Date**: 2025-08-03\
**Scope**: Complete rewrite of SuperConfig core based on Grok3 multi-format design\
**Target**: v2.1 release with nested key/value support, profiles, swappable backends, and multi-format input/output

**Related Documents:**

- [24a-existing-files-review.txt](./24a-existing-files-review.txt) - File retrieval strategy for preserving v2.0 components
- [24b-logging-addendum.md](./24b-logging-addendum.md) - **LogFFI Universal Architecture** - Complete logging and error handling system that makes LogFFI the universal Rust logging standard

---

## 🤖 LLM Working Practices & Implementation Guidelines

### **MANDATORY WORKING PATTERN**

This implementation **MUST** follow these strict working practices:

#### **1. Phase-by-Phase Execution**

- ✅ **Work on ONE phase at a time** - never jump ahead or work on multiple phases
- ✅ **Get approval before starting each phase** - explain what you plan to do and get user confirmation
- ✅ **Complete ALL tasks in a phase** before moving to the next phase
- ❌ **NEVER skip phases or work out of order**

#### **2. Pre-Phase Approval Process**

Before starting each phase:

1. **Explain exactly what you plan to implement** in that phase
2. **List the specific files you'll create/modify** and what each will contain
3. **Describe the testing approach** you'll use for that phase
4. **Get explicit user approval** before proceeding with any code changes
5. **Clarify any uncertainties** or design decisions with the user

#### **3. Post-Phase Testing & Review Process**

After completing each phase:

1. **Test thoroughly yourself** - run all tests, check compilation, verify functionality
2. **Document any issues encountered** and how you resolved them
3. **Provide detailed testing instructions** for the user to verify the phase
4. **Include specific commands to run** and expected outputs
5. **Wait for user approval** before marking the phase complete
6. **Only then tick off the tasks** and move to the next phase

#### **4. Problem Resolution Protocol**

When encountering any issues:

- ❌ **NEVER solve problems independently** without consulting the user first
- ✅ **Stop work immediately** and explain the problem to the user
- ✅ **Describe the issue clearly** with context and potential impact
- ✅ **Propose your intended solution** and get approval before proceeding
- ✅ **Document the solution** for future reference

#### **5. Technology Standards & Research Requirements**

- ✅ **Use Rust 2024 edition** for all code
- ✅ **Check latest crate versions** using context7 or internet research before adding dependencies
- ✅ **Use latest documentation** for Rust and all libraries
- ✅ **Verify compatibility** between crate versions before integration
- ✅ **Document version choices** and reasoning

#### **6. Quality Assurance Standards**

- ✅ **Write comprehensive tests** for each component as you implement it
- ✅ **Ensure clean compilation** with no warnings
- ✅ **Run cargo clippy** and fix all suggestions
- ✅ **Run cargo fmt** for consistent code formatting
- ✅ **Test edge cases** and error conditions
- ✅ **Verify performance requirements** are met

#### **7. Communication Requirements**

- ✅ **Provide regular status updates** during long implementation phases
- ✅ **Ask for clarification** when requirements are ambiguous
- ✅ **Explain design decisions** and trade-offs made
- ✅ **Document any deviations** from the original plan
- ✅ **Seek approval for any plan modifications**

### **Example Phase Workflow**

```
1. User approves overall plan
2. LLM: "I'm about to start Phase 1. I plan to create files X, Y, Z with functionality A, B, C. 
   I'll test by doing P, Q, R. Do you approve?"
3. User: "Approved"
4. LLM implements Phase 1 completely
5. LLM: "Phase 1 complete. Please test by running: [specific commands]. Expected results: [detailed description]"
6. User tests and approves
7. LLM: ✅ marks Phase 1 complete, asks approval for Phase 2
8. Repeat...
```

### **Failure Protocol**

If at any point the LLM:

- Skips the approval process
- Works on multiple phases simultaneously
- Solves problems without consultation
- Uses outdated dependencies
- Provides insufficient testing instructions

The user should **immediately stop the work** and redirect back to proper working practices.

---

## Architecture Overview

```
┌────────────────────────────────────────────────────────────────────────┐
│                   SuperConfig v2.1 Multi-Format Architecture           │
└────────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────────┐
│                           Public API Layer                             │
├────────────────────────────────────────────────────────────────────────┤
│  SuperConfig                                                           │
│  ├── new() -> Self                                                     │
│  ├── select(profile: &str) -> ConfigRegistry                           │
│  ├── get<T>(key: &str) -> Option<T>                                    │
│  ├── get_handle<T>(key: &str) -> Option<ConfigHandle<T>>               │
│  ├── set<T>(profile: &str, key: &str, data: T) -> ConfigHandle<T>      │
│  ├── merge_file(path: &str) -> Result<(), String>     [Auto-detect]    │
│  ├── merge_string(content: &str) -> Result<(), String> [Auto-detect]   │
│  ├── merge_env(prefix: &str) -> Result<(), String>    [Env vars]       │
│  ├── merge_cli(args: &[String]) -> Result<(), String> [CLI args]       │
│  ├── to_format(profile, format) -> Result<String, String>              │
│  └── extract<T: Deserialize>() -> Result<T, String>                    │
└────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                         ConfigRegistry Layer                           │
├────────────────────────────────────────────────────────────────────────┤
│  ConfigRegistry                                                        │
│  ├── backend: Arc<dyn ConfigRegistryBackend>    [DataMap Layer]        │
│  ├── keymaps: Arc<SccHashMap<Profile, KeyMap>>  [Profile→Key Map]      │
│  ├── trees: Arc<SccHashMap<Profile, toml::Val>> [TOML Extract Trees]   │
│  ├── dirty_profiles: Arc<SccHashMap<Profile, bool>> [Rebuild flags]    │
│  └── selected_profile: Profile                  [Current Context]      │
└────────────────────────────────────────────────────────────────────────┘
                                    │
                      ┌─────────────┼─────────────┐
                      ▼             ▼             ▼
    ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
    │  Backend Layer  │ │  KeyMap Layer   │ │   Tree Layer    │
    ├─────────────────┤ ├─────────────────┤ ├─────────────────┤
    │ConfigRegistry   │ │Profile-Specific │ │TOML Value Trees │
    │Backend (Trait)  │ │Key → HandleID   │ │for Deserialize  │
    │                 │ │Mappings         │ │                 │
    │┌───────────────┐│ │                 │ │Profile → Tree   │
    ││   SccBackend  ││ │Example:         │ │                 │
    ││               ││ │"default.storage │ │Used by:         │
    ││ DataMap:      ││ │ .db.host" → 123 │ │extract<T>()     │
    ││ SccHashMap    ││ │                 │ │rebuild_struct() │
    ││ <HandleID,    ││ │Enables:         │ │                 │
    ││  Arc<Data>>   ││ │- Nested keys    │ │Enables:         │
    ││               ││ │- Profile scope  │ │- Struct deser   │
    │└───────────────┘│ │- Fast lookup    │ │- Full tree      │
    │                 │ │                 │ │  access         │
    │Swappable:       │ │Per Profile:     │ │Per Profile:     │
    │- SCC Backend    │ │KeyMap =         │ │toml::Value =    │
    │- Redis Backend  │ │Arc<SccHashMap   │ │Nested structure │
    │- Memory Backend │ │<String, u64>>   │ │from input       │
    └─────────────────┘ └─────────────────┘ └─────────────────┘
                        
┌────────────────────────────────────────────────────────────────────────┐
│                  Format & Source Integration Layer                     │
├────────────────────────────────────────────────────────────────────────┤
│                                                                        │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐     │
│  │  Formats Layer  │    │  Sources Layer  │    │  Detection      │     │
│  ├─────────────────┤    ├─────────────────┤    ├─────────────────┤     │
│  │ ConfigFormat    │    │ Input Sources   │    │ Format Auto-    │     │
│  │ trait           │    │ (Key/Value)     │    │ Detection       │     │
│  │                 │    │                 │    │                 │     │
│  │ Implementations:│    │ Sources:        │    │ Methods:        │     │
│  │ - TomlFormat    │    │ - EnvVars       │    │ - File ext      │     │
│  │ - JsonFormat    │    │ - CLI Args      │    │ - Content       │     │
│  │ - YamlFormat    │    │                 │    │   heuristics    │     │
│  │ - IniFormat     │    │ Processing:     │    │ - Parse attempt │     │
│  │                 │    │ - Key flattening│    │                 │     │
│  │ Each provides:  │    │ - Profile detect│    │ Fallback order: │     │
│  │ - parse()       │    │ - Type convert  │    │ 1. Extension    │     │
│  │ - serialize()   │    │                 │    │ 2. JSON (fast)  │     │
│  │ - flatten()     │    │ Input examples: │    │ 3. YAML         │     │
│  │ - reconstruct() │    │ APP_DB_HOST=x   │    │ 4. TOML         │     │
│  │                 │    │ --db.host=x     │    │ 5. INI          │     │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘     │
└────────────────────────────────────────────────────────────────────────┘
```

### Multi-Format Data Flow Example

```
┌────────────────────────────────────────────────────────────────────────┐
│                     Multi-Format Data Flow Example                     │
├────────────────────────────────────────────────────────────────────────┤
│                                                                        │
│ 1. Multiple Input Sources:                                             │
│    ┌─────────────────────────────────────────────────────────────────┐ │
│    │ File (app.toml):                                                │ │
│    │ [default.storage.db]                                            │ │
│    │ host = "localhost"                                              │ │
│    │ port = 5432                                                     │ │
│    │                                                                 │ │
│    │ JSON String:                                                    │ │
│    │ {"staging": {"storage": {"db": {"host": "staging-db"}}}}        │ │
│    │                                                                 │ │
│    │ Environment Variables:                                          │ │
│    │ APP_PROD_STORAGE_DB_HOST=prod-db                                │ │
│    │ APP_PROD_STORAGE_DB_PORT=5433                                   │ │
│    │                                                                 │ │
│    │ CLI Arguments:                                                  │ │
│    │ --dev.storage.db.host=dev-db --dev.storage.db.port=5434         │ │
│    └─────────────────────────────────────────────────────────────────┘ │
│                                     │                                  │
│                                     ▼                                  │
│ 2. Auto-Detection & Processing:                                        │
│    • File: .toml extension → TomlFormat → flatten                      │
│    • String: starts with { → JsonFormat → flatten                      │ 
│    • EnvVars: APP_ prefix → key conversion → direct insert             │
│    • CLI: --key.path → key conversion → direct insert                  │
│                                                                        │
│ 3. Unified Storage in DataMap:                                         │
│    • DataMap[123] = Arc<String>("localhost")                           │
│    • DataMap[124] = Arc<i64>(5432)                                     │
│    • DataMap[456] = Arc<String>("staging-db")                          │
│    • DataMap[789] = Arc<String>("prod-db")                             │
│    • DataMap[101] = Arc<String>("dev-db")                              │
│                                                                        │
│ 4. Profile-Based KeyMaps:                                              │
│    • default: {"default.storage.db.host" → 123, ...}                   │
│    • staging: {"staging.storage.db.host" → 456, ...}                   │
│    • prod:    {"prod.storage.db.host" → 789, ...}                      │
│    • dev:     {"dev.storage.db.host" → 101, ...}                       │
│                                                                        │
│ 5. Access Examples:                                                    │
│    • config.select("default").get("storage.db.host") → "localhost"     │
│    • config.select("staging").get("storage.db.host") → "staging-db"    │
│    • config.select("prod").get("storage.db.host") → "prod-db"          │
│                                                                        │
│ 6. Output to Any Format:                                               │
│    • config.to_format("prod", &YamlFormat) → YAML string               │
│    • config.to_format("dev", &JsonFormat) → JSON string                │
│    • config.extract::<Config>() → Struct from tree                     │
└────────────────────────────────────────────────────────────────────────┘
```

### Memory Layout with Multi-Format Support

```
┌────────────────────────────────────────────────────────────────────────┐
│                     Memory Layout - Multi-Format                       │
├────────────────────────────────────────────────────────────────────────┤
│                                                                        │
│ Global Registry:                                                       │
│ ┌──────────────────────────────────────────────────────────────────┐   │
│ │ CONFIG_REGISTRY: Lazy<ConfigRegistry>                            │   │
│ │ NEXT_HANDLE_ID: AtomicU64                                        │   │
│ └──────────────────────────────────────────────────────────────────┘   │
│                                                                        │
│ DataMap (Unified Backend Storage):                                     │
│ ┌──────────────────────────────────────────────────────────────────┐   │
│ │ SccHashMap<HandleID, Arc<dyn Any + Send + Sync>>                 │   │
│ │ ├── 123 → Arc<String>("localhost")    [from TOML]                │   │
│ │ ├── 124 → Arc<i64>(5432)              [from TOML]                │   │
│ │ ├── 456 → Arc<String>("staging-db")   [from JSON string]         │   │
│ │ ├── 789 → Arc<String>("prod-db")      [from ENV var]             │   │
│ │ ├── 101 → Arc<String>("dev-db")       [from CLI arg]             │   │
│ │ └── 202 → Arc<Config>({...})          [rebuilt struct]           │   │
│ └──────────────────────────────────────────────────────────────────┘   │
│                                                                        │
│ Multi-Profile KeyMaps:                                                 │
│ ┌──────────────────────────────────────────────────────────────────┐   │
│ │ SccHashMap<Profile, Arc<SccHashMap<String, HandleID>>>           │   │
│ │ ├── "default" → {                                                │   │
│ │ │     "default.storage.db.host" → 123,                           │   │
│ │ │     "default.storage.db.port" → 124,                           │   │
│ │ │     "default" → 202  [full struct handle]                      │   │
│ │ │   }                                                            │   │
│ │ ├── "staging" → {                                                │   │
│ │ │     "staging.storage.db.host" → 456,                           │   │
│ │ │     "staging" → 303  [full struct handle]                      │   │
│ │ │   }                                                            │   │
│ │ ├── "prod" → {                                                   │   │
│ │ │     "prod.storage.db.host" → 789,                              │   │
│ │ │     "prod.storage.db.port" → 790,                              │   │
│ │ │     "prod" → 404  [full struct handle]                         │   │
│ │ │   }                                                            │   │
│ │ └── "dev" → {                                                    │   │
│ │       "dev.storage.db.host" → 101,                               │   │
│ │       "dev.storage.db.port" → 102,                               │   │
│ │       "dev" → 505  [full struct handle]                          │   │
│ │     }                                                            │   │
│ └──────────────────────────────────────────────────────────────────┘   │
│                                                                        │
│ Trees for Struct Deserialization:                                      │
│ ┌──────────────────────────────────────────────────────────────────┐   │
│ │ SccHashMap<Profile, toml::Value>                                 │   │
│ │ ├── "default" → Table({storage: {db: {host: "localhost"}}})      │   │
│ │ ├── "staging" → Table({storage: {db: {host: "staging-db"}}})     │   │
│ │ ├── "prod" → Table({storage: {db: {host: "prod-db"}}})           │   │
│ │ └── "dev" → Table({storage: {db: {host: "dev-db"}}})             │   │
│ └──────────────────────────────────────────────────────────────────┘   │
│                                                                        │
│ Dirty Profile Tracking:                                                │
│ ┌──────────────────────────────────────────────────────────────────┐   │
│ │ SccHashMap<Profile, bool>                                        │   │
│ │ ├── "staging" → true   [needs struct rebuild]                    │   │
│ │ ├── "prod" → false     [struct is current]                       │   │
│ │ └── "dev" → true       [needs struct rebuild]                    │   │
│ └──────────────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────────────┘
```

---

## Overview

This plan implements the enhanced Grok3 design from document 23a, providing full key/value support with multi-format input/output capabilities. The design incorporates:

### Key Features from Enhanced Grok3 Design

- **Multi-Format Input**: TOML, JSON, YAML, INI file support with auto-detection
- **String Input**: Arbitrary config strings with format auto-detection
- **Environment Variables**: `APP_STORAGE_DB_HOST` → `storage.db.host` conversion
- **CLI Arguments**: `--storage.db.host=value` parsing
- **Multi-Format Output**: Export to any supported format
- **Nested Keys**: `storage.a.b.c` support via flattened keymaps
- **Profiles**: Per-profile configurations (`default`, `staging`, `prod`, etc.)
- **Swappable Backends**: Plugin architecture for different storage engines
- **DataMap**: Structured data store using K,D format (Key-Data mapping)
- **Arc-based**: Memory efficient sharing throughout
- **FFI Ready**: Python/Node.js support with ~51-56μs performance

---

## Current State Analysis

### What We're Replacing

- Current handle-only system in `crates/superconfig/src/core/`
- Limited to handle-based access only
- No nested key support
- No profile support
- No multi-format support
- DashMap-based simple registry

### What We're Keeping

- Benchmark infrastructure (`benches/`)
- Moon build configuration (`moon.yml`)
- Core project structure
- Performance targets and testing approach
- Handle-based access as compatibility layer

### What We're Adding

- Multi-format input/output system
- Environment variable integration
- CLI argument parsing
- Format auto-detection
- Sources abstraction layer
- Comprehensive format support

---

## File Structure & Organization

```
crates/superconfig/
├── Cargo.toml                    # Dependencies: scc, serde, toml, serde_json, serde_yaml
├── src/
│   ├── lib.rs                    # Public API exports and documentation
│   │
│   ├── api/                      # Public API Layer
│   │   ├── mod.rs                # API module exports
│   │   └── superconfig.rs        # SuperConfig main struct with all merge methods
│   │
│   ├── core/                     # Core Registry System
│   │   ├── mod.rs                # Core module exports
│   │   ├── registry.rs           # ConfigRegistry with multi-format support
│   │   ├── profile.rs            # Profile type and management
│   │   └── handle.rs             # ConfigHandle<T> implementation
│   │
│   ├── backend/                  # Swappable Backend System
│   │   ├── mod.rs                # Backend module exports
│   │   ├── traits.rs             # ConfigRegistryBackend trait
│   │   ├── scc_backend.rs        # SCC HashMap DataMap implementation
│   │   └── memory_backend.rs     # Simple in-memory backend (optional)
│   │
│   ├── keymap/                   # Key Management System
│   │   ├── mod.rs                # Keymap module exports
│   │   ├── manager.rs            # KeyMap management and operations
│   │   ├── key_utils.rs          # Key flattening and validation
│   │   └── profile_keys.rs       # Per-profile key operations
│   │
│   ├── formats/                  # Multi-Format Support System
│   │   ├── mod.rs                # Format module exports
│   │   ├── traits.rs             # ConfigFormat trait definition
│   │   ├── toml.rs               # TOML format implementation
│   │   ├── json.rs               # JSON format implementation
│   │   ├── yaml.rs               # YAML format implementation
│   │   ├── ini.rs                # INI format implementation
│   │   ├── flatten.rs            # Flattening utilities for all formats
│   │   ├── reconstruct.rs        # Reconstruction utilities for output
│   │   └── detect.rs             # Format auto-detection logic
│   │
│   ├── sources/                  # Input Source System (Non-format inputs)
│   │   ├── mod.rs                # Sources module exports
│   │   ├── env.rs                # Environment variable parsing
│   │   └── cli.rs                # CLI argument parsing
│   │
│   ├── trees/                    # Tree Management for Struct Deserialization
│   │   ├── mod.rs                # Tree module exports
│   │   ├── tree_manager.rs       # Per-profile tree storage and management
│   │   ├── extractor.rs          # Struct deserialization from trees
│   │   └── rebuilder.rs          # Tree rebuilding and synchronization
│   │
│   ├── types/                    # Type Definitions and Utilities
│   │   ├── mod.rs                # Types module exports
│   │   ├── handle_id.rs          # HandleID type and generation
│   │   ├── errors.rs             # Error types and handling
│   │   ├── config_data.rs        # Data type utilities
│   │   └── dynamic_types.rs      # DynDeserialize/DynSerialize types
│   │
│   └── config_flags.rs           # Configuration flags (kept from v2.0)
│
├── tests/                        # External Test Suite
│   ├── integration/              # Integration tests
│   │   ├── basic_operations.rs   # Basic CRUD operations
│   │   ├── toml_loading.rs       # TOML file integration tests
│   │   ├── json_loading.rs       # JSON format tests
│   │   ├── yaml_loading.rs       # YAML format tests
│   │   ├── ini_loading.rs        # INI format tests
│   │   ├── env_loading.rs        # Environment variable tests
│   │   ├── cli_loading.rs        # CLI argument tests
│   │   ├── multi_format.rs       # Mixed format integration tests
│   │   ├── auto_detection.rs     # Format detection tests
│   │   ├── profile_switching.rs  # Profile management tests
│   │   └── concurrent_access.rs  # Thread safety tests
│   │
│   ├── unit/                     # Unit tests per module
│   │   ├── backend_tests.rs      # Backend trait and implementations
│   │   ├── keymap_tests.rs       # Key mapping and flattening
│   │   ├── registry_tests.rs     # Registry operations
│   │   ├── format_tests.rs       # Format parsing and serialization
│   │   ├── source_tests.rs       # Environment and CLI parsing
│   │   ├── tree_tests.rs         # Tree management and extraction
│   │   └── detection_tests.rs    # Auto-detection unit tests
│   │
│   └── performance/              # Performance tests
│       ├── benchmark_compat.rs   # Compatibility with existing benchmarks
│       ├── memory_usage.rs       # Memory usage validation
│       ├── format_perf.rs        # Format parsing/serialization performance
│       └── operation_speed.rs    # Operation timing tests
│
├── examples/                     # Example Applications
│   ├── basic_usage.rs            # Simple key/value operations
│   ├── toml_config.rs            # TOML file configuration
│   ├── json_config.rs            # JSON configuration examples
│   ├── yaml_config.rs            # YAML configuration examples
│   ├── env_cli_config.rs         # Environment and CLI examples
│   ├── multi_format_demo.rs      # Multiple format demonstration
│   ├── profile_demo.rs           # Multi-profile demonstration
│   ├── auto_detection_demo.rs    # Format detection examples
│   └── migration_guide.rs        # Migration from v2.0 to v2.1
│
├── benches/                      # Performance Benchmarks (existing + new)
│   ├── registry_bench.rs         # Updated for new API
│   ├── format_bench.rs           # Format parsing/serialization benchmarks
│   ├── detection_bench.rs        # Auto-detection performance
│   └── ...                       # Other existing benchmarks
│
└── crates/                       # Test Application Crate
    └── superconfig-test/         # Separate crate for real-world testing
        ├── Cargo.toml            # Test crate dependencies
        ├── src/
        │   ├── main.rs           # CLI for testing functionality
        │   └── scenarios/        # Different usage scenarios
        │       ├── basic.rs      # Basic operations
        │       ├── multi_format.rs # Multi-format scenarios
        │       ├── env_cli.rs    # Environment and CLI scenarios
        │       └── concurrent.rs # Concurrent access scenarios
        └── configs/              # Sample files for testing
            ├── app.toml          # Basic TOML configuration
            ├── app.json          # JSON configuration
            ├── app.yaml          # YAML configuration
            ├── app.ini           # INI configuration
            ├── multi-profile.toml # Multi-environment TOML config
            ├── nested.json       # Deep nesting examples
            └── complex.yaml      # Complex YAML structures
```

---

## Implementation Plan

### Phase 0: LogFFI 0.2.0 Universal Architecture Implementation

**Duration**: 2-3 hours\
**Goal**: Implement the revolutionary LogFFI universal logging system with runtime backend switching
**Reference**: Document 24b contains complete implementation instructions

#### Tasks:

1. **Update LogFFI Crate Structure**
   - [ ] update cargo.toml with any needee versions (ensuring latest versions) and
   - [ ] Add universal backend dependencies to `crates/logffi/Cargo.toml`
   - [ ] Add tracing, tracing-subscriber, slog, slog-term, slog-json, paste, thiserror
   - [ ] Update version constraints to latest stable versions
   - [ ] Verify compatibility between all dependencies

2. **Core Universal Backend System**
   - [ ] Implement `Backend` enum (Log, Tracing, Slog) in `crates/logffi/src/lib.rs`
   - [ ] Add `CURRENT_BACKEND` atomic variable for runtime switching
   - [ ] Add `LOGGER_INSTANCE` OnceLock for singleton pattern
   - [ ] Add `FORCE_NATIVE_BACKENDS` atomic flag for dual mode support
   - [ ] Rename `FFI_CALLBACK` to `CALLBACK` for universal naming
   - [ ] Implement `logger()` function (renamed from global)

3. **Backend Management Functions**
   - [ ] Implement `set_backend(backend: Backend)` for runtime switching
   - [ ] Implement `current_backend()` -> Backend for detection
   - [ ] Rename `set_ffi_callback` to `set_callback` for universal usage
   - [ ] Rename `call_ffi_callback` to `call_callback` for consistency
   - [ ] Add environment variable support for backend selection

4. **Universal Macro System**
   - [ ] Create `generate_log_macro!` meta-macro in `crates/logffi/src/macros.rs`
   - [ ] Replace all existing logging macros (error!, warn!, info!, debug!, trace!)
   - [ ] Implement callback detection logic in macros
   - [ ] Add dual-mode support (callback + native backends)
   - [ ] Preserve full backend functionality in macro calls

5. **Enhanced define_errors! Macro**
   - [ ] Create `crates/logffi/src/error_macros.rs`
   - [ ] Implement complete `define_errors!` macro with error codes
   - [ ] Add source error chaining support with std::error::Error
   - [ ] Add automatic LogFFI integration with structured logging
   - [ ] Add FFI-friendly error mapping (kind() method)
   - [ ] Add constructor methods (new_variant_name pattern)

6. **Backend Implementations**
   - [ ] Create backend wrappers (TracingBackend, LogBackend, SlogBackend)
   - [ ] Implement Deref pattern for full API access without functionality loss
   - [ ] Add auto-initialization with environment variable detection
   - [ ] Add smart defaults (tracing backend, text format)
   - [ ] Implement LOGFFI_BACKEND, LOGFFI_FORMAT, LOGFFI_FORCE_NATIVE support

7. **Environment Variable Integration**
   - [ ] Add `LOGFFI_BACKEND=tracing|log|slog` support (default: tracing)
   - [ ] Add `LOGFFI_FORMAT=text|json|compact` support (default: text)
   - [ ] Add `LOGFFI_FORCE_NATIVE=true|false` support (default: false)
   - [ ] Maintain compatibility with `RUST_LOG` standard
   - [ ] Add auto-initialization on first macro use

8. **Testing & Validation**
   - [ ] Update existing logffi tests for new universal architecture
   - [ ] Add tests for runtime backend switching
   - [ ] Add tests for callback mode detection
   - [ ] Add tests for dual-mode functionality (callback + native)
   - [ ] Add tests for environment variable configuration
   - [ ] Verify all macro variations work correctly

**Verification Steps:**

- [ ] LogFFI compiles with all new dependencies
- [ ] Runtime backend switching works (log ↔ tracing ↔ slog)
- [ ] Callback mode detection works correctly
- [ ] Dual-mode logging works (callback + native simultaneously)
- [ ] All macros (error!, warn!, info!, debug!, trace!) preserve full functionality
- [ ] Environment variable configuration works
- [ ] define_errors! macro generates complete error types
- [ ] FFI error mapping works for cross-language consistency

**Why This Phase is Critical:**

- LogFFI becomes the universal Rust logging standard that SuperConfig will use
- Provides runtime backend switching (unique in Rust ecosystem)
- Enables universal FFI bridging for Python/Node.js integration
- Zero functionality loss via Deref pattern
- Revolutionary callback mode for custom routing
- Makes SuperConfig's error handling system enterprise-ready

---

### Phase 1: Core Architecture & Backend System

**Duration**: 1-2 hours\
**Goal**: Implement swappable backend system and basic registry with enhanced types

#### Tasks:

1. **Create new core architecture**
   - [ ] Clear current `src/core/` contents
   - [ ] Create new file structure as defined above
   - [ ] Implement enhanced `Profile` type in `src/core/profile.rs`
   - [ ] Set up module structure and exports

2. **Enhanced Type System**
   - [ ] Create `DynDeserialize`/`DynSerialize` types in `src/types/dynamic_types.rs`
   - [ ] Implement conversions for all basic types (String, i64, f64, bool)
   - [ ] Add support for nested structures
   - [ ] Type-safe error handling

3. **Backend Infrastructure**
   - [ ] Define enhanced `ConfigRegistryBackend` trait in `src/backend/traits.rs`
   - [ ] Implement `SccBackend` with DataMap in `src/backend/scc_backend.rs`
   - [ ] Add support for dirty tracking
   - [ ] Enhanced error handling for backend operations

4. **Registry Foundation**
   - [ ] Implement enhanced `ConfigRegistry` in `src/core/registry.rs`
   - [ ] Add `dirty_profiles` tracking system
   - [ ] Integration with backend system
   - [ ] Thread safety setup

**Verification Steps:**

- [ ] Clean compile with new file structure
- [ ] Basic registry creation works
- [ ] Profile creation and selection works
- [ ] Backend trait can be implemented
- [ ] Dynamic type conversions work

---

### Phase 2: Multi-Format System Implementation

**Duration**: 2-3 hours\
**Goal**: Implement comprehensive format support with auto-detection

#### Tasks:

1. **ConfigFormat Trait System**
   - [ ] Define `ConfigFormat` trait in `src/formats/traits.rs`
   - [ ] Specify `parse()` and `serialize()` methods
   - [ ] Error handling standardization
   - [ ] Performance optimization hooks

2. **Format Implementations**
   - [ ] Implement `TomlFormat` in `src/formats/toml.rs`
   - [ ] Implement `JsonFormat` in `src/formats/json.rs`
   - [ ] Implement `YamlFormat` in `src/formats/yaml.rs`
   - [ ] Implement `IniFormat` in `src/formats/ini.rs`

3. **Flattening & Reconstruction**
   - [ ] Universal flattening utilities in `src/formats/flatten.rs`
   - [ ] Universal reconstruction utilities in `src/formats/reconstruct.rs`
   - [ ] Support for nested structures across all formats
   - [ ] Profile-aware key generation

4. **Auto-Detection System**
   - [ ] Implement format detection in `src/formats/detect.rs`
   - [ ] File extension-based detection
   - [ ] Content heuristic analysis
   - [ ] Parse attempt fallback system
   - [ ] Caching for performance

**Verification Steps:**

- [ ] All formats can parse and serialize correctly
- [ ] Flattening works consistently across formats
- [ ] Auto-detection accurately identifies formats
- [ ] Performance meets targets (~20-150μs parsing)
- [ ] Error handling is robust

---

### Phase 3: Sources System Implementation

**Duration**: 1-2 hours\
**Goal**: Environment variable and CLI argument integration

#### Tasks:

1. **Environment Variable Support**
   - [ ] Implement env parsing in `src/sources/env.rs`
   - [ ] Prefix-based filtering (`APP_`, `CONFIG_`, etc.)
   - [ ] Key name conversion (`APP_DB_HOST` → `db.host`)
   - [ ] Type inference and conversion
   - [ ] Profile detection from env vars

2. **CLI Argument Support**
   - [ ] Implement CLI parsing in `src/sources/cli.rs`
   - [ ] Support for `--key.path=value` format
   - [ ] Support for `--key.path value` format
   - [ ] Boolean flag handling
   - [ ] Profile-specific arguments

3. **Sources Integration**
   - [ ] Unified interface for all sources
   - [ ] Integration with main registry
   - [ ] Consistent error handling
   - [ ] Performance optimization

**Verification Steps:**

- [ ] Environment variables parse correctly
- [ ] CLI arguments parse correctly
- [ ] Key conversion works as expected
- [ ] Integration with registry works
- [ ] Performance within targets (~10-30μs)

---

### Phase 4: Tree Management & Enhanced Registry

**Duration**: 1-2 hours\
**Goal**: Struct deserialization and tree synchronization

#### Tasks:

1. **Tree Management System**
   - [ ] Implement tree manager in `src/trees/tree_manager.rs`
   - [ ] Per-profile tree storage
   - [ ] Tree synchronization with DataMap
   - [ ] Memory optimization

2. **Struct Extraction System**
   - [ ] Implement extractor in `src/trees/extractor.rs`
   - [ ] Generic struct deserialization
   - [ ] Type-safe extraction
   - [ ] Error handling for invalid structures

3. **Tree Rebuilding**
   - [ ] Implement rebuilder in `src/trees/rebuilder.rs`
   - [ ] Automatic tree reconstruction
   - [ ] Dirty tracking integration
   - [ ] Performance optimization

4. **Enhanced Registry Operations**
   - [ ] Integrate all systems in registry
   - [ ] `merge_string()`, `merge_file()`, `merge_env()`, `merge_cli()` methods
   - [ ] `to_format()` method for output
   - [ ] Profile switching with tree management

**Verification Steps:**

- [ ] Trees stay synchronized with DataMap
- [ ] Struct deserialization works correctly
- [ ] Dirty tracking triggers rebuilds appropriately
- [ ] All merge methods work correctly
- [ ] Output generation works for all formats

---

### Phase 5: SuperConfig API & Integration

**Duration**: 30-60 minutes\
**Goal**: Complete public API matching enhanced Grok3 design

#### Tasks:

1. **SuperConfig Main API**
   - [ ] Implement enhanced `SuperConfig` in `src/api/superconfig.rs`
   - [ ] All merge methods: `merge_file()`, `merge_string()`, `merge_env()`, `merge_cli()`
   - [ ] Output methods: `to_format()`
   - [ ] Enhanced error handling and user feedback

2. **Handle System Integration**
   - [ ] Update `ConfigHandle` in `src/core/handle.rs`
   - [ ] Integration with new DataMap system
   - [ ] Handle lifecycle management
   - [ ] Backward compatibility with v2.0

3. **API Finalization**
   - [ ] Update `src/lib.rs` with all exports
   - [ ] Global registry integration
   - [ ] Documentation updates
   - [ ] Example code verification

**Verification Steps:**

- [ ] Full enhanced Grok3 API examples work
- [ ] All input methods work correctly
- [ ] Output methods work for all formats
- [ ] Handle compatibility maintained
- [ ] Global registry accessible

---

### Phase 6: Testing & Benchmarking

**Duration**: 2-3 hours\
**Goal**: Comprehensive test suite and performance validation

#### Tasks:

1. **Test Suite Creation**
   - [ ] Create comprehensive test structure in `tests/`
   - [ ] Integration tests for all formats
   - [ ] Source integration tests (env, CLI)
   - [ ] Auto-detection tests
   - [ ] Multi-format interaction tests
   - [ ] Performance regression tests

2. **Benchmark Updates**
   - [ ] Update existing benchmarks in `benches/`
   - [ ] Add format-specific benchmarks
   - [ ] Source parsing benchmarks
   - [ ] Auto-detection performance benchmarks
   - [ ] Memory usage validation
   - [ ] Performance comparison with baseline

3. **Example Applications**
   - [ ] Create `crates/superconfig-test` crate
   - [ ] Real-world multi-format scenarios
   - [ ] Sample configuration files for all formats
   - [ ] CLI for interactive testing
   - [ ] Performance demonstration

**Verification Steps:**

- [ ] All tests pass
- [ ] Performance targets met (~20-150μs parsing, ~48-54μs operations)
- [ ] Memory usage within targets (~82-93KB)
- [ ] Auto-detection accuracy > 99%
- [ ] Examples demonstrate all functionality
- [ ] No performance regressions

---

### Phase 7: Documentation & Finalization

**Duration**: 30-60 minutes\
**Goal**: Complete documentation and final polish

#### Tasks:

1. **Documentation Updates**
   - [ ] Update `src/lib.rs` with comprehensive documentation
   - [ ] API documentation for all public methods
   - [ ] Format-specific usage examples
   - [ ] Auto-detection guidance
   - [ ] Migration guide from v2.0
   - [ ] Performance characteristics documentation

2. **Final Integration**
   - [ ] Integration with existing build system
   - [ ] Version bumping to v2.1
   - [ ] Changelog updates
   - [ ] Final code cleanup and optimization
   - [ ] Lint and format checks

**Verification Steps:**

- [ ] All documentation builds correctly
- [ ] Examples in docs work
- [ ] Clean compile with all warnings addressed
- [ ] Ready for release

---

## Todo List & Progress Tracking

### Current Status: Planning Phase

- [x] Analyze current codebase and Grok3 requirements
- [x] Incorporate multi-format requirements from 23a
- [ ] **PHASE 1**: Core Architecture & Backend System (1-2 hours)
- [ ] **PHASE 2**: Multi-Format System Implementation (2-3 hours)
- [ ] **PHASE 3**: Sources System Implementation (1-2 hours)
- [ ] **PHASE 4**: Tree Management & Enhanced Registry (1-2 hours)
- [ ] **PHASE 5**: SuperConfig API & Integration (30-60 min)
- [ ] **PHASE 6**: Testing & Benchmarking (2-3 hours)
- [ ] **PHASE 7**: Documentation & Finalization (30-60 min)

**Total Estimated Time**: 8-12 hours for complete implementation

### Working Pattern

1. **Get approval** for this plan from user
2. **Implement one phase at a time**
3. **Test each phase** before proceeding
4. **Show progress** and get approval for next phase
5. **Ask for guidance** when encountering issues

---

## Technical Specifications

### Core Components

#### Enhanced DataMap Architecture

```rust
// Enhanced backend trait for multi-format support
pub trait ConfigRegistryBackend: Send + Sync {
    fn insert<T: 'static + Send + Sync>(&self, id: u64, data: T) -> Result<(), ()>;
    fn get<T: 'static + Clone>(&self, id: &u64) -> Option<T>;
    fn remove(&self, id: &u64) -> Option<Arc<dyn std::any::Any + Send + Sync>>;
}

// Dynamic type system for multi-format support
#[derive(Clone)]
pub enum DynDeserialize {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    // Future: Arrays, Tables, etc.
}

// ConfigFormat trait for extensible format support
pub trait ConfigFormat: Send + Sync {
    fn parse(&self, content: &str) -> Result<HashMap<String, DynDeserialize>, String>;
    fn serialize(&self, data: &HashMap<String, DynSerialize>) -> Result<String, String>;
    fn format_name(&self) -> &'static str;
}

// Enhanced registry with multi-format support
pub struct ConfigRegistry {
    backend: Arc<dyn ConfigRegistryBackend>,                      // DataMap layer
    keymaps: Arc<SccHashMap<Profile, Arc<SccHashMap<String, u64>>>>, // Key→HandleID
    trees: Arc<SccHashMap<Profile, toml::Value>>,                // TOML trees
    dirty_profiles: Arc<SccHashMap<Profile, bool>>,              // Rebuild tracking
    selected_profile: Profile,                                    // Current profile
}
```

### Dependencies Required

**Note**: See [25-crate-research-findings.md](./25-crate-research-findings.md) for detailed analysis of technology choices.

```toml
[dependencies]
# Core dependencies
scc = "2.0" # High-performance concurrent collections
serde = { version = "1.0", features = ["derive"] }

# Format support (updated based on research)
toml = "0.8" # TOML parsing and serialization
serde_json = "1.0" # JSON format support
serde-yaml-bw = "0.3" # YAML support with security hardening (YAML 1.1)
ini = "1.3" # INI format support

# CLI parsing (optional)
clap = { version = "4.0", optional = true }

# Note: Using std::sync::LazyLock instead of once_cell (Rust 1.80+ native)
```

**Key Technology Decisions**:

- **YAML Library**: `serde-yaml-bw` chosen over YAML 1.2 alternatives for superior serde integration and security features
- **Lazy Initialization**: `std::sync::LazyLock` replaces `once_cell` (native in Rust 1.80+)
- **Security Focus**: Enhanced security with panic-free YAML parsing and attack protection

### Performance Targets (from Enhanced Grok3 design)

- **Format Parsing**: JSON (~20-50μs), TOML (~50-100μs), YAML (~50-150μs)
- **Auto-Detection**: ~0.1-0.5μs (heuristics) + ~20-150μs (parse attempt)
- **Environment Variables**: ~10-30μs for 100 vars
- **CLI Arguments**: ~10-30μs for 100 args
- **Core Operations**: ~48-54μs per operation (unchanged)
- **FFI Operations**: ~51-56μs per operation
- **Memory Usage**: ~82-93KB for 100 configs + ~20-30KB per profile
- **Throughput**: ~25,000-130,000 ops/sec

### API Compatibility

- Full backward compatibility with handle-based access
- Enhanced key/value API as primary interface
- Multi-format input support with auto-detection
- Profile-based configuration management
- Comprehensive output format support
- Environment variable and CLI integration

---

## Success Criteria

### Functional Requirements

- [x] Nested key access: `config.get("storage.db.host")`
- [x] Profile switching: `config.select("staging")`
- [x] Multi-format loading: `config.merge_file("app.toml")`, `config.merge_string(json)`
- [x] Environment variables: `config.merge_env("APP_")`
- [x] CLI arguments: `config.merge_cli(&args)`
- [x] Format auto-detection: automatic format detection for files and strings
- [x] Multi-format output: `config.to_format("profile", &YamlFormat)`
- [x] Struct deserialization: `config.extract::<Config>()`
- [x] Handle compatibility: existing handle code works

### Performance Requirements

- [x] Parsing within targets: JSON ~20-50μs, TOML ~50-100μs, YAML ~50-150μs
- [x] Core operations within ~48-54μs target
- [x] Memory usage within ~82-93KB target
- [x] Auto-detection accuracy > 99%
- [x] No significant performance regression
- [x] Thread safety maintained

### Quality Requirements

- [x] Comprehensive test coverage for all formats
- [x] Clean, maintainable code with proper separation of concerns
- [x] Good documentation with examples for all features
- [x] Example applications demonstrating real-world usage
- [x] Migration path from v2.0 clearly documented

---

This enhanced plan provides a systematic approach to implementing the complete Grok3 multi-format design while maintaining the performance and reliability characteristics of SuperConfig. The clear file structure ensures maintainable code organization, and each phase builds on the previous one with verification steps to ensure quality and correctness. The multi-format support, auto-detection, and sources integration make SuperConfig a comprehensive configuration management solution.
