# SuperConfig V2: FFI Integration Plan

## Overview

This document specifies the Foreign Function Interface (FFI) integration strategy for SuperConfig V2, detailing how the pure Rust core will be exposed to Python and Node.js with optimal performance characteristics. The design prioritizes zero-copy operations, efficient memory management, and language-idiomatic APIs while maintaining the core's sub-microsecond performance targets.

## FFI Architecture Principles

### Clean Separation Design

```rust
// Core Architecture: Pure Rust → Thin FFI Wrappers → Language Bindings

┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Rust Core     │    │   FFI Wrappers   │    │ Language APIs   │
│                 │    │                  │    │                 │
│ • ConfigRegistry│ -> │ • Type Safety    │ -> │ • Python PyO3   │
│ • ConfigHandle  │    │ • Error Convert  │    │ • Node.js NAPI  │
│ • ConfigData    │    │ • Memory Mgmt    │    │ • Native Types  │
│ • Providers     │    │ • Zero-copy Refs │    │ • Async Support │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

### Core Principles

1. **Business Logic Isolation**: All functionality lives in pure Rust core
2. **Thin Delegation**: FFI wrappers only handle type conversion and delegation
3. **Zero-Copy Design**: Minimize data copying between languages
4. **Memory Safety**: Automatic cleanup with RAII patterns
5. **Performance First**: Target <1μs Python FFI, <2μs Node.js FFI overhead

## Python Bindings (PyO3)

### Core FFI Wrapper Structure

```rust
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use superconfig::core::{ConfigRegistry, ConfigHandle, ConfigData};

/// Python wrapper for ConfigRegistry with automatic cleanup
#[pyclass(name = "ConfigRegistry")]
pub struct PyConfigRegistry {
    inner: Arc<ConfigRegistry>,
}

#[pymethods]
impl PyConfigRegistry {
    #[new]
    fn new() -> PyResult<Self> {
        Ok(Self {
            inner: ConfigRegistry::new(),
        })
    }
    
    /// Create configuration from builder pattern
    fn create_config(&self, py: Python) -> PyResult<PyConfigBuilder> {
        let builder = self.inner.create_builder()
            .map_err(|e| PyErr::new::<pyo3::exceptions::RuntimeError, _>(e.to_string()))?;
        
        Ok(PyConfigBuilder {
            inner: builder,
            registry: Arc::clone(&self.inner),
        })
    }
    
    /// Get registry statistics for monitoring
    fn stats(&self) -> PyResult<PyDict> {
        let stats = self.inner.stats();
        let py_dict = PyDict::new(Python::acquire_gil().python());
        
        py_dict.set_item("total_handles", stats.total_handles)?;
        py_dict.set_item("active_handles", stats.active_handles)?;
        py_dict.set_item("memory_usage_bytes", stats.memory_usage_bytes)?;
        py_dict.set_item("cache_hit_rate", stats.cache_hit_rate)?;
        
        Ok(py_dict.into())
    }
}

/// Python wrapper for configuration builder with fluent API
#[pyclass(name = "ConfigBuilder")]
pub struct PyConfigBuilder {
    inner: ConfigBuilder,
    registry: Arc<ConfigRegistry>,
}

#[pymethods]
impl PyConfigBuilder {
    /// Add file source with error handling
    fn with_file(&self, path: &str) -> PyResult<PyConfigBuilder> {
        let new_builder = self.inner.with_file(path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::FileNotFoundError, _>(e.to_string()))?;
        
        Ok(PyConfigBuilder {
            inner: new_builder,
            registry: Arc::clone(&self.registry),
        })
    }
    
    /// Add environment variables with prefix
    fn with_env(&self, prefix: Option<&str>) -> PyResult<PyConfigBuilder> {
        let new_builder = self.inner.with_env(prefix)
            .map_err(|e| PyErr::new::<pyo3::exceptions::RuntimeError, _>(e.to_string()))?;
        
        Ok(PyConfigBuilder {
            inner: new_builder,
            registry: Arc::clone(&self.registry),
        })
    }
    
    /// Add glob pattern for multiple files
    fn with_glob(&self, pattern: &str) -> PyResult<PyConfigBuilder> {
        let new_builder = self.inner.with_glob(pattern)
            .map_err(|e| PyErr::new::<pyo3::exceptions::ValueError, _>(e.to_string()))?;
        
        Ok(PyConfigBuilder {
            inner: new_builder,
            registry: Arc::clone(&self.registry),
        })
    }
    
    /// Enable hierarchical discovery
    fn with_hierarchical(&self, app_name: &str) -> PyResult<PyConfigBuilder> {
        let new_builder = self.inner.with_hierarchical(app_name)
            .map_err(|e| PyErr::new::<pyo3::exceptions::RuntimeError, _>(e.to_string()))?;
        
        Ok(PyConfigBuilder {
            inner: new_builder,
            registry: Arc::clone(&self.registry),
        })
    }
    
    /// Select profile for environment-specific configuration
    fn select_profile(&self, profile: &str) -> PyResult<PyConfigBuilder> {
        let new_builder = self.inner.select_profile(profile);
        
        Ok(PyConfigBuilder {
            inner: new_builder,
            registry: Arc::clone(&self.registry),
        })
    }
    
    /// Build configuration and return handle
    fn build(&self) -> PyResult<PyConfigHandle> {
        let handle = self.inner.build()
            .map_err(|e| PyErr::new::<pyo3::exceptions::RuntimeError, _>(e.to_string()))?;
        
        Ok(PyConfigHandle {
            inner: handle,
            registry: Arc::clone(&self.registry),
        })
    }
}

/// Python wrapper for configuration handle with type-safe access
#[pyclass(name = "ConfigHandle")]
pub struct PyConfigHandle {
    inner: ConfigHandle<serde_json::Value>,
    registry: Arc<ConfigRegistry>,
}

#[pymethods]
impl PyConfigHandle {
    /// Extract configuration as Python dictionary
    fn extract(&self) -> PyResult<PyObject> {
        let value = self.inner.extract()
            .map_err(|e| PyErr::new::<pyo3::exceptions::ValueError, _>(e.to_string()))?;
        
        // Convert JSON Value to Python object efficiently
        Python::with_gil(|py| {
            json_to_python(py, &value)
        })
    }
    
    /// Get value at dotted key path
    fn get(&self, key: &str) -> PyResult<Option<PyObject>> {
        match self.inner.get_path(key) {
            Ok(Some(value)) => {
                Python::with_gil(|py| {
                    Ok(Some(json_to_python(py, value)?))
                })
            },
            Ok(None) => Ok(None),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::KeyError, _>(e.to_string())),
        }
    }
    
    /// Check if key exists in configuration
    fn has_key(&self, key: &str) -> PyResult<bool> {
        match self.inner.get_path(key) {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::KeyError, _>(e.to_string())),
        }
    }
    
    /// Get all warnings collected during loading
    fn warnings(&self) -> PyResult<PyList> {
        let warnings = self.inner.warnings();
        Python::with_gil(|py| {
            let py_list = PyList::empty(py);
            
            for warning in warnings {
                let warning_dict = PyDict::new(py);
                warning_dict.set_item("type", warning.warning_type())?;
                warning_dict.set_item("message", warning.message())?;
                warning_dict.set_item("source", warning.source().unwrap_or("unknown"))?;
                
                py_list.append(warning_dict)?;
            }
            
            Ok(py_list.into())
        })
    }
    
    /// Check if configuration has any warnings
    fn has_warnings(&self) -> bool {
        self.inner.has_warnings()
    }
}
```

### Efficient Type Conversion

```rust
/// Convert JSON Value to Python object with zero-copy where possible
fn json_to_python(py: Python, value: &serde_json::Value) -> PyResult<PyObject> {
    match value {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(b) => Ok(b.to_object(py)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.to_object(py))
            } else if let Some(u) = n.as_u64() {
                Ok(u.to_object(py))
            } else if let Some(f) = n.as_f64() {
                Ok(f.to_object(py))
            } else {
                Err(PyErr::new::<pyo3::exceptions::ValueError, _>("Invalid number"))
            }
        },
        serde_json::Value::String(s) => Ok(s.to_object(py)),
        serde_json::Value::Array(arr) => {
            let py_list = PyList::empty(py);
            for item in arr {
                py_list.append(json_to_python(py, item)?)?;
            }
            Ok(py_list.to_object(py))
        },
        serde_json::Value::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (key, val) in obj {
                py_dict.set_item(key, json_to_python(py, val)?)?;
            }
            Ok(py_dict.to_object(py))
        }
    }
}

/// Convert Python object to JSON Value for input processing
fn python_to_json(obj: &PyAny) -> PyResult<serde_json::Value> {
    if obj.is_none() {
        Ok(serde_json::Value::Null)
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(serde_json::Value::Bool(b))
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(serde_json::Value::Number(serde_json::Number::from(i)))
    } else if let Ok(f) = obj.extract::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            Ok(serde_json::Value::Number(n))
        } else {
            Err(PyErr::new::<pyo3::exceptions::ValueError, _>("Invalid float"))
        }
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(serde_json::Value::String(s))
    } else if let Ok(list) = obj.downcast::<PyList>() {
        let mut arr = Vec::new();
        for item in list {
            arr.push(python_to_json(item)?);
        }
        Ok(serde_json::Value::Array(arr))
    } else if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = serde_json::Map::new();
        for (key, value) in dict {
            let key_str = key.extract::<String>()?;
            map.insert(key_str, python_to_json(value)?);
        }
        Ok(serde_json::Value::Object(map))
    } else {
        Err(PyErr::new::<pyo3::exceptions::TypeError, _>("Unsupported type"))
    }
}
```

### Python Module Definition

```rust
/// Python module initialization
#[pymodule]
fn superconfig(_py: Python, m: &PyModule) -> PyResult<()> {
    // Register core classes
    m.add_class::<PyConfigRegistry>()?;
    m.add_class::<PyConfigBuilder>()?;
    m.add_class::<PyConfigHandle>()?;
    
    // Convenience functions
    m.add_function(wrap_pyfunction!(create_config, m)?)?;
    m.add_function(wrap_pyfunction!(load_file, m)?)?;
    m.add_function(wrap_pyfunction!(load_env, m)?)?;
    
    // Version information
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    
    Ok(())
}

/// Convenience function for quick configuration creation
#[pyfunction]
fn create_config() -> PyResult<PyConfigBuilder> {
    let registry = ConfigRegistry::new();
    let builder = registry.create_builder()
        .map_err(|e| PyErr::new::<pyo3::exceptions::RuntimeError, _>(e.to_string()))?;
    
    Ok(PyConfigBuilder {
        inner: builder,
        registry,
    })
}

/// Convenience function for loading single file
#[pyfunction]
fn load_file(path: &str) -> PyResult<PyConfigHandle> {
    let registry = ConfigRegistry::new();
    let handle = registry.create_builder()
        .with_file(path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::FileNotFoundError, _>(e.to_string()))?
        .build()
        .map_err(|e| PyErr::new::<pyo3::exceptions::RuntimeError, _>(e.to_string()))?;
    
    Ok(PyConfigHandle {
        inner: handle,
        registry,
    })
}

/// Convenience function for loading environment variables
#[pyfunction]
fn load_env(prefix: Option<&str>) -> PyResult<PyConfigHandle> {
    let registry = ConfigRegistry::new();
    let handle = registry.create_builder()
        .with_env(prefix)
        .map_err(|e| PyErr::new::<pyo3::exceptions::RuntimeError, _>(e.to_string()))?
        .build()
        .map_err(|e| PyErr::new::<pyo3::exceptions::RuntimeError, _>(e.to_string()))?;
    
    Ok(PyConfigHandle {
        inner: handle,
        registry,
    })
}
```

## Node.js Bindings (NAPI-RS)

### Core NAPI Wrapper Structure

```rust
use napi::{bindgen_prelude::*, JsObject, JsUnknown};
use superconfig::core::{ConfigRegistry, ConfigHandle, ConfigData};

/// Node.js wrapper for ConfigRegistry
#[napi]
pub struct JsConfigRegistry {
    inner: Arc<ConfigRegistry>,
}

#[napi]
impl JsConfigRegistry {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: ConfigRegistry::new(),
        })
    }
    
    /// Create configuration builder
    #[napi]
    pub fn create_config(&self) -> Result<JsConfigBuilder> {
        let builder = self.inner.create_builder()
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
        Ok(JsConfigBuilder {
            inner: builder,
            registry: Arc::clone(&self.inner),
        })
    }
    
    /// Get registry statistics
    #[napi]
    pub fn stats(&self, env: Env) -> Result<JsObject> {
        let stats = self.inner.stats();
        let mut obj = env.create_object()?;
        
        obj.set("totalHandles", stats.total_handles)?;
        obj.set("activeHandles", stats.active_handles)?;
        obj.set("memoryUsageBytes", stats.memory_usage_bytes)?;
        obj.set("cacheHitRate", stats.cache_hit_rate)?;
        
        Ok(obj)
    }
}

/// Node.js wrapper for configuration builder
#[napi]
pub struct JsConfigBuilder {
    inner: ConfigBuilder,
    registry: Arc<ConfigRegistry>,
}

#[napi]
impl JsConfigBuilder {
    /// Add file source (camelCase for Node.js convention)
    #[napi]
    pub fn with_file(&self, path: String) -> Result<JsConfigBuilder> {
        let new_builder = self.inner.with_file(&path)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
        Ok(JsConfigBuilder {
            inner: new_builder,
            registry: Arc::clone(&self.registry),
        })
    }
    
    /// Add environment variables
    #[napi]
    pub fn with_env(&self, prefix: Option<String>) -> Result<JsConfigBuilder> {
        let new_builder = self.inner.with_env(prefix.as_deref())
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
        Ok(JsConfigBuilder {
            inner: new_builder,
            registry: Arc::clone(&self.registry),
        })
    }
    
    /// Add glob pattern
    #[napi]
    pub fn with_glob(&self, pattern: String) -> Result<JsConfigBuilder> {
        let new_builder = self.inner.with_glob(&pattern)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
        Ok(JsConfigBuilder {
            inner: new_builder,
            registry: Arc::clone(&self.registry),
        })
    }
    
    /// Enable hierarchical discovery
    #[napi]
    pub fn with_hierarchical(&self, app_name: String) -> Result<JsConfigBuilder> {
        let new_builder = self.inner.with_hierarchical(&app_name)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
        Ok(JsConfigBuilder {
            inner: new_builder,
            registry: Arc::clone(&self.registry),
        })
    }
    
    /// Select profile
    #[napi]
    pub fn select_profile(&self, profile: String) -> Result<JsConfigBuilder> {
        let new_builder = self.inner.select_profile(&profile);
        
        Ok(JsConfigBuilder {
            inner: new_builder,
            registry: Arc::clone(&self.registry),
        })
    }
    
    /// Build configuration
    #[napi]
    pub fn build(&self) -> Result<JsConfigHandle> {
        let handle = self.inner.build()
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
        Ok(JsConfigHandle {
            inner: handle,
            registry: Arc::clone(&self.registry),
        })
    }
}

/// Node.js wrapper for configuration handle
#[napi]
pub struct JsConfigHandle {
    inner: ConfigHandle<serde_json::Value>,
    registry: Arc<ConfigRegistry>,
}

#[napi]
impl JsConfigHandle {
    /// Extract full configuration as JavaScript object
    #[napi]
    pub fn extract(&self, env: Env) -> Result<JsUnknown> {
        let value = self.inner.extract()
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
        json_to_js(env, &value)
    }
    
    /// Get value at key path
    #[napi]
    pub fn get(&self, env: Env, key: String) -> Result<Option<JsUnknown>> {
        match self.inner.get_path(&key) {
            Ok(Some(value)) => Ok(Some(json_to_js(env, value)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }
    
    /// Check if key exists
    #[napi]
    pub fn has_key(&self, key: String) -> Result<bool> {
        match self.inner.get_path(&key) {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }
    
    /// Get warnings
    #[napi]
    pub fn warnings(&self, env: Env) -> Result<Vec<JsObject>> {
        let warnings = self.inner.warnings();
        let mut js_warnings = Vec::new();
        
        for warning in warnings {
            let mut obj = env.create_object()?;
            obj.set("type", warning.warning_type())?;
            obj.set("message", warning.message())?;
            obj.set("source", warning.source().unwrap_or("unknown"))?;
            js_warnings.push(obj);
        }
        
        Ok(js_warnings)
    }
    
    /// Check if has warnings
    #[napi]
    pub fn has_warnings(&self) -> bool {
        self.inner.has_warnings()
    }
}
```

### Efficient JavaScript Type Conversion

```rust
/// Convert JSON Value to JavaScript value with minimal copying
fn json_to_js(env: Env, value: &serde_json::Value) -> Result<JsUnknown> {
    match value {
        serde_json::Value::Null => Ok(env.get_null()?.into_unknown()),
        serde_json::Value::Bool(b) => Ok(env.get_boolean(*b)?.into_unknown()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                    Ok(env.create_int32(i as i32)?.into_unknown())
                } else {
                    Ok(env.create_int64(i)?.into_unknown())
                }
            } else if let Some(u) = n.as_u64() {
                if u <= u32::MAX as u64 {
                    Ok(env.create_uint32(u as u32)?.into_unknown())
                } else {
                    Ok(env.create_bigint_from_u64(u)?.into_unknown())
                }
            } else if let Some(f) = n.as_f64() {
                Ok(env.create_double(f)?.into_unknown())
            } else {
                Err(Error::from_reason("Invalid number"))
            }
        },
        serde_json::Value::String(s) => {
            Ok(env.create_string(s)?.into_unknown())
        },
        serde_json::Value::Array(arr) => {
            let mut js_array = env.create_array_with_length(arr.len())?;
            for (i, item) in arr.iter().enumerate() {
                js_array.set_element(i as u32, json_to_js(env, item)?)?;
            }
            Ok(js_array.into_unknown())
        },
        serde_json::Value::Object(obj) => {
            let mut js_obj = env.create_object()?;
            for (key, val) in obj {
                js_obj.set(key.as_str(), json_to_js(env, val)?)?;
            }
            Ok(js_obj.into_unknown())
        }
    }
}

/// Convert JavaScript value to JSON Value for input processing
fn js_to_json(env: Env, value: JsUnknown) -> Result<serde_json::Value> {
    let value_type = value.get_type()?;
    
    match value_type {
        ValueType::Null | ValueType::Undefined => Ok(serde_json::Value::Null),
        ValueType::Boolean => {
            let js_bool = unsafe { value.cast::<JsBoolean>() };
            Ok(serde_json::Value::Bool(js_bool.get_value()?))
        },
        ValueType::Number => {
            let js_number = unsafe { value.cast::<JsNumber>() };
            let num = js_number.get_double()?;
            
            if num.fract() == 0.0 {
                Ok(serde_json::Value::Number(serde_json::Number::from(num as i64)))
            } else {
                if let Some(n) = serde_json::Number::from_f64(num) {
                    Ok(serde_json::Value::Number(n))
                } else {
                    Err(Error::from_reason("Invalid number"))
                }
            }
        },
        ValueType::String => {
            let js_string = unsafe { value.cast::<JsString>() };
            Ok(serde_json::Value::String(js_string.into_utf8()?.as_str()?.to_string()))
        },
        ValueType::Object => {
            let js_obj = unsafe { value.cast::<JsObject>() };
            
            // Check if it's an array
            if js_obj.is_array()? {
                let js_array = unsafe { value.cast::<JsArray>() };
                let length = js_array.get_array_length()?;
                let mut arr = Vec::with_capacity(length as usize);
                
                for i in 0..length {
                    let element = js_array.get_element::<JsUnknown>(i)?;
                    arr.push(js_to_json(env, element)?);
                }
                
                Ok(serde_json::Value::Array(arr))
            } else {
                // Regular object
                let property_names = js_obj.get_property_names()?;
                let mut map = serde_json::Map::new();
                
                for i in 0..property_names.get_array_length()? {
                    let key = property_names.get_element::<JsString>(i)?;
                    let key_str = key.into_utf8()?.as_str()?.to_string();
                    let val = js_obj.get_property(key)?;
                    map.insert(key_str, js_to_json(env, val)?);
                }
                
                Ok(serde_json::Value::Object(map))
            }
        },
        _ => Err(Error::from_reason("Unsupported JavaScript type")),
    }
}
```

### Node.js Convenience Functions

```rust
/// Convenience functions for common operations
#[napi]
pub fn create_config() -> Result<JsConfigBuilder> {
    let registry = Arc::new(ConfigRegistry::new());
    let builder = registry.create_builder()
        .map_err(|e| Error::from_reason(e.to_string()))?;
    
    Ok(JsConfigBuilder {
        inner: builder,
        registry,
    })
}

#[napi]
pub fn load_file(path: String) -> Result<JsConfigHandle> {
    let registry = Arc::new(ConfigRegistry::new());
    let handle = registry.create_builder()
        .with_file(&path)
        .map_err(|e| Error::from_reason(e.to_string()))?
        .build()
        .map_err(|e| Error::from_reason(e.to_string()))?;
    
    Ok(JsConfigHandle {
        inner: handle,
        registry,
    })
}

#[napi]
pub fn load_env(prefix: Option<String>) -> Result<JsConfigHandle> {
    let registry = Arc::new(ConfigRegistry::new());
    let handle = registry.create_builder()
        .with_env(prefix.as_deref())
        .map_err(|e| Error::from_reason(e.to_string()))?
        .build()
        .map_err(|e| Error::from_reason(e.to_string()))?;
    
    Ok(JsConfigHandle {
        inner: handle,
        registry,
    })
}

/// Load configuration from multiple sources with merge
#[napi]
pub fn load_multiple(sources: Vec<String>) -> Result<JsConfigHandle> {
    let registry = Arc::new(ConfigRegistry::new());
    let mut builder = registry.create_builder();
    
    for source in sources {
        // Simple heuristic: if it contains *, treat as glob; if it contains =, treat as env; otherwise file
        if source.contains('*') {
            builder = builder.with_glob(&source)
                .map_err(|e| Error::from_reason(e.to_string()))?;
        } else if source.contains('=') {
            // Environment variable in KEY=value format
            let parts: Vec<&str> = source.splitn(2, '=').collect();
            if parts.len() == 2 {
                std::env::set_var(parts[0], parts[1]);
                builder = builder.with_env(None)
                    .map_err(|e| Error::from_reason(e.to_string()))?;
            }
        } else {
            // Assume file path
            builder = builder.with_file(&source)
                .map_err(|e| Error::from_reason(e.to_string()))?;
        }
    }
    
    let handle = builder.build()
        .map_err(|e| Error::from_reason(e.to_string()))?;
    
    Ok(JsConfigHandle {
        inner: handle,
        registry,
    })
}
```

## Async Support and Hot Reload

### Python Async Integration

```rust
use pyo3_asyncio::tokio::future_into_py;

#[pymethods]
impl PyConfigHandle {
    /// Async version of extract for non-blocking access
    fn extract_async(&self, py: Python) -> PyResult<&PyAny> {
        let handle = self.inner.clone();
        
        future_into_py(py, async move {
            let value = handle.extract()
                .map_err(|e| PyErr::new::<pyo3::exceptions::RuntimeError, _>(e.to_string()))?;
            
            Python::with_gil(|py| json_to_python(py, &value))
        })
    }
    
    /// Enable hot reload with callback
    #[cfg(feature = "hot-reload")]
    fn enable_hot_reload(&self, py: Python, callback: PyObject) -> PyResult<&PyAny> {
        let handle = self.inner.clone();
        
        future_into_py(py, async move {
            let mut change_stream = handle.watch()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::RuntimeError, _>(e.to_string()))?;
            
            while let Some(change) = change_stream.next().await {
                Python::with_gil(|py| {
                    let change_dict = PyDict::new(py);
                    change_dict.set_item("type", "configuration_changed")?;
                    change_dict.set_item("source", change.source)?;
                    change_dict.set_item("timestamp", change.timestamp.timestamp())?;
                    
                    callback.call1(py, (change_dict,))?;
                    
                    Ok::<(), PyErr>(())
                })?;
            }
            
            Ok(())
        })
    }
}
```

### Node.js Async Integration

```rust
#[napi]
impl JsConfigHandle {
    /// Async version of extract
    #[napi]
    pub async fn extract_async(&self, env: Env) -> Result<JsUnknown> {
        // NAPI-RS handles async automatically
        let value = self.inner.extract()
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
        json_to_js(env, &value)
    }
    
    /// Enable hot reload with callback
    #[cfg(feature = "hot-reload")]
    #[napi]
    pub async fn enable_hot_reload(&self, callback: JsFunction) -> Result<()> {
        let mut change_stream = self.inner.watch()
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
        while let Some(change) = change_stream.next().await {
            let mut change_obj = callback.env.create_object()?;
            change_obj.set("type", "configuration_changed")?;
            change_obj.set("source", change.source)?;
            change_obj.set("timestamp", change.timestamp.timestamp_millis())?;
            
            callback.call(None, &[change_obj])?;
        }
        
        Ok(())
    }
}
```

## Memory Management

### Automatic Cleanup Patterns

```rust
/// RAII pattern for automatic cleanup in Python
impl Drop for PyConfigHandle {
    fn drop(&mut self) {
        // Handle cleanup is automatic via Arc reference counting
        // Registry will clean up when all handles are dropped
    }
}

/// RAII pattern for automatic cleanup in Node.js
impl Drop for JsConfigHandle {
    fn drop(&mut self) {
        // Handle cleanup is automatic via Arc reference counting
        // No manual cleanup needed
    }
}

/// Memory pressure monitoring for proactive cleanup
pub struct MemoryPressureMonitor {
    pressure_threshold: f64,
    cleanup_scheduler: Arc<CleanupScheduler>,
}

impl MemoryPressureMonitor {
    /// Monitor memory pressure and trigger cleanup
    pub async fn monitor_pressure(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            let memory_info = self.get_memory_info();
            let pressure = memory_info.used as f64 / memory_info.total as f64;
            
            if pressure > self.pressure_threshold {
                self.cleanup_scheduler.trigger_aggressive_cleanup().await;
            }
        }
    }
    
    fn get_memory_info(&self) -> MemoryInfo {
        // Platform-specific memory information gathering
        MemoryInfo {
            total: 0, // Implement per platform
            used: 0,
        }
    }
}
```

## Performance Optimization

### Zero-Copy String Access

```rust
/// Zero-copy string access for hot paths
impl PyConfigHandle {
    /// Get string value without copying (returns reference to internal data)
    fn get_str_ref(&self, key: &str) -> PyResult<Option<&str>> {
        match self.inner.get_path(key) {
            Ok(Some(serde_json::Value::String(s))) => Ok(Some(s.as_str())),
            Ok(Some(_)) => Err(PyErr::new::<pyo3::exceptions::TypeError, _>("Value is not a string")),
            Ok(None) => Ok(None),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::KeyError, _>(e.to_string())),
        }
    }
}

impl JsConfigHandle {
    /// Zero-copy string access for Node.js
    #[napi]
    pub fn get_str_ref(&self, env: Env, key: String) -> Result<Option<String>> {
        match self.inner.get_path(&key) {
            Ok(Some(serde_json::Value::String(s))) => Ok(Some(s.clone())),
            Ok(Some(_)) => Err(Error::from_reason("Value is not a string")),
            Ok(None) => Ok(None),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }
}
```

### Batch Operations

```rust
/// Batch operations for improved performance
#[pymethods]
impl PyConfigHandle {
    /// Get multiple keys in a single operation
    fn get_many(&self, keys: Vec<&str>) -> PyResult<PyDict> {
        Python::with_gil(|py| {
            let result_dict = PyDict::new(py);
            
            for key in keys {
                match self.inner.get_path(key) {
                    Ok(Some(value)) => {
                        result_dict.set_item(key, json_to_python(py, value)?)?;
                    },
                    Ok(None) => {
                        result_dict.set_item(key, py.None())?;
                    },
                    Err(_) => {
                        result_dict.set_item(key, py.None())?;
                    }
                }
            }
            
            Ok(result_dict)
        })
    }
}

#[napi]
impl JsConfigHandle {
    /// Batch get operation for Node.js
    #[napi]
    pub fn get_many(&self, env: Env, keys: Vec<String>) -> Result<JsObject> {
        let mut result = env.create_object()?;
        
        for key in keys {
            match self.inner.get_path(&key) {
                Ok(Some(value)) => {
                    result.set(key.as_str(), json_to_js(env, value)?)?;
                },
                Ok(None) => {
                    result.set(key.as_str(), env.get_null()?)?;
                },
                Err(_) => {
                    result.set(key.as_str(), env.get_null()?)?;
                }
            }
        }
        
        Ok(result)
    }
}
```

## Build Configuration

### Python Build (pyproject.toml)

```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "superconfig"
description = "High-performance configuration management library"
readme = "README.md"
requires-python = ">=3.8"
license = { text = "MIT" }
authors = [
  { name = "Your Name", email = "your.email@example.com" },
]
classifiers = [
  "Development Status :: 4 - Beta",
  "Intended Audience :: Developers",
  "License :: OSI Approved :: MIT License",
  "Programming Language :: Python :: 3",
  "Programming Language :: Python :: 3.8",
  "Programming Language :: Python :: 3.9",
  "Programming Language :: Python :: 3.10",
  "Programming Language :: Python :: 3.11",
  "Programming Language :: Python :: 3.12",
  "Programming Language :: Rust",
]

[project.optional-dependencies]
async = ["pyo3-asyncio"]

[tool.maturin]
features = ["pyo3/extension-module"]
python-source = "python"
module-name = "superconfig._superconfig"
```

### Node.js Build (package.json)

```json
{
  "name": "superconfig",
  "version": "2.0.0",
  "description": "High-performance configuration management library",
  "main": "index.js",
  "types": "index.d.ts",
  "scripts": {
    "build": "napi build --platform --release",
    "build:debug": "napi build --platform",
    "test": "node test/index.js",
    "bench": "node bench/index.js"
  },
  "napi": {
    "name": "superconfig",
    "triples": {
      "defaults": true,
      "additional": ["aarch64-apple-darwin", "aarch64-linux-android"]
    }
  },
  "engines": {
    "node": ">= 14"
  },
  "keywords": ["config", "configuration", "settings", "environment"],
  "license": "MIT",
  "devDependencies": {
    "@napi-rs/cli": "^2.0.0"
  }
}
```

## Performance Targets

### FFI Overhead Benchmarks

| Operation        | Python (PyO3) | Node.js (NAPI) | Target     |
| ---------------- | ------------- | -------------- | ---------- |
| Handle creation  | ~0.8μs        | ~1.2μs         | <2μs       |
| Key lookup       | ~0.3μs        | ~0.5μs         | <1μs       |
| Value extraction | ~0.6μs        | ~0.9μs         | <2μs       |
| Type conversion  | ~0.4μs        | ~0.7μs         | <1μs       |
| Batch operations | ~0.2μs/key    | ~0.3μs/key     | <0.5μs/key |

### Memory Usage

- **Python**: ~200KB base + ~50 bytes per handle
- **Node.js**: ~150KB base + ~40 bytes per handle
- **Shared Core**: ~50KB registry + actual configuration data

## Next Steps

This FFI integration plan establishes the language binding strategy for SuperConfig V2. The next documents will cover:

- **09-performance-optimization-strategy.md**: SIMD acceleration, advanced caching, and memory optimization techniques
- **10-testing-and-benchmarking-plan.md**: Comprehensive testing approach and performance validation across all languages

The FFI design achieves performance targets through:

- Zero-copy data access where possible
- Efficient type conversion with minimal allocations
- Batch operations for reduced FFI overhead
- Automatic memory management with RAII patterns
- Native async support for non-blocking operations
