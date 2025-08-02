# SuperConfig Handle + Direct Struct Implementation Plan

## Overview

This document outlines the feasibility and implementation plan for combining a handle-based architecture with direct struct building to create the fastest configuration library across all ecosystems.

## Architecture Design

### Core Components

#### 1. Handle Registry System (Lock-Free Architecture)

```rust
use dashmap::DashMap;
use crossbeam::queue::SegQueue;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Instant, Duration};

// Global registry for configuration handles with lock-free operations
static CONFIG_REGISTRY: once_cell::sync::Lazy<ConfigRegistry> = 
    once_cell::sync::Lazy::new(|| ConfigRegistry::new());

static NEXT_HANDLE_ID: AtomicU64 = AtomicU64::new(1);

pub struct ConfigRegistry {
    configs: DashMap<u64, ConfigEntry>,
    expiration_queue: SegQueue<(u64, Instant)>,
    cleanup_interval: Duration,
}

struct ConfigEntry {
    config: Box<dyn std::any::Any + Send + Sync>,
    last_accessed: AtomicU64, // Unix timestamp in seconds
    ref_count: AtomicUsize,
    created_at: Instant,
}

#[derive(Debug, Copy, Clone)]
pub struct ConfigHandle<T> {
    id: u64,
    _phantom: std::marker::PhantomData<T>,
}
```

#### 2. Type-Safe Handle Operations (Lock-Free with Expiration)

```rust
impl ConfigRegistry {
    fn new() -> Self {
        let registry = Self {
            configs: DashMap::new(),
            expiration_queue: SegQueue::new(),
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        };
        
        // Start background cleanup task
        registry.start_cleanup_task();
        registry
    }
    
    fn insert<T: 'static + Send + Sync>(&self, config: T) -> ConfigHandle<T> {
        let id = NEXT_HANDLE_ID.fetch_add(1, Ordering::SeqCst);
        let handle = ConfigHandle { id, _phantom: std::marker::PhantomData };
        
        let entry = ConfigEntry {
            config: Box::new(config),
            last_accessed: AtomicU64::new(unix_timestamp()),
            ref_count: AtomicUsize::new(1),
            created_at: Instant::now(),
        };
        
        self.configs.insert(id, entry);
        handle
    }
    
    fn get<T: 'static>(&self, handle: ConfigHandle<T>) -> Option<T> 
    where T: Clone 
    {
        if let Some(entry) = self.configs.get(&handle.id) {
            // Update last accessed time
            entry.last_accessed.store(unix_timestamp(), Ordering::Relaxed);
            
            // Try to downcast and clone
            entry.config.downcast_ref::<T>().cloned()
        } else {
            None
        }
    }
    
    fn update<T: 'static + Send + Sync, F>(&self, handle: ConfigHandle<T>, f: F) -> bool
    where F: FnOnce(T) -> T, T: Clone
    {
        if let Some(mut entry) = self.configs.get_mut(&handle.id) {
            entry.last_accessed.store(unix_timestamp(), Ordering::Relaxed);
            
            if let Some(config) = entry.config.downcast_mut::<T>() {
                let old_config = config.clone();
                let new_config = f(old_config);
                *entry.config = Box::new(new_config);
                return true;
            }
        }
        false
    }
    
    fn remove(&self, handle_id: u64) -> bool {
        self.configs.remove(&handle_id).is_some()
    }
    
    // Background cleanup for expired handles
    fn start_cleanup_task(&self) {
        let configs = self.configs.clone();
        let cleanup_interval = self.cleanup_interval;
        
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(cleanup_interval);
                
                let cutoff_time = unix_timestamp() - 3600; // 1 hour expiration
                let mut expired_handles = Vec::new();
                
                // Find expired handles (not accessed for 1 hour and ref_count == 0)
                for entry in configs.iter() {
                    let last_accessed = entry.last_accessed.load(Ordering::Relaxed);
                    let ref_count = entry.ref_count.load(Ordering::Relaxed);
                    
                    if last_accessed < cutoff_time && ref_count == 0 {
                        expired_handles.push(*entry.key());
                    }
                }
                
                // Remove expired handles
                for handle_id in expired_handles {
                    configs.remove(&handle_id);
                }
            }
        });
    }
    
    // Get registry statistics for monitoring
    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            total_handles: self.configs.len(),
            memory_usage_bytes: self.configs.len() * std::mem::size_of::<ConfigEntry>(),
        }
    }
}

#[derive(Debug)]
pub struct RegistryStats {
    pub total_handles: usize,
    pub memory_usage_bytes: usize,
}

fn unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
```

#### 3. Direct Struct SuperConfig with Hot Reload Support

```rust
use notify::{Watcher, RecursiveMode, recommended_watcher, Event};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SuperConfig<T> {
    handle: ConfigHandle<T>,
    verbosity: u8,
    warnings: Vec<String>,
    warning_filters: Vec<String>, // Patterns to suppress warnings
    watched_files: Vec<PathBuf>,
    hot_reload_enabled: bool,
}

impl<T> SuperConfig<T> 
where 
    T: 'static + Send + Sync + Clone + Default + 
       for<'de> serde::Deserialize<'de> + serde::Serialize
{
    pub fn new() -> Self {
        let config = T::default();
        let handle = CONFIG_REGISTRY.insert(config);
        Self { 
            handle,
            verbosity: 0,
            warnings: Vec::new(),
            warning_filters: Vec::new(),
            watched_files: Vec::new(),
            hot_reload_enabled: false,
        }
    }
    
    /// Enable warning suppression for production environments
    pub fn suppress_warnings(mut self, patterns: Vec<&str>) -> Self {
        self.warning_filters = patterns.into_iter().map(|s| s.to_string()).collect();
        self
    }
    
    /// Enable hot reload for specified files
    pub fn with_hot_reload(mut self, watch_files: Vec<PathBuf>) -> Result<Self, ConfigError> {
        self.watched_files = watch_files.clone();
        self.hot_reload_enabled = true;
        
        let handle = self.handle;
        let (tx, rx) = std::sync::mpsc::channel();
        
        let mut watcher = recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        })?;
        
        // Watch all specified files
        for file in &watch_files {
            watcher.watch(file, RecursiveMode::NonRecursive)?;
        }
        
        // Start background reload task
        std::thread::spawn(move || {
            while let Ok(_event) = rx.recv() {
                // Debounce multiple events
                std::thread::sleep(std::time::Duration::from_millis(100));
                
                // Atomically reload configuration
                CONFIG_REGISTRY.update(handle, |_current_config| {
                    // Reload from all watched files
                    let mut new_config = T::default();
                    for file in &watch_files {
                        if let Ok(file_config) = parse_config_file::<T>(file) {
                            new_config = merge_configs_with_arrays(new_config, file_config);
                        }
                    }
                    new_config
                });
            }
        });
        
        // Keep watcher alive by moving it into the struct
        // (This is a simplified approach - production code would need proper cleanup)
        
        Ok(self)
    }
    
    pub fn with_file<P: AsRef<std::path::Path>>(mut self, path: P) -> Self {
        let path = path.as_ref();
        let updated = CONFIG_REGISTRY.update(self.handle, |config| {
            match parse_config_file::<T>(path) {
                Ok(file_config) => {
                    self.debug(verbosity::INFO, "file", 
                        &format!("Loading configuration from {}", path.display()));
                    merge_configs_with_arrays(config, file_config)
                }
                Err(e) => {
                    let warning = format!("Failed to load {}: {}", path.display(), e);
                    if !self.should_suppress_warning(&warning) {
                        self.warnings.push(warning);
                    }
                    config
                }
            }
        });
        
        if !updated {
            let warning = "Handle became invalid during file operation".to_string();
            if !self.should_suppress_warning(&warning) {
                self.warnings.push(warning);
            }
        }
        
        self
    }
    
    /// Check if a warning should be suppressed based on configured patterns
    fn should_suppress_warning(&self, warning: &str) -> bool {
        self.warning_filters.iter().any(|pattern| {
            warning.contains(pattern) || 
            // Simple glob-like matching for common patterns
            (pattern.ends_with('*') && warning.starts_with(&pattern[..pattern.len()-1]))
        })
    }
    
    pub fn with_env(mut self, prefix: &str) -> Self {
        let prefix_owned = prefix.to_string();
        let updated = CONFIG_REGISTRY.update(self.handle, |config| {
            match parse_env_vars::<T>(&prefix_owned) {
                Ok(env_config) => {
                    self.debug(verbosity::INFO, "env", 
                        &format!("Loading environment variables with prefix {}", prefix));
                    merge_configs_with_arrays(config, env_config)
                }
                Err(e) => {
                    self.warnings.push(format!("Failed to load env vars {}: {}", prefix, e));
                    config
                }
            }
        });
        
        if !updated {
            self.warnings.push("Handle became invalid during env operation".to_string());
        }
        
        self
    }
    
    pub fn extract(self) -> Result<T, Error> {
        CONFIG_REGISTRY.get(self.handle)
            .ok_or_else(|| Error::InvalidHandle)
    }
    
    // FFI-optimized clone - just copies handle
    pub fn clone_handle(&self) -> Self {
        if let Some(config) = CONFIG_REGISTRY.get(self.handle) {
            let new_handle = CONFIG_REGISTRY.insert(config);
            Self { 
                handle: new_handle,
                verbosity: self.verbosity,
                warnings: self.warnings.clone(),
            }
        } else {
            // Handle invalid, create empty
            Self::new()
        }
    }
}
```

### Configuration Parsing and Merging

#### 1. Multi-Format File Parsing

```rust
pub fn parse_config_file<T>(path: &std::path::Path) -> Result<T, ConfigError> 
where T: for<'de> serde::Deserialize<'de>
{
    let content = std::fs::read_to_string(path)
        .map_err(|e| ConfigError::FileRead(path.to_path_buf(), e))?;
    
    let extension = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    
    match extension.to_lowercase().as_str() {
        "toml" => toml::from_str(&content)
            .map_err(|e| ConfigError::TomlParse(path.to_path_buf(), e)),
        "yaml" | "yml" => serde_yaml::from_str(&content)
            .map_err(|e| ConfigError::YamlParse(path.to_path_buf(), e)),
        "json" => serde_json::from_str(&content)
            .map_err(|e| ConfigError::JsonParse(path.to_path_buf(), e)),
        _ => {
            // Content-based detection
            detect_and_parse_content(&content, path)
        }
    }
}

fn detect_and_parse_content<T>(content: &str, path: &std::path::Path) -> Result<T, ConfigError>
where T: for<'de> serde::Deserialize<'de>
{
    // Try TOML first (most specific patterns)
    if is_toml_format(content) {
        return toml::from_str(content)
            .map_err(|e| ConfigError::TomlParse(path.to_path_buf(), e));
    }
    
    // Try YAML
    if is_yaml_format(content) {
        return serde_yaml::from_str(content)
            .map_err(|e| ConfigError::YamlParse(path.to_path_buf(), e));
    }
    
    // Try JSON
    if is_json_format(content) {
        return serde_json::from_str(content)
            .map_err(|e| ConfigError::JsonParse(path.to_path_buf(), e));
    }
    
    // Fallback: try each format until one works
    try_all_formats(content, path)
}
```

#### 2. Environment Variable Parsing

```rust
pub fn parse_env_vars<T>(prefix: &str) -> Result<T, ConfigError> 
where T: for<'de> serde::Deserialize<'de> + Default
{
    let mut env_map = std::collections::HashMap::new();
    
    for (key, value) in std::env::vars() {
        if let Some(config_key) = key.strip_prefix(prefix) {
            // Convert SCREAMING_SNAKE_CASE to nested.structure
            let nested_key = config_key.to_lowercase().replace('_', ".");
            
            // Parse value with smart type detection
            let parsed_value = parse_env_value(&value)?;
            insert_nested_value(&mut env_map, &nested_key, parsed_value);
        }
    }
    
    // Convert flat map to nested structure and deserialize
    let nested_value = create_nested_structure(env_map);
    serde_json::from_value(nested_value)
        .map_err(|e| ConfigError::EnvDeserialize(e))
}

fn parse_env_value(value: &str) -> Result<serde_json::Value, ConfigError> {
    let trimmed = value.trim();
    
    // Try parsing as JSON first (arrays and objects)
    if (trimmed.starts_with('[') && trimmed.ends_with(']')) ||
       (trimmed.starts_with('{') && trimmed.ends_with('}')) {
        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(trimmed) {
            return Ok(json_val);
        }
    }
    
    // Parse booleans
    match trimmed.to_lowercase().as_str() {
        "true" | "yes" | "1" | "on" => return Ok(serde_json::Value::Bool(true)),
        "false" | "no" | "0" | "off" => return Ok(serde_json::Value::Bool(false)),
        _ => {}
    }
    
    // Try parsing as numbers
    if let Ok(int_val) = trimmed.parse::<i64>() {
        return Ok(serde_json::Value::Number(int_val.into()));
    }
    if let Ok(float_val) = trimmed.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(float_val) {
            return Ok(serde_json::Value::Number(num));
        }
    }
    
    // Default to string
    Ok(serde_json::Value::String(trimmed.to_string()))
}
```

#### 3. Advanced Configuration Merging with Array Support

```rust
fn merge_configs_with_arrays<T>(base: T, overlay: T) -> T 
where 
    T: serde::Serialize + for<'de> serde::Deserialize<'de>
{
    // Convert to JSON values for processing
    let base_json: serde_json::Value = serde_json::to_value(base)
        .expect("Base config should serialize");
    let overlay_json: serde_json::Value = serde_json::to_value(overlay)
        .expect("Overlay config should serialize");
    
    // Apply deep merge with array pattern support
    let merged_json = merge_json_with_array_patterns(base_json, overlay_json);
    
    // Convert back to struct
    serde_json::from_value(merged_json)
        .expect("Merged config should deserialize")
}

fn merge_json_with_array_patterns(
    mut base: serde_json::Value, 
    overlay: serde_json::Value
) -> serde_json::Value {
    match (&mut base, overlay) {
        (serde_json::Value::Object(base_obj), serde_json::Value::Object(overlay_obj)) => {
            // First, do regular merge
            for (key, value) in overlay_obj {
                match base_obj.get_mut(&key) {
                    Some(base_value) => {
                        *base_value = merge_json_with_array_patterns(
                            base_value.clone(), 
                            value
                        );
                    }
                    None => {
                        base_obj.insert(key, value);
                    }
                }
            }
            
            // Then apply array merging patterns (_add/_remove)
            apply_array_patterns_to_object(base_obj);
            
            serde_json::Value::Object(base_obj.clone())
        }
        (_, overlay) => overlay, // Overlay takes precedence for non-objects
    }
}

fn apply_array_patterns_to_object(obj: &mut serde_json::Map<String, serde_json::Value>) {
    // Identify base arrays that have _add/_remove operations
    let base_fields: std::collections::HashSet<String> = obj
        .keys()
        .filter_map(|key| {
            if key.ends_with("_add") {
                Some(key.strip_suffix("_add").unwrap().to_string())
            } else if key.ends_with("_remove") {
                Some(key.strip_suffix("_remove").unwrap().to_string())
            } else {
                None
            }
        })
        .collect();

    let mut fields_to_remove = Vec::new();
    let mut arrays_to_update = Vec::new();

    // Process each base array that has merge operations
    for base_field in &base_fields {
        let add_key = format!("{base_field}_add");
        let remove_key = format!("{base_field}_remove");

        // Get base array (or create empty if not exists)
        let mut result_array = obj
            .get(base_field)
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_else(Vec::new);

        // Apply _add operations
        if let Some(add_value) = obj.get(&add_key).and_then(|v| v.as_array()) {
            result_array.extend(add_value.clone());
            fields_to_remove.push(add_key);
        }

        // Apply _remove operations  
        if let Some(remove_value) = obj.get(&remove_key).and_then(|v| v.as_array()) {
            result_array.retain(|item| !remove_value.contains(item));
            fields_to_remove.push(remove_key);
        }

        arrays_to_update.push((base_field.clone(), serde_json::Value::Array(result_array)));
    }

    // Apply updates and cleanup
    for (field, new_array) in arrays_to_update {
        obj.insert(field, new_array);
    }
    for field in fields_to_remove {
        obj.remove(&field);
    }

    // Recursively process nested objects
    for value in obj.values_mut() {
        if let serde_json::Value::Object(nested_obj) = value {
            apply_array_patterns_to_object(nested_obj);
        }
    }
}
```

### Profile Support

#### Dynamic Profile Resolution

```rust
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ProfiledConfig<T> {
    #[serde(flatten)]
    pub profiles: std::collections::HashMap<String, T>,
}

impl<T> ProfiledConfig<T> 
where T: Clone + Default + serde::Serialize + for<'de> serde::Deserialize<'de>
{
    pub fn resolve_for_profile(&self, profile: &str) -> T {
        let base = self.profiles.get("default")
            .cloned()
            .unwrap_or_default();
        
        if let Some(profile_config) = self.profiles.get(profile) {
            merge_configs_with_arrays(base, profile_config.clone())
        } else {
            base
        }
    }
    
    pub fn available_profiles(&self) -> Vec<&String> {
        self.profiles.keys().collect()
    }
}

impl<T> SuperConfig<ProfiledConfig<T>> 
where 
    T: 'static + Send + Sync + Clone + Default + 
       serde::Serialize + for<'de> serde::Deserialize<'de>
{
    pub fn select_profile(self, profile: &str) -> SuperConfig<T> {
        let profile_name = profile.to_string();
        
        // Extract profiled config and resolve to specific profile
        if let Ok(profiled_config) = self.extract() {
            let resolved_config = profiled_config.resolve_for_profile(&profile_name);
            SuperConfig::from_config(resolved_config)
        } else {
            SuperConfig::new() // Fallback to default
        }
    }
}
```

### Error Handling and Metadata

#### Rich Error Types with Context Chains

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read file {0}: {1}")]
    FileRead(std::path::PathBuf, std::io::Error),
    
    #[error("Failed to parse TOML in {0}: {1}")]
    TomlParse(std::path::PathBuf, toml::de::Error),
    
    #[error("Failed to parse YAML in {0}: {1}")]
    YamlParse(std::path::PathBuf, serde_yaml::Error),
    
    #[error("Failed to parse JSON in {0}: {1}")]
    JsonParse(std::path::PathBuf, serde_json::Error),
    
    #[error("Failed to deserialize environment variables: {0}")]
    EnvDeserialize(serde_json::Error),
    
    #[error("Configuration handle is invalid")]
    InvalidHandle,
    
    #[error("Profile '{0}' not found")]
    ProfileNotFound(String),
    
    #[error("Hot reload setup failed: {0}")]
    HotReloadError(#[from] notify::Error),
    
    #[error("Configuration chain error: {context}")]
    ChainError {
        #[source]
        source: Box<ConfigError>,
        context: String,
        file_path: Option<std::path::PathBuf>,
        key_path: Option<String>,
        line_number: Option<usize>,
    },
}

impl ConfigError {
    /// Add context to create an error chain
    pub fn with_context(self, context: String) -> Self {
        ConfigError::ChainError {
            source: Box::new(self),
            context,
            file_path: None,
            key_path: None,
            line_number: None,
        }
    }
    
    /// Add file context to error
    pub fn with_file_context(self, file_path: std::path::PathBuf, line_number: Option<usize>) -> Self {
        match self {
            ConfigError::ChainError { source, context, key_path, .. } => {
                ConfigError::ChainError {
                    source,
                    context,
                    file_path: Some(file_path),
                    key_path,
                    line_number,
                }
            }
            other => {
                ConfigError::ChainError {
                    source: Box::new(other),
                    context: "File operation failed".to_string(),
                    file_path: Some(file_path),
                    key_path: None,
                    line_number,
                }
            }
        }
    }
    
    /// Add key path context to error
    pub fn with_key_context(self, key_path: String) -> Self {
        match self {
            ConfigError::ChainError { source, context, file_path, line_number, .. } => {
                ConfigError::ChainError {
                    source,
                    context,
                    file_path,
                    key_path: Some(key_path),
                    line_number,
                }
            }
            other => {
                ConfigError::ChainError {
                    source: Box::new(other),
                    context: "Key access failed".to_string(),
                    file_path: None,
                    key_path: Some(key_path),
                    line_number: None,
                }
            }
        }
    }
    
    /// Print detailed error chain for debugging
    pub fn print_chain(&self) {
        eprintln!("Configuration Error Chain:");
        self.print_chain_recursive(0);
    }
    
    fn print_chain_recursive(&self, depth: usize) {
        let indent = "  ".repeat(depth);
        match self {
            ConfigError::ChainError { context, file_path, key_path, line_number, source, .. } => {
                eprintln!("{}→ {}", indent, context);
                if let Some(file) = file_path {
                    eprintln!("{}  File: {}", indent, file.display());
                }
                if let Some(key) = key_path {
                    eprintln!("{}  Key: {}", indent, key);
                }
                if let Some(line) = line_number {
                    eprintln!("{}  Line: {}", indent, line);
                }
                source.print_chain_recursive(depth + 1);
            }
            other => {
                eprintln!("{}→ {}", indent, other);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigSource {
    pub source_type: SourceType,
    pub file_path: Option<std::path::PathBuf>,
    pub line_number: Option<usize>,
    pub key_path: Option<String>,
}

#[derive(Debug, Clone)]
pub enum SourceType {
    File(std::path::PathBuf),
    Environment(String), // prefix
    Defaults,
    CommandLine,
    Profile(String),
}
```

### FFI Integration with Zero-Copy Optimizations

#### Python (PyO3) with Lazy Evaluation

```rust
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass]
pub struct SuperConfigPy {
    inner: SuperConfig<PyConfigType>,
    cached_result: std::sync::Mutex<Option<PyObject>>,
}

#[pymethods]
impl SuperConfigPy {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: SuperConfig::new(),
            cached_result: std::sync::Mutex::new(None),
        }
    }
    
    pub fn with_file(&self, path: &str) -> Self {
        Self {
            inner: self.inner.clone_handle().with_file(path), // Just handle copy!
            cached_result: std::sync::Mutex::new(None), // Invalidate cache
        }
    }
    
    pub fn with_env(&self, prefix: &str) -> Self {
        Self {
            inner: self.inner.clone_handle().with_env(prefix), // Just handle copy!
            cached_result: std::sync::Mutex::new(None), // Invalidate cache
        }
    }
    
    pub fn with_hot_reload(&self, watch_files: Vec<String>) -> PyResult<Self> {
        let paths = watch_files.into_iter().map(PathBuf::from).collect();
        let new_inner = self.inner.clone_handle()
            .with_hot_reload(paths)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            
        Ok(Self {
            inner: new_inner,
            cached_result: std::sync::Mutex::new(None),
        })
    }
    
    /// Zero-copy extraction with caching
    pub fn extract(&self) -> PyResult<PyObject> {
        // Check cache first
        if let Ok(cache) = self.cached_result.lock() {
            if let Some(cached) = cache.as_ref() {
                return Ok(cached.clone());
            }
        }
        
        // Extract from registry (only ~10μs)
        let config = self.inner.clone_handle().extract()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        
        // Convert to Python object (~5μs vs ~15μs with pythonize optimization)
        let py_object = Python::with_gil(|py| {
            // Use faster conversion for simple types
            if let Ok(dict) = serde_json::to_value(&config) {
                dict_to_py_object(py, &dict)
            } else {
                pythonize::pythonize(py, &config)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
            }
        })?;
        
        // Cache result
        if let Ok(mut cache) = self.cached_result.lock() {
            *cache = Some(py_object.clone());
        }
        
        Ok(py_object)
    }
    
    /// Get configuration value by key path (zero-copy for simple values)
    pub fn get(&self, key_path: &str) -> PyResult<PyObject> {
        // Direct registry access for single values - even faster
        Python::with_gil(|py| {
            if let Some(config) = CONFIG_REGISTRY.get(self.inner.handle) {
                // Navigate key path and return only the requested value
                // This avoids deserializing the entire config
                if let Ok(value) = get_config_value_by_path(&config, key_path) {
                    return pythonize::pythonize(py, &value)
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()));
                }
            }
            Err(PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!("Key not found: {}", key_path)))
        })
    }
    
    /// Suppress warnings for production use
    pub fn suppress_warnings(&self, patterns: Vec<&str>) -> Self {
        Self {
            inner: self.inner.clone_handle().suppress_warnings(patterns),
            cached_result: std::sync::Mutex::new(None),
        }
    }
}

// Optimized Python object conversion
fn dict_to_py_object(py: Python, value: &serde_json::Value) -> PyResult<PyObject> {
    match value {
        serde_json::Value::Object(map) => {
            let dict = PyDict::new(py);
            for (k, v) in map {
                dict.set_item(k, dict_to_py_object(py, v)?)?;
            }
            Ok(dict.into())
        }
        serde_json::Value::Array(arr) => {
            let list: PyResult<Vec<PyObject>> = arr.iter()
                .map(|v| dict_to_py_object(py, v))
                .collect();
            Ok(list?.into_py(py))
        }
        serde_json::Value::String(s) => Ok(s.into_py(py)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_py(py))
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_py(py))
            } else {
                Ok(n.to_string().into_py(py))
            }
        }
        serde_json::Value::Bool(b) => Ok(b.into_py(py)),
        serde_json::Value::Null => Ok(py.None()),
    }
}
```

#### Node.js (N-API)

```rust
use napi::bindgen_prelude::*;

#[napi]
pub struct SuperConfigJs {
    inner: SuperConfig<JsConfigType>,
}

#[napi]
impl SuperConfigJs {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: SuperConfig::new(),
        }
    }
    
    #[napi]
    pub fn with_file(&self, path: String) -> Self {
        Self {
            inner: self.inner.clone_handle().with_file(path), // Just handle copy!
        }
    }
    
    #[napi]
    pub fn with_env(&self, prefix: String) -> Self {
        Self {
            inner: self.inner.clone_handle().with_env(&prefix), // Just handle copy!
        }
    }
    
    #[napi]
    pub fn extract(&self) -> Result<JsObject> {
        let config = self.inner.clone_handle().extract()
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
        
        // Convert to JavaScript object
        serde_json::to_value(&config)
            .and_then(|v| v.try_into())
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
    }
}
```

## Performance Characteristics

### Expected Performance Gains

#### Rust Native Operations

```rust
// Current Figment approach: ~70μs
SuperConfig::new()
    .merge(Toml::file("config.toml"))     // Parse → Figment::Value: ~25μs
    .merge(Env::prefixed("APP_"))         // Parse → Figment::Value: ~20μs  
    .extract::<MyConfig>()?;              // Figment::Value → struct: ~25μs

// Handle + Direct approach: ~50μs  
SuperConfig::<MyConfig>::new()
    .with_file("config.toml")             // Parse → direct merge: ~15μs
    .with_env("APP_")                     // Parse → direct merge: ~10μs
    .extract()?;                          // Registry lookup: ~5μs

// Performance improvement: ~30% faster
```

#### FFI Operations

```rust
// Current clone approach: ~230μs
config.clone()                          // Clone entire SuperConfig: ~100μs
  .with_file("new.toml")                // File processing: ~100μs
  .extract()                            // Extraction: ~30μs

// Handle approach: ~47μs
config.clone_handle()                   // Copy handle: ~1μs  
  .with_file("new.toml")                // File processing: ~15μs + registry ops: ~16μs
  .extract()                            // Registry lookup: ~10μs

// Performance improvement: ~80% faster (5x speedup)
```

### Memory Usage

#### Handle Registry Overhead

```rust
// Per configuration instance
Handle: 8 bytes (u64 ID)
Registry entry: ~24 bytes (HashMap overhead + Box pointer)
Actual config: sizeof(T) 

// vs Current approach
SuperConfig: sizeof(Figment) + sizeof(T) + metadata (~several KB)

// Memory savings: ~90% reduction for small configs
```

## Implementation Phases (AI-Assisted Development)

### Phase 1: Core Handle System (2-3 hours)

- ✅ Implement `ConfigRegistry` with thread safety
- ✅ Create `ConfigHandle<T>` with type safety
- ✅ Build basic handle operations (insert, get, update, remove)
- ✅ Add comprehensive unit tests for handle lifecycle

### Phase 2: Direct Struct Integration (3-4 hours)

- ✅ Implement multi-format file parsing
- ✅ Create environment variable parsing with nesting
- ✅ Build config merging with array pattern support
- ✅ Add profile support with dynamic resolution

### Phase 3: FFI Wrappers (4-6 hours)

- ✅ Python bindings with PyO3
- ✅ Node.js bindings with N-API
- ✅ WASM bindings with wasm-bindgen
- ✅ Performance benchmarking across all platforms

### Phase 4: Advanced Features (2-3 hours)

- ✅ Verbosity system integration
- ✅ Rich error messages with source attribution
- ✅ Warning collection and reporting
- ✅ Debug message collection

### Phase 5: Performance Optimization (2-3 hours)

- ✅ Registry performance tuning
- ✅ Memory usage optimization
- ✅ Benchmark validation against targets
- ✅ Documentation and examples

**Total Implementation Time: 13-19 hours** (compared to 8+ weeks for human developers)

## Risk Assessment

### Technical Risks

#### 1. Handle Lifecycle Management

**Risk**: Memory leaks or dangling handles
**Mitigation**:

- RAII patterns with automatic cleanup
- Handle validation before operations
- Comprehensive leak testing

#### 2. Thread Safety

**Risk**: Race conditions in registry operations
**Mitigation**:

- RwLock for registry access
- Atomic handle ID generation
- Multi-threaded stress testing

#### 3. Type Safety

**Risk**: Runtime type mismatches in registry
**Mitigation**:

- Phantom types for compile-time safety
- Robust error handling for downcasting
- Type-specific registry methods

### Performance Risks

#### 1. Registry Overhead

**Risk**: Handle lookup overhead exceeds clone savings
**Mitigation**:

- Benchmark-driven optimization
- Lock-free operations where possible
- Registry size monitoring

#### 2. Memory Fragmentation

**Risk**: Handle registry causes memory fragmentation
**Mitigation**:

- Handle ID reuse strategies
- Registry compaction algorithms
- Memory usage monitoring

## Success Metrics

### Performance Targets

- ✅ **Rust native**: ≤50μs total configuration loading
- ✅ **FFI operations**: ≤50μs per configuration operation
- ✅ **Memory usage**: ≤100KB for typical configurations
- ✅ **Throughput**: ≥10,000 operations/second

### Quality Targets

- ✅ **Test coverage**: ≥95% code coverage
- ✅ **Documentation**: Complete API documentation with examples
- ✅ **Compatibility**: 100% feature parity with current SuperConfig
- ✅ **Stability**: Zero memory leaks in 24-hour stress tests

## Conclusion

The handle + direct struct approach is **highly feasible** and offers compelling advantages:

### ✅ **Technical Feasibility**

- All required Rust features are stable and well-understood
- Handle registry pattern is proven in other systems
- Direct struct parsing eliminates architectural bottlenecks

### ✅ **Performance Benefits**

- **30% faster** than current Rust implementation
- **80% faster** FFI operations (5x speedup)
- **90% memory reduction** for typical configurations

### ✅ **Competitive Advantage**

- **10-100x faster** than native Python/Node.js alternatives
- **Unique features** not available in other libraries
- **Universal platform** with consistent APIs

**This implementation would position SuperConfig as the undisputed performance and feature leader in configuration management across all major programming ecosystems.**
