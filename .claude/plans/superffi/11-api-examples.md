# SuperConfig FFI API Examples

This document provides comprehensive usage examples for SuperConfig across all supported language bindings, demonstrating the consistent API design and practical usage patterns.

## Installation

### Python
```bash
pip install superconfig
```

### Node.js
```bash
npm install superconfig
```

### WebAssembly
```bash
npm install superconfig-wasm
```

## Basic Usage Examples

### Python

```python
from superconfig import SuperConfig

# Basic configuration loading
config = SuperConfig.new()
config = config.with_file("config.toml")
config = config.with_env("APP_")

# Extract configuration as dictionary
data = config.extract_json()
print(f"Database URL: {data.get('database', {}).get('url')}")

# Method chaining
config = (SuperConfig.new()
    .with_file("base.toml")
    .with_file("local.toml")
    .with_env("MYAPP_")
    .set_debug(True))
```

### Node.js

```javascript
const { SuperConfig } = require('superconfig');

// Basic configuration loading (note camelCase methods)
let config = SuperConfig.new();
config = config.withFile("config.toml");
config = config.withEnv("APP_");

// Extract configuration as object
const data = config.extractJson();
console.log(`Database URL: ${data.database?.url}`);

// Method chaining
config = SuperConfig.new()
    .withFile("base.toml")
    .withFile("local.toml")
    .withEnv("MYAPP_")
    .setDebug(true);
```

### WebAssembly

```javascript
import { SuperConfig } from 'superconfig-wasm';

// Identical API to Node.js (camelCase methods)
let config = SuperConfig.new();
config = config.withFile("config.toml");
config = config.withEnv("APP_");

// Extract configuration as object
const data = config.extractJson();
console.log(`Database URL: ${data.database?.url}`);

// Method chaining works identically to Node.js
config = SuperConfig.new()
    .withFile("base.toml")
    .withFile("local.toml")
    .withEnv("MYAPP_")
    .setDebug(true);
```

## Advanced Configuration Examples

### Wildcard File Discovery

#### Python
```python
from superconfig import SuperConfig

# Simple wildcard configuration
wildcard_config = {
    "pattern": "*.toml"
}
config = SuperConfig.new().with_wildcard(wildcard_config)

# Advanced wildcard with recursive search
advanced_wildcard = {
    "pattern": "**/*.json",
    "search": {
        "type": "recursive",
        "root": "./config",
        "max_depth": 3
    },
    "merge_order": {
        "type": "custom",
        "patterns": ["base.*", "env-*.json", "local.*"]
    }
}
config = SuperConfig.new().with_wildcard(advanced_wildcard)
```

#### Node.js
```javascript
const { SuperConfig } = require('superconfig');

// Simple wildcard configuration (camelCase method)
const wildcardConfig = {
    pattern: "*.toml"
};
const config = SuperConfig.new().withWildcard(wildcardConfig);

// Advanced wildcard with recursive search
const advancedWildcard = {
    pattern: "**/*.json",
    search: {
        type: "recursive",
        root: "./config",
        maxDepth: 3
    },
    mergeOrder: {
        type: "custom",
        patterns: ["base.*", "env-*.json", "local.*"]
    }
};
const config = SuperConfig.new().withWildcard(advancedWildcard);
```

#### WebAssembly
```javascript
import { SuperConfig } from 'superconfig-wasm';

// Identical to Node.js API
const wildcardConfig = {
    pattern: "*.toml"
};
const config = SuperConfig.new().withWildcard(wildcardConfig);

// Advanced configuration identical to Node.js
const advancedWildcard = {
    pattern: "**/*.json",
    search: {
        type: "recursive",
        root: "./config",
        maxDepth: 3
    },
    mergeOrder: {
        type: "custom",
        patterns: ["base.*", "env-*.json", "local.*"]
    }
};
const config = SuperConfig.new().withWildcard(advancedWildcard);
```

## Real-World Usage Scenarios

### Scenario 1: Web Application Configuration

#### Python (Flask/Django)
```python
from superconfig import SuperConfig
import os

def load_app_config():
    """Load configuration for Flask/Django application."""
    env = os.getenv('ENVIRONMENT', 'development')
    
    config = (SuperConfig.new()
        .with_file("config/base.toml")
        .with_file(f"config/{env}.toml")
        .with_env("WEBAPP_")
        .set_debug(env == 'development'))
    
    return config.extract_json()

# Usage
app_config = load_app_config()
DATABASE_URL = app_config['database']['url']
SECRET_KEY = app_config['security']['secret_key']
```

#### Node.js (Express)
```javascript
const { SuperConfig } = require('superconfig');

function loadAppConfig() {
    const env = process.env.NODE_ENV || 'development';
    
    const config = SuperConfig.new()
        .withFile("config/base.toml")
        .withFile(`config/${env}.toml`)
        .withEnv("WEBAPP_")
        .setDebug(env === 'development');
    
    return config.extractJson();
}

// Usage
const appConfig = loadAppConfig();
const DATABASE_URL = appConfig.database.url;
const SECRET_KEY = appConfig.security.secretKey;
```

### Scenario 2: Microservice Configuration

#### Python
```python
from superconfig import SuperConfig

class MicroserviceConfig:
    def __init__(self, service_name: str):
        self.service_name = service_name
        self.config = self._load_config()
    
    def _load_config(self):
        wildcard_config = {
            "pattern": "*.toml",
            "search": {
                "type": "directories",
                "directories": [
                    "/etc/myapp",
                    f"/etc/myapp/{self.service_name}",
                    "./config"
                ]
            },
            "merge_order": {
                "type": "custom",
                "patterns": ["global.*", f"{self.service_name}.*", "local.*"]
            }
        }
        
        return (SuperConfig.new()
            .with_wildcard(wildcard_config)
            .with_env(f"{self.service_name.upper()}_")
            .extract_json())
    
    def get_database_config(self):
        return self.config.get('database', {})
    
    def get_redis_config(self):
        return self.config.get('redis', {})

# Usage
auth_service = MicroserviceConfig('auth')
db_config = auth_service.get_database_config()
```

#### Node.js
```javascript
const { SuperConfig } = require('superconfig');

class MicroserviceConfig {
    constructor(serviceName) {
        this.serviceName = serviceName;
        this.config = this._loadConfig();
    }
    
    _loadConfig() {
        const wildcardConfig = {
            pattern: "*.toml",
            search: {
                type: "directories",
                directories: [
                    "/etc/myapp",
                    `/etc/myapp/${this.serviceName}`,
                    "./config"
                ]
            },
            mergeOrder: {
                type: "custom",
                patterns: ["global.*", `${this.serviceName}.*`, "local.*"]
            }
        };
        
        return SuperConfig.new()
            .withWildcard(wildcardConfig)
            .withEnv(`${this.serviceName.toUpperCase()}_`)
            .extractJson();
    }
    
    getDatabaseConfig() {
        return this.config.database || {};
    }
    
    getRedisConfig() {
        return this.config.redis || {};
    }
}

// Usage
const authService = new MicroserviceConfig('auth');
const dbConfig = authService.getDatabaseConfig();
```

## Configuration Debugging & Introspection

### Python
```python
from superconfig import SuperConfig

config = (SuperConfig.new()
    .with_file("base.toml")
    .with_env("APP_")
    .set_debug(True))

# Find specific configuration value
database_url = config.find("database.url")
if database_url:
    print(f"Database URL found: {database_url}")

# Get metadata about configuration sources
metadata = config.get_metadata()
print("Configuration sources:")
for source in metadata['sources']:
    print(f"  {source['priority']}: {source['name']} ({source['source']})")
```

### Node.js
```javascript
const { SuperConfig } = require('superconfig');

const config = SuperConfig.new()
    .withFile("base.toml")
    .withEnv("APP_")
    .setDebug(true);

// Find specific configuration value (camelCase method)
const databaseUrl = config.find("database.url");
if (databaseUrl) {
    console.log(`Database URL found: ${databaseUrl}`);
}

// Get metadata about configuration sources (camelCase method)
const metadata = config.getMetadata();
console.log("Configuration sources:");
metadata.sources.forEach(source => {
    console.log(`  ${source.priority}: ${source.name} (${source.source})`);
});
```

## Error Handling

### Python
```python
from superconfig import SuperConfig

try:
    config = SuperConfig.new().with_file("nonexistent.toml")
except Exception as e:
    print(f"Configuration error: {e}")
    # Error message will contain "SuperConfig" context and helpful details

# Validate wildcard configuration
try:
    invalid_wildcard = {
        "search": {"type": "current"}
        # Missing required 'pattern' field
    }
    config = SuperConfig.new().with_wildcard(invalid_wildcard)
except Exception as e:
    print(f"Wildcard validation error: {e}")
    # Will get SuperConfig-specific error about missing pattern
```

### Node.js
```javascript
const { SuperConfig } = require('superconfig');

try {
    const config = SuperConfig.new().withFile("nonexistent.toml");
} catch (error) {
    console.error(`Configuration error: ${error.message}`);
    // Error message will contain "SuperConfig" context and helpful details
}

// Validate wildcard configuration
try {
    const invalidWildcard = {
        search: { type: "current" }
        // Missing required 'pattern' field
    };
    const config = SuperConfig.new().withWildcard(invalidWildcard);
} catch (error) {
    console.error(`Wildcard validation error: ${error.message}`);
    // Will get SuperConfig-specific error about missing pattern
}
```

## TypeScript Support (Node.js)

```typescript
import { SuperConfig, WildcardConfig } from 'superconfig';

interface DatabaseConfig {
    url: string;
    pool_size: number;
    timeout: number;
}

interface AppConfig {
    database: DatabaseConfig;
    redis: {
        url: string;
        max_connections: number;
    };
    features: {
        auth_enabled: boolean;
        cache_enabled: boolean;
    };
}

function loadTypedConfig(): AppConfig {
    const wildcardConfig: WildcardConfig = {
        pattern: "*.toml",
        search: {
            type: "recursive",
            root: "./config",
            maxDepth: 2
        }
    };
    
    const config = SuperConfig.new()
        .withWildcard(wildcardConfig)
        .withEnv("APP_")
        .setDebug(process.env.NODE_ENV === 'development');
    
    return config.extractJson() as AppConfig;
}

const appConfig = loadTypedConfig();
// TypeScript knows the structure of appConfig
console.log(`Database pool size: ${appConfig.database.pool_size}`);
```

## API Consistency Demonstration

The following examples demonstrate identical JavaScript APIs across Node.js and WebAssembly:

```javascript
// This exact code works in both Node.js and WebAssembly environments
function createUniversalConfig(platform) {
    const config = SuperConfig.new()
        .withFile("config.toml")           // ✅ Same method name
        .withEnv("APP_")                   // ✅ Same method name  
        .withWildcard({                    // ✅ Same method name
            pattern: "*.toml"
        })
        .setDebug(true);                   // ✅ Same method name
    
    return {
        data: config.extractJson(),        // ✅ Same method name
        metadata: config.getMetadata(),    // ✅ Same method name
        platform: platform
    };
}

// Works identically in both environments:
// - Node.js: const config = createUniversalConfig('nodejs');
// - WASM:    const config = createUniversalConfig('wasm');
```

This consistent API design enables:
- **Code Reuse**: Same configuration logic across server and client
- **Developer Experience**: Learn once, use everywhere  
- **Migration Flexibility**: Easy switching between Node.js and WASM
- **Testing Consistency**: Same test patterns across environments

---

*See [`12-testing-strategy.md`](./12-testing-strategy.md) for comprehensive testing examples and [`03-architecture.md`](./03-architecture.md) for technical implementation details.*