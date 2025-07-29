# Phase 3: Complex Type Handling

**Status**: ⏳ PENDING\
**Estimated Duration**: 3-4 hours\
**Dependencies**: Phase 2 Complete

## Overview

Phase 3 extends the SuperConfig FFI wrapper with JSON interfaces for SuperConfig's complex Rust types. This phase focuses on converting SuperConfig-specific types (Wildcard, SearchStrategy, etc.) into simple JSON parameters that SuperFFI can easily handle across all target languages.

## Architecture Clarification

### **Separation of Concerns**

**SuperFFI's Role** (Generic):

- Takes any Rust function with any parameter types
- Automatically generates FFI bindings for Python/Node.js/WASM
- Handles JSON boundary crossing generically
- No knowledge of SuperConfig domain types

**SuperConfig-FFI's Role** (Domain-Specific):

- Converts SuperConfig's complex Rust types to simple JSON interfaces
- Handles SuperConfig-specific JSON-to-Rust-type conversion
- Provides SuperConfig domain knowledge and validation
- Exposes JSON-based APIs that SuperFFI can easily bind

### **Data Flow**

```
User (Python/JS) → JSON → SuperFFI → SuperConfig-FFI → SuperConfig (Rust types)
                                    ↑
                              JSON parsing &
                            domain validation
```

**Example**:

```rust
// SuperConfig original API:
fn with_wildcard(&self, wildcard: Wildcard) -> Result<Self, Error>

// SuperConfig-FFI converts to JSON interface:
#[superffi]
pub fn with_wildcard(&self, wildcard_json: serde_json::Value) -> Result<Self, String> {
    let wildcard = parse_wildcard_from_json(wildcard_json)?;  // Domain-specific conversion
    self.inner.with_wildcard(wildcard)                       // Call original method
        .map(|inner| Self { inner })
        .map_err(|e| e.to_string())
}

// SuperFFI automatically generates:
// Python: def with_wildcard(self, wildcard_json: dict) -> SuperConfig
// Node.js: withWildcard(wildcardJson: object): SuperConfig
// WASM: with_wildcard(wildcard_json: JsValue): SuperConfig
```

## Deliverables

### 🎯 **Core Objectives**

1. **JSON interfaces for SuperConfig complex types** - Convert Wildcard, SearchStrategy, MergeOrder to JSON
2. **SuperConfig-specific validation** - Domain knowledge for proper JSON schema validation
3. **Figment method exposure** - JSON interfaces for debugging and introspection
4. **Comprehensive error handling** - SuperConfig domain-aware error messages

### 📋 **Implementation Tasks**

#### Task 1: Wildcard Provider JSON Interface (1-2 hours)

**SuperConfig-Specific JSON Schema**:

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

**SuperConfig-FFI Implementation**:

```rust
#[superffi]
impl SuperConfig {
    /// Configure wildcard file discovery - SuperConfig domain-specific JSON interface
    pub fn with_wildcard(&self, config: serde_json::Value) -> Result<Self, String> {
        // SuperConfig-specific JSON parsing
        let wildcard = parse_superconfig_wildcard_json(config)?;
        
        // Call original SuperConfig method
        self.inner.with_wildcard(wildcard)
            .map(|inner| Self { inner })
            .map_err(|e| format!("SuperConfig wildcard error: {}", e))
    }
}

// SuperConfig domain-specific conversion functions
fn parse_superconfig_wildcard_json(config: serde_json::Value) -> Result<Wildcard, String> {
    let pattern = config["pattern"].as_str()
        .ok_or("SuperConfig wildcard requires 'pattern' field")?;
        
    let search_strategy = parse_superconfig_search_strategy(&config["search"])?;
    let merge_order = parse_superconfig_merge_order(&config["merge_order"])?;
    
    Ok(Wildcard::from_pattern(pattern)
        .with_search_strategy(search_strategy)
        .with_merge_order(merge_order))
}

fn parse_superconfig_search_strategy(config: &serde_json::Value) -> Result<SearchStrategy, String> {
    match config["type"].as_str().unwrap_or("current") {
        "current" => Ok(SearchStrategy::Current),
        
        "recursive" => {
            let root = config["root"].as_str().unwrap_or(".");
            let max_depth = config["max_depth"].as_u64().map(|d| d as usize);
            
            Ok(SearchStrategy::Recursive {
                roots: vec![PathBuf::from(root)],
                max_depth,
            })
        },
        
        "directories" => {
            let dirs_array = config["directories"].as_array()
                .ok_or("SuperConfig search strategy 'directories' requires 'directories' array")?;
                
            let dirs: Result<Vec<_>, _> = dirs_array
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    v.as_str()
                        .map(PathBuf::from)
                        .ok_or_else(|| format!("SuperConfig directory at index {} must be a string", i))
                })
                .collect();
                
            Ok(SearchStrategy::Directories(dirs?))
        },
        
        invalid => Err(format!("SuperConfig invalid search strategy: '{}'. Valid: current, recursive, directories", invalid))
    }
}
```

#### Task 2: Other SuperConfig Complex Types (1 hour)

**Additional SuperConfig JSON Interfaces**:

```rust
#[superffi]
impl SuperConfig {
    /// Configure custom provider - SuperConfig-specific JSON interface  
    pub fn with_provider(&self, provider_config: serde_json::Value) -> Result<Self, String> {
        let provider = parse_superconfig_provider_json(provider_config)?;
        self.inner.with_provider(provider)
            .map(|inner| Self { inner })
            .map_err(|e| format!("SuperConfig provider error: {}", e))
    }
    
    /// Configure merge strategy - SuperConfig-specific JSON interface
    pub fn with_merge_strategy(&self, strategy_config: serde_json::Value) -> Result<Self, String> {
        let strategy = parse_superconfig_merge_strategy_json(strategy_config)?;
        self.inner.with_merge_strategy(strategy)
            .map(|inner| Self { inner })
            .map_err(|e| format!("SuperConfig merge strategy error: {}", e))
    }
}
```

#### Task 3: SuperConfig Figment Introspection (1-2 hours)

**SuperConfig-Specific Debugging APIs**:

```rust
#[superffi]
impl SuperConfig {
    /// Extract SuperConfig state as JSON for debugging
    pub fn extract_json(&self) -> Result<serde_json::Value, String> {
        let figment_value = self.inner.extract::<figment::value::Value>()
            .map_err(|e| format!("SuperConfig extraction failed: {}", e))?;
        convert_superconfig_figment_to_json(figment_value)
    }
    
    /// Find SuperConfig value by path
    pub fn find(&self, path: String) -> Result<Option<serde_json::Value>, String> {
        match self.inner.find_ref(&path) {
            Some(figment_value) => {
                let json_value = convert_superconfig_figment_to_json(figment_value.clone())?;
                Ok(Some(json_value))
            },
            None => Ok(None)
        }
    }
    
    /// Get SuperConfig metadata and source information
    pub fn get_metadata(&self) -> Result<serde_json::Value, String> {
        let metadata = self.inner.metadata();
        let sources: Vec<serde_json::Value> = metadata
            .iter()
            .enumerate()
            .map(|(priority, meta)| {
                json!({
                    "priority": priority,
                    "name": meta.name,
                    "source": format!("{:?}", meta.source),
                    "interpolated": meta.interpolated,
                })
            })
            .collect();
            
        Ok(json!({
            "sources": sources,
            "total_sources": sources.len(),
            "superconfig_version": env!("CARGO_PKG_VERSION")
        }))
    }
}

// SuperConfig-specific Figment conversion
fn convert_superconfig_figment_to_json(figment_value: figment::value::Value) -> Result<serde_json::Value, String> {
    use figment::value::Value as FV;
    
    match figment_value {
        FV::Bool(_, b) => Ok(serde_json::Value::Bool(b)),
        FV::Num(_, n) => {
            if let Ok(i) = n.to_i64() {
                Ok(serde_json::Value::Number(serde_json::Number::from(i)))
            } else if let Ok(f) = n.to_f64() {
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .ok_or_else(|| format!("SuperConfig invalid number: {}", f))
            } else {
                Err(format!("SuperConfig unsupported number type: {:?}", n))
            }
        },
        FV::String(_, s) => Ok(serde_json::Value::String(s)),
        FV::Dict(_, dict) => {
            let mut map = serde_json::Map::new();
            for (key, value) in dict {
                let json_value = convert_superconfig_figment_to_json(value)?;
                map.insert(key.to_string(), json_value);
            }
            Ok(serde_json::Value::Object(map))
        },
        FV::Array(_, array) => {
            let json_array: Result<Vec<_>, _> = array
                .into_iter()
                .map(convert_superconfig_figment_to_json)
                .collect();
            Ok(serde_json::Value::Array(json_array?))
        },
    }
}
```

## SuperConfig Domain Knowledge

### **Why SuperConfig-Specific JSON Schemas**

1. **Domain Validation**: SuperConfig knows which combinations of search strategies and merge orders are valid
2. **Error Messages**: Can provide SuperConfig-specific error context and suggestions
3. **Default Values**: Understands SuperConfig's default behaviors and conventions
4. **Type Safety**: Validates JSON matches SuperConfig's expected Rust type constraints

### **SuperConfig JSON Schema Standards**

**Search Strategy Types**:

- `"current"` - Search only current directory
- `"recursive"` - Recursive search with optional max_depth and root(s)
- `"directories"` - Search specific directories only

**Merge Order Types**:

- `"alphabetical"` - Standard alphabetical ordering
- `"reverse_alphabetical"` - Reverse alphabetical ordering
- `"custom"` - User-defined pattern ordering
- `"modification_time"` - Order by file modification time

**Validation Rules**:

- `pattern` is always required for wildcard configuration
- `recursive` type requires valid directory paths
- `custom` merge order requires `patterns` array
- All paths must be valid UTF-8 strings

## Testing Strategy

### **SuperConfig-Specific JSON Validation**

```rust
#[test]
fn test_superconfig_wildcard_basic() {
    let config = SuperConfig::new();
    let wildcard_config = json!({
        "pattern": "*.toml"  // Minimal SuperConfig wildcard
    });
    let result = config.with_wildcard(wildcard_config);
    assert!(result.is_ok());
}

#[test]
fn test_superconfig_search_strategies() {
    let config = SuperConfig::new();
    
    // Test SuperConfig recursive search
    let recursive_config = json!({
        "pattern": "**/*.json",
        "search": {
            "type": "recursive",
            "root": "./config",
            "max_depth": 3
        }
    });
    assert!(config.with_wildcard(recursive_config).is_ok());
    
    // Test SuperConfig directory search  
    let directory_config = json!({
        "pattern": "*.toml",
        "search": {
            "type": "directories",
            "directories": ["./config", "./settings"]
        }
    });
    assert!(config.with_wildcard(directory_config).is_ok());
}

#[test]
fn test_superconfig_validation_errors() {
    let config = SuperConfig::new();
    
    // SuperConfig-specific error: missing pattern
    let invalid_config = json!({
        "search": {"type": "current"}
    });
    let error = config.with_wildcard(invalid_config).unwrap_err();
    assert!(error.contains("SuperConfig wildcard requires 'pattern'"));
    
    // SuperConfig-specific error: invalid search type
    let invalid_config = json!({
        "pattern": "*.toml",
        "search": {"type": "invalid_superconfig_type"}
    });
    let error = config.with_wildcard(invalid_config).unwrap_err();
    assert!(error.contains("SuperConfig invalid search strategy"));
}
```

### **SuperConfig Figment Integration**

```rust
#[test]
fn test_superconfig_extract_json() {
    let config = SuperConfig::new()
        .with_file("test.toml".to_string())
        .unwrap();
        
    let json_result = config.extract_json();
    assert!(json_result.is_ok());
    
    let json_value = json_result.unwrap();
    assert!(json_value.is_object());
}

#[test]
fn test_superconfig_metadata() {
    let config = SuperConfig::new()
        .with_file("test.toml".to_string())
        .unwrap();
        
    let metadata = config.get_metadata().unwrap();
    assert!(metadata["sources"].is_array());
    assert!(metadata["superconfig_version"].is_string());
}
```

## Success Metrics

### **Completion Criteria**

- [ ] All SuperConfig complex types have JSON interfaces
- [ ] SuperConfig domain validation provides meaningful error messages
- [ ] Figment introspection methods work through JSON interface
- [ ] JSON schemas documented for SuperConfig API consumers

### **Quality Targets**

- [ ] Error messages include "SuperConfig" context for clarity
- [ ] JSON validation catches SuperConfig-specific invalid configurations
- [ ] Performance suitable for typical SuperConfig usage patterns
- [ ] Type safety maintained between JSON and SuperConfig Rust types

### **Integration Readiness**

- [ ] SuperFFI can automatically bind all JSON-based methods
- [ ] Ready for Phase 4 build system and packaging
- [ ] API suitable for real-world SuperConfig usage

## Architecture Benefits

### **Clear Separation**

- **SuperFFI**: Generic FFI binding generation (reusable)
- **SuperConfig-FFI**: Domain-specific JSON conversion (SuperConfig-specific)

### **Maintainability**

- SuperConfig changes only affect SuperConfig-FFI layer
- SuperFFI remains generic and reusable for other projects
- Clear boundaries between generic and domain-specific code

### **User Experience**

- Language users get clean JSON interfaces
- SuperConfig domain knowledge preserved in error messages
- Type safety maintained throughout the conversion chain

---

_Builds on Phase 2 foundations with corrected architecture understanding. See [`architecture.md`](./architecture.md) for complete separation of concerns._
