# SuperConfig V2: API Design Reference

## Overview

This document provides complete API specifications for SuperConfig V2 across all target languages. The API design prioritizes consistency, performance, and language-specific conventions while maintaining feature parity across platforms.

## Core Design Principles

### Unified Handle-Based Architecture

- **Zero-Copy Access**: Handles reference internal data without duplication
- **Type Safety**: Strong typing prevents runtime errors across all languages
- **Consistent Semantics**: Same behavior regardless of language binding
- **Performance First**: Sub-microsecond access patterns for all operations

### Language-Specific Conventions

- **Rust**: Native ownership patterns with `&str` and `&[T]` references
- **Python**: Snake_case naming with Pythonic error handling
- **Node.js**: CamelCase conversion with Promise-based async patterns
- **WebAssembly**: Browser-optimized with automatic memory management

## Rust Core API

### Primary Configuration Interface

```rust
// Core configuration manager
pub struct SuperConfig {
    registry: HandleRegistry,
    providers: Vec<Box<dyn Provider>>,
    cache: ConfigCache,
}

impl SuperConfig {
    // Configuration loading
    pub fn new() -> Result<Self, ConfigError> { /* ... */ }
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> { /* ... */ }
    pub fn from_env() -> Result<Self, ConfigError> { /* ... */ }
    pub fn from_providers(providers: Vec<Box<dyn Provider>>) -> Result<Self, ConfigError> { /* ... */ }
    
    // Handle management
    pub fn get_handle<T>(&self, key: &str) -> Result<Handle<T>, ConfigError>
    where T: ConfigValue + 'static { /* ... */ }
    
    pub fn get_handle_with_default<T>(&self, key: &str, default: T) -> Handle<T>
    where T: ConfigValue + Clone + 'static { /* ... */ }
    
    // Direct access (higher overhead)
    pub fn get<T>(&self, key: &str) -> Result<T, ConfigError>
    where T: ConfigValue { /* ... */ }
    
    pub fn set<T>(&mut self, key: &str, value: T) -> Result<(), ConfigError>
    where T: ConfigValue { /* ... */ }
    
    // Bulk operations
    pub fn merge(&mut self, other: SuperConfig) -> Result<(), ConfigError> { /* ... */ }
    pub fn extract_section(&self, prefix: &str) -> Result<SuperConfig, ConfigError> { /* ... */ }
    
    // Hot reload
    pub fn enable_hot_reload(&mut self) -> Result<HotReloadHandle, ConfigError> { /* ... */ }
    pub fn reload(&mut self) -> Result<Vec<String>, ConfigError> { /* ... */ }
}

// Zero-copy value handle
pub struct Handle<T> {
    registry_ref: &'static HandleRegistry,
    handle_id: HandleId,
    _phantom: PhantomData<T>,
}

impl<T: ConfigValue> Handle<T> {
    pub fn get(&self) -> &T { /* Sub-microsecond access */ }
    pub fn is_valid(&self) -> bool { /* ... */ }
    pub fn key(&self) -> &str { /* ... */ }
    pub fn source(&self) -> &str { /* ... */ }
}

// Configuration value types
pub trait ConfigValue: Send + Sync + Clone + Debug {
    fn from_config_data(data: &ConfigData) -> Result<Self, ConfigError>;
    fn to_config_data(&self) -> ConfigData;
}

// Built-in implementations
impl ConfigValue for String { /* ... */ }
impl ConfigValue for i32 { /* ... */ }
impl ConfigValue for f64 { /* ... */ }
impl ConfigValue for bool { /* ... */ }
impl<T: ConfigValue> ConfigValue for Vec<T> { /* ... */ }
impl<T: ConfigValue> ConfigValue for HashMap<String, T> { /* ... */ }

// Custom types
#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub ssl: bool,
}

impl ConfigValue for DatabaseConfig { /* ... */ }
```

### Error Handling

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    KeyNotFound { key: String },
    TypeMismatch { key: String, expected: String, found: String },
    ParseError { key: String, source: String, error: String },
    ProviderError { provider: String, error: String },
    ValidationError { key: String, constraint: String },
    IoError { path: String, error: String },
    HotReloadError { error: String },
}

impl std::error::Error for ConfigError { /* ... */ }
impl Display for ConfigError { /* ... */ }
```

### Provider System

```rust
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn load(&self) -> Result<ConfigData, ConfigError>;
    fn watch(&self) -> Result<Box<dyn Stream<Item = ConfigEvent>>, ConfigError>;
    fn supports_hot_reload(&self) -> bool { false }
}

// Built-in providers
pub struct FileProvider {
    path: PathBuf,
    format: FileFormat,
}

pub struct EnvironmentProvider {
    prefix: Option<String>,
    separator: String,
}

pub struct HierarchicalProvider {
    providers: Vec<Box<dyn Provider>>,
    merge_strategy: MergeStrategy,
}

// Provider builders
impl FileProvider {
    pub fn new<P: AsRef<Path>>(path: P) -> Self { /* ... */ }
    pub fn with_format(mut self, format: FileFormat) -> Self { /* ... */ }
    pub fn with_hot_reload(mut self) -> Self { /* ... */ }
}

impl EnvironmentProvider {
    pub fn new() -> Self { /* ... */ }
    pub fn with_prefix<S: Into<String>>(mut self, prefix: S) -> Self { /* ... */ }
    pub fn with_separator<S: Into<String>>(mut self, separator: S) -> Self { /* ... */ }
}
```

## Python API (superconfig-py)

### Installation and Import

```python
# Installation
pip install superconfig

# Import
from superconfig import SuperConfig, ConfigError, Handle
```

### Core Interface

```python
class SuperConfig:
    """High-performance configuration manager with zero-copy access."""
    
    def __init__(self) -> None:
        """Create empty configuration."""
    
    @classmethod
    def from_file(cls, path: str) -> 'SuperConfig':
        """Load configuration from file."""
    
    @classmethod
    def from_env(cls, prefix: str = None) -> 'SuperConfig':
        """Load configuration from environment variables."""
    
    @classmethod
    def from_providers(cls, providers: List[Provider]) -> 'SuperConfig':
        """Load configuration from custom providers."""
    
    def get_handle(self, key: str, type_hint: Type[T] = None) -> Handle[T]:
        """Get zero-copy handle to configuration value."""
    
    def get_handle_with_default(self, key: str, default: T) -> Handle[T]:
        """Get handle with fallback default value."""
    
    def get(self, key: str, type_hint: Type[T] = None) -> T:
        """Get configuration value (creates copy)."""
    
    def set(self, key: str, value: Any) -> None:
        """Set configuration value."""
    
    def merge(self, other: 'SuperConfig') -> None:
        """Merge another configuration."""
    
    def extract_section(self, prefix: str) -> 'SuperConfig':
        """Extract configuration section."""
    
    def enable_hot_reload(self) -> HotReloadHandle:
        """Enable file system watching for hot reload."""
    
    def reload(self) -> List[str]:
        """Manually reload configuration from sources."""

class Handle:
    """Zero-copy reference to configuration value."""
    
    def get(self) -> T:
        """Get current value (sub-microsecond access)."""
    
    def is_valid(self) -> bool:
        """Check if handle is still valid."""
    
    @property
    def key(self) -> str:
        """Configuration key."""
    
    @property
    def source(self) -> str:
        """Source provider name."""

# Type annotations
from typing import TypeVar, Generic, Union, Dict, List, Any

T = TypeVar('T')

# Supported types
ConfigValue = Union[
    str, int, float, bool,
    List[Any], Dict[str, Any],
    # Custom types with proper serialization
]
```

### Error Handling

```python
class ConfigError(Exception):
    """Base configuration error."""
    
    def __init__(self, message: str, key: str = None, source: str = None):
        super().__init__(message)
        self.key = key
        self.source = source

class KeyNotFoundError(ConfigError):
    """Configuration key not found."""

class TypeMismatchError(ConfigError):
    """Type conversion failed."""

class ValidationError(ConfigError):
    """Configuration validation failed."""
```

### Usage Examples

```python
# Basic usage
config = SuperConfig.from_file("app.toml")

# Zero-copy handles (preferred for hot paths)
db_host_handle = config.get_handle("database.host", str)
db_port_handle = config.get_handle("database.port", int)

# Access values (sub-microsecond)
host = db_host_handle.get()  # ~0.1μs
port = db_port_handle.get()  # ~0.1μs

# Direct access (higher overhead)
debug = config.get("app.debug", bool)  # ~1-2μs

# Hot reload
hot_reload = config.enable_hot_reload()

def on_config_change(changed_keys: List[str]):
    print(f"Configuration changed: {changed_keys}")

hot_reload.on_change(on_config_change)

# Custom types
from dataclasses import dataclass

@dataclass
class DatabaseConfig:
    host: str
    port: int
    database: str
    ssl: bool = False

db_config = config.get("database", DatabaseConfig)
```

## Node.js API (superconfig-napi)

### Installation and Import

```javascript
// Installation
npm install @superconfig/core

// Import (ESM)
import { SuperConfig, ConfigError } from '@superconfig/core';

// Import (CommonJS)
const { SuperConfig, ConfigError } = require('@superconfig/core');
```

### Core Interface

```typescript
class SuperConfig {
  /**
   * Create empty configuration
   */
  constructor();
  
  /**
   * Load configuration from file
   */
  static fromFile(path: string): Promise<SuperConfig>;
  
  /**
   * Load configuration from environment variables
   */
  static fromEnv(prefix?: string): Promise<SuperConfig>;
  
  /**
   * Load configuration from custom providers
   */
  static fromProviders(providers: Provider[]): Promise<SuperConfig>;
  
  /**
   * Get zero-copy handle to configuration value
   */
  getHandle<T>(key: string): Handle<T>;
  
  /**
   * Get handle with fallback default value
   */
  getHandleWithDefault<T>(key: string, defaultValue: T): Handle<T>;
  
  /**
   * Get configuration value (creates copy)
   */
  get<T>(key: string): T;
  
  /**
   * Set configuration value
   */
  set<T>(key: string, value: T): void;
  
  /**
   * Merge another configuration
   */
  merge(other: SuperConfig): void;
  
  /**
   * Extract configuration section
   */
  extractSection(prefix: string): SuperConfig;
  
  /**
   * Enable file system watching for hot reload
   */
  enableHotReload(): Promise<HotReloadHandle>;
  
  /**
   * Manually reload configuration from sources
   */
  reload(): Promise<string[]>;
}

class Handle<T> {
  /**
   * Get current value (sub-microsecond access)
   */
  get(): T;
  
  /**
   * Check if handle is still valid
   */
  isValid(): boolean;
  
  /**
   * Configuration key
   */
  readonly key: string;
  
  /**
   * Source provider name
   */
  readonly source: string;
}

// Type definitions
type ConfigValue = 
  | string 
  | number 
  | boolean 
  | ConfigValue[] 
  | { [key: string]: ConfigValue }
  | null;

interface Provider {
  name: string;
  load(): Promise<Record<string, ConfigValue>>;
  watch?(): AsyncIterable<ConfigEvent>;
}

interface HotReloadHandle {
  onChange(callback: (changedKeys: string[]) => void): void;
  stop(): void;
}
```

### Error Handling

```typescript
class ConfigError extends Error {
  constructor(
    message: string, 
    public readonly key?: string,
    public readonly source?: string
  ) {
    super(message);
    this.name = 'ConfigError';
  }
}

class KeyNotFoundError extends ConfigError {
  constructor(key: string) {
    super(`Configuration key not found: ${key}`, key);
    this.name = 'KeyNotFoundError';
  }
}

class TypeMismatchError extends ConfigError {
  constructor(key: string, expected: string, received: string) {
    super(`Type mismatch for key '${key}': expected ${expected}, got ${received}`, key);
    this.name = 'TypeMismatchError';
  }
}
```

### Usage Examples

```typescript
// Basic usage
const config = await SuperConfig.fromFile('app.json');

// Zero-copy handles (preferred for hot paths)
const dbHostHandle = config.getHandle<string>('database.host');
const dbPortHandle = config.getHandle<number>('database.port');

// Access values (sub-microsecond)
const host = dbHostHandle.get(); // ~0.2μs
const port = dbPortHandle.get(); // ~0.2μs

// Direct access (higher overhead)
const debug = config.get<boolean>('app.debug'); // ~2-3μs

// Hot reload
const hotReload = await config.enableHotReload();

hotReload.onChange((changedKeys: string[]) => {
  console.log(`Configuration changed: ${changedKeys.join(', ')}`);
});

// Custom types with validation
interface DatabaseConfig {
  host: string;
  port: number;
  database: string;
  ssl?: boolean;
}

const dbConfig = config.get<DatabaseConfig>('database');

// Async providers
class RemoteProvider implements Provider {
  name = 'remote';
  
  async load(): Promise<Record<string, ConfigValue>> {
    const response = await fetch('https://config.example.com/app.json');
    return response.json();
  }
  
  async* watch(): AsyncIterable<ConfigEvent> {
    // WebSocket or polling implementation
    const ws = new WebSocket('wss://config.example.com/watch');
    
    while (true) {
      yield await new Promise(resolve => {
        ws.onmessage = (event) => resolve(JSON.parse(event.data));
      });
    }
  }
}

const remoteConfig = await SuperConfig.fromProviders([
  new RemoteProvider()
]);
```

## WebAssembly API

### Installation and Import

```javascript
// Installation
npm install @superconfig/wasm

// Import (Browser ESM)
import init, { SuperConfig } from '@superconfig/wasm';

// Initialize WASM module
await init();
```

### Browser-Optimized Interface

```typescript
class SuperConfig {
  /**
   * Create empty configuration
   */
  constructor();
  
  /**
   * Load configuration from URL
   */
  static fromUrl(url: string): Promise<SuperConfig>;
  
  /**
   * Load configuration from object
   */
  static fromObject(obj: Record<string, any>): SuperConfig;
  
  /**
   * Load configuration from localStorage
   */
  static fromLocalStorage(key?: string): SuperConfig;
  
  /**
   * Get zero-copy handle to configuration value
   */
  getHandle<T>(key: string): Handle<T>;
  
  /**
   * Get handle with fallback default value
   */
  getHandleWithDefault<T>(key: string, defaultValue: T): Handle<T>;
  
  /**
   * Get configuration value
   */
  get<T>(key: string): T;
  
  /**
   * Set configuration value
   */
  set<T>(key: string, value: T): void;
  
  /**
   * Save to localStorage
   */
  saveToLocalStorage(key?: string): void;
  
  /**
   * Enable URL watching for hot reload
   */
  enableUrlReload(url: string, intervalMs?: number): HotReloadHandle;
}

// Memory-optimized handle
class Handle<T> {
  get(): T;
  isValid(): boolean;
  readonly key: string;
  readonly source: string;
}

// Browser-specific features
interface HotReloadHandle {
  onChange(callback: (changedKeys: string[]) => void): void;
  stop(): void;
}
```

### Usage Examples

```javascript
// Initialize
import init, { SuperConfig } from '@superconfig/wasm';
await init();

// Load from URL
const config = await SuperConfig.fromUrl('/config.json');

// Zero-copy handles
const themeHandle = config.getHandle('ui.theme');
const debugHandle = config.getHandle('app.debug');

// Fast access in render loops
function render() {
  const theme = themeHandle.get(); // ~0.1μs
  const debug = debugHandle.get(); // ~0.1μs
  
  // Render UI based on config
  updateTheme(theme);
  showDebugInfo(debug);
  
  requestAnimationFrame(render);
}

// Hot reload from URL
const hotReload = config.enableUrlReload('/config.json', 5000);

hotReload.onChange((changedKeys) => {
  console.log('Config updated:', changedKeys);
  // Re-render affected components
});

// localStorage integration
config.set('user.preferences.theme', 'dark');
config.saveToLocalStorage('app-config');

// Next session
const savedConfig = SuperConfig.fromLocalStorage('app-config');
```

## Cross-Language Type Mapping

### Primitive Types

| Rust     | Python  | Node.js   | WebAssembly | Notes            |
| -------- | ------- | --------- | ----------- | ---------------- |
| `String` | `str`   | `string`  | `string`    | UTF-8 everywhere |
| `i32`    | `int`   | `number`  | `number`    | 32-bit signed    |
| `i64`    | `int`   | `bigint`  | `bigint`    | 64-bit signed    |
| `f64`    | `float` | `number`  | `number`    | IEEE 754 double  |
| `bool`   | `bool`  | `boolean` | `boolean`   | Same semantics   |

### Complex Types

| Rust                 | Python         | Node.js             | WebAssembly         | Notes             |
| -------------------- | -------------- | ------------------- | ------------------- | ----------------- |
| `Vec<T>`             | `List[T]`      | `T[]`               | `T[]`               | Dynamic arrays    |
| `HashMap<String, T>` | `Dict[str, T]` | `Record<string, T>` | `Record<string, T>` | String-keyed maps |
| `Option<T>`          | `Optional[T]`  | `T \| null`         | `T \| null`         | Nullable types    |
| `Result<T, E>`       | Exception      | `Promise<T>`        | `Promise<T>`        | Error handling    |

### Custom Types

```rust
// Rust definition
#[derive(ConfigValue, Clone, Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls: Option<TlsConfig>,
    pub workers: u32,
}

#[derive(ConfigValue, Clone, Debug)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
    pub protocols: Vec<String>,
}
```

```python
# Python equivalent
from dataclasses import dataclass
from typing import Optional, List

@dataclass
class TlsConfig:
    cert_path: str
    key_path: str
    protocols: List[str]

@dataclass
class ServerConfig:
    host: str
    port: int
    tls: Optional[TlsConfig]
    workers: int
```

```typescript
// TypeScript equivalent
interface TlsConfig {
  certPath: string;
  keyPath: string;
  protocols: string[];
}

interface ServerConfig {
  host: string;
  port: number;
  tls?: TlsConfig;
  workers: number;
}
```

## Performance Characteristics

### Operation Timings (Target)

| Operation       | Rust      | Python    | Node.js   | WebAssembly |
| --------------- | --------- | --------- | --------- | ----------- |
| Handle creation | ~20-30μs  | ~25-35μs  | ~30-40μs  | ~25-35μs    |
| Handle access   | ~0.1μs    | ~0.5μs    | ~1μs      | ~0.2μs      |
| Direct get()    | ~1-2μs    | ~2-3μs    | ~3-4μs    | ~2-3μs      |
| Config reload   | ~50-100μs | ~60-120μs | ~80-150μs | ~70-130μs   |

### Memory Usage

| Language    | Handle Overhead | Config Overhead  | Notes                  |
| ----------- | --------------- | ---------------- | ---------------------- |
| Rust        | 24 bytes        | ~1x data size    | Zero-copy references   |
| Python      | 48 bytes        | ~1.2x data size  | Python object overhead |
| Node.js     | 40 bytes        | ~1.1x data size  | V8 optimization        |
| WebAssembly | 32 bytes        | ~1.05x data size | Linear memory layout   |

## Validation and Constraints

### Built-in Validators

```rust
// Rust validators
use superconfig::validators::*;

let config = SuperConfig::new()
    .with_validator("database.port", range(1, 65535))
    .with_validator("app.name", regex(r"^[a-zA-Z][a-zA-Z0-9_]*$"))
    .with_validator("features", each(one_of(&["auth", "logging", "metrics"])));
```

```python
# Python validators
from superconfig.validators import range_validator, regex_validator, each_validator

config = SuperConfig() \
    .with_validator("database.port", range_validator(1, 65535)) \
    .with_validator("app.name", regex_validator(r"^[a-zA-Z][a-zA-Z0-9_]*$")) \
    .with_validator("features", each_validator(["auth", "logging", "metrics"]))
```

```typescript
// TypeScript validators
import { rangeValidator, regexValidator, eachValidator } from '@superconfig/core';

const config = new SuperConfig()
  .withValidator('database.port', rangeValidator(1, 65535))
  .withValidator('app.name', regexValidator(/^[a-zA-Z][a-zA-Z0-9_]*$/))
  .withValidator('features', eachValidator(['auth', 'logging', 'metrics']));
```

### Custom Validators

```rust
// Rust custom validator
fn validate_url(value: &ConfigData) -> Result<(), ValidationError> {
    match value {
        ConfigData::String(s) => {
            url::Url::parse(s)
                .map(|_| ())
                .map_err(|e| ValidationError::new("Invalid URL", &e.to_string()))
        }
        _ => Err(ValidationError::new("Expected string", "Got non-string value"))
    }
}

config.with_validator("api.base_url", validate_url);
```

```python
# Python custom validator
def validate_url(value: Any) -> None:
    if not isinstance(value, str):
        raise ValidationError("Expected string")
    
    try:
        from urllib.parse import urlparse
        result = urlparse(value)
        if not all([result.scheme, result.netloc]):
            raise ValidationError("Invalid URL format")
    except Exception as e:
        raise ValidationError(f"URL validation failed: {e}")

config.with_validator("api.base_url", validate_url)
```

## Migration Utilities

### V1 to V2 Migration

```rust
// Rust migration helper
use superconfig::migration::*;

let v1_config = figment::Figment::new()
    .merge(figment::providers::Toml::file("config.toml"));

let v2_config = SuperConfig::migrate_from_figment(v1_config)?;
```

```python
# Python migration helper
from superconfig.migration import migrate_from_dict

# From any dict-like config
old_config = {
    "database": {"host": "localhost", "port": 5432},
    "app": {"debug": True}
}

config = SuperConfig.migrate_from_dict(old_config)
```

### Compatibility Layer

```rust
// Temporary compatibility trait
trait FigmentCompat {
    fn extract<T: serde::Deserialize>(&self) -> Result<T, ConfigError>;
}

impl FigmentCompat for SuperConfig {
    fn extract<T: serde::Deserialize>(&self) -> Result<T, ConfigError> {
        // Convert internal data to serde-compatible format
        // Allow gradual migration
    }
}
```

## Integration Examples

### Framework Integration

```rust
// Axum integration
use axum::{Extension, Router};

let config = SuperConfig::from_file("app.toml")?;
let app = Router::new()
    .route("/", get(handler))
    .layer(Extension(config));

async fn handler(Extension(config): Extension<SuperConfig>) -> String {
    let app_name = config.get_handle::<String>("app.name").get();
    format!("Hello from {}", app_name)
}
```

```python
# FastAPI integration
from fastapi import FastAPI, Depends
from superconfig import SuperConfig

config = SuperConfig.from_file("app.toml")
app = FastAPI()

def get_config() -> SuperConfig:
    return config

@app.get("/")
async def root(config: SuperConfig = Depends(get_config)):
    app_name = config.get("app.name", str)
    return {"message": f"Hello from {app_name}"}
```

```typescript
// Express integration
import express from 'express';
import { SuperConfig } from '@superconfig/core';

const config = await SuperConfig.fromFile('app.json');
const app = express();

app.locals.config = config;

app.get('/', (req, res) => {
  const appName = req.app.locals.config.get<string>('app.name');
  res.json({ message: `Hello from ${appName}` });
});
```

This comprehensive API reference provides the foundation for consistent, high-performance configuration management across all target platforms while maintaining language-specific idioms and best practices.
