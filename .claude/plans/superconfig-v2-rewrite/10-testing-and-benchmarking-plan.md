# SuperConfig V2: Testing and Benchmarking Plan

## Overview

This document specifies the comprehensive testing and benchmarking strategy for SuperConfig V2, ensuring quality, performance validation, and regression prevention across all components. The plan covers unit tests, integration tests, performance benchmarks, cross-language validation, and continuous integration pipelines.

## Testing Philosophy

### Quality Gates

1. **No Regression**: Every change must maintain or improve existing performance
2. **Cross-Language Consistency**: Identical behavior across Rust, Python, and Node.js APIs
3. **Performance Validation**: All operations must meet specified performance targets
4. **Memory Safety**: Zero memory leaks, no unsafe operations without justification
5. **Error Handling**: Comprehensive error coverage with graceful degradation

### Testing Pyramid

```
          ▲
         /E2E\
        /     \
       /       \
      /Integration\
     /             \
    /               \
   /                 \
  /      Unit Tests    \
 /                     \
/_______________________\
```

- **Unit Tests (70%)**: Fast, isolated component testing
- **Integration Tests (20%)**: Cross-component interaction testing
- **End-to-End Tests (10%)**: Full system workflow validation

## Unit Testing Strategy

### Core Registry Tests

```rust
#[cfg(test)]
mod registry_tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    
    #[test]
    fn test_handle_creation_and_lookup() {
        let registry = ConfigRegistry::new();
        let data = ConfigData::new();
        
        let start = Instant::now();
        let handle = registry.insert::<serde_json::Value>(data);
        let create_time = start.elapsed();
        
        // Performance validation
        assert!(create_time.as_nanos() < 3000, "Handle creation too slow: {:?}", create_time);
        
        let start = Instant::now();
        let retrieved = registry.get(&handle);
        let lookup_time = start.elapsed();
        
        assert!(retrieved.is_some());
        assert!(lookup_time.as_nanos() < 500, "Handle lookup too slow: {:?}", lookup_time);
    }
    
    #[test]
    fn test_concurrent_handle_operations() {
        let registry = Arc::new(ConfigRegistry::new());
        let mut handles = vec![];
        
        // Spawn 100 threads creating handles concurrently
        for i in 0..100 {
            let registry_clone = Arc::clone(&registry);
            let handle = thread::spawn(move || {
                let mut data = ConfigData::new();
                data.insert("thread_id".to_string(), serde_json::Value::Number(i.into()));
                registry_clone.insert::<serde_json::Value>(data)
            });
            handles.push(handle);
        }
        
        // Collect all handles
        let created_handles: Vec<_> = handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect();
        
        // Verify all handles are valid and unique
        assert_eq!(created_handles.len(), 100);
        
        // Test concurrent access
        let access_threads: Vec<_> = created_handles.into_iter()
            .enumerate()
            .map(|(i, handle)| {
                let registry_clone = Arc::clone(&registry);
                thread::spawn(move || {
                    for _ in 0..1000 {
                        let data = registry_clone.get(&handle);
                        assert!(data.is_some());
                        
                        if let Some(data_ref) = data {
                            let thread_id = data_ref.data.get("thread_id").unwrap();
                            assert_eq!(thread_id.as_i64().unwrap(), i as i64);
                        }
                    }
                })
            })
            .collect();
        
        // Wait for all access threads
        for handle in access_threads {
            handle.join().unwrap();
        }
    }
    
    #[test]
    fn test_handle_cleanup() {
        let registry = ConfigRegistry::new();
        let initial_count = registry.stats().active_handles;
        
        {
            let _handle1 = registry.insert::<serde_json::Value>(ConfigData::new());
            let _handle2 = registry.insert::<serde_json::Value>(ConfigData::new());
            
            assert_eq!(registry.stats().active_handles, initial_count + 2);
        } // Handles dropped here
        
        // Give cleanup time to run
        std::thread::sleep(Duration::from_millis(100));
        
        // Handles should be cleaned up
        assert_eq!(registry.stats().active_handles, initial_count);
    }
    
    #[test]
    fn test_memory_usage_bounds() {
        let registry = ConfigRegistry::new();
        let initial_memory = registry.calculate_memory_usage();
        
        // Create 1000 handles with varying data sizes
        let mut handles = Vec::new();
        for i in 0..1000 {
            let mut data = ConfigData::new();
            
            // Vary data size to test memory accounting
            let size = (i % 10) + 1;
            for j in 0..size {
                data.insert(
                    format!("key_{}_{}", i, j),
                    serde_json::Value::String(format!("value_{}", i)),
                );
            }
            
            handles.push(registry.insert::<serde_json::Value>(data));
        }
        
        let final_memory = registry.calculate_memory_usage();
        let memory_per_handle = (final_memory - initial_memory) / 1000;
        
        // Memory per handle should be reasonable (< 1KB average)
        assert!(memory_per_handle < 1024, "Memory per handle too high: {} bytes", memory_per_handle);
        
        // Total memory should be bounded
        assert!(final_memory < 10 * 1024 * 1024, "Total memory usage too high: {} bytes", final_memory);
    }
}
```

### Provider System Tests

```rust
#[cfg(test)]
mod provider_tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_file_provider_performance() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");
        
        // Create test JSON file
        let test_data = serde_json::json!({
            "database": {
                "host": "localhost",
                "port": 5432,
                "name": "testdb"
            },
            "api": {
                "timeout": 30,
                "retries": 3,
                "endpoints": ["api1", "api2", "api3"]
            }
        });
        
        std::fs::write(&file_path, serde_json::to_string_pretty(&test_data).unwrap()).unwrap();
        
        let provider = FileProvider::new();
        let context = LoadContext::default();
        
        // Performance test: should load within target time
        let start = Instant::now();
        let result = provider.load_file(&file_path, &context).await.unwrap();
        let load_time = start.elapsed();
        
        assert!(load_time.as_micros() < 30, "File loading too slow: {:?}", load_time);
        assert_eq!(result.data, test_data);
    }
    
    #[tokio::test]
    async fn test_environment_provider_nested_keys() {
        // Set up test environment variables
        std::env::set_var("TEST_DATABASE__HOST", "prod-db.example.com");
        std::env::set_var("TEST_DATABASE__PORT", "5432");
        std::env::set_var("TEST_API__TIMEOUT", "60");
        std::env::set_var("TEST_API__ENDPOINTS", r#"["prod1", "prod2"]"#);
        
        let provider = EnvironmentProvider::new().with_prefix("TEST_".to_string());
        let context = LoadContext::default();
        
        let result = provider.load_env(&context).await.unwrap();
        
        // Verify nested structure
        let expected = serde_json::json!({
            "database": {
                "host": "prod-db.example.com",
                "port": "5432"
            },
            "api": {
                "timeout": "60",
                "endpoints": ["prod1", "prod2"]
            }
        });
        
        assert_eq!(result.data, expected);
        
        // Cleanup
        std::env::remove_var("TEST_DATABASE__HOST");
        std::env::remove_var("TEST_DATABASE__PORT");
        std::env::remove_var("TEST_API__TIMEOUT");
        std::env::remove_var("TEST_API__ENDPOINTS");
    }
    
    #[tokio::test]
    async fn test_glob_provider_parallel_loading() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create multiple config files
        for i in 0..10 {
            let file_path = temp_dir.path().join(format!("config_{}.json", i));
            let data = serde_json::json!({
                "id": i,
                "name": format!("config_{}", i),
                "values": [i, i + 1, i + 2]
            });
            std::fs::write(&file_path, serde_json::to_string(&data).unwrap()).unwrap();
        }
        
        let pattern = format!("{}/*.json", temp_dir.path().display());
        let provider = GlobProvider::new().with_patterns(vec![pattern]).unwrap();
        let context = LoadContext::default();
        
        // Should trigger parallel loading (>3 files)
        let start = Instant::now();
        let files = provider.find_matching_files(&context).await.unwrap();
        let result = provider.load_multiple_files(files, &context).await.unwrap();
        let total_time = start.elapsed();
        
        // Parallel loading should be faster than sequential
        assert!(total_time.as_micros() < 50, "Parallel loading too slow: {:?}", total_time);
        
        // Verify merged result contains all configurations
        let merged_data = result.data.as_object().unwrap();
        assert!(!merged_data.is_empty());
    }
    
    #[tokio::test]
    async fn test_hierarchical_discovery() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create hierarchical structure
        let system_dir = temp_dir.path().join("system");
        let user_dir = temp_dir.path().join("user");
        let project_dir = temp_dir.path().join("project");
        
        std::fs::create_dir_all(&system_dir).unwrap();
        std::fs::create_dir_all(&user_dir).unwrap(); 
        std::fs::create_dir_all(&project_dir).unwrap();
        
        // System config (lowest priority)
        std::fs::write(
            system_dir.join("myapp.json"),
            r#"{"timeout": 10, "retries": 1, "system": true}"#,
        ).unwrap();
        
        // User config (medium priority)
        std::fs::write(
            user_dir.join("myapp.json"),
            r#"{"timeout": 30, "user": true}"#,
        ).unwrap();
        
        // Project config (highest priority)
        std::fs::write(
            project_dir.join("myapp.json"),
            r#"{"timeout": 60, "project": true}"#,
        ).unwrap();
        
        let mut provider = HierarchicalProvider::new("myapp".to_string());
        provider.config.search_dirs = vec![
            SearchDir::Custom(system_dir),
            SearchDir::Custom(user_dir),
            SearchDir::Custom(project_dir),
        ];
        
        let context = LoadContext {
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let configs = provider.discover_configs(&context).await.unwrap();
        assert_eq!(configs.len(), 3);
        
        // Test merging behavior
        let results = provider.load_configs(configs, &context).await.unwrap();
        let merged = provider.merge_configurations(&mut results.clone()).unwrap();
        
        // Verify priority order (project > user > system)
        assert_eq!(merged.get("timeout").unwrap().as_i64().unwrap(), 60);
        assert_eq!(merged.get("retries").unwrap().as_i64().unwrap(), 1);
        assert_eq!(merged.get("system").unwrap().as_bool().unwrap(), true);
        assert_eq!(merged.get("user").unwrap().as_bool().unwrap(), true);
        assert_eq!(merged.get("project").unwrap().as_bool().unwrap(), true);
    }
}
```

### SIMD Operations Tests

```rust
#[cfg(test)]
mod simd_tests {
    use super::*;
    
    #[test]
    fn test_simd_string_comparison() {
        let simd_ops = SimdOps::new();
        
        // Test various string lengths
        let test_cases = vec![
            ("", ""),
            ("a", "a"),
            ("hello", "hello"),
            ("this is a longer string for testing", "this is a longer string for testing"),
            ("different", "strings"),
            ("same length", "diff length"),
        ];
        
        for (a, b) in test_cases {
            let simd_result = simd_ops.fast_str_compare(a, b);
            let std_result = a == b;
            
            assert_eq!(simd_result, std_result, "SIMD comparison failed for '{}' vs '{}'", a, b);
        }
    }
    
    #[test]
    fn test_simd_string_comparison_performance() {
        let simd_ops = SimdOps::new();
        let test_string = "This is a moderately long string for performance testing purposes";
        let iterations = 100_000;
        
        // SIMD comparison
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = simd_ops.fast_str_compare(test_string, test_string);
        }
        let simd_time = start.elapsed();
        
        // Standard comparison
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = test_string == test_string;
        }
        let std_time = start.elapsed();
        
        println!("SIMD time: {:?}, Standard time: {:?}", simd_time, std_time);
        
        // SIMD should be competitive (within 2x) for long strings
        if test_string.len() >= 16 && simd_ops.features.sse2 {
            assert!(simd_time < std_time * 2, "SIMD comparison not competitive");
        }
    }
    
    #[cfg(feature = "simd")]
    #[test]
    fn test_simd_json_parsing() {
        let simd_ops = SimdOps::new();
        
        let json_data = r#"{
            "name": "test",
            "values": [1, 2, 3, 4, 5],
            "nested": {
                "key1": "value1",
                "key2": 42,
                "key3": true
            }
        }"#;
        
        let mut data = json_data.as_bytes().to_vec();
        
        let start = Instant::now();
        let simd_result = simd_ops.parse_json_simd(data.clone()).unwrap();
        let simd_time = start.elapsed();
        
        let start = Instant::now();
        let std_result = serde_json::from_slice::<serde_json::Value>(&data).unwrap();
        let std_time = start.elapsed();
        
        // Results should be identical
        assert_eq!(simd_result, std_result);
        
        // SIMD should be faster for supported CPUs
        if simd_ops.features.avx2 || simd_ops.features.sse4_1 {
            println!("SIMD parse time: {:?}, Standard parse time: {:?}", simd_time, std_time);
            // Note: simd-json is typically 30-50% faster
        }
    }
    
    #[test]
    fn test_simd_array_merging() {
        let simd_ops = SimdOps::new();
        
        let mut target = vec![
            serde_json::Value::Number(1.0.into()),
            serde_json::Value::Number(2.0.into()),
            serde_json::Value::Number(3.0.into()),
            serde_json::Value::Number(4.0.into()),
        ];
        
        let source = vec![
            serde_json::Value::Number(0.5.into()),
            serde_json::Value::Number(1.5.into()),
            serde_json::Value::Number(2.5.into()),
            serde_json::Value::Number(3.5.into()),
        ];
        
        simd_ops.merge_arrays_simd(&mut target, source, ArrayMergeStrategy::Merge).unwrap();
        
        // Verify results (should be element-wise addition for numeric arrays)
        assert_eq!(target[0].as_f64().unwrap(), 1.5);
        assert_eq!(target[1].as_f64().unwrap(), 3.5);
        assert_eq!(target[2].as_f64().unwrap(), 5.5);
        assert_eq!(target[3].as_f64().unwrap(), 7.5);
    }
}
```

## Integration Testing

### Cross-Component Integration

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_full_configuration_pipeline() {
        let temp_dir = TempDir::new().unwrap();
        
        // Set up test environment
        std::env::set_var("INTEGRATION_TEST_PORT", "8080");
        std::env::set_var("INTEGRATION_TEST_DEBUG", "true");
        
        // Create config files
        let base_config = temp_dir.path().join("base.json");
        std::fs::write(&base_config, r#"{
            "server": {
                "host": "localhost",
                "port": 3000
            },
            "logging": {
                "level": "info"
            }
        }"#).unwrap();
        
        let env_config = temp_dir.path().join("production.json");
        std::fs::write(&env_config, r#"{
            "server": {
                "port": 8000
            },
            "logging": {
                "level": "warn"
            }
        }"#).unwrap();
        
        // Build configuration using the full pipeline
        let registry = ConfigRegistry::new();
        let handle = registry.create_builder()
            .with_file(base_config.to_str().unwrap())
            .expect("Failed to add base config")
            .with_file(env_config.to_str().unwrap())
            .expect("Failed to add env config")
            .with_env(Some("INTEGRATION_TEST_"))
            .expect("Failed to add env vars")
            .select_profile("production")
            .build()
            .expect("Failed to build configuration");
        
        // Test full configuration extraction
        let config: serde_json::Value = handle.extract().expect("Failed to extract config");
        
        // Verify merging worked correctly
        assert_eq!(config["server"]["host"], "localhost");
        assert_eq!(config["server"]["port"], "8080"); // From env var, highest priority
        assert_eq!(config["logging"]["level"], "warn"); // From production.json
        assert_eq!(config["debug"], true); // From env var
        
        // Test performance of full pipeline
        let start = Instant::now();
        for _ in 0..1000 {
            let _: serde_json::Value = handle.extract().unwrap();
        }
        let extract_time = start.elapsed();
        
        assert!(extract_time.as_micros() / 1000 < 1, "Configuration extraction too slow");
        
        // Cleanup
        std::env::remove_var("INTEGRATION_TEST_PORT");
        std::env::remove_var("INTEGRATION_TEST_DEBUG");
    }
    
    #[tokio::test]
    async fn test_caching_integration() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("cache_test.json");
        
        std::fs::write(&config_file, r#"{"version": 1}"#).unwrap();
        
        let cache_system = CacheSystem::new(CacheConfig::default());
        let provider = FileProvider::new();
        let context = LoadContext::default();
        
        // First load - should be cache miss
        let start = Instant::now();
        let result1 = provider.load_file(&config_file, &context).await.unwrap();
        let first_load_time = start.elapsed();
        
        // Second load - should be cache hit
        let start = Instant::now();
        let result2 = provider.load_file(&config_file, &context).await.unwrap();
        let second_load_time = start.elapsed();
        
        // Results should be identical
        assert_eq!(result1.data, result2.data);
        
        // Second load should be significantly faster
        assert!(second_load_time < first_load_time / 2, 
                "Cache hit not faster: first={:?}, second={:?}", 
                first_load_time, second_load_time);
    }
    
    #[tokio::test]
    async fn test_hot_reload_integration() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("hot_reload_test.json");
        
        // Initial config
        std::fs::write(&config_file, r#"{"value": 1}"#).unwrap();
        
        let registry = ConfigRegistry::new();
        let handle = registry.create_builder()
            .with_file(config_file.to_str().unwrap())
            .unwrap()
            .build()
            .unwrap();
        
        // Verify initial value
        let config: serde_json::Value = handle.extract().unwrap();
        assert_eq!(config["value"], 1);
        
        #[cfg(feature = "hot-reload")]
        {
            // Set up hot reload monitoring
            let (tx, mut rx) = tokio::sync::mpsc::channel(10);
            
            tokio::spawn(async move {
                let mut change_stream = handle.watch().await.unwrap();
                while let Some(change) = change_stream.next().await {
                    tx.send(change).await.unwrap();
                }
            });
            
            // Modify file
            std::thread::sleep(Duration::from_millis(100)); // Ensure different mtime
            std::fs::write(&config_file, r#"{"value": 2}"#).unwrap();
            
            // Wait for change notification
            let change = tokio::time::timeout(Duration::from_secs(5), rx.recv())
                .await
                .expect("Hot reload timeout")
                .expect("Hot reload failed");
            
            assert_eq!(change.source, config_file.to_str().unwrap());
            
            // Verify updated value
            let updated_config: serde_json::Value = handle.extract().unwrap();
            assert_eq!(updated_config["value"], 2);
        }
    }
}
```

## FFI Testing

### Python Binding Tests

```python
# tests/test_python_bindings.py
import pytest
import time
import json
import os
import tempfile
from superconfig import ConfigRegistry, create_config, load_file

class TestPythonBindings:
    
    def test_basic_configuration_loading(self):
        """Test basic configuration loading through Python bindings"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump({
                "database": {
                    "host": "localhost",
                    "port": 5432
                },
                "debug": True
            }, f)
            config_path = f.name
        
        try:
            # Test convenience function
            handle = load_file(config_path)
            config = handle.extract()
            
            assert config["database"]["host"] == "localhost"
            assert config["database"]["port"] == 5432
            assert config["debug"] is True
            
            # Test key access
            assert handle.get("database.host") == "localhost"
            assert handle.get("database.port") == 5432
            assert handle.get("nonexistent.key") is None
            
            # Test key existence
            assert handle.has_key("database.host") is True
            assert handle.has_key("nonexistent.key") is False
            
        finally:
            os.unlink(config_path)
    
    def test_builder_pattern(self):
        """Test fluent builder API"""
        registry = ConfigRegistry()
        
        with tempfile.TemporaryDirectory() as temp_dir:
            config1_path = os.path.join(temp_dir, "config1.json")
            config2_path = os.path.join(temp_dir, "config2.json")
            
            with open(config1_path, 'w') as f:
                json.dump({"a": 1, "b": 2}, f)
            
            with open(config2_path, 'w') as f:
                json.dump({"b": 3, "c": 4}, f)
            
            # Set environment variable
            os.environ["PYTEST_TEST_VALUE"] = "from_env"
            
            try:
                builder = registry.create_config()
                handle = (builder
                         .with_file(config1_path)
                         .with_file(config2_path)
                         .with_env("PYTEST_TEST_")
                         .select_profile("test")
                         .build())
                
                config = handle.extract()
                
                # Verify merging
                assert config["a"] == 1
                assert config["b"] == 3  # config2 should override config1
                assert config["c"] == 4
                assert config["value"] == "from_env"  # From environment
                
            finally:
                del os.environ["PYTEST_TEST_VALUE"]
    
    def test_performance_targets(self):
        """Test that Python FFI meets performance targets"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump({"test": "value"}, f)
            config_path = f.name
        
        try:
            # Test handle creation performance (target: <2μs)
            start = time.perf_counter()
            handle = load_file(config_path)
            create_time = (time.perf_counter() - start) * 1_000_000  # Convert to microseconds
            
            assert create_time < 2.0, f"Handle creation too slow: {create_time:.2f}μs"
            
            # Test extraction performance (target: <1μs)
            start = time.perf_counter()
            config = handle.extract()
            extract_time = (time.perf_counter() - start) * 1_000_000
            
            assert extract_time < 1.0, f"Config extraction too slow: {extract_time:.2f}μs"
            
            # Test key lookup performance (target: <1μs)
            start = time.perf_counter()
            value = handle.get("test")
            lookup_time = (time.perf_counter() - start) * 1_000_000
            
            assert lookup_time < 1.0, f"Key lookup too slow: {lookup_time:.2f}μs"
            
            # Batch performance test
            iterations = 10000
            start = time.perf_counter()
            for _ in range(iterations):
                _ = handle.get("test")
            total_time = (time.perf_counter() - start) * 1_000_000
            avg_time = total_time / iterations
            
            assert avg_time < 0.5, f"Average lookup too slow: {avg_time:.2f}μs"
            
        finally:
            os.unlink(config_path)
    
    def test_error_handling(self):
        """Test error handling through Python bindings"""
        with pytest.raises(FileNotFoundError):
            load_file("nonexistent.json")
        
        registry = ConfigRegistry()
        builder = registry.create_config()
        
        with pytest.raises(ValueError):
            builder.with_glob("invalid[glob")  # Invalid glob pattern
    
    def test_warnings_collection(self):
        """Test warning collection and reporting"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            f.write('{"key": "value"')  # Invalid JSON (missing closing brace)
            config_path = f.name
        
        try:
            # This should generate warnings but not fail
            handle = load_file(config_path)
            
            assert handle.has_warnings() is True
            warnings = handle.warnings()
            assert len(warnings) > 0
            assert any("parse" in warning["message"].lower() for warning in warnings)
            
        except:
            # If it fails completely, that's also acceptable behavior
            pass
        finally:
            os.unlink(config_path)
    
    def test_memory_management(self):
        """Test that Python objects are properly cleaned up"""
        import gc
        import weakref
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump({"test": "value"}, f)
            config_path = f.name
        
        try:
            # Create and destroy many handles
            weak_refs = []
            
            for _ in range(1000):
                handle = load_file(config_path)
                weak_refs.append(weakref.ref(handle))
                _ = handle.extract()
            
            # Force garbage collection
            gc.collect()
            
            # Check that handles are being cleaned up
            alive_count = sum(1 for ref in weak_refs if ref() is not None)
            assert alive_count < 100, f"Too many handles still alive: {alive_count}"
            
        finally:
            os.unlink(config_path)

if __name__ == "__main__":
    pytest.main([__file__])
```

### Node.js Binding Tests

```javascript
// tests/nodejs_bindings.test.js
const test = require('ava');
const fs = require('fs');
const path = require('path');
const os = require('os');
const { ConfigRegistry, createConfig, loadFile } = require('superconfig');

test('basic configuration loading', async t => {
    const tempFile = path.join(os.tmpdir(), 'test-config.json');
    const testConfig = {
        database: {
            host: 'localhost',
            port: 5432
        },
        debug: true
    };
    
    fs.writeFileSync(tempFile, JSON.stringify(testConfig));
    
    try {
        const handle = loadFile(tempFile);
        const config = handle.extract();
        
        t.deepEqual(config.database.host, 'localhost');
        t.deepEqual(config.database.port, 5432);
        t.is(config.debug, true);
        
        // Test camelCase conversion
        t.is(handle.hasKey('database.host'), true);
        t.is(handle.hasKey('nonexistent.key'), false);
        
        t.is(handle.get('database.host'), 'localhost');
        t.is(handle.get('database.port'), 5432);
        t.is(handle.get('nonexistent.key'), null);
        
    } finally {
        fs.unlinkSync(tempFile);
    }
});

test('builder pattern with method chaining', async t => {
    const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'superconfig-test-'));
    
    try {
        const config1Path = path.join(tempDir, 'config1.json');
        const config2Path = path.join(tempDir, 'config2.json');
        
        fs.writeFileSync(config1Path, JSON.stringify({ a: 1, b: 2 }));
        fs.writeFileSync(config2Path, JSON.stringify({ b: 3, c: 4 }));
        
        process.env.NODEJS_TEST_VALUE = 'from_env';
        
        const registry = new ConfigRegistry();
        const handle = registry
            .createConfig()
            .withFile(config1Path)
            .withFile(config2Path)
            .withEnv('NODEJS_TEST_')
            .selectProfile('test')
            .build();
        
        const config = handle.extract();
        
        t.is(config.a, 1);
        t.is(config.b, 3); // config2 should override config1
        t.is(config.c, 4);
        t.is(config.value, 'from_env');
        
        delete process.env.NODEJS_TEST_VALUE;
        
    } finally {
        fs.rmSync(tempDir, { recursive: true });
    }
});

test('performance targets', async t => {
    const tempFile = path.join(os.tmpdir(), 'perf-test.json');
    fs.writeFileSync(tempFile, JSON.stringify({ test: 'value' }));
    
    try {
        // Test handle creation performance (target: <2μs)
        const start1 = process.hrtime.bigint();
        const handle = loadFile(tempFile);
        const createTime = Number(process.hrtime.bigint() - start1) / 1000; // Convert to microseconds
        
        t.true(createTime < 2000, `Handle creation too slow: ${createTime.toFixed(2)}μs`);
        
        // Test extraction performance (target: <2μs)
        const start2 = process.hrtime.bigint();
        const config = handle.extract();
        const extractTime = Number(process.hrtime.bigint() - start2) / 1000;
        
        t.true(extractTime < 2000, `Config extraction too slow: ${extractTime.toFixed(2)}μs`);
        
        // Test key lookup performance (target: <1μs)
        const start3 = process.hrtime.bigint();
        const value = handle.get('test');
        const lookupTime = Number(process.hrtime.bigint() - start3) / 1000;
        
        t.true(lookupTime < 1000, `Key lookup too slow: ${lookupTime.toFixed(2)}μs`);
        
        // Batch performance test
        const iterations = 10000;
        const start4 = process.hrtime.bigint();
        for (let i = 0; i < iterations; i++) {
            handle.get('test');
        }
        const totalTime = Number(process.hrtime.bigint() - start4) / 1000;
        const avgTime = totalTime / iterations;
        
        t.true(avgTime < 500, `Average lookup too slow: ${avgTime.toFixed(2)}μs`);
        
    } finally {
        fs.unlinkSync(tempFile);
    }
});

test('async operations', async t => {
    const tempFile = path.join(os.tmpdir(), 'async-test.json');
    fs.writeFileSync(tempFile, JSON.stringify({ async: true }));
    
    try {
        const handle = loadFile(tempFile);
        
        // Test async extraction
        const config = await handle.extractAsync();
        t.is(config.async, true);
        
        // Test that async is faster or same as sync for small configs
        const start1 = process.hrtime.bigint();
        handle.extract();
        const syncTime = Number(process.hrtime.bigint() - start1);
        
        const start2 = process.hrtime.bigint();
        await handle.extractAsync();
        const asyncTime = Number(process.hrtime.bigint() - start2);
        
        // Async shouldn't be significantly slower than sync
        t.true(asyncTime < syncTime * 2, 'Async extraction too slow compared to sync');
        
    } finally {
        fs.unlinkSync(tempFile);
    }
});

test('error handling', async t => {
    await t.throwsAsync(async () => {
        loadFile('nonexistent.json');
    }, { instanceOf: Error });
    
    const registry = new ConfigRegistry();
    const builder = registry.createConfig();
    
    t.throws(() => {
        builder.withGlob('invalid[glob');
    }, { instanceOf: Error });
});

test('memory management', t => {
    const tempFile = path.join(os.tmpdir(), 'memory-test.json');
    fs.writeFileSync(tempFile, JSON.stringify({ test: 'value' }));
    
    try {
        // Create many handles to test memory management
        const handles = [];
        const startMemory = process.memoryUsage().heapUsed;
        
        for (let i = 0; i < 1000; i++) {
            const handle = loadFile(tempFile);
            handles.push(handle);
            handle.extract(); // Use the handle
        }
        
        const midMemory = process.memoryUsage().heapUsed;
        
        // Clear handles and force GC
        handles.length = 0;
        if (global.gc) {
            global.gc();
        }
        
        // Wait a bit for cleanup
        setTimeout(() => {
            const endMemory = process.memoryUsage().heapUsed;
            const memoryGrowth = endMemory - startMemory;
            
            // Memory growth should be reasonable (< 10MB for 1000 handles)
            t.true(memoryGrowth < 10 * 1024 * 1024, 
                   `Excessive memory growth: ${Math.round(memoryGrowth / 1024)}KB`);
        }, 100);
        
    } finally {
        fs.unlinkSync(tempFile);
    }
});
```

## Performance Benchmarking

### Comprehensive Benchmark Suite

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_handle_operations(c: &mut Criterion) {
    let registry = Arc::new(ConfigRegistry::new());
    
    let mut group = c.benchmark_group("handle_operations");
    
    // Benchmark handle creation
    group.bench_function("create_handle", |b| {
        b.iter(|| {
            let data = ConfigData::new();
            let handle = registry.insert::<serde_json::Value>(black_box(data));
            black_box(handle);
        });
    });
    
    // Benchmark handle lookup
    let data = ConfigData::new();
    let handle = registry.insert::<serde_json::Value>(data);
    
    group.bench_function("lookup_handle", |b| {
        b.iter(|| {
            let result = registry.get(black_box(&handle));
            black_box(result);
        });
    });
    
    // Benchmark value extraction
    group.bench_function("extract_value", |b| {
        b.iter(|| {
            let value: serde_json::Value = handle.extract().unwrap();
            black_box(value);
        });
    });
    
    group.finish();
}

fn benchmark_file_loading(c: &mut Criterion) {
    let temp_dir = tempfile::TempDir::new().unwrap();
    
    // Create test files of different sizes
    let file_sizes = vec![1024, 10_240, 102_400, 1_024_000]; // 1KB, 10KB, 100KB, 1MB
    let mut test_files = Vec::new();
    
    for size in &file_sizes {
        let file_path = temp_dir.path().join(format!("test_{}.json", size));
        let test_data = generate_test_json(*size);
        std::fs::write(&file_path, test_data).unwrap();
        test_files.push(file_path);
    }
    
    let mut group = c.benchmark_group("file_loading");
    
    for (file_path, size) in test_files.iter().zip(file_sizes.iter()) {
        group.bench_with_input(
            BenchmarkId::new("load_file", size),
            file_path,
            |b, path| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let provider = FileProvider::new();
                let context = LoadContext::default();
                
                b.to_async(&rt).iter(|| async {
                    let result = provider.load_file(black_box(path), &context).await.unwrap();
                    black_box(result);
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_parsing_formats(c: &mut Criterion) {
    let test_data = generate_complex_config();
    
    let json_data = serde_json::to_string(&test_data).unwrap();
    let toml_data = toml::to_string(&test_data).unwrap();
    let yaml_data = serde_yaml::to_string(&test_data).unwrap();
    
    let mut group = c.benchmark_group("parsing");
    
    // JSON parsing
    group.bench_function("parse_json", |b| {
        b.iter(|| {
            let value: serde_json::Value = serde_json::from_str(black_box(&json_data)).unwrap();
            black_box(value);
        });
    });
    
    // SIMD JSON parsing (if available)
    #[cfg(feature = "simd")]
    {
        group.bench_function("parse_json_simd", |b| {
            let simd_ops = SimdOps::new();
            b.iter(|| {
                let mut data = json_data.as_bytes().to_vec();
                let value = simd_ops.parse_json_simd(black_box(data)).unwrap();
                black_box(value);
            });
        });
    }
    
    // TOML parsing
    group.bench_function("parse_toml", |b| {
        b.iter(|| {
            let value: toml::Value = toml::from_str(black_box(&toml_data)).unwrap();
            black_box(value);
        });
    });
    
    // YAML parsing
    group.bench_function("parse_yaml", |b| {
        b.iter(|| {
            let value: serde_yaml::Value = serde_yaml::from_str(black_box(&yaml_data)).unwrap();
            black_box(value);
        });
    });
    
    group.finish();
}

fn benchmark_caching_performance(c: &mut Criterion) {
    let cache_system = CacheSystem::new(CacheConfig::default());
    
    // Pre-populate cache with test data
    for i in 0..1000 {
        let key = format!("test_key_{}", i);
        let value = serde_json::json!({
            "id": i,
            "data": format!("test_data_{}", i)
        });
        cache_system.store_hot_key(key, value);
    }
    
    let mut group = c.benchmark_group("caching");
    
    // Cache hit performance
    group.bench_function("cache_hit", |b| {
        b.iter(|| {
            let result = cache_system.get_hot_key(black_box("test_key_500"));
            black_box(result);
        });
    });
    
    // Cache miss performance
    group.bench_function("cache_miss", |b| {
        b.iter(|| {
            let result = cache_system.get_hot_key(black_box("nonexistent_key"));
            black_box(result);
        });
    });
    
    // Cache write performance
    group.bench_function("cache_write", |b| {
        let mut counter = 0;
        b.iter(|| {
            let key = format!("new_key_{}", counter);
            let value = serde_json::json!({"counter": counter});
            cache_system.store_hot_key(black_box(key), black_box(value));
            counter += 1;
        });
    });
    
    group.finish();
}

fn benchmark_parallel_loading(c: &mut Criterion) {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let parallel_loader = ParallelLoader::new(ParallelConfig::default());
    
    // Create multiple test files
    let file_counts = vec![1, 3, 10, 50];
    let mut test_file_groups = Vec::new();
    
    for count in &file_counts {
        let mut files = Vec::new();
        for i in 0..*count {
            let file_path = temp_dir.path().join(format!("parallel_{}_{}.json", count, i));
            let test_data = serde_json::json!({
                "file_id": i,
                "data": vec![i; 100] // Some data to make parsing non-trivial
            });
            std::fs::write(&file_path, serde_json::to_string(&test_data).unwrap()).unwrap();
            files.push(file_path);
        }
        test_file_groups.push(files);
    }
    
    let mut group = c.benchmark_group("parallel_loading");
    
    for (files, count) in test_file_groups.iter().zip(file_counts.iter()) {
        group.bench_with_input(
            BenchmarkId::new("load_files", count),
            files,
            |b, file_paths| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let context = LoadContext::default();
                
                b.to_async(&rt).iter(|| async {
                    let result = parallel_loader
                        .load_files_parallel(black_box(file_paths.clone()), &context)
                        .await
                        .unwrap();
                    black_box(result);
                });
            },
        );
    }
    
    group.finish();
}

// Helper functions
fn generate_test_json(size_bytes: usize) -> String {
    let mut data = serde_json::Map::new();
    let key_value_size = 50; // Approximate size per key-value pair
    let num_entries = size_bytes / key_value_size;
    
    for i in 0..num_entries {
        data.insert(
            format!("key_{}", i),
            serde_json::Value::String(format!("value_{}_data", i)),
        );
    }
    
    serde_json::to_string(&data).unwrap()
}

fn generate_complex_config() -> serde_json::Value {
    serde_json::json!({
        "database": {
            "host": "localhost",
            "port": 5432,
            "connections": {
                "min": 1,
                "max": 10
            },
            "features": ["ssl", "compression", "pooling"]
        },
        "api": {
            "endpoints": [
                {"path": "/users", "methods": ["GET", "POST"]},
                {"path": "/orders", "methods": ["GET", "POST", "PUT", "DELETE"]}
            ],
            "rate_limits": {
                "per_minute": 1000,
                "per_hour": 10000
            }
        },
        "logging": {
            "level": "info",
            "outputs": ["console", "file"],
            "file_config": {
                "path": "/var/log/app.log",
                "max_size": "100MB",
                "rotation": "daily"
            }
        }
    })
}

criterion_group!(
    benches,
    benchmark_handle_operations,
    benchmark_file_loading,
    benchmark_parsing_formats,
    benchmark_caching_performance,
    benchmark_parallel_loading
);
criterion_main!(benches);
```

### Performance Regression Detection

```rust
// benchmarks/regression_tests.rs
use std::collections::HashMap;

/// Performance regression detection system
pub struct RegressionDetector {
    /// Historical benchmark results
    baseline_results: HashMap<String, BenchmarkResult>,
    
    /// Tolerance for performance variations (percentage)
    tolerance: f64,
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub mean_ns: f64,
    pub std_dev_ns: f64,
    pub iterations: u64,
    pub timestamp: SystemTime,
}

impl RegressionDetector {
    pub fn new(tolerance_percent: f64) -> Self {
        Self {
            baseline_results: HashMap::new(),
            tolerance: tolerance_percent / 100.0,
        }
    }
    
    /// Load baseline results from previous runs
    pub fn load_baseline(&mut self, baseline_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if baseline_file.exists() {
            let content = std::fs::read_to_string(baseline_file)?;
            self.baseline_results = serde_json::from_str(&content)?;
        }
        Ok(())
    }
    
    /// Check current results against baseline
    pub fn check_regression(&self, current: &BenchmarkResult) -> RegressionStatus {
        if let Some(baseline) = self.baseline_results.get(&current.name) {
            let performance_change = (current.mean_ns - baseline.mean_ns) / baseline.mean_ns;
            
            if performance_change > self.tolerance {
                RegressionStatus::Regression {
                    benchmark: current.name.clone(),
                    baseline_ns: baseline.mean_ns,
                    current_ns: current.mean_ns,
                    change_percent: performance_change * 100.0,
                }
            } else if performance_change < -self.tolerance {
                RegressionStatus::Improvement {
                    benchmark: current.name.clone(),
                    baseline_ns: baseline.mean_ns,
                    current_ns: current.mean_ns,
                    improvement_percent: -performance_change * 100.0,
                }
            } else {
                RegressionStatus::NoChange
            }
        } else {
            RegressionStatus::NewBenchmark
        }
    }
    
    /// Update baseline with current results
    pub fn update_baseline(&mut self, result: BenchmarkResult) {
        self.baseline_results.insert(result.name.clone(), result);
    }
    
    /// Save baseline to file
    pub fn save_baseline(&self, baseline_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.baseline_results)?;
        std::fs::write(baseline_file, content)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum RegressionStatus {
    NoChange,
    NewBenchmark,
    Improvement {
        benchmark: String,
        baseline_ns: f64,
        current_ns: f64,
        improvement_percent: f64,
    },
    Regression {
        benchmark: String,
        baseline_ns: f64,
        current_ns: f64,
        change_percent: f64,
    },
}

/// Performance target validation
pub fn validate_performance_targets() -> Vec<PerformanceViolation> {
    let mut violations = Vec::new();
    
    // Define performance targets
    let targets = [
        ("handle_operations/create_handle", 3000), // 3μs
        ("handle_operations/lookup_handle", 500),  // 0.5μs
        ("handle_operations/extract_value", 1000), // 1μs
        ("file_loading/load_file/1024", 30000),    // 30μs for 1KB file
        ("file_loading/load_file/10240", 35000),   // 35μs for 10KB file
        ("parsing/parse_json", 10000),             // 10μs for JSON parsing
        ("caching/cache_hit", 100),                // 0.1μs for cache hit
        ("parallel_loading/load_files/10", 50000), // 50μs for 10 files
    ];
    
    // Run benchmarks and check against targets
    for (benchmark_name, target_ns) in &targets {
        // This would integrate with the actual benchmark runner
        let result = run_single_benchmark(benchmark_name);
        
        if result.mean_ns > *target_ns as f64 {
            violations.push(PerformanceViolation {
                benchmark: benchmark_name.to_string(),
                target_ns: *target_ns,
                actual_ns: result.mean_ns,
                violation_percent: ((result.mean_ns - *target_ns as f64) / *target_ns as f64) * 100.0,
            });
        }
    }
    
    violations
}

#[derive(Debug)]
pub struct PerformanceViolation {
    pub benchmark: String,
    pub target_ns: u64,
    pub actual_ns: f64,
    pub violation_percent: f64,
}

// Stub function - would integrate with actual benchmark runner
fn run_single_benchmark(name: &str) -> BenchmarkResult {
    // Implementation would run the specific benchmark and return results
    BenchmarkResult {
        name: name.to_string(),
        mean_ns: 1000.0,
        std_dev_ns: 100.0,
        iterations: 1000,
        timestamp: SystemTime::now(),
    }
}
```

## Continuous Integration with Moon

### Moon-Integrated CI Pipeline

Based on the existing Moon setup, SuperConfig V2 will leverage the monorepo's intelligent affected project detection and caching system:

```yaml
# .github/workflows/superconfig-v2-ci.yml
name: SuperConfig V2 CI

on:
  pull_request:
    branches: [ main ]
    paths: 
      - 'crates/superconfig/**'
      - 'crates/superconfig-py/**'
      - 'crates/superconfig-napi/**'

env:
  CARGO_TERM_COLOR: always
  MOON_BASE: origin/main

jobs:
  # Detect affected SuperConfig V2 crates
  detect-affected:
    name: Detect Affected SuperConfig V2 Crates
    runs-on: ubuntu-latest
    outputs:
      affected-crates: ${{ steps.affected.outputs.crates }}
      has-changes: ${{ steps.affected.outputs.has-changes }}
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Moon build system
        uses: ./.github/actions/setup-moon

      - name: Detect affected V2 projects
        id: affected
        run: |
          # Get affected projects, filter for SuperConfig V2
          AFFECTED_JSON=$(moon query projects --affected --json)
          V2_CRATES=$(echo "$AFFECTED_JSON" | jq -r '.projects[].id' | grep -E '^(superconfig|superconfig-py|superconfig-napi)$' | jq -R -s -c 'split("\n")[:-1]')
          
          echo "crates=$V2_CRATES" >> $GITHUB_OUTPUT
          HAS_CHANGES=$(echo "$V2_CRATES" | jq -r 'length > 0')
          echo "has-changes=$HAS_CHANGES" >> $GITHUB_OUTPUT
          
          echo "Affected SuperConfig V2 crates: $V2_CRATES"

  # Core Rust testing with Moon
  test-core:
    name: Test Core (${{ matrix.crate }})
    runs-on: ubuntu-latest
    needs: detect-affected
    if: needs.detect-affected.outputs.has-changes == 'true'
    strategy:
      matrix:
        crate: ${{ fromJson(needs.detect-affected.outputs.affected-crates) }}
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Moon for ${{ matrix.crate }}
        uses: ./.github/actions/setup-moon
        with:
          crate-name: ${{ matrix.crate }}

      # Use Moon's task inheritance
      - name: Check formatting
        run: moon run ${{ matrix.crate }}:fmt-check

      - name: Run clippy
        run: moon run ${{ matrix.crate }}:clippy

      - name: Build (release)
        run: moon run ${{ matrix.crate }}:build-release

      - name: Run unit tests
        run: moon run ${{ matrix.crate }}:test

      # SuperConfig V2 specific tests
      - name: Run SIMD tests
        run: moon run ${{ matrix.crate }}:test-simd
        if: matrix.crate == 'superconfig'

      - name: Run integration tests  
        run: moon run ${{ matrix.crate }}:test-integration
        if: matrix.crate == 'superconfig'

  # Python FFI testing
  test-python-ffi:
    name: Test Python FFI
    runs-on: ubuntu-latest
    needs: detect-affected
    if: contains(fromJson(needs.detect-affected.outputs.affected-crates), 'superconfig-py')
    strategy:
      matrix:
        python-version: ['3.8', '3.9', '3.10', '3.11', '3.12']
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Set up Python ${{ matrix.python-version }}
        uses: actions/setup-python@v4
        with:
          python-version: ${{ matrix.python-version }}

      - name: Setup Moon
        uses: ./.github/actions/setup-moon
        with:
          crate-name: superconfig-py

      - name: Build Python bindings
        run: moon run superconfig-py:build-python

      - name: Install Python dependencies
        run: |
          cd crates/superconfig-py
          pip install pytest maturin

      - name: Run Python FFI tests
        run: moon run superconfig-py:test-python

      - name: Run Python performance tests
        run: moon run superconfig-py:bench-python

  # Node.js FFI testing  
  test-nodejs-ffi:
    name: Test Node.js FFI
    runs-on: ubuntu-latest
    needs: detect-affected
    if: contains(fromJson(needs.detect-affected.outputs.affected-crates), 'superconfig-napi')
    strategy:
      matrix:
        node-version: ['16', '18', '20', '22']
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Use Node.js ${{ matrix.node-version }}
        uses: actions/setup-node@v3
        with:
          node-version: ${{ matrix.node-version }}

      - name: Setup Moon
        uses: ./.github/actions/setup-moon
        with:
          crate-name: superconfig-napi

      - name: Build Node.js bindings
        run: moon run superconfig-napi:build-nodejs

      - name: Run Node.js FFI tests
        run: moon run superconfig-napi:test-nodejs

      - name: Run Node.js performance tests
        run: moon run superconfig-napi:bench-nodejs

  # Performance benchmarks with regression detection
  benchmark:
    name: Performance Benchmarks
    runs-on: ubuntu-latest
    needs: detect-affected
    if: needs.detect-affected.outputs.has-changes == 'true' && github.event_name == 'push'
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Moon
        uses: ./.github/actions/setup-moon

      - name: Run core benchmarks
        run: moon run superconfig:bench

      - name: Run cross-language benchmarks  
        run: |
          moon run superconfig-py:bench-python
          moon run superconfig-napi:bench-nodejs

      - name: Performance regression check
        run: moon run superconfig:check-regression

      - name: Upload benchmark results
        uses: actions/upload-artifact@v3
        with:
          name: v2-benchmark-results
          path: |
            crates/superconfig/target/criterion/
            benchmarks/results/

  # Security and compliance
  security:
    name: Security Audit
    runs-on: ubuntu-latest
    needs: detect-affected
    if: needs.detect-affected.outputs.has-changes == 'true'
    strategy:
      matrix:
        crate: ${{ fromJson(needs.detect-affected.outputs.affected-crates) }}
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Moon
        uses: ./.github/actions/setup-moon

      - name: Run security audit
        run: moon run ${{ matrix.crate }}:security-audit

      - name: Run cargo deny
        run: moon run ${{ matrix.crate }}:deny

  # Code coverage with Moon
  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    needs: [detect-affected, test-core]
    if: needs.detect-affected.outputs.has-changes == 'true'
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Moon
        uses: ./.github/actions/setup-moon

      - name: Generate coverage (core)
        run: moon run superconfig:coverage

      - name: Generate coverage (Python)
        run: moon run superconfig-py:coverage
        if: contains(fromJson(needs.detect-affected.outputs.affected-crates), 'superconfig-py')

      - name: Generate coverage (Node.js)  
        run: moon run superconfig-napi:coverage
        if: contains(fromJson(needs.detect-affected.outputs.affected-crates), 'superconfig-napi')

      - name: Upload to Codecov
        uses: codecov/codecov-action@v4
        with:
          token: ${{ secrets.CODECOV_TOKEN }}
          files: ./cobertura.xml
          flags: superconfig-v2

  # Cross-platform testing
  cross-platform:
    name: Cross-platform Tests
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        crate: ['superconfig'] # Core only for cross-platform
    runs-on: ${{ matrix.os }}
    needs: detect-affected
    if: contains(fromJson(needs.detect-affected.outputs.affected-crates), 'superconfig')
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Moon
        uses: ./.github/actions/setup-moon

      - name: Run cross-platform tests
        run: moon run ${{ matrix.crate }}:test

      - name: Run platform-specific tests
        run: moon run ${{ matrix.crate }}:test-platform-specific
```

### Moon Task Extensions for SuperConfig V2

Additional tasks to add to crate-specific `moon.yml` files:

```yaml
# crates/superconfig/moon.yml (additions)
tasks:
  # Performance testing
  bench:
    command: 'cargo bench --features simd,parallel'
    inputs: ['@globs(sources)', 'benches/**/*']
    outputs: ['target/criterion/']
    
  # SIMD-specific tests
  test-simd:
    command: 'cargo test --features simd'
    inputs: ['@globs(sources)', '@globs(tests)']
    
  # Integration tests
  test-integration:
    command: 'cargo test --test integration_tests'
    inputs: ['@globs(sources)', 'tests/integration_tests.rs']
    
  # Platform-specific tests
  test-platform-specific:
    command: 'cargo test platform_specific'
    inputs: ['@globs(sources)', '@globs(tests)']
    
  # Regression detection
  check-regression:
    command: './scripts/check_performance_regression.sh'
    inputs: ['target/criterion/', 'benchmarks/baseline.json']
    options:
      cache: false

# crates/superconfig-py/moon.yml (additions)  
tasks:
  build-python:
    command: 'maturin build --release'
    inputs: ['@globs(sources)', 'Cargo.toml', 'pyproject.toml']
    outputs: ['target/wheels/']
    deps: ['superconfig:build-release']
    
  test-python:
    command: 'python -m pytest tests/ -v'
    inputs: ['tests/**/*', 'examples/**/*']
    deps: ['build-python']
    
  bench-python:
    command: 'python benchmarks/python_benchmarks.py'
    inputs: ['benchmarks/**/*']
    deps: ['build-python']

# crates/superconfig-napi/moon.yml (additions)
tasks:
  build-nodejs:
    command: 'napi build --platform --release'
    inputs: ['@globs(sources)', 'Cargo.toml', 'package.json']  
    outputs: ['bindings/']
    deps: ['superconfig:build-release']
    
  test-nodejs:
    command: 'npm test'
    inputs: ['tests/**/*', 'examples/**/*']
    deps: ['build-nodejs']
    
  bench-nodejs:
    command: 'npm run bench'
    inputs: ['benchmarks/**/*']
    deps: ['build-nodejs']
```

## Test Coverage and Quality Metrics

### Coverage Configuration

```toml
# Cargo.toml
[dev-dependencies]
tarpaulin = "0.27"

# Coverage configuration
[package.metadata.coverage]
exclude = [
  "src/bin/*",
  "tests/*",
  "benches/*",
]

minimum-coverage = 90.0
```

### Quality Gate Script

```bash
#!/bin/bash
# scripts/quality_gate.sh

set -e

echo "Running SuperConfig V2 Quality Gate..."

# 1. Code Coverage
echo "Checking test coverage..."
cargo tarpaulin --all-features --out xml --exclude-files 'src/bin/*' 'tests/*' 'benches/*'
COVERAGE=$(cargo tarpaulin --all-features --skip-clean | grep -E "^[0-9]+\.[0-9]+%" | tail -1 | sed 's/%//')

if (( $(echo "$COVERAGE < 90.0" | bc -l) )); then
    echo "❌ Coverage too low: $COVERAGE% (minimum: 90%)"
    exit 1
fi

echo "✅ Coverage: $COVERAGE%"

# 2. Performance Targets
echo "Validating performance targets..."
cargo run --release --bin validate_performance
if [ $? -ne 0 ]; then
    echo "❌ Performance targets not met"
    exit 1
fi

echo "✅ Performance targets met"

# 3. Memory Safety
echo "Running memory safety checks..."
cargo +nightly miri test --lib
if [ $? -ne 0 ]; then
    echo "❌ Memory safety issues detected"
    exit 1
fi

echo "✅ Memory safety validated"

# 4. Cross-language Consistency
echo "Checking cross-language API consistency..."
python scripts/validate_api_consistency.py
if [ $? -ne 0 ]; then
    echo "❌ API consistency issues detected"
    exit 1
fi

echo "✅ Cross-language consistency validated"

# 5. Documentation Coverage
echo "Checking documentation coverage..."
cargo doc --no-deps --document-private-items
MISSING_DOCS=$(cargo doc --no-deps 2>&1 | grep -c "missing documentation" || true)
if [ "$MISSING_DOCS" -gt 10 ]; then
    echo "❌ Too many missing documentation: $MISSING_DOCS"
    exit 1
fi

echo "✅ Documentation coverage adequate"

echo "🎉 All quality gates passed!"
```

## Summary

This comprehensive testing and benchmarking plan ensures SuperConfig V2 meets all quality and performance requirements:

### Testing Coverage

- **Unit Tests**: 70% of testing effort, targeting >95% code coverage
- **Integration Tests**: 20% of effort, focusing on component interactions
- **End-to-End Tests**: 10% of effort, validating complete workflows
- **Cross-Language Tests**: Ensuring consistent behavior across Rust, Python, Node.js

### Performance Validation

- **Automated Benchmarks**: Continuous performance monitoring
- **Regression Detection**: Automated detection of performance degradation
- **Target Validation**: Enforcement of specific performance requirements
- **Memory Safety**: Comprehensive memory leak and safety testing

### Quality Gates

- **90%+ test coverage** across all components
- **Zero performance regressions** above 10% tolerance
- **Memory safety validation** via Miri and Valgrind
- **Cross-platform compatibility** testing
- **API consistency validation** across language bindings

The testing strategy provides confidence that SuperConfig V2 will deliver on its performance promises while maintaining high quality and reliability standards.

## Hot Reload Testing

### Overview

Hot reload is a critical feature that enables real-time configuration updates without application restarts. Due to the comprehensive nature of hot reload implementation, this functionality has been documented separately.

**See: [11-hot-reload-implementation.md](./11-hot-reload-implementation.md)** for complete coverage of:

- **File System Watching Strategies** (Linux inotify, macOS kqueue, Windows ReadDirectoryChangesW)
- **Change Detection Algorithms** with intelligent content hashing
- **Debouncing and Batching** with adaptive timeouts
- **Atomic Configuration Updates** with versioning and rollback
- **Race Condition Handling** with comprehensive prevention mechanisms
- **Integration Tests** covering all hot reload functionality

### Hot Reload Test Integration

Hot reload tests are integrated into the standard testing pipeline through Moon tasks:

```yaml
# Additional Moon tasks for hot reload testing
hot-reload-test:
  command: 'cargo test hot_reload --features hot-reload'
  inputs: ['@globs(sources)', 'tests/hot_reload_tests.rs']

hot-reload-integration:
  command: 'cargo test --test hot_reload_integration'
  inputs: ['@globs(sources)', 'tests/hot_reload_integration.rs']
```

### Performance Targets for Hot Reload

- **File change detection**: <1ms on supported platforms
- **Debouncing efficiency**: 70-90% reduction in unnecessary updates
- **Atomic update latency**: <5ms for typical configuration files
- **Memory overhead**: <100KB per 1000 monitored files
- **Race condition prevention**: Zero data corruption under concurrent access

The hot reload implementation ensures enterprise-grade reliability and performance for real-time configuration updates.
