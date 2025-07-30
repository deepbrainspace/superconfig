# SuperConfig V2: Provider System Design

## Overview

This document specifies the provider system for SuperConfig V2 - the comprehensive configuration loading infrastructure that supports multiple data sources, formats, and discovery patterns. The provider system is designed for maximum performance with intelligent caching, parallel loading, and advanced pattern matching capabilities.

## Provider Architecture

### Core Provider Interface

```rust
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Common interface for all configuration providers
#[async_trait]
pub trait ConfigProvider: Send + Sync {
    /// Load configuration data from this provider
    async fn load(&self, context: &LoadContext) -> Result<ProviderResult, ProviderError>;
    
    /// Check if this provider can handle the given source
    fn can_handle(&self, source: &ConfigSource) -> bool;
    
    /// Get provider priority for source resolution
    fn priority(&self) -> u32;
    
    /// Provider-specific metadata for debugging
    fn metadata(&self) -> ProviderMetadata;
    
    /// Optional: Watch for changes (hot reload support)
    #[cfg(feature = "hot-reload")]
    async fn watch(&self, context: &LoadContext) -> Result<ChangeStream, ProviderError>;
}

/// Context passed to providers during loading
#[derive(Debug, Clone)]
pub struct LoadContext {
    /// Selected profile (dev, prod, test, etc.)
    pub profile: Option<String>,
    
    /// Base directory for relative path resolution
    pub base_dir: PathBuf,
    
    /// Environment variables snapshot
    pub env_vars: HashMap<String, String>,
    
    /// Loading options and feature flags
    pub options: LoadOptions,
    
    /// Warning collector for non-fatal issues
    pub warnings: Arc<Mutex<Vec<ConfigWarning>>>,
}

/// Result returned by providers
pub struct ProviderResult {
    /// Parsed configuration data
    pub data: serde_json::Value,
    
    /// Source information for debugging
    pub source_info: SourceInfo,
    
    /// Loading metrics
    pub metrics: LoadingMetrics,
    
    /// File modification time (for caching)
    pub mtime: Option<std::time::SystemTime>,
}
```

### Provider Registry

```rust
/// Central registry for all configuration providers
pub struct ProviderRegistry {
    /// Registered providers sorted by priority
    providers: Vec<Box<dyn ConfigProvider>>,
    
    /// Provider lookup cache for performance
    cache: DashMap<ConfigSource, ProviderRef>,
    
    /// Provider-specific configuration
    config: ProviderConfig,
}

impl ProviderRegistry {
    /// Create new registry with default providers
    pub fn new() -> Self {
        let mut registry = Self {
            providers: Vec::new(),
            cache: DashMap::new(),
            config: ProviderConfig::default(),
        };
        
        // Register default providers in priority order
        registry.register(Box::new(FileProvider::new()));
        registry.register(Box::new(EnvironmentProvider::new()));
        registry.register(Box::new(HierarchicalProvider::new()));
        registry.register(Box::new(GlobProvider::new()));
        
        registry
    }
    
    /// Register a new provider
    pub fn register(&mut self, provider: Box<dyn ConfigProvider>) {
        self.providers.push(provider);
        
        // Sort by priority (higher priority first)
        self.providers.sort_by(|a, b| b.priority().cmp(&a.priority()));
        
        // Clear cache to force re-resolution
        self.cache.clear();
    }
    
    /// Find appropriate provider for source
    pub fn find_provider(&self, source: &ConfigSource) -> Option<&dyn ConfigProvider> {
        // Check cache first
        if let Some(provider_ref) = self.cache.get(source) {
            return self.providers.get(provider_ref.index).map(|p| p.as_ref());
        }
        
        // Find matching provider
        for (index, provider) in self.providers.iter().enumerate() {
            if provider.can_handle(source) {
                // Cache the result
                self.cache.insert(source.clone(), ProviderRef { index });
                return Some(provider.as_ref());
            }
        }
        
        None
    }
}
```

## File Provider

### High-Performance File Loading

```rust
use memmap2::Mmap;
use std::fs::File;
use std::io::Read;

/// High-performance file provider with memory mapping and caching
pub struct FileProvider {
    /// File content cache with mtime-based invalidation
    cache: Arc<RwLock<HashMap<PathBuf, CachedFile>>>,
    
    /// Configuration for file loading behavior
    config: FileProviderConfig,
}

/// Cached file entry with validation
struct CachedFile {
    /// Parsed configuration data
    data: serde_json::Value,
    
    /// File modification time for cache validation
    mtime: std::time::SystemTime,
    
    /// Source information for this file
    source_info: SourceInfo,
    
    /// Cache creation timestamp
    cached_at: std::time::Instant,
}

impl FileProvider {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            config: FileProviderConfig::default(),
        }
    }
    
    /// Load file with intelligent caching and format detection
    async fn load_file(&self, path: &Path, context: &LoadContext) -> Result<ProviderResult, ProviderError> {
        // Check cache first
        if let Some(cached) = self.check_cache(path).await? {
            return Ok(cached);
        }
        
        // Determine loading strategy based on file size
        let metadata = std::fs::metadata(path)
            .map_err(|e| ProviderError::FileNotFound { path: path.to_path_buf(), source: e })?;
        
        let content = if metadata.len() > self.config.mmap_threshold {
            self.load_with_mmap(path).await?
        } else {
            self.load_in_memory(path).await?
        };
        
        // Detect format and parse
        let format = self.detect_format(path, &content)?;
        let data = self.parse_content(&content, format, context).await?;
        
        // Cache the result
        let result = ProviderResult {
            data: data.clone(),
            source_info: SourceInfo {
                source_type: SourceType::File { format },
                source_id: path.to_string_lossy().to_string(),
                location: None,
                priority: 100,
            },
            metrics: LoadingMetrics::default(),
            mtime: Some(metadata.modified().unwrap_or_else(|_| std::time::SystemTime::now())),
        };
        
        self.update_cache(path, &result).await;
        
        Ok(result)
    }
    
    /// Load file using memory mapping for large files
    async fn load_with_mmap(&self, path: &Path) -> Result<Vec<u8>, ProviderError> {
        let file = File::open(path)
            .map_err(|e| ProviderError::FileReadError { path: path.to_path_buf(), source: e })?;
        
        let mmap = unsafe { Mmap::map(&file) }
            .map_err(|e| ProviderError::MmapError { path: path.to_path_buf(), source: e })?;
        
        Ok(mmap.to_vec())
    }
    
    /// Load file into memory for small files
    async fn load_in_memory(&self, path: &Path) -> Result<Vec<u8>, ProviderError> {
        let mut content = Vec::new();
        let mut file = File::open(path)
            .map_err(|e| ProviderError::FileReadError { path: path.to_path_buf(), source: e })?;
        
        file.read_to_end(&mut content)
            .map_err(|e| ProviderError::FileReadError { path: path.to_path_buf(), source: e })?;
        
        Ok(content)
    }
    
    /// Intelligent format detection using file extension and content analysis
    fn detect_format(&self, path: &Path, content: &[u8]) -> Result<ConfigFormat, ProviderError> {
        // Try extension-based detection first
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "json" => return Ok(ConfigFormat::Json),
                "toml" => return Ok(ConfigFormat::Toml),
                "yaml" | "yml" => return Ok(ConfigFormat::Yaml),
                "env" => return Ok(ConfigFormat::Env),
                _ => {} // Fall through to content-based detection
            }
        }
        
        // Content-based detection for extensionless files
        self.detect_format_by_content(content)
    }
    
    /// Content-based format detection using heuristics
    fn detect_format_by_content(&self, content: &[u8]) -> Result<ConfigFormat, ProviderError> {
        if content.is_empty() {
            return Ok(ConfigFormat::Json); // Default to JSON for empty files
        }
        
        let text = String::from_utf8_lossy(content);
        let trimmed = text.trim();
        
        // JSON detection
        if (trimmed.starts_with('{') && trimmed.ends_with('}')) ||
           (trimmed.starts_with('[') && trimmed.ends_with(']')) {
            return Ok(ConfigFormat::Json);
        }
        
        // YAML detection (starts with ---, has key: value patterns)
        if trimmed.starts_with("---") || 
           text.lines().any(|line| line.trim().contains(": ") && !line.trim_start().starts_with('#')) {
            return Ok(ConfigFormat::Yaml);
        }
        
        // TOML detection (has [section] or key = value patterns)
        if text.lines().any(|line| {
            let line = line.trim();
            line.starts_with('[') && line.ends_with(']') ||
            line.contains('=') && !line.starts_with('#')
        }) {
            return Ok(ConfigFormat::Toml);
        }
        
        // Environment file detection (KEY=value patterns)
        if text.lines().all(|line| {
            let line = line.trim();
            line.is_empty() || line.starts_with('#') || line.contains('=')
        }) {
            return Ok(ConfigFormat::Env);
        }
        
        // Default to JSON if unable to detect
        Ok(ConfigFormat::Json)
    }
}
```

### Format Parsing

```rust
/// Configuration format types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigFormat {
    Json,
    Toml,
    Yaml,
    Env,
}

impl FileProvider {
    /// Parse content based on detected format
    async fn parse_content(
        &self,
        content: &[u8],
        format: ConfigFormat,
        context: &LoadContext,
    ) -> Result<serde_json::Value, ProviderError> {
        let start_time = std::time::Instant::now();
        
        let result = match format {
            ConfigFormat::Json => self.parse_json(content).await,
            ConfigFormat::Toml => self.parse_toml(content).await,
            ConfigFormat::Yaml => self.parse_yaml(content).await,
            ConfigFormat::Env => self.parse_env(content, context).await,
        };
        
        let parse_time = start_time.elapsed();
        if parse_time > Duration::from_millis(10) {
            let warning = ConfigWarning::SlowParsing {
                format,
                duration: parse_time,
                size_bytes: content.len(),
            };
            context.warnings.lock().unwrap().push(warning);
        }
        
        result
    }
    
    /// Parse JSON with optional SIMD acceleration
    async fn parse_json(&self, content: &[u8]) -> Result<serde_json::Value, ProviderError> {
        #[cfg(feature = "simd")]
        {
            simd_json::from_slice(&mut content.to_vec())
                .map_err(|e| ProviderError::ParseError {
                    format: ConfigFormat::Json,
                    source: Box::new(e),
                })
        }
        
        #[cfg(not(feature = "simd"))]
        {
            serde_json::from_slice(content)
                .map_err(|e| ProviderError::ParseError {
                    format: ConfigFormat::Json,
                    source: Box::new(e),
                })
        }
    }
    
    /// Parse TOML with detailed error reporting
    async fn parse_toml(&self, content: &[u8]) -> Result<serde_json::Value, ProviderError> {
        let text = String::from_utf8_lossy(content);
        let toml_value: toml::Value = text.parse()
            .map_err(|e| ProviderError::ParseError {
                format: ConfigFormat::Toml,
                source: Box::new(e),
            })?;
        
        // Convert TOML to JSON for unified handling
        serde_json::to_value(toml_value)
            .map_err(|e| ProviderError::ConversionError {
                from: ConfigFormat::Toml,
                to: ConfigFormat::Json,
                source: Box::new(e),
            })
    }
    
    /// Parse YAML with safe loading
    async fn parse_yaml(&self, content: &[u8]) -> Result<serde_json::Value, ProviderError> {
        let text = String::from_utf8_lossy(content);
        serde_yaml::from_str(&text)
            .map_err(|e| ProviderError::ParseError {
                format: ConfigFormat::Yaml,
                source: Box::new(e),
            })
    }
    
    /// Parse environment file format with variable expansion
    async fn parse_env(&self, content: &[u8], context: &LoadContext) -> Result<serde_json::Value, ProviderError> {
        let text = String::from_utf8_lossy(content);
        let mut result = serde_json::Map::new();
        
        for (line_num, line) in text.lines().enumerate() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Parse KEY=value pairs
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let mut value = value.trim();
                
                // Remove quotes if present
                if (value.starts_with('"') && value.ends_with('"')) ||
                   (value.starts_with('\'') && value.ends_with('\'')) {
                    value = &value[1..value.len() - 1];
                }
                
                // Variable expansion
                let expanded_value = self.expand_env_vars(value, &context.env_vars)?;
                
                // Convert nested keys (KEY__NESTED=value -> {"KEY": {"NESTED": value}})
                self.insert_nested_key(&mut result, key, &expanded_value);
            } else {
                let warning = ConfigWarning::InvalidEnvLine {
                    line_number: line_num + 1,
                    content: line.to_string(),
                };
                context.warnings.lock().unwrap().push(warning);
            }
        }
        
        Ok(serde_json::Value::Object(result))
    }
    
    /// Expand environment variables in values
    fn expand_env_vars(&self, value: &str, env_vars: &HashMap<String, String>) -> Result<String, ProviderError> {
        let mut result = value.to_string();
        
        // Simple variable expansion: ${VAR} or $VAR
        let var_pattern = regex::Regex::new(r"\$\{([^}]+)\}|\$([A-Z_][A-Z0-9_]*)")
            .map_err(|e| ProviderError::RegexError(e))?;
        
        for captures in var_pattern.captures_iter(value) {
            let var_name = captures.get(1).or_else(|| captures.get(2)).unwrap().as_str();
            let full_match = captures.get(0).unwrap().as_str();
            
            if let Some(var_value) = env_vars.get(var_name) {
                result = result.replace(full_match, var_value);
            } else {
                // Leave unexpanded if variable not found
                continue;
            }
        }
        
        Ok(result)
    }
    
    /// Insert nested key into JSON object (KEY__NESTED -> {"KEY": {"NESTED": value}})
    fn insert_nested_key(&self, object: &mut serde_json::Map<String, serde_json::Value>, key: &str, value: &str) {
        let parts: Vec<&str> = key.split("__").collect();
        
        if parts.len() == 1 {
            // Simple key
            object.insert(key.to_string(), serde_json::Value::String(value.to_string()));
        } else {
            // Nested key
            let mut current = object;
            
            for (i, part) in parts.iter().enumerate() {
                if i == parts.len() - 1 {
                    // Last part - insert the value
                    current.insert(part.to_string(), serde_json::Value::String(value.to_string()));
                } else {
                    // Intermediate part - ensure nested object exists
                    let entry = current.entry(part.to_string()).or_insert_with(|| {
                        serde_json::Value::Object(serde_json::Map::new())
                    });
                    
                    if let serde_json::Value::Object(obj) = entry {
                        current = obj;
                    } else {
                        // Conflict - existing value is not an object
                        // Convert to object and preserve old value as "_value"
                        let old_value = entry.clone();
                        *entry = serde_json::Value::Object(serde_json::Map::new());
                        if let serde_json::Value::Object(obj) = entry {
                            obj.insert("_value".to_string(), old_value);
                            current = obj;
                        }
                    }
                }
            }
        }
    }
}
```

## Environment Provider

### Advanced Environment Variable Processing

```rust
/// Environment variable provider with prefix support and nesting
pub struct EnvironmentProvider {
    /// Configuration for environment processing
    config: EnvironmentProviderConfig,
}

#[derive(Debug, Clone)]
pub struct EnvironmentProviderConfig {
    /// Environment variable prefix to filter by
    pub prefix: Option<String>,
    
    /// Separator for nested keys (default: "__")
    pub separator: String,
    
    /// Whether to preserve case or convert to lowercase
    pub preserve_case: bool,
    
    /// Whether to strip prefix from keys
    pub strip_prefix: bool,
}

impl EnvironmentProvider {
    pub fn new() -> Self {
        Self {
            config: EnvironmentProviderConfig {
                prefix: None,
                separator: "__".to_string(),
                preserve_case: false,
                strip_prefix: true,
            }
        }
    }
    
    pub fn with_prefix(mut self, prefix: String) -> Self {
        self.config.prefix = Some(prefix);
        self
    }
    
    /// Load environment variables into nested configuration
    async fn load_env(&self, context: &LoadContext) -> Result<ProviderResult, ProviderError> {
        let mut result = serde_json::Map::new();
        
        for (key, value) in &context.env_vars {
            if let Some(ref prefix) = self.config.prefix {
                if !key.starts_with(prefix) {
                    continue;
                }
            }
            
            let processed_key = if let Some(ref prefix) = self.config.prefix {
                if self.config.strip_prefix {
                    key.strip_prefix(prefix)
                        .and_then(|k| k.strip_prefix('_'))
                        .unwrap_or(key)
                } else {
                    key
                }
            } else {
                key
            };
            
            let final_key = if self.config.preserve_case {
                processed_key.to_string()
            } else {
                processed_key.to_lowercase()
            };
            
            self.insert_nested_env_key(&mut result, &final_key, value);
        }
        
        Ok(ProviderResult {
            data: serde_json::Value::Object(result),
            source_info: SourceInfo {
                source_type: SourceType::Environment { 
                    prefix: self.config.prefix.clone() 
                },
                source_id: "environment".to_string(),
                location: None,
                priority: 80,
            },
            metrics: LoadingMetrics::default(),
            mtime: None,
        })
    }
    
    /// Insert environment variable with nested key support
    fn insert_nested_env_key(&self, object: &mut serde_json::Map<String, serde_json::Value>, key: &str, value: &str) {
        let parts: Vec<&str> = key.split(&self.config.separator).collect();
        
        if parts.len() == 1 {
            // Simple key - try to parse as JSON first, fall back to string
            let parsed_value = serde_json::from_str(value)
                .unwrap_or_else(|_| serde_json::Value::String(value.to_string()));
            
            object.insert(key.to_string(), parsed_value);
        } else {
            // Nested key
            let mut current = object;
            
            for (i, part) in parts.iter().enumerate() {
                if i == parts.len() - 1 {
                    // Last part - insert the value
                    let parsed_value = serde_json::from_str(value)
                        .unwrap_or_else(|_| serde_json::Value::String(value.to_string()));
                    
                    current.insert(part.to_string(), parsed_value);
                } else {
                    // Intermediate part - ensure nested object exists
                    let entry = current.entry(part.to_string()).or_insert_with(|| {
                        serde_json::Value::Object(serde_json::Map::new())
                    });
                    
                    if let serde_json::Value::Object(obj) = entry {
                        current = obj;
                    }
                }
            }
        }
    }
}
```

## Hierarchical Provider

### Git-Style Configuration Discovery

```rust
use std::path::{Path, PathBuf};

/// Hierarchical provider for system → user → project configuration discovery
pub struct HierarchicalProvider {
    /// Configuration for discovery behavior
    config: HierarchicalConfig,
}

#[derive(Debug, Clone)]
pub struct HierarchicalConfig {
    /// Application name for configuration files
    pub app_name: String,
    
    /// Configuration file names to search for
    pub config_names: Vec<String>,
    
    /// Whether to merge configs or use first found
    pub merge_configs: bool,
    
    /// Search directories in order of priority
    pub search_dirs: Vec<SearchDir>,
}

#[derive(Debug, Clone)]
pub enum SearchDir {
    /// System-wide configuration directory (/etc, etc.)
    System,
    
    /// User home directory (~/.config, ~/.appname)
    UserHome,
    
    /// User config directory (XDG_CONFIG_HOME)
    UserConfig,
    
    /// Current working directory
    CurrentDir,
    
    /// Project root (nearest .git, package.json, etc.)
    ProjectRoot,
    
    /// Custom directory path
    Custom(PathBuf),
}

impl HierarchicalProvider {
    pub fn new(app_name: String) -> Self {
        Self {
            config: HierarchicalConfig {
                app_name: app_name.clone(),
                config_names: vec![
                    format!("{}.json", app_name),
                    format!("{}.toml", app_name),
                    format!("{}.yaml", app_name),
                    format!(".{}rc", app_name),
                    "config.json".to_string(),
                    "config.toml".to_string(),
                ],
                merge_configs: true,
                search_dirs: vec![
                    SearchDir::System,
                    SearchDir::UserHome,
                    SearchDir::UserConfig,
                    SearchDir::ProjectRoot,
                    SearchDir::CurrentDir,
                ],
            }
        }
    }
    
    /// Discover configuration files in hierarchical order
    async fn discover_configs(&self, context: &LoadContext) -> Result<Vec<PathBuf>, ProviderError> {
        let mut found_configs = Vec::new();
        
        for search_dir in &self.config.search_dirs {
            let dir_path = self.resolve_search_dir(search_dir, context)?;
            
            if let Some(dir) = dir_path {
                for config_name in &self.config.config_names {
                    let config_path = dir.join(config_name);
                    
                    if config_path.exists() && config_path.is_file() {
                        found_configs.push(config_path);
                        
                        // If not merging, stop at first found config in each directory
                        if !self.config.merge_configs {
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(found_configs)
    }
    
    /// Resolve search directory to actual path
    fn resolve_search_dir(&self, search_dir: &SearchDir, context: &LoadContext) -> Result<Option<PathBuf>, ProviderError> {
        match search_dir {
            SearchDir::System => {
                // Try common system config directories
                let system_dirs = [
                    PathBuf::from("/etc"),
                    PathBuf::from("/usr/local/etc"),
                ];
                
                for dir in &system_dirs {
                    let app_dir = dir.join(&self.config.app_name);
                    if app_dir.exists() {
                        return Ok(Some(app_dir));
                    }
                    if dir.exists() {
                        return Ok(Some(dir.clone()));
                    }
                }
                Ok(None)
            },
            
            SearchDir::UserHome => {
                if let Some(home_dir) = dirs::home_dir() {
                    let app_config_dir = home_dir.join(format!(".{}", self.config.app_name));
                    if app_config_dir.exists() {
                        Ok(Some(app_config_dir))
                    } else {
                        Ok(Some(home_dir))
                    }
                } else {
                    Ok(None)
                }
            },
            
            SearchDir::UserConfig => {
                if let Some(config_dir) = dirs::config_dir() {
                    let app_config_dir = config_dir.join(&self.config.app_name);
                    if app_config_dir.exists() {
                        Ok(Some(app_config_dir))
                    } else {
                        Ok(Some(config_dir))
                    }
                } else {
                    Ok(None)
                }
            },
            
            SearchDir::CurrentDir => {
                Ok(Some(context.base_dir.clone()))
            },
            
            SearchDir::ProjectRoot => {
                self.find_project_root(&context.base_dir)
            },
            
            SearchDir::Custom(path) => {
                Ok(if path.exists() { Some(path.clone()) } else { None })
            }
        }
    }
    
    /// Find project root by looking for common project markers
    fn find_project_root(&self, start_dir: &Path) -> Result<Option<PathBuf>, ProviderError> {
        let project_markers = [
            ".git",
            "package.json",
            "Cargo.toml",
            "pyproject.toml",
            "pom.xml",
            "build.gradle",
            ".project",
        ];
        
        let mut current_dir = start_dir;
        
        loop {
            for marker in &project_markers {
                let marker_path = current_dir.join(marker);
                if marker_path.exists() {
                    return Ok(Some(current_dir.to_path_buf()));
                }
            }
            
            // Move up to parent directory
            if let Some(parent) = current_dir.parent() {
                current_dir = parent;
            } else {
                // Reached filesystem root
                break;
            }
        }
        
        Ok(None)
    }
}
```

## Glob Provider

### High-Performance Pattern Matching

```rust
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::PathBuf;

/// Glob provider for wildcard-based file discovery
pub struct GlobProvider {
    /// Compiled glob patterns for efficient matching
    globset: Option<GlobSet>,
    
    /// Original patterns for debugging
    patterns: Vec<String>,
    
    /// Provider configuration
    config: GlobConfig,
}

#[derive(Debug, Clone)]
pub struct GlobConfig {
    /// Base directory for relative patterns
    pub base_dir: PathBuf,
    
    /// Whether to follow symbolic links
    pub follow_links: bool,
    
    /// Maximum recursion depth
    pub max_depth: Option<usize>,
    
    /// File size limit for globbed files
    pub max_file_size: Option<u64>,
    
    /// Parallel loading threshold (number of files)
    pub parallel_threshold: usize,
}

impl GlobProvider {
    pub fn new() -> Self {
        Self {
            globset: None,
            patterns: Vec::new(),
            config: GlobConfig {
                base_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
                follow_links: false,
                max_depth: Some(10),
                max_file_size: Some(10 * 1024 * 1024), // 10MB
                parallel_threshold: 3,
            }
        }
    }
    
    pub fn with_patterns(mut self, patterns: Vec<String>) -> Result<Self, ProviderError> {
        self.patterns = patterns;
        self.compile_patterns()?;
        Ok(self)
    }
    
    /// Compile glob patterns for efficient matching
    fn compile_patterns(&mut self) -> Result<(), ProviderError> {
        let mut builder = GlobSetBuilder::new();
        
        for pattern in &self.patterns {
            let glob = Glob::new(pattern)
                .map_err(|e| ProviderError::InvalidGlobPattern { 
                    pattern: pattern.clone(), 
                    error: e.to_string() 
                })?;
            builder.add(glob);
        }
        
        self.globset = Some(builder.build()
            .map_err(|e| ProviderError::GlobCompileError(e.to_string()))?);
        
        Ok(())
    }
    
    /// Find all files matching glob patterns
    async fn find_matching_files(&self, context: &LoadContext) -> Result<Vec<PathBuf>, ProviderError> {
        let globset = self.globset.as_ref()
            .ok_or(ProviderError::GlobNotCompiled)?;
        
        let mut matching_files = Vec::new();
        
        // Walk directory tree and collect matching files
        let walker = walkdir::WalkDir::new(&self.config.base_dir)
            .follow_links(self.config.follow_links)
            .max_depth(self.config.max_depth.unwrap_or(usize::MAX));
        
        for entry in walker {
            let entry = entry.map_err(|e| ProviderError::WalkDirError(e.to_string()))?;
            
            if !entry.file_type().is_file() {
                continue;
            }
            
            let path = entry.path();
            
            // Check file size limit
            if let Some(max_size) = self.config.max_file_size {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.len() > max_size {
                        let warning = ConfigWarning::FileTooLarge {
                            path: path.to_path_buf(),
                            size: metadata.len(),
                            limit: max_size,
                        };
                        context.warnings.lock().unwrap().push(warning);
                        continue;
                    }
                }
            }
            
            // Test against glob patterns
            if globset.is_match(path) {
                matching_files.push(path.to_path_buf());
            }
        }
        
        // Sort files for deterministic ordering
        matching_files.sort();
        
        Ok(matching_files)
    }
    
    /// Load multiple files in parallel if above threshold
    async fn load_multiple_files(&self, files: Vec<PathBuf>, context: &LoadContext) -> Result<ProviderResult, ProviderError> {
        if files.len() >= self.config.parallel_threshold {
            self.load_files_parallel(files, context).await
        } else {
            self.load_files_sequential(files, context).await
        }
    }
    
    /// Load files in parallel using rayon
    #[cfg(feature = "parallel")]
    async fn load_files_parallel(&self, files: Vec<PathBuf>, context: &LoadContext) -> Result<ProviderResult, ProviderError> {
        use rayon::prelude::*;
        
        let file_provider = FileProvider::new();
        
        // Load files in parallel
        let results: Result<Vec<_>, _> = files.par_iter()
            .map(|file| {
                // Create async runtime for each thread
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(file_provider.load_file(file, context))
            })
            .collect();
        
        let mut loaded_results = results?;
        
        // Merge all configurations
        let merged_data = self.merge_configurations(&mut loaded_results)?;
        
        Ok(ProviderResult {
            data: merged_data,
            source_info: SourceInfo {
                source_type: SourceType::Glob { 
                    patterns: self.patterns.clone(),
                    file_count: files.len(),
                },
                source_id: "glob".to_string(),
                location: None,
                priority: 60,
            },
            metrics: LoadingMetrics::default(),
            mtime: None,
        })
    }
    
    /// Load files sequentially
    async fn load_files_sequential(&self, files: Vec<PathBuf>, context: &LoadContext) -> Result<ProviderResult, ProviderError> {
        let file_provider = FileProvider::new();
        let mut loaded_results = Vec::new();
        
        for file in files {
            match file_provider.load_file(&file, context).await {
                Ok(result) => loaded_results.push(result),
                Err(e) => {
                    let warning = ConfigWarning::FileLoadFailed {
                        path: file,
                        error: e.to_string(),
                    };
                    context.warnings.lock().unwrap().push(warning);
                }
            }
        }
        
        // Merge all configurations
        let merged_data = self.merge_configurations(&mut loaded_results)?;
        
        Ok(ProviderResult {
            data: merged_data,
            source_info: SourceInfo {
                source_type: SourceType::Glob { 
                    patterns: self.patterns.clone(),
                    file_count: loaded_results.len(),
                },
                source_id: "glob".to_string(),
                location: None,
                priority: 60,
            },
            metrics: LoadingMetrics::default(),
            mtime: None,
        })
    }
    
    /// Merge multiple configuration results using merge engine
    fn merge_configurations(&self, results: &mut [ProviderResult]) -> Result<serde_json::Value, ProviderError> {
        if results.is_empty() {
            return Ok(serde_json::Value::Object(serde_json::Map::new()));
        }
        
        if results.len() == 1 {
            return Ok(results[0].data.clone());
        }
        
        // Sort by source priority (higher priority first)
        results.sort_by(|a, b| b.source_info.priority.cmp(&a.source_info.priority));
        
        // Start with the highest priority configuration
        let mut merged = results[0].data.clone();
        
        // Merge remaining configurations
        for result in &results[1..] {
            merged = self.deep_merge_values(merged, result.data.clone())?;
        }
        
        Ok(merged)
    }
    
    /// Deep merge two JSON values with array merge support
    fn deep_merge_values(&self, mut target: serde_json::Value, source: serde_json::Value) -> Result<serde_json::Value, ProviderError> {
        match (&mut target, source) {
            (serde_json::Value::Object(target_obj), serde_json::Value::Object(source_obj)) => {
                for (key, source_value) in source_obj {
                    if let Some(target_value) = target_obj.get_mut(&key) {
                        *target_value = self.deep_merge_values(target_value.clone(), source_value)?;
                    } else {
                        target_obj.insert(key, source_value);
                    }
                }
                Ok(target)
            },
            (serde_json::Value::Array(target_arr), serde_json::Value::Array(source_arr)) => {
                // Array merge strategy: append source to target
                target_arr.extend(source_arr);
                Ok(target)
            },
            (_, source) => {
                // Source overrides target for non-object values
                Ok(source)
            }
        }
    }
}
```

## Provider Error Handling

### Comprehensive Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf, source: std::io::Error },
    
    #[error("Failed to read file: {path}")]
    FileReadError { path: PathBuf, source: std::io::Error },
    
    #[error("Memory mapping failed for file: {path}")]
    MmapError { path: PathBuf, source: std::io::Error },
    
    #[error("Parse error in {format:?} format")]
    ParseError { format: ConfigFormat, source: Box<dyn std::error::Error + Send + Sync> },
    
    #[error("Format conversion error from {from:?} to {to:?}")]
    ConversionError { 
        from: ConfigFormat, 
        to: ConfigFormat, 
        source: Box<dyn std::error::Error + Send + Sync> 
    },
    
    #[error("Invalid glob pattern: {pattern} - {error}")]
    InvalidGlobPattern { pattern: String, error: String },
    
    #[error("Failed to compile glob patterns: {0}")]
    GlobCompileError(String),
    
    #[error("Glob patterns not compiled")]
    GlobNotCompiled,
    
    #[error("Directory walk error: {0}")]
    WalkDirError(String),
    
    #[error("Regular expression error")]
    RegexError(#[from] regex::Error),
    
    #[error("Provider not found for source: {source:?}")]
    ProviderNotFound { source: ConfigSource },
    
    #[error("Provider operation timed out after {timeout_ms}ms")]
    ProviderTimeout { timeout_ms: u64 },
}
```

## Configuration Sources

### Source Type Definitions

```rust
/// Configuration source types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConfigSource {
    /// Single file source
    File { path: PathBuf },
    
    /// Directory source (searches for config files)
    Directory { path: PathBuf },
    
    /// Environment variables with optional prefix
    Environment { prefix: Option<String> },
    
    /// Glob pattern for multiple files
    Glob { patterns: Vec<String> },
    
    /// Hierarchical discovery starting from base directory
    Hierarchical { app_name: String, base_dir: PathBuf },
    
    /// URL source (future extension)
    Url { url: String },
    
    /// Custom source (user-defined provider)
    Custom { provider_id: String, config: serde_json::Value },
}

/// Loading options and feature flags
#[derive(Debug, Clone)]
pub struct LoadOptions {
    /// Whether to merge configurations or use first found
    pub merge_strategy: MergeStrategy,
    
    /// Profile to select during loading
    pub profile: Option<String>,
    
    /// Whether to validate schema during loading
    pub validate_schema: bool,
    
    /// Loading timeout in milliseconds
    pub timeout_ms: Option<u64>,
    
    /// Cache behavior
    pub cache_behavior: CacheBehavior,
    
    /// Feature flags
    pub features: LoadFeatures,
}

#[derive(Debug, Clone)]
pub enum MergeStrategy {
    /// Replace target with source (last wins)
    Replace,
    
    /// Deep merge objects, replace arrays
    DeepMerge,
    
    /// Deep merge objects, append arrays
    DeepMergeAppendArrays,
    
    /// Deep merge objects, merge arrays by index
    DeepMergeMergeArrays,
}

#[derive(Debug, Clone)]
pub struct LoadFeatures {
    /// Enable SIMD acceleration
    pub simd: bool,
    
    /// Enable parallel loading
    pub parallel: bool,
    
    /// Enable hot reload watching
    pub hot_reload: bool,
    
    /// Enable performance profiling
    pub profiling: bool,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            merge_strategy: MergeStrategy::DeepMerge,
            profile: None,
            validate_schema: false,
            timeout_ms: None,
            cache_behavior: CacheBehavior::Smart,
            features: LoadFeatures {
                simd: cfg!(feature = "simd"),
                parallel: cfg!(feature = "parallel"),
                hot_reload: cfg!(feature = "hot-reload"),
                profiling: cfg!(feature = "profiling"),
            },
        }
    }
}
```

## Performance Optimization

### Caching Strategy

```rust
#[derive(Debug, Clone)]
pub enum CacheBehavior {
    /// No caching
    None,
    
    /// Smart caching based on mtime
    Smart,
    
    /// Aggressive caching (ignore mtime)
    Aggressive,
    
    /// Custom cache duration
    Duration(std::time::Duration),
}

/// File cache with mtime-based invalidation
pub struct FileCache {
    /// Cache entries with modification time validation
    entries: Arc<RwLock<HashMap<PathBuf, CacheEntry>>>,
    
    /// Cache configuration
    config: CacheConfig,
}

struct CacheEntry {
    /// Cached configuration data
    data: serde_json::Value,
    
    /// File modification time when cached
    mtime: std::time::SystemTime,
    
    /// Cache entry creation time
    cached_at: std::time::Instant,
    
    /// Source information
    source_info: SourceInfo,
}

impl FileCache {
    /// Get cached data if still valid
    pub fn get(&self, path: &Path) -> Option<ProviderResult> {
        let entries = self.entries.read().ok()?;
        let entry = entries.get(path)?;
        
        // Check if cache is still valid
        if self.is_cache_valid(path, entry) {
            Some(ProviderResult {
                data: entry.data.clone(),
                source_info: entry.source_info.clone(),
                metrics: LoadingMetrics::default(),
                mtime: Some(entry.mtime),
            })
        } else {
            None
        }
    }
    
    /// Check if cache entry is still valid
    fn is_cache_valid(&self, path: &Path, entry: &CacheEntry) -> bool {
        match self.config.behavior {
            CacheBehavior::None => false,
            CacheBehavior::Aggressive => true,
            CacheBehavior::Smart => {
                // Check file modification time
                if let Ok(metadata) = std::fs::metadata(path) {
                    if let Ok(mtime) = metadata.modified() {
                        return mtime <= entry.mtime;
                    }
                }
                false
            },
            CacheBehavior::Duration(duration) => {
                entry.cached_at.elapsed() < duration
            }
        }
    }
}
```

## Next Steps

This provider system design establishes the comprehensive configuration loading infrastructure for SuperConfig V2. The next documents will detail:

- **08-ffi-integration-plan.md**: FFI wrapper patterns and binding generation for Python/Node.js
- **09-performance-optimization-strategy.md**: SIMD acceleration, caching strategies, and advanced performance techniques
- **10-testing-and-benchmarking-plan.md**: Comprehensive testing approach and performance validation

The provider system achieves target performance through:

- Memory-mapped file loading for large configurations
- Intelligent caching with mtime-based invalidation
- Parallel loading for glob patterns (3+ files)
- SIMD-accelerated parsing where available
- Lock-free registry integration for zero-copy access
