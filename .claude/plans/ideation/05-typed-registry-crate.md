# Typed Registry - Reusable Crate Design

**Status**: 💡 CONCEPT - REUSABLE ARCHITECTURE\
**Priority**: MEDIUM (Future Enhancement)\
**Dependencies**: SuperConfig V2 Core ✅\
**Estimated Effort**: 2-3 weeks for full crate

## Overview

Extract the SuperConfig V2 registry architecture into a standalone `typed-registry` crate that provides high-performance, type-safe, heterogeneous storage for any Rust application.

## 🎯 **Abstraction Benefits**

### **Reusability Across Domains**

- **Configuration Management** (SuperConfig)
- **Service Registries** (Dependency Injection)
- **Plugin Systems** (Dynamic Loading)
- **Component Systems** (Game Engines)
- **Cache Systems** (Multi-type Caching)
- **Asset Management** (Game/Web Assets)

### **Performance Characteristics**

- Sub-microsecond lookups via handle-based access
- Lock-free concurrent operations via DashMap
- Zero-copy sharing via Arc references
- Minimal memory overhead with type erasure

## 🏗️ **Crate Architecture**

### **Core Abstraction**

```rust
// typed-registry/src/lib.rs
pub struct TypedRegistry<K = u64, M = ()> 
where 
    K: Eq + Hash + Copy + Send + Sync,
    M: Clone + Send + Sync,
{
    entries: DashMap<K, RegistryEntry<M>>,
    next_key: AtomicU64,
    stats: Arc<RwLock<RegistryStats>>,
    metadata: M,
}

pub struct RegistryEntry<M> {
    data: Box<dyn Any + Send + Sync>,
    type_name: &'static str,
    created_at: Instant,
    last_accessed: AtomicU64,
    metadata: M,
    data_size: usize,
}

pub struct Handle<T, K = u64> {
    key: K,
    _phantom: PhantomData<T>,
}
```

### **Generic Interface**

```rust
impl<K, M> TypedRegistry<K, M> 
where 
    K: Eq + Hash + Copy + Send + Sync,
    M: Clone + Send + Sync,
{
    pub fn new(metadata: M) -> Self { ... }
    
    pub fn insert<T>(&self, data: T) -> Handle<T, K> 
    where T: 'static + Send + Sync { ... }
    
    pub fn insert_with_key<T>(&self, key: K, data: T) -> Handle<T, K>
    where T: 'static + Send + Sync { ... }
    
    pub fn get<T>(&self, handle: &Handle<T, K>) -> Result<Arc<T>, RegistryError> 
    where T: 'static { ... }
    
    pub fn update<T>(&self, handle: &Handle<T, K>, data: T) -> Result<(), RegistryError>
    where T: 'static + Send + Sync { ... }
    
    pub fn remove<T>(&self, handle: &Handle<T, K>) -> Result<Arc<T>, RegistryError>
    where T: 'static { ... }
}
```

### **Specialized Implementations**

```rust
// Default implementation (like SuperConfig)
pub type DefaultRegistry = TypedRegistry<u64, ()>;

// String-keyed registry
pub type NamedRegistry<M = ()> = TypedRegistry<String, M>;

// Custom key + metadata
pub type CustomRegistry<K, M> = TypedRegistry<K, M>;
```

## 🎯 **Usage Examples**

### **1. Configuration Management (SuperConfig)**

```rust
use typed_registry::DefaultRegistry;

#[derive(Clone)]
struct ConfigMetadata {
    startup_flags: u32,
    runtime_flags: Arc<RwLock<u64>>,
}

struct ConfigRegistry {
    registry: DefaultRegistry,
    metadata: ConfigMetadata,
}

impl ConfigRegistry {
    fn new() -> Self {
        let metadata = ConfigMetadata {
            startup_flags: 0,
            runtime_flags: Arc::new(RwLock::new(0)),
        };
        
        Self {
            registry: DefaultRegistry::new(()),
            metadata,
        }
    }
    
    fn create<T>(&self, data: T) -> Handle<T> 
    where T: 'static + Send + Sync {
        self.registry.insert(data)
    }
    
    fn read<T>(&self, handle: &Handle<T>) -> Result<Arc<T>, RegistryError> 
    where T: 'static {
        self.registry.get(handle)
    }
}
```

### **2. Service Registry (Dependency Injection)**

```rust
use typed_registry::NamedRegistry;

#[derive(Clone)]
struct ServiceMetadata {
    lifecycle: ServiceLifecycle,
    dependencies: Vec<String>,
}

struct ServiceContainer {
    registry: NamedRegistry<ServiceMetadata>,
}

impl ServiceContainer {
    fn register<T>(&self, name: &str, service: T, lifecycle: ServiceLifecycle) -> Handle<T, String>
    where T: 'static + Send + Sync {
        let metadata = ServiceMetadata {
            lifecycle,
            dependencies: Vec::new(),
        };
        
        self.registry.insert_with_key(name.to_string(), service)
    }
    
    fn resolve<T>(&self, name: &str) -> Result<Arc<T>, RegistryError>
    where T: 'static {
        let handle = Handle::new(name.to_string());
        self.registry.get(&handle)
    }
}

// Usage
let container = ServiceContainer::new();

// Register services
let db_handle = container.register("database", DatabaseService::new(), ServiceLifecycle::Singleton);
let api_handle = container.register("api", ApiService::new(), ServiceLifecycle::Transient);

// Resolve services  
let db_service: Arc<DatabaseService> = container.resolve("database")?;
let api_service: Arc<ApiService> = container.resolve("api")?;
```

### **3. Plugin System**

```rust
use typed_registry::NamedRegistry;

#[derive(Clone)]
struct PluginMetadata {
    version: String,
    author: String,
    capabilities: Vec<String>,
}

struct PluginManager {
    registry: NamedRegistry<PluginMetadata>,
}

impl PluginManager {
    fn load_plugin<T>(&self, name: &str, plugin: T, metadata: PluginMetadata) -> Handle<T, String>
    where T: Plugin + 'static + Send + Sync {
        self.registry.insert_with_key_and_metadata(name.to_string(), plugin, metadata)
    }
    
    fn get_plugin<T>(&self, name: &str) -> Result<Arc<T>, RegistryError>
    where T: Plugin + 'static {
        let handle = Handle::new(name.to_string());
        self.registry.get(&handle)
    }
}

// Usage
let manager = PluginManager::new();

// Load different types of plugins
let auth_plugin = manager.load_plugin("auth", AuthPlugin::new(), PluginMetadata { ... });
let logger_plugin = manager.load_plugin("logger", LoggerPlugin::new(), PluginMetadata { ... });

// Use plugins
let auth: Arc<AuthPlugin> = manager.get_plugin("auth")?;
let logger: Arc<LoggerPlugin> = manager.get_plugin("logger")?;
```

### **4. Component System (Game Engine)**

```rust
use typed_registry::DefaultRegistry;

#[derive(Clone)]
struct ComponentMetadata {
    entity_id: u64,
    component_flags: u32,
}

struct World {
    registry: TypedRegistry<u64, ComponentMetadata>,
    next_entity: AtomicU64,
}

impl World {
    fn spawn_entity(&self) -> EntityId {
        let id = self.next_entity.fetch_add(1, Ordering::Relaxed);
        EntityId(id)
    }
    
    fn add_component<T>(&self, entity: EntityId, component: T) -> Handle<T>
    where T: Component + 'static + Send + Sync {
        let metadata = ComponentMetadata {
            entity_id: entity.0,
            component_flags: T::FLAGS,
        };
        
        self.registry.insert_with_metadata(component, metadata)
    }
    
    fn get_component<T>(&self, handle: &Handle<T>) -> Result<Arc<T>, RegistryError>
    where T: Component + 'static {
        self.registry.get(handle)
    }
}

// Usage
let world = World::new();
let entity = world.spawn_entity();

// Add different component types to same registry
let transform_handle = world.add_component(entity, Transform { x: 0.0, y: 0.0 });
let sprite_handle = world.add_component(entity, Sprite { texture: "player.png".to_string() });
let health_handle = world.add_component(entity, Health { current: 100, max: 100 });
```

### **5. Multi-Type Cache**

```rust
use typed_registry::NamedRegistry;

#[derive(Clone)]
struct CacheMetadata {
    ttl: Duration,
    accessed_count: AtomicU64,
    expires_at: Instant,
}

struct TypedCache {
    registry: NamedRegistry<CacheMetadata>,
}

impl TypedCache {
    fn set<T>(&self, key: &str, value: T, ttl: Duration) -> Handle<T, String>
    where T: 'static + Send + Sync {
        let metadata = CacheMetadata {
            ttl,
            accessed_count: AtomicU64::new(0),
            expires_at: Instant::now() + ttl,
        };
        
        self.registry.insert_with_key_and_metadata(key.to_string(), value, metadata)
    }
    
    fn get<T>(&self, key: &str) -> Result<Arc<T>, RegistryError>
    where T: 'static {
        let handle = Handle::new(key.to_string());
        
        // Check TTL and increment access count
        if let Ok(entry) = self.registry.get_entry(&handle) {
            if entry.metadata.expires_at < Instant::now() {
                return Err(RegistryError::Expired);
            }
            entry.metadata.accessed_count.fetch_add(1, Ordering::Relaxed);
        }
        
        self.registry.get(&handle)
    }
}

// Usage - cache different types with same keys
let cache = TypedCache::new();

cache.set("user:123", User { name: "Alice".to_string() }, Duration::from_secs(300));
cache.set("session:abc", Session { token: "xyz".to_string() }, Duration::from_secs(3600));
cache.set("config:db", DatabaseConfig { host: "localhost".to_string() }, Duration::from_secs(60));

// Type-safe retrieval
let user: Arc<User> = cache.get("user:123")?;
let session: Arc<Session> = cache.get("session:abc")?;
let db_config: Arc<DatabaseConfig> = cache.get("config:db")?;
```

## 🔧 **Crate Features**

### **Core Features**

```toml
[features]
default = ["std", "serde", "statistics"]

# Standard library support
std = ["dashmap/std"]

# Serialization support
serde = ["dep:serde", "serde_json"]

# Statistics and monitoring
statistics = ["dep:metrics"]

# Async support
async = ["tokio", "async-trait"]

# Custom allocators
custom-alloc = []

# No-std support (embedded)
no-std = ["hashbrown", "spin"]
```

### **Optional Integrations**

```rust
// Serde integration
#[cfg(feature = "serde")]
impl<K, M> TypedRegistry<K, M> {
    pub fn get_json<T>(&self, handle: &Handle<T, K>) -> Result<String, RegistryError>
    where T: Serialize + 'static { ... }
    
    pub fn set_json<T>(&self, key: K, json: &str) -> Result<Handle<T, K>, RegistryError>
    where T: DeserializeOwned + 'static + Send + Sync { ... }
}

// Metrics integration
#[cfg(feature = "statistics")]
impl<K, M> TypedRegistry<K, M> {
    pub fn metrics(&self) -> RegistryMetrics { ... }
    pub fn export_prometheus(&self) -> String { ... }
}

// Async support
#[cfg(feature = "async")]
impl<K, M> TypedRegistry<K, M> {
    pub async fn get_async<T>(&self, handle: &Handle<T, K>) -> Result<Arc<T>, RegistryError> { ... }
    pub async fn set_async<T>(&self, key: K, future: impl Future<Output = T>) -> Handle<T, K> { ... }
}
```

## 📦 **Crate Structure**

```
typed-registry/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Main registry implementation
│   ├── entry.rs            # RegistryEntry implementation  
│   ├── handle.rs           # Handle implementation
│   ├── error.rs            # Error types
│   ├── stats.rs            # Statistics and monitoring
│   ├── serde.rs            # Serialization support
│   ├── async_support.rs    # Async operations
│   └── prelude.rs          # Convenience imports
├── examples/
│   ├── config.rs           # Configuration management
│   ├── services.rs         # Service registry
│   ├── plugins.rs          # Plugin system
│   ├── components.rs       # Component system
│   └── cache.rs            # Multi-type cache
├── benches/
│   ├── performance.rs      # Performance benchmarks
│   └── memory.rs           # Memory usage tests
└── tests/
    ├── integration.rs      # Integration tests
    ├── concurrent.rs       # Concurrency tests
    └── serde.rs           # Serialization tests
```

## 🎯 **SuperConfig Integration**

SuperConfig would become a specialized wrapper:

```rust
// superconfig/src/core/registry.rs
use typed_registry::{TypedRegistry, Handle};

#[derive(Clone)]
struct SuperConfigMetadata {
    startup_flags: u32,
    runtime_flags: Arc<parking_lot::RwLock<u64>>,
    verbosity: Arc<parking_lot::RwLock<u8>>,
    collected_errors: Arc<parking_lot::RwLock<Vec<FluentError>>>,
}

pub struct ConfigRegistry {
    inner: TypedRegistry<u64, SuperConfigMetadata>,
}

impl ConfigRegistry {
    pub fn new() -> Self {
        let metadata = SuperConfigMetadata {
            startup_flags: 0,
            runtime_flags: Arc::new(parking_lot::RwLock::new(0)),
            verbosity: Arc::new(parking_lot::RwLock::new(0)),
            collected_errors: Arc::new(parking_lot::RwLock::new(Vec::new())),
        };
        
        Self {
            inner: TypedRegistry::new(metadata),
        }
    }
    
    pub fn create<T>(&self, data: T) -> Result<ConfigHandle<T>, RegistryError>
    where T: 'static + Send + Sync {
        let handle = self.inner.insert(data);
        Ok(ConfigHandle::from(handle))
    }
    
    pub fn read<T>(&self, handle: &ConfigHandle<T>) -> Result<Arc<T>, RegistryError>
    where T: 'static {
        self.inner.get(&handle.inner)
    }
    
    // SuperConfig-specific methods
    pub fn enable(&self, flags: u64) -> Result<&Self, RegistryError> { ... }
    pub fn try_enable(self: Arc<Self>, flags: u64) -> Arc<Self> { ... }
    pub fn catch(&self) -> Vec<FluentError> { ... }
}

pub type ConfigHandle<T> = Handle<T, u64>;
```

## 🚀 **Benefits of Abstraction**

### **Code Reuse**

- Core type-safe storage logic reused across projects
- Battle-tested performance optimizations
- Consistent API patterns

### **Community Impact**

- Enable other projects to benefit from this architecture
- Potential for ecosystem adoption
- Contributions and improvements from community

### **Maintenance**

- Single crate to maintain core storage logic
- Bug fixes benefit all users
- Easier to optimize and benchmark

### **Innovation**

- Foundation for more advanced patterns
- Plugin for different storage backends
- Custom metadata and key types

## 📋 **Implementation Plan**

### **Phase 1: Core Extraction (1 week)**

- Extract core registry logic from SuperConfig
- Generalize key and metadata types
- Basic handle operations (insert, get, update, remove)

### **Phase 2: Advanced Features (1 week)**

- Statistics and monitoring
- Serde integration
- Error handling improvements

### **Phase 3: Ecosystem Integration (1 week)**

- Async support
- No-std compatibility
- Performance optimizations
- Documentation and examples

### **Phase 4: Community (Ongoing)**

- Publish to crates.io
- Community feedback integration
- Ecosystem adoption

## 🎯 **Success Criteria**

- **Performance**: Maintains sub-microsecond lookup times
- **Safety**: Zero unsafe code, full type safety
- **Ergonomics**: Easy to use for common patterns
- **Flexibility**: Supports diverse use cases
- **Quality**: Comprehensive tests and documentation

---

This abstraction would create a powerful, reusable foundation for type-safe storage patterns across the Rust ecosystem, while allowing SuperConfig to focus on its core configuration management features.
