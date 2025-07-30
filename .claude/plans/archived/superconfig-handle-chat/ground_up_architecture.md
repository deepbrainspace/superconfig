# SuperConfig V2: Ground-Up Rewrite Architecture Plan

## Executive Summary

This document outlines a complete ground-up rewrite of SuperConfig optimized for both native Rust performance and FFI from day one, using a handle-based registry architecture. All timelines reflect AI-assisted development with Claude Code/Sonnet 4.

## 🏗️ **Multi-Crate Architecture (Final Design)**

Based on analysis of your existing codebase and the complete SuperFFI implementation, here's the refined 3-crate architecture that maximizes the value of your SuperFFI investment:

**Architecture Strengths:**

- **Pure Core Performance**: superconfig crate has zero FFI dependencies for maximum performance
- **Universal FFI Interface**: superconfig-ffi uses #[superffi] macros for one universal binding approach
- **Complete FFI Encapsulation**: superffi contains all FFI dependencies (pyo3, napi, wasm-bindgen)
- **Clean Separation**: Each crate has a single, focused responsibility
- **Consistent APIs**: SuperFFI ensures natural naming conventions across all languages

**Key Clarification - Bindings Location:**

- **SuperFFI crate**: Contains ONLY the procedural macro implementation - no generated bindings
- **SuperConfig-FFI crate**: Contains the actual generated language packages in its `bindings/` directory
- **Build Process**: SuperFFI macro expands at compile time, generating FFI code within superconfig-ffi
- **Output**: Language packages (Python wheel, NPM package, WASM bundle) are built in `superconfig-ffi/bindings/`

```
superconfig/                    # Workspace root
├── Cargo.toml                 # Workspace configuration
├── crates/
│   ├── superconfig/           # Pure Rust core (NO FFI dependencies)
│   │   ├── Cargo.toml         # Only pure Rust dependencies
│   │   └── src/
│   │       ├── lib.rs         # Public Rust API
│   │       ├── core/          # Core engine (handle-based)
│   │       │   ├── mod.rs
│   │       │   ├── registry.rs    # Lock-free handle registry
│   │       │   ├── config.rs      # Zero-copy configuration data
│   │       │   ├── loader.rs      # SIMD-optimized file loading
│   │       │   ├── merge.rs       # Advanced array merging engine
│   │       │   └── hotreload.rs   # Tokio-based hot reload system
│   │       ├── builder.rs     # Fluent builder API
│   │       ├── extract.rs     # Type-safe extraction
│   │       └── error.rs       # Rich error types with source chains
│   ├── superconfig-ffi/       # FFI wrapper (uses #[superffi] macros)
│   │   ├── Cargo.toml         # Depends ONLY on superconfig + superffi
│   │   ├── src/
│   │   │   ├── lib.rs         # SuperFFI-annotated wrapper types
│   │   │   ├── fluent.rs      # FFI-friendly fluent API methods
│   │   │   ├── access.rs      # Configuration access methods
│   │   │   └── verbosity.rs   # Verbosity/debugging support
│   │   └── bindings/          # Generated language packages (OUTPUT from SuperFFI)
│   │       ├── python/        # Python wheel (via maturin + SuperFFI python feature)
│   │       ├── nodejs/        # NPM package (via napi-rs + SuperFFI nodejs feature)
│   │       └── wasm/          # WASM package (via wasm-pack + SuperFFI wasm feature)
│   └── superffi/              # Procedural macro bridge (NO generated bindings here)
│       ├── Cargo.toml         # Contains ALL FFI deps (pyo3, napi, wasm-bindgen)
│       └── src/
│           ├── lib.rs         # #[superffi] macro implementation only
│           └── tests.rs       # Macro transformation tests
└── benchmarks/                # Performance validation suite
```

**Crate Dependencies & Data Flow:**

```
┌─────────────────┐    depends on    ┌─────────────────┐    uses macro    ┌─────────────────┐
│   superconfig   │ ←──────────────── │ superconfig-ffi │ ──────────────→ │    superffi     │
│ (pure Rust)     │                  │(#[superffi] only)│                  │(macro bridge)   │
│ NO FFI deps     │                  │                 │                  │pyo3,napi,wasm   │
└─────────────────┘                  └─────────────────┘                  └─────────────────┘
        ↑                                      ↓                                   ↓
        │                            ┌─────────────────┐                ┌─────────────────┐
        │                            │  Native Rust    │                │  Macro expands  │
        └──── High Performance ──────│     Users       │                │  to FFI code    │
               Handle Registry        └─────────────────┘                └─────────────────┘
                                                ↓                                   ↓
                                      ┌─────────────────┐                ┌─────────────────┐
                                      │ superconfig-ffi │                │   Generated     │
                                      │   /bindings/    │ ←──────────── │   Bindings      │
                                      │ Py/JS/WASM pkgs │                │(in ffi crate)   │
                                      └─────────────────┘                └─────────────────┘
```

**Dependency Isolation Strategy:**

```toml
# superconfig/Cargo.toml (Pure Rust Core - NO FFI)
[dependencies]
# Core performance dependencies
dashmap = "5.5"
parking_lot = "0.12"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
memmap2 = "0.9" # Memory-mapped file loading
dirs = "5.0" # Hierarchical config discovery

# Optional performance dependencies
simd-json = { version = "0.13", optional = true }
tokio = { version = "1.0", optional = true, features = ["fs", "time", "rt"] }
notify = { version = "6.0", optional = true }
tracing = { version = "0.1", optional = true }
serde_yaml = { version = "0.9", optional = true }
toml = { version = "0.8", optional = true }

[features]
default = []
# Performance optimizations
simd = ["simd-json", "std-simd"] # Both simd-json crate + std::simd API
hot-reload = ["notify", "tokio"] # Tokio-based file watching
profiling = ["tracing"] # Performance profiling
serde-formats = ["serde_yaml", "toml"] # Additional format support

# superconfig-ffi/Cargo.toml (FFI Wrapper - Uses SuperFFI Only)
[dependencies]
superconfig = { path = "../superconfig" }
superffi = { path = "../superffi" }
# NO direct FFI dependencies - superffi handles everything

[features]
default = ["python", "nodejs", "wasm"]
python = ["superffi/python"] # Forwards to superffi
nodejs = ["superffi/nodejs"] # Forwards to superffi
wasm = ["superffi/wasm"] # Forwards to superffi
hot-reload = ["superconfig/hot-reload"]
all = ["python", "nodejs", "wasm", "hot-reload"]

# superffi/Cargo.toml (Contains ALL FFI Dependencies)
[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full"] }

# Feature-gated FFI dependencies
pyo3 = { version = "0.25", optional = true }
napi = { version = "2.16", optional = true }
napi-derive = { version = "2.16", optional = true }
wasm-bindgen = { version = "0.2", optional = true }
js-sys = { version = "0.3", optional = true }

[features]
default = ["python", "nodejs", "wasm"]
python = ["pyo3"]
nodejs = ["napi", "napi-derive"]
wasm = ["wasm-bindgen", "js-sys"]
all = ["python", "nodejs", "wasm"]
```

## 🎯 **Complete Feature List & Implementation**

### **Core Features (Always Available)**

#### 1. **Handle Registry System**

**Implementation Time**: 3-4 hours

```rust
// src/core/registry.rs
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub type HandleId = u64;

pub struct ConfigRegistry {
    configs: DashMap<HandleId, Arc<ConfigEntry>>,
    next_id: AtomicU64,
    #[cfg(feature = "hot-reload")]
    watchers: DashMap<HandleId, notify::RecommendedWatcher>,
}

impl ConfigRegistry {
    pub fn create(&self, data: ConfigData) -> HandleId {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.configs.insert(id, Arc::new(ConfigEntry::new(data)));
        id
    }
    
    pub fn get(&self, id: HandleId) -> Option<Arc<ConfigEntry>> {
        self.configs.get(&id).map(|entry| entry.clone())
    }
    
    pub fn update(&self, id: HandleId, data: ConfigData) -> bool {
        if let Some(mut entry) = self.configs.get_mut(&id) {
            *entry = Arc::new(ConfigEntry::new(data));
            true
        } else {
            false
        }
    }
    
    pub fn remove(&self, id: HandleId) -> bool {
        self.configs.remove(&id).is_some()
    }
}
```

#### 2. **Zero-Copy Configuration Data**

**Implementation Time**: 2-3 hours

```rust
// src/core/config.rs
use serde_json::Value;
use std::collections::HashMap;

pub struct ConfigData {
    // Primary JSON representation for fast access
    json: Value,
    
    // Cached extractions for performance
    cache: parking_lot::RwLock<HashMap<String, CachedValue>>,
    
    // Source tracking for debugging
    sources: Vec<ConfigSource>,
    
    // Metadata
    created_at: std::time::Instant,
}

#[derive(Clone)]
pub enum CachedValue {
    String(String),
    Integer(i64),
    Float(f64), 
    Boolean(bool),
    Array(Vec<Value>),
    Object(Value),
}

#[derive(Clone, Debug)]
pub struct ConfigSource {
    provider: String,    // "file", "env", "cli", etc.
    location: String,    // file path, env var name, etc.
    line: Option<usize>, // line number for debugging
}

impl ConfigData {
    pub fn new(json: Value, sources: Vec<ConfigSource>) -> Self {
        Self {
            json,
            cache: parking_lot::RwLock::new(HashMap::new()),
            sources,
            created_at: std::time::Instant::now(),
        }
    }
    
    /// Zero-copy JSON access
    pub fn as_json(&self) -> &Value {
        &self.json
    }
    
    /// Cached value extraction
    pub fn get_cached<T>(&self, key: &str) -> Option<T> 
    where 
        T: for<'de> serde::Deserialize<'de> + Clone + 'static
    {
        // Check cache first
        if let Some(cached) = self.cache.read().get(key) {
            return self.extract_from_cache(cached);
        }
        
        // Extract from JSON and cache result
        if let Some(value) = self.json.pointer(key) {
            if let Ok(extracted) = serde_json::from_value::<T>(value.clone()) {
                self.cache.write().insert(
                    key.to_string(), 
                    self.to_cached_value(&extracted)
                );
                return Some(extracted);
            }
        }
        
        None
    }
}
```

#### 3. **High-Performance File Loading**

**Implementation Time**: 4-5 hours

```rust
// src/core/loader.rs
use memmap2::{Mmap, MmapOptions};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub struct ConfigLoader {
    // File cache with modification time tracking
    file_cache: DashMap<PathBuf, CachedFile>,
    
    // Format detection cache
    format_cache: DashMap<PathBuf, ConfigFormat>,
}

struct CachedFile {
    content: Arc<str>,
    parsed: Value,
    modified: SystemTime,
    format: ConfigFormat,
}

#[derive(Clone, Copy, Debug)]
pub enum ConfigFormat {
    Json,
    Toml,
    Yaml,
    Env,
    Auto, // Auto-detect
}

impl ConfigLoader {
    pub fn load_file<P: AsRef<Path>>(
        &self, 
        path: P, 
        format: ConfigFormat
    ) -> Result<Value, LoadError> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path)?;
        let modified = metadata.modified()?;
        
        // Check cache first
        if let Some(cached) = self.file_cache.get(path) {
            if cached.modified >= modified {
                return Ok(cached.parsed.clone());
            }
        }
        
        // Load and parse file
        let content = if metadata.len() > 1024 * 1024 { // 1MB threshold
            self.load_with_mmap(path)?
        } else {
            std::fs::read_to_string(path)?
        };
        
        let detected_format = if format == ConfigFormat::Auto {
            self.detect_format(path, &content)
        } else {
            format
        };
        
        let parsed = self.parse_content(&content, detected_format)?;
        
        // Cache result
        self.file_cache.insert(path.to_path_buf(), CachedFile {
            content: content.into(),
            parsed: parsed.clone(),
            modified,
            format: detected_format,
        });
        
        Ok(parsed)
    }
    
    /// Memory-mapped loading for large files
    fn load_with_mmap<P: AsRef<Path>>(&self, path: P) -> Result<String, LoadError> {
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        Ok(String::from_utf8_lossy(&mmap).into_owned())
    }
    
    /// SIMD-accelerated format detection
    #[cfg(feature = "simd")]
    fn detect_format(&self, path: &Path, content: &str) -> ConfigFormat {
        // Check file extension first (fastest)
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "json" => return ConfigFormat::Json,
                "toml" => return ConfigFormat::Toml, 
                "yaml" | "yml" => return ConfigFormat::Yaml,
                "env" => return ConfigFormat::Env,
                _ => {}
            }
        }
        
        // Content-based detection using SIMD
        let bytes = content.as_bytes();
        if bytes.len() < 4 {
            return ConfigFormat::Json; // Default
        }
        
        // Look for JSON indicators: '{', '['
        if bytes[0] == b'{' || bytes[0] == b'[' {
            return ConfigFormat::Json;
        }
        
        // Look for YAML indicators: '---'
        if bytes.starts_with(b"---") {
            return ConfigFormat::Yaml;
        }
        
        // Look for TOML indicators: '[section]' or 'key = value'
        if content.contains(" = ") || content.contains("[") {
            return ConfigFormat::Toml;
        }
        
        // Look for ENV indicators: 'KEY=value'
        if content.lines().any(|line| {
            line.trim_start().chars().next().map_or(false, |c| c.is_ascii_uppercase())
                && line.contains('=')
        }) {
            return ConfigFormat::Env;
        }
        
        ConfigFormat::Json // Default fallback
    }
    
    fn parse_content(&self, content: &str, format: ConfigFormat) -> Result<Value, LoadError> {
        match format {
            ConfigFormat::Json => {
                #[cfg(feature = "simd")]
                {
                    let mut bytes = content.as_bytes().to_vec();
                    simd_json::from_slice(&mut bytes).map_err(LoadError::JsonParse)
                }
                #[cfg(not(feature = "simd"))]
                {
                    serde_json::from_str(content).map_err(LoadError::JsonParse)
                }
            },
            ConfigFormat::Toml => {
                toml::from_str(content).map_err(LoadError::TomlParse)
            },
            ConfigFormat::Yaml => {
                serde_yaml::from_str(content).map_err(LoadError::YamlParse)
            },
            ConfigFormat::Env => {
                self.parse_env_format(content)
            },
            ConfigFormat::Auto => unreachable!("Auto format should be resolved before parsing"),
        }
    }
    
    fn parse_env_format(&self, content: &str) -> Result<Value, LoadError> {
        let mut map = serde_json::Map::new();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"').trim_matches('\'');
                
                // Try to parse as JSON first for nested structures
                let parsed_value = if value.starts_with('{') || value.starts_with('[') {
                    serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.to_string()))
                } else if let Ok(num) = value.parse::<i64>() {
                    Value::Number(num.into())
                } else if let Ok(float) = value.parse::<f64>() {
                    Value::Number(serde_json::Number::from_f64(float).unwrap_or_else(|| 0.into()))
                } else if let Ok(bool_val) = value.parse::<bool>() {
                    Value::Bool(bool_val)
                } else {
                    Value::String(value.to_string())
                };
                
                // Support nested keys like DATABASE_HOST -> {"database": {"host": "..."}}
                self.insert_nested_key(&mut map, key, parsed_value);
            }
        }
        
        Ok(Value::Object(map))
    }
    
    fn insert_nested_key(&self, map: &mut serde_json::Map<String, Value>, key: &str, value: Value) {
        let parts: Vec<&str> = key.split('_').map(|s| s.to_lowercase()).collect();
        
        if parts.len() == 1 {
            map.insert(parts[0].to_string(), value);
            return;
        }
        
        let mut current = map;
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                current.insert(part.to_string(), value.clone());
            } else {
                let entry = current.entry(part.to_string())
                    .or_insert_with(|| Value::Object(serde_json::Map::new()));
                
                if let Value::Object(ref mut obj) = entry {
                    current = obj;
                } else {
                    // Conflict: overwrite with object
                    *entry = Value::Object(serde_json::Map::new());
                    if let Value::Object(ref mut obj) = entry {
                        current = obj;
                    }
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    JsonParse(serde_json::Error),
    #[error("TOML parse error: {0}")]
    TomlParse(toml::de::Error),
    #[error("YAML parse error: {0}")]
    YamlParse(serde_yaml::Error),
    #[error("Format detection failed")]
    UnknownFormat,
}
```

#### 4. **Advanced Array Merging Engine**

**Implementation Time**: 5-6 hours

```rust
// src/core/merge.rs
use serde_json::{Value, Map};
use std::collections::{HashMap, HashSet};

pub struct ArrayMergeEngine {
    // Pre-compiled patterns for performance
    add_suffix: &'static str,
    remove_suffix: &'static str,
}

impl ArrayMergeEngine {
    pub fn new() -> Self {
        Self {
            add_suffix: "_add",
            remove_suffix: "_remove",
        }
    }
    
    pub fn merge_configurations(&self, configs: &[Value]) -> Value {
        if configs.is_empty() {
            return Value::Null;
        }
        
        if configs.len() == 1 {
            return configs[0].clone();
        }
        
        // Start with first config as base
        let mut result = configs[0].clone();
        
        // Merge each subsequent config
        for config in &configs[1..] {
            self.merge_into(&mut result, config);
        }
        
        // Apply array merge operations
        self.process_array_operations(&mut result);
        
        result
    }
    
    fn merge_into(&self, target: &mut Value, source: &Value) {
        match (target, source) {
            (Value::Object(target_map), Value::Object(source_map)) => {
                for (key, source_value) in source_map {
                    match target_map.get_mut(key) {
                        Some(target_value) => {
                            // Recursive merge for nested objects
                            self.merge_into(target_value, source_value);
                        }
                        None => {
                            // Insert new key-value pair
                            target_map.insert(key.clone(), source_value.clone());
                        }
                    }
                }
            }
            (target_val, source_val) => {
                // For non-objects, source overwrites target
                *target_val = source_val.clone();
            }
        }
    }
    
    fn process_array_operations(&self, config: &mut Value) {
        if let Value::Object(map) = config {
            self.process_object_array_operations(map);
        }
    }
    
    fn process_object_array_operations(&self, map: &mut Map<String, Value>) {
        // Collect all array operation keys
        let mut operations: HashMap<String, ArrayOperations> = HashMap::new();
        
        let keys: Vec<String> = map.keys().cloned().collect();
        for key in &keys {
            if let Some(base_key) = key.strip_suffix(self.add_suffix) {
                operations.entry(base_key.to_string())
                    .or_default()
                    .add_operations
                    .push(key.clone());
            } else if let Some(base_key) = key.strip_suffix(self.remove_suffix) {
                operations.entry(base_key.to_string())
                    .or_default()
                    .remove_operations
                    .push(key.clone());
            }
        }
        
        // Apply operations to base arrays
        for (base_key, ops) in operations {
            self.apply_array_operations(map, &base_key, ops);
        }
        
        // Recursively process nested objects
        for value in map.values_mut() {
            if let Value::Object(nested_map) = value {
                self.process_object_array_operations(nested_map);
            } else if let Value::Array(array) = value {
                for item in array {
                    if let Value::Object(item_map) = item {
                        self.process_object_array_operations(item_map);
                    }
                }
            }
        }
    }
    
    fn apply_array_operations(
        &self, 
        map: &mut Map<String, Value>, 
        base_key: &str, 
        ops: ArrayOperations
    ) {
        // Get or create base array
        let mut base_array = match map.get(base_key) {
            Some(Value::Array(arr)) => arr.clone(),
            Some(_) => {
                // Base key exists but is not an array - convert or skip
                return;
            }
            None => Vec::new(), // Create new array
        };
        
        // Apply add operations
        for add_key in &ops.add_operations {
            if let Some(Value::Array(add_items)) = map.get(add_key) {
                base_array.extend(add_items.clone());
            }
        }
        
        // Apply remove operations
        for remove_key in &ops.remove_operations {
            if let Some(Value::Array(remove_items)) = map.get(remove_key) {
                // Convert to HashSet for efficient removal
                let remove_set: HashSet<&Value> = remove_items.iter().collect();
                base_array.retain(|item| !remove_set.contains(item));
            }
        }
        
        // Update base array
        map.insert(base_key.to_string(), Value::Array(base_array));
        
        // Remove operation keys
        for key in ops.add_operations.iter().chain(ops.remove_operations.iter()) {
            map.remove(key);
        }
    }
}

#[derive(Default)]
struct ArrayOperations {
    add_operations: Vec<String>,
    remove_operations: Vec<String>,
}
```

#### 5. **Hot Reload System** (Optional Feature)

**Implementation Time**: 6-7 hours

```rust
// src/core/hotreload.rs
#[cfg(feature = "hot-reload")]
use notify::{Watcher, RecommendedWatcher, Config, Event, EventKind};
use tokio::sync::mpsc;
use std::path::PathBuf;

#[cfg(feature = "hot-reload")]
pub struct HotReloadManager {
    watchers: DashMap<HandleId, WatcherInfo>,
    reload_tx: mpsc::UnboundedSender<ReloadEvent>,
    registry: Arc<ConfigRegistry>,
}

#[cfg(feature = "hot-reload")]
struct WatcherInfo {
    _watcher: RecommendedWatcher, // Keep alive
    files: Vec<PathBuf>,
}

#[cfg(feature = "hot-reload")]
struct ReloadEvent {
    handle_id: HandleId,
    changed_files: Vec<PathBuf>,
}

#[cfg(feature = "hot-reload")]
impl HotReloadManager {
    pub fn new(registry: Arc<ConfigRegistry>) -> Self {
        let (reload_tx, mut reload_rx) = mpsc::unbounded_channel();
        let registry_clone = registry.clone();
        
        // Background task to handle reload events
        tokio::spawn(async move {
            while let Some(event) = reload_rx.recv().await {
                if let Err(e) = Self::handle_reload_event(&registry_clone, event).await {
                    eprintln!("Hot reload error: {}", e);
                }
            }
        });
        
        Self {
            watchers: DashMap::new(),
            reload_tx,
            registry,
        }
    }
    
    pub fn enable_hot_reload(
        &self, 
        handle_id: HandleId, 
        files: Vec<PathBuf>
    ) -> Result<(), notify::Error> {
        let tx = self.reload_tx.clone();
        
        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        let _ = tx.send(ReloadEvent {
                            handle_id,
                            changed_files: event.paths,
                        });
                    }
                }
            },
            Config::default(),
        )?;
        
        // Watch all specified files
        for file in &files {
            watcher.watch(file, notify::RecursiveMode::NonRecursive)?;
        }
        
        self.watchers.insert(handle_id, WatcherInfo {
            _watcher: watcher,
            files: files.clone(),
        });
        
        Ok(())
    }
    
    pub fn disable_hot_reload(&self, handle_id: HandleId) {
        self.watchers.remove(&handle_id);
    }
    
    async fn handle_reload_event(
        registry: &ConfigRegistry, 
        event: ReloadEvent
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Get current config to reconstruct with new file contents
        let current_config = registry.get(event.handle_id)
            .ok_or("Handle not found")?;
        
        // Reload changed files and reconstruct configuration
        let loader = ConfigLoader::new();
        let mut new_configs = Vec::new();
        
        for file in &event.changed_files {
            match loader.load_file(file, ConfigFormat::Auto) {
                Ok(config) => new_configs.push(config),
                Err(e) => {
                    eprintln!("Failed to reload {}: {}", file.display(), e);
                    continue;
                }
            }
        }
        
        if !new_configs.is_empty() {
            let merger = ArrayMergeEngine::new();
            let merged = merger.merge_configurations(&new_configs);
            
            let new_data = ConfigData::new(
                merged,
                vec![ConfigSource {
                    provider: "hot-reload".to_string(),
                    location: format!("{:?}", event.changed_files),
                    line: None,
                }]
            );
            
            registry.update(event.handle_id, new_data);
        }
        
        Ok(())
    }
}

#[cfg(not(feature = "hot-reload"))]
pub struct HotReloadManager;

#[cfg(not(feature = "hot-reload"))]
impl HotReloadManager {
    pub fn new(_registry: Arc<ConfigRegistry>) -> Self {
        Self
    }
    
    pub fn enable_hot_reload(&self, _handle_id: HandleId, _files: Vec<PathBuf>) -> Result<(), &'static str> {
        Err("Hot reload not enabled - compile with 'hot-reload' feature")
    }
    
    pub fn disable_hot_reload(&self, _handle_id: HandleId) {}
}
```

### **Rust API Layer** (Feature: rust-api)

#### 6. **Fluent Builder API**

**Implementation Time**: 3-4 hours

```rust
// src/rust_api/builder.rs
use crate::core::*;

pub struct SuperConfig {
    handle: HandleId,
    registry: Arc<ConfigRegistry>,
    #[cfg(feature = "hot-reload")]
    hot_reload: Arc<HotReloadManager>,
}

impl SuperConfig {
    pub fn new() -> Self {
        let registry = GLOBAL_REGISTRY.clone();
        let handle = registry.create(ConfigData::new(
            serde_json::Value::Object(serde_json::Map::new()),
            vec![]
        ));
        
        Self {
            handle,
            registry,
            #[cfg(feature = "hot-reload")]
            hot_reload: GLOBAL_HOT_RELOAD_MANAGER.clone(),
        }
    }
    
    pub fn with_file<P: AsRef<std::path::Path>>(self, path: P) -> Result<Self, ConfigError> {
        let loader = ConfigLoader::new();
        let config = loader.load_file(path.as_ref(), ConfigFormat::Auto)?;
        
        // Get current config and merge
        let current = self.registry.get(self.handle).unwrap();
        let merger = ArrayMergeEngine::new();
        let merged = merger.merge_configurations(&[current.data.as_json().clone(), config]);
        
        let new_data = ConfigData::new(
            merged,
            vec![ConfigSource {
                provider: "file".to_string(),
                location: path.as_ref().display().to_string(),
                line: None,
            }]
        );
        
        let new_handle = self.registry.create(new_data);
        Ok(Self { handle: new_handle, ..self })
    }
    
    pub fn with_env(self, prefix: &str) -> Result<Self, ConfigError> {
        let env_config = self.load_env_config(prefix)?;
        
        let current = self.registry.get(self.handle).unwrap();
        let merger = ArrayMergeEngine::new();
        let merged = merger.merge_configurations(&[current.data.as_json().clone(), env_config]);
        
        let new_data = ConfigData::new(
            merged,
            vec![ConfigSource {
                provider: "env".to_string(),
                location: format!("prefix:{}", prefix),
                line: None,
            }]
        );
        
        let new_handle = self.registry.create(new_data);
        Ok(Self { handle: new_handle, ..self })
    }
    
    pub fn with_hierarchical<S: AsRef<str>>(self, app_name: S) -> Result<Self, ConfigError> {
        let paths = self.discover_hierarchical_paths(app_name.as_ref());
        let mut configs = vec![self.registry.get(self.handle).unwrap().data.as_json().clone()];
        
        let loader = ConfigLoader::new();
        for path in paths {
            if path.exists() {
                match loader.load_file(&path, ConfigFormat::Auto) {
                    Ok(config) => configs.push(config),
                    Err(_) => continue, // Skip files that can't be loaded
                }
            }
        }
        
        let merger = ArrayMergeEngine::new();
        let merged = merger.merge_configurations(&configs);
        
        let new_data = ConfigData::new(
            merged,
            vec![ConfigSource {
                provider: "hierarchical".to_string(),
                location: app_name.as_ref().to_string(),
                line: None,
            }]
        );
        
        let new_handle = self.registry.create(new_data);
        Ok(Self { handle: new_handle, ..self })
    }
    
    #[cfg(feature = "hot-reload")]
    pub fn with_hot_reload(self, files: Vec<std::path::PathBuf>) -> Result<Self, ConfigError> {
        self.hot_reload.enable_hot_reload(self.handle, files)?;
        Ok(self)
    }
    
    pub fn extract<T>(&self) -> Result<T, ConfigError> 
    where 
        T: for<'de> serde::Deserialize<'de>
    {
        let config = self.registry.get(self.handle).unwrap();
        serde_json::from_value(config.data.as_json().clone())
            .map_err(ConfigError::Extraction)
    }
    
    // Helper methods
    fn load_env_config(&self, prefix: &str) -> Result<serde_json::Value, ConfigError> {
        let mut map = serde_json::Map::new();
        let prefix_with_separator = format!("{}_", prefix);
        
        for (key, value) in std::env::vars() {
            if key.starts_with(&prefix_with_separator) {
                let config_key = key.strip_prefix(&prefix_with_separator).unwrap();
                let parsed_value = self.parse_env_value(&value);
                self.insert_nested_env_key(&mut map, config_key, parsed_value);
            }
        }
        
        Ok(serde_json::Value::Object(map))
    }
    
    fn parse_env_value(&self, value: &str) -> serde_json::Value {
        // Try parsing as JSON first
        if value.starts_with('{') || value.starts_with('[') {
            if let Ok(json) = serde_json::from_str(value) {
                return json;
            }
        }
        
        // Try parsing as number
        if let Ok(int) = value.parse::<i64>() {
            return serde_json::Value::Number(int.into());
        }
        
        if let Ok(float) = value.parse::<f64>() {
            if let Some(num) = serde_json::Number::from_f64(float) {
                return serde_json::Value::Number(num);
            }
        }
        
        // Try parsing as boolean
        if let Ok(bool_val) = value.parse::<bool>() {
            return serde_json::Value::Bool(bool_val);
        }
        
        // Default to string
        serde_json::Value::String(value.to_string())
    }
    
    fn insert_nested_env_key(
        &self, 
        map: &mut serde_json::Map<String, serde_json::Value>, 
        key: &str, 
        value: serde_json::Value
    ) {
        let parts: Vec<&str> = key.split('_').map(|s| s.to_lowercase()).collect();
        
        if parts.len() == 1 {
            map.insert(parts[0].to_string(), value);
            return;
        }
        
        let mut current = map;
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                current.insert(part.to_string(), value.clone());
            } else {
                let entry = current.entry(part.to_string())
                    .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
                
                if let serde_json::Value::Object(ref mut obj) = entry {
                    current = obj;
                }
            }
        }
    }
    
    fn discover_hierarchical_paths(&self, app_name: &str) -> Vec<std::path::PathBuf> {
        let mut paths = Vec::new();
        
        // System-wide config
        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join(app_name).join("config.toml"));
            paths.push(config_dir.join(app_name).join("config.yaml"));
            paths.push(config_dir.join(app_name).join("config.json"));
        }
        
        // User config
        if let Some(home_dir) = dirs::home_dir() {
            paths.push(home_dir.join(format!(".{}", app_name)).join("config.toml"));
            paths.push(home_dir.join(format!(".{}", app_name)).join("config.yaml"));
            paths.push(home_dir.join(format!(".{}", app_name)).join("config.json"));
        }
        
        // Project config (current directory and parents)
        if let Ok(current_dir) = std::env::current_dir() {
            let mut dir = current_dir.as_path();
            loop {
                paths.push(dir.join("config.toml"));
                paths.push(dir.join("config.yaml")); 
                paths.push(dir.join("config.json"));
                paths.push(dir.join(format!("{}.toml", app_name)));
                paths.push(dir.join(format!("{}.yaml", app_name)));
                paths.push(dir.join(format!("{}.json", app_name)));
                
                if let Some(parent) = dir.parent() {
                    dir = parent;
                } else {
                    break;
                }
            }
        }
        
        paths
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Load error: {0}")]
    Load(#[from] LoadError),
    #[error("Extraction error: {0}")]
    Extraction(serde_json::Error),
    #[cfg(feature = "hot-reload")]
    #[error("Hot reload error: {0}")]
    HotReload(#[from] notify::Error),
}
```

### **SuperConfig-FFI Layer** (Thin Wrapper)

#### 7. **Thin FFI Wrapper Implementation**

**Implementation Time**: 2-3 hours

```rust
// superconfig-ffi/src/lib.rs - Thin wrapper using SuperFFI macros
use superffi::superffi;
use superconfig::core::{SuperConfigCore, ConfigBuilder, ConfigError};

#[superffi]
pub struct SuperConfig {
    inner: SuperConfigCore,
}

#[superffi]
impl SuperConfig {
    pub fn new() -> Self {
        Self { 
            inner: SuperConfigCore::new() 
        }
    }
    
    pub fn with_file(&self, path: String) -> Result<SuperConfig, String> {
        // Pure delegation - no business logic
        self.inner.with_file(&path)
            .map(|inner| SuperConfig { inner })
            .map_err(|e| e.to_string())
    }
    
    pub fn with_env(&self, prefix: String) -> Result<SuperConfig, String> {
        // Pure delegation - no business logic
        self.inner.with_env(&prefix)
            .map(|inner| SuperConfig { inner })
            .map_err(|e| e.to_string())
    }
    
    pub fn with_hierarchical(&self, app_name: String) -> Result<SuperConfig, String> {
        // Pure delegation - no business logic
        self.inner.with_hierarchical(&app_name)
            .map(|inner| SuperConfig { inner })
            .map_err(|e| e.to_string())
    }
    
    pub fn with_glob_pattern(&self, pattern: String) -> Result<SuperConfig, String> {
        // Pure delegation - no business logic
        self.inner.with_glob_pattern(&pattern)
            .map(|inner| SuperConfig { inner })
            .map_err(|e| e.to_string())
    }
    
    pub fn select_profile(&self, profile: String) -> Result<SuperConfig, String> {
        // Pure delegation - no business logic
        self.inner.select_profile(&profile)
            .map(|inner| SuperConfig { inner })
            .map_err(|e| e.to_string())
    }
    
    pub fn join_hierarchical(&self, app_name: String) -> Result<SuperConfig, String> {
        // Pure delegation - no business logic
        self.inner.join_hierarchical(&app_name)
            .map(|inner| SuperConfig { inner })
            .map_err(|e| e.to_string())
    }
    
    #[cfg(feature = "hot-reload")]
    pub fn with_hot_reload(&self, files: Vec<String>) -> Result<SuperConfig, String> {
        // Convert String to PathBuf and delegate
        let file_paths: Vec<std::path::PathBuf> = files.into_iter()
            .map(std::path::PathBuf::from)
            .collect();
            
        self.inner.with_hot_reload(file_paths)
            .map(|inner| SuperConfig { inner })
            .map_err(|e| e.to_string())
    }
    
    pub fn extract_json(&self) -> Result<String, String> {
        // Pure delegation - no business logic
        self.inner.extract_json()
            .map_err(|e| e.to_string())
    }
    
    pub fn has_warnings(&self) -> bool {
        // Pure delegation - no business logic
        self.inner.has_warnings()
    }
    
    pub fn get_warnings(&self) -> Vec<String> {
        // Pure delegation with type conversion
        self.inner.get_warnings()
            .into_iter()
            .map(|w| w.to_string())
            .collect()
    }
    
    pub fn extract_typed<T>(&self, type_name: String) -> Result<String, String> 
    where 
        T: serde::de::DeserializeOwned + serde::Serialize
    {
        // Pure delegation with JSON serialization for FFI compatibility
        self.inner.extract::<T>()
            .and_then(|typed| serde_json::to_string(&typed).map_err(ConfigError::from))
            .map_err(|e| e.to_string())
    }
}

// Note: All business logic (file loading, env parsing, hierarchical discovery, 
// array merging, etc.) stays in the superconfig core crate.
// This wrapper only does:
// 1. Type conversion (Rust types ↔ FFI-friendly types)  
// 2. Error conversion (ConfigError → String)
// 3. Simple delegation to core methods
```

### **Language Bindings - Generated by SuperFFI**

#### 8. **Automatic Multi-Language Binding Generation**

**Implementation Time**: 0 hours (Generated automatically by SuperFFI)

The SuperFFI macro automatically generates native bindings for all target languages from the Rust implementation above. No manual language-specific code is required.

**Generated Bindings:**

**Python (Generated automatically):**

```python
# Automatically generated by SuperFFI
import superconfig

config = superconfig.SuperConfig()
config = config.with_file("config.toml")
config = config.with_env("APP")
data = config.extract_json()

if config.has_warnings():
    for warning in config.get_warnings():
        print(f"Warning: {warning}")
```

**Node.js/JavaScript (Generated automatically):**

```javascript
// Automatically generated by SuperFFI with camelCase conversion
const { SuperConfig } = require('superconfig');

const config = new SuperConfig()
    .withFile('config.toml')       // Auto-converted from with_file
    .withEnv('APP')                // Auto-converted from with_env
    .withHierarchical('myapp');    // Auto-converted from with_hierarchical

const data = JSON.parse(config.extractJson());

if (config.hasWarnings()) {
    config.getWarnings().forEach(warning => console.warn(warning));
}
```

**WebAssembly (Generated automatically):**

```javascript
// Automatically generated by SuperFFI for browser environments
import { SuperConfig } from './superconfig_wasm.js';

const config = new SuperConfig()
    .withFile('config.json')
    .withEnv('APP');

const data = JSON.parse(config.extractJson());
```

**SuperFFI Benefits:**

- **Zero Manual FFI Code**: All bindings generated from single Rust implementation
- **Consistent APIs**: Same functionality across all languages
- **Automatic Naming**: `snake_case` → `camelCase` conversion for JavaScript
- **Type Safety**: Proper error handling in each language
- **Memory Management**: Automatic cleanup via language-specific patterns

## ⚡ **Realistic AI Development Timeline**

### **Phase 1: Core Implementation** (13-16 hours)

- **Registry System**: 3-4 hours
- **Config Data Structure**: 2-3 hours
- **File Loader**: 4-5 hours
- **Array Merge Engine**: 5-6 hours
- **Basic Testing**: 2-3 hours

### **Phase 2: API Layers** (4-5 hours)

- **Rust Core API**: 3-4 hours
- **Thin FFI Wrapper**: 2-3 hours
- **Error Handling**: 1-2 hours

### **Phase 3: SuperFFI Integration** (2-3 hours)

- **SuperFFI Macro Integration**: 1-2 hours
- **Build System Setup**: 1-2 hours
- **Language Package Generation**: 0 hours (Automatic)

### **Phase 4: Advanced Features** (8-10 hours)

- **Hot Reload**: 6-7 hours
- **SIMD Optimizations**: 2-3 hours

### **Phase 5: Polish & Package** (6-8 hours)

- **Documentation**: 2-3 hours
- **Examples**: 2-3 hours
- **CI/CD Setup**: 2-3 hours

**Total: 33-42 hours** (approximately 4-6 days of focused AI-assisted development)

## 🚀 **Expected Performance**

With this architecture, we expect:

- **Config Loading**: ~20-30μs (vs current ~100μs)
- **FFI Overhead**: ~0.5-1μs (vs current ~100μs)
- **Memory Usage**: ~100 bytes per handle + 50KB registry
- **Array Merging**: ~5-10μs with SIMD optimizations
- **Hot Reload**: ~2-5μs update time

**Total end-to-end performance: ~25-35μs** across all languages and platforms.

This multi-crate architecture with SuperFFI integration provides the best balance of performance, maintainability, and automatic multi-language support while being realistic for AI-assisted development timelines.
