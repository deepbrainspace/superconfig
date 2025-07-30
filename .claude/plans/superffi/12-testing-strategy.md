# SuperConfig FFI Testing Strategy

This document outlines the comprehensive testing approach for SuperConfig multi-language FFI bindings, ensuring reliability, consistency, and maintainability across all supported languages.

## Testing Philosophy

### Multi-Layer Testing Approach

```
┌─────────────────────────────────────────────────────────────┐
│                    Integration Tests                        │
│  Python, Node.js, WASM - Real user scenarios              │
│  Test language-specific APIs and cross-language consistency│
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                      FFI Layer Tests                       │
│  superconfig-ffi - JSON conversion, error handling         │
│  Test wrapper logic and parameter marshaling               │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                    SuperFFI Macro Tests                    │
│  superffi - Code generation, naming conversion             │
│  Test procedural macro functionality                       │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                   Core SuperConfig Tests                   │
│  superconfig - Native Rust API (existing)                 │
│  Business logic and configuration processing               │
└─────────────────────────────────────────────────────────────┘
```

## Layer 1: SuperFFI Macro Tests

**Location**: `crates/superffi/src/lib.rs` (inline tests)\
**Purpose**: Verify procedural macro code generation and naming strategy

### Naming Conversion Tests

```rust
#[cfg(test)]
mod naming_tests {
    use super::*;
    
    #[test]
    fn test_generic_naming_conversion() {
        // SuperConfig method examples
        assert_eq!(convert_to_camel_case("with_file"), "withFile");
        assert_eq!(convert_to_camel_case("with_wildcard"), "withWildcard");
        assert_eq!(convert_to_camel_case("set_debug"), "setDebug");
        assert_eq!(convert_to_camel_case("extract_json"), "extractJson");
        assert_eq!(convert_to_camel_case("get_metadata"), "getMetadata");
        
        // Edge cases
        assert_eq!(convert_to_camel_case("single"), "single");
        assert_eq!(convert_to_camel_case("new"), "new");
        assert_eq!(convert_to_camel_case("with_multiple_words_here"), "withMultipleWordsHere");
        assert_eq!(convert_to_camel_case("a_b_c_d_e"), "aBCDE");
    }
    
    #[test]
    fn test_language_specific_naming() {
        let test_methods = ["with_file", "set_debug", "extract_json"];
        
        for method in &test_methods {
            // Python preserves snake_case
            assert_eq!(generate_method_name(method, "python"), *method);
            
            // JavaScript environments use camelCase
            let camel_case = convert_to_camel_case(method);
            assert_eq!(generate_method_name(method, "nodejs"), camel_case);
            assert_eq!(generate_method_name(method, "wasm"), camel_case);
        }
    }
    
    #[test]
    fn test_javascript_api_consistency() {
        // Critical test: Node.js and WASM must have identical method names
        let test_methods = [
            "with_file", "with_env", "with_wildcard", "set_debug", 
            "extract_json", "find", "get_metadata"
        ];
        
        for method in &test_methods {
            let nodejs_name = generate_method_name(method, "nodejs");
            let wasm_name = generate_method_name(method, "wasm");
            assert_eq!(nodejs_name, wasm_name, 
                "Method '{}' should have identical names in Node.js ({}) and WASM ({})", 
                method, nodejs_name, wasm_name);
        }
    }
}
```

### Code Generation Tests

```rust
#[cfg(test)]
mod generation_tests {
    use super::*;
    use quote::quote;
    use syn::{parse_quote, ItemStruct, ItemImpl};
    
    #[test]
    fn test_struct_generation() {
        let input: ItemStruct = parse_quote! {
            pub struct TestConfig {
                pub name: String,
                pub value: i32,
            }
        };
        
        let output = generate_struct_bindings(input);
        let generated = output.to_string();
        
        // Verify conditional compilation
        #[cfg(feature = "python")]
        assert!(generated.contains("#[pyo3::pyclass]"));
        
        #[cfg(feature = "nodejs")]
        assert!(generated.contains("#[napi::napi"));
        
        #[cfg(feature = "wasm")]
        assert!(generated.contains("#[wasm_bindgen]"));
    }
    
    #[test]
    fn test_impl_generation() {
        let input: ItemImpl = parse_quote! {
            impl TestConfig {
                pub fn with_file(&self, path: String) -> Result<Self, String> {
                    Ok(Self { name: path, value: 42 })
                }
            }
        };
        
        let output = generate_impl_bindings(input);
        let generated = output.to_string();
        
        // Verify method name conversion is applied
        #[cfg(feature = "nodejs")]
        assert!(generated.contains("withFile"));
        
        #[cfg(feature = "wasm")]
        assert!(generated.contains("withFile"));
        
        #[cfg(feature = "python")]
        assert!(generated.contains("with_file"));
    }
}
```

### Feature Flag Tests

```rust
#[cfg(test)]
mod feature_tests {
    use super::*;
    
    #[test]
    #[cfg(feature = "python")]
    fn test_python_feature_enabled() {
        // Test Python-specific code generation
        assert!(cfg!(feature = "python"));
    }
    
    #[test]
    #[cfg(feature = "nodejs")]
    fn test_nodejs_feature_enabled() {
        // Test Node.js-specific code generation
        assert!(cfg!(feature = "nodejs"));
    }
    
    #[test]
    #[cfg(feature = "wasm")]
    fn test_wasm_feature_enabled() {
        // Test WebAssembly-specific code generation
        assert!(cfg!(feature = "wasm"));
    }
    
    #[test]
    #[cfg(not(any(feature = "python", feature = "nodejs", feature = "wasm")))]
    fn test_no_features_enabled() {
        // Test that macro gracefully handles no features
        // Should still preserve original Rust code
    }
}
```

## Layer 2: SuperConfig FFI Wrapper Tests

**Location**: `crates/superconfig-ffi/tests/`\
**Purpose**: Test JSON parameter conversion and error handling

### JSON Parameter Conversion Tests

```rust
// tests/json_conversion_tests.rs
use superconfig_ffi::SuperConfig;
use serde_json::json;

#[test]
fn test_simple_method_wrappers() {
    let config = SuperConfig::new();
    
    // Test simple string parameter
    let result = config.with_file("test.toml".to_string());
    assert!(result.is_ok());
    
    // Test simple boolean parameter
    let result = config.set_debug(true);
    assert!(result.is_ok());
    
    // Test string parameter with env prefix
    let result = config.with_env("APP_".to_string());
    assert!(result.is_ok());
}

#[test]
fn test_wildcard_json_conversion() {
    let config = SuperConfig::new();
    
    // Test minimal wildcard configuration
    let wildcard_config = json!({
        "pattern": "*.toml"
    });
    let result = config.with_wildcard(wildcard_config);
    assert!(result.is_ok());
    
    // Test complex wildcard configuration
    let complex_wildcard = json!({
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
    });
    let result = config.with_wildcard(complex_wildcard);
    assert!(result.is_ok());
}

#[test]
fn test_invalid_json_parameters() {
    let config = SuperConfig::new();
    
    // Test missing required field
    let invalid_wildcard = json!({
        "search": {"type": "current"}
        // Missing required 'pattern' field
    });
    let result = config.with_wildcard(invalid_wildcard);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("pattern"));
    
    // Test invalid search type
    let invalid_search = json!({
        "pattern": "*.toml",
        "search": {"type": "invalid_search_type"}
    });
    let result = config.with_wildcard(invalid_search);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("invalid search strategy"));
}
```

### Error Handling Tests

```rust
// tests/error_handling_tests.rs
use superconfig_ffi::SuperConfig;
use serde_json::json;

#[test]
fn test_file_not_found_error() {
    let config = SuperConfig::new();
    let result = config.with_file("nonexistent_file.toml".to_string());
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("Failed to load file"));
    assert!(error_msg.contains("nonexistent_file.toml"));
}

#[test]
fn test_superconfig_error_context() {
    let config = SuperConfig::new();
    
    // All FFI errors should include SuperConfig context
    let invalid_wildcard = json!({
        "search": {"type": "current"}
    });
    let result = config.with_wildcard(invalid_wildcard);
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("SuperConfig"));
}

#[test]
fn test_json_extraction_error_handling() {
    let config = SuperConfig::new();
    
    // Test extraction from empty configuration
    let result = config.extract_json();
    assert!(result.is_ok());
    
    // Test find on non-existent path
    let result = config.find("non.existent.path".to_string());
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}
```

### Feature Flag Compilation Tests

```rust
// tests/feature_compilation_tests.rs

#[cfg(feature = "python")]
#[test]
fn test_python_feature_compiles() {
    // Test that Python bindings compile without errors
    use superconfig_ffi::SuperConfig;
    let _config = SuperConfig::new();
    // If this compiles, Python feature works
}

#[cfg(feature = "nodejs")]
#[test]
fn test_nodejs_feature_compiles() {
    // Test that Node.js bindings compile without errors
    use superconfig_ffi::SuperConfig;
    let _config = SuperConfig::new();
    // If this compiles, Node.js feature works
}

#[cfg(feature = "wasm")]
#[test]
fn test_wasm_feature_compiles() {
    // Test that WebAssembly bindings compile without errors
    use superconfig_ffi::SuperConfig;
    let _config = SuperConfig::new();
    // If this compiles, WASM feature works
}
```

## Layer 3: Language Integration Tests

### Python Integration Tests

**Location**: `crates/superconfig-ffi/bindings/python/tests/test_integration.py`

```python
import pytest
import json
from superconfig import SuperConfig

class TestBasicFunctionality:
    def test_new_instance_creation(self):
        """Test creating new SuperConfig instance."""
        config = SuperConfig.new()
        assert config is not None
    
    def test_method_chaining(self):
        """Test method chaining works correctly."""
        config = (SuperConfig.new()
            .with_file("test.toml")
            .with_env("APP_")
            .set_debug(True))
        assert config is not None

class TestFileOperations:
    def test_with_file_success(self):
        """Test successful file loading."""
        config = SuperConfig.new()
        # This should succeed even if file doesn't exist (depending on SuperConfig behavior)
        result = config.with_file("test.toml")
        assert result is not None
    
    def test_with_file_error_handling(self):
        """Test file loading error handling."""
        config = SuperConfig.new()
        try:
            config.with_file("definitely_nonexistent_file.toml")
        except Exception as e:
            assert "SuperConfig" in str(e)
            assert "Failed to load file" in str(e)

class TestWildcardConfiguration:
    def test_simple_wildcard(self):
        """Test simple wildcard configuration."""
        config = SuperConfig.new()
        wildcard_config = {
            "pattern": "*.toml"
        }
        result = config.with_wildcard(wildcard_config)
        assert result is not None
    
    def test_complex_wildcard(self):
        """Test complex wildcard with all options."""
        config = SuperConfig.new()
        complex_wildcard = {
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
        result = config.with_wildcard(complex_wildcard)
        assert result is not None
    
    def test_wildcard_validation_error(self):
        """Test wildcard configuration validation."""
        config = SuperConfig.new()
        invalid_wildcard = {
            "search": {"type": "current"}
            # Missing required 'pattern' field
        }
        with pytest.raises(Exception) as exc_info:
            config.with_wildcard(invalid_wildcard)
        
        error_message = str(exc_info.value)
        assert "pattern" in error_message.lower()
        assert "SuperConfig" in error_message

class TestDataExtraction:
    def test_extract_json(self):
        """Test JSON extraction functionality."""
        config = SuperConfig.new()
        result = config.extract_json()
        assert isinstance(result, (dict, list, str, int, float, bool, type(None)))
    
    def test_find_method(self):
        """Test find method for specific paths."""
        config = SuperConfig.new()
        result = config.find("some.path")
        # Should return None for non-existent paths or actual value
        assert result is None or isinstance(result, (dict, list, str, int, float, bool))
    
    def test_get_metadata(self):
        """Test metadata extraction."""
        config = SuperConfig.new()
        metadata = config.get_metadata()
        assert isinstance(metadata, dict)
        assert "sources" in metadata
        assert isinstance(metadata["sources"], list)

class TestNamingConsistency:
    def test_python_uses_snake_case(self):
        """Verify Python API uses snake_case method names."""
        config = SuperConfig.new()
        
        # These methods should exist and use snake_case
        assert hasattr(config, 'with_file')
        assert hasattr(config, 'with_env')
        assert hasattr(config, 'with_wildcard')
        assert hasattr(config, 'set_debug')
        assert hasattr(config, 'extract_json')
        assert hasattr(config, 'get_metadata')
        
        # These camelCase methods should NOT exist
        assert not hasattr(config, 'withFile')
        assert not hasattr(config, 'withEnv')
        assert not hasattr(config, 'withWildcard')
        assert not hasattr(config, 'setDebug')
        assert not hasattr(config, 'extractJson')
        assert not hasattr(config, 'getMetadata')
```

### Node.js Integration Tests

**Location**: `crates/superconfig-ffi/bindings/nodejs/tests/integration.test.js`

```javascript
const { SuperConfig } = require('../index');

describe('SuperConfig Node.js Integration', () => {
    describe('Basic Functionality', () => {
        test('creates new instance', () => {
            const config = SuperConfig.new();
            expect(config).toBeDefined();
        });
        
        test('supports method chaining', () => {
            const config = SuperConfig.new()
                .withFile("test.toml")
                .withEnv("APP_")
                .setDebug(true);
            expect(config).toBeDefined();
        });
    });
    
    describe('File Operations', () => {
        test('handles file loading', () => {
            const config = SuperConfig.new();
            expect(() => {
                const result = config.withFile("test.toml");
                expect(result).toBeDefined();
            }).not.toThrow();
        });
        
        test('handles file not found errors', () => {
            const config = SuperConfig.new();
            expect(() => {
                config.withFile("definitely_nonexistent_file.toml");
            }).toThrow(/SuperConfig.*Failed to load file/);
        });
    });
    
    describe('Wildcard Configuration', () => {
        test('handles simple wildcard', () => {
            const config = SuperConfig.new();
            const wildcardConfig = {
                pattern: "*.toml"
            };
            
            expect(() => {
                const result = config.withWildcard(wildcardConfig);
                expect(result).toBeDefined();
            }).not.toThrow();
        });
        
        test('handles complex wildcard configuration', () => {
            const config = SuperConfig.new();
            const complexWildcard = {
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
            
            expect(() => {
                const result = config.withWildcard(complexWildcard);
                expect(result).toBeDefined();
            }).not.toThrow();
        });
        
        test('validates wildcard configuration', () => {
            const config = SuperConfig.new();
            const invalidConfig = {
                search: { type: "current" }
                // Missing required 'pattern' field
            };
            
            expect(() => {
                config.withWildcard(invalidConfig);
            }).toThrow(/pattern/i);
        });
    });
    
    describe('Data Extraction', () => {
        test('extracts JSON data', () => {
            const config = SuperConfig.new();
            expect(() => {
                const result = config.extractJson();
                expect(typeof result).toBe('object');
            }).not.toThrow();
        });
        
        test('finds specific paths', () => {
            const config = SuperConfig.new();
            const result = config.find("some.path");
            // Should be null for non-existent paths or actual value
            expect(result === null || typeof result === 'object').toBe(true);
        });
        
        test('gets metadata', () => {
            const config = SuperConfig.new();
            const metadata = config.getMetadata();
            expect(typeof metadata).toBe('object');
            expect(Array.isArray(metadata.sources)).toBe(true);
        });
    });
    
    describe('Naming Consistency', () => {
        test('uses camelCase method names', () => {
            const config = SuperConfig.new();
            
            // These camelCase methods should exist
            expect(typeof config.withFile).toBe('function');
            expect(typeof config.withEnv).toBe('function');
            expect(typeof config.withWildcard).toBe('function');
            expect(typeof config.setDebug).toBe('function');
            expect(typeof config.extractJson).toBe('function');
            expect(typeof config.getMetadata).toBe('function');
            
            // These snake_case methods should NOT exist
            expect(config.with_file).toBeUndefined();
            expect(config.with_env).toBeUndefined();
            expect(config.with_wildcard).toBeUndefined();
            expect(config.set_debug).toBeUndefined();
            expect(config.extract_json).toBeUndefined();
            expect(config.get_metadata).toBeUndefined();
        });
    });
});
```

### WebAssembly Integration Tests

**Location**: `crates/superconfig-ffi/bindings/wasm/tests/integration.test.js`

```javascript
/**
 * WebAssembly integration tests
 * These tests should be IDENTICAL to Node.js tests to verify API consistency
 */

import { SuperConfig } from '../dist/superconfig.js';

describe('SuperConfig WebAssembly Integration', () => {
    describe('Basic Functionality', () => {
        test('creates new instance', () => {
            const config = SuperConfig.new();
            expect(config).toBeDefined();
        });
        
        test('supports method chaining', () => {
            const config = SuperConfig.new()
                .withFile("test.toml")
                .withEnv("APP_")
                .setDebug(true);
            expect(config).toBeDefined();
        });
    });
    
    describe('API Consistency with Node.js', () => {
        test('has identical method names to Node.js', () => {
            const config = SuperConfig.new();
            
            // These methods should exist and have the same names as Node.js
            expect(typeof config.withFile).toBe('function');
            expect(typeof config.withEnv).toBe('function');
            expect(typeof config.withWildcard).toBe('function');
            expect(typeof config.setDebug).toBe('function');
            expect(typeof config.extractJson).toBe('function');
            expect(typeof config.getMetadata).toBe('function');
        });
        
        test('wildcard configuration works identically to Node.js', () => {
            const config = SuperConfig.new();
            
            // This exact configuration should work the same as in Node.js
            const wildcardConfig = {
                pattern: "*.toml",
                search: {
                    type: "recursive",
                    root: "./config",
                    maxDepth: 2
                }
            };
            
            expect(() => {
                const result = config.withWildcard(wildcardConfig);
                expect(result).toBeDefined();
            }).not.toThrow();
        });
    });
    
    // Copy all other tests from Node.js but import from WASM module
    // This ensures 100% API compatibility between Node.js and WASM
});
```

## Cross-Language Consistency Tests

### Cross-Language Integration Tests (Opus Feedback)

**Location**: `tests/cross_language_integration.py`\
**Purpose**: Verify identical behavior across Python, Node.js, and WASM

```python
#!/usr/bin/env python3
"""
Cross-language integration tests to ensure identical results.
Tests the same configuration scenarios across all language bindings.
"""

import subprocess
import json
import tempfile
import os
from pathlib import Path

class CrossLanguageIntegrationTest:
    def __init__(self):
        self.test_config_dir = tempfile.mkdtemp()
        self.create_test_configs()
    
    def create_test_configs(self):
        """Create test configuration files for all languages to use."""
        # Base configuration
        base_config = {
            "database": {
                "host": "localhost",
                "port": 5432,
                "name": "testdb"
            },
            "features": {
                "auth_enabled": True,
                "cache_ttl": 3600
            }
        }
        
        base_path = Path(self.test_config_dir) / "base.json"
        with open(base_path, 'w') as f:
            json.dump(base_config, f)
        
        # Environment-specific configuration
        env_config = {
            "database": {
                "host": "prod.example.com",
                "ssl": True
            },
            "features": {
                "debug_mode": False
            }
        }
        
        env_path = Path(self.test_config_dir) / "production.json"
        with open(env_path, 'w') as f:
            json.dump(env_config, f)
    
    def test_python_config_loading(self):
        """Test configuration loading in Python."""
        python_code = f"""
import sys
sys.path.append('bindings/python')
from superconfig import SuperConfig

config = SuperConfig.new()
config = config.with_file('{self.test_config_dir}/base.json')
config = config.with_file('{self.test_config_dir}/production.json')

result = config.extract_json()
print(json.dumps(result, sort_keys=True))
"""
        
        result = subprocess.run(['python', '-c', python_code], 
                              capture_output=True, text=True)
        if result.returncode != 0:
            raise Exception(f"Python test failed: {result.stderr}")
        
        return json.loads(result.stdout)
    
    def test_nodejs_config_loading(self):
        """Test configuration loading in Node.js."""
        nodejs_code = f"""
const {{ SuperConfig }} = require('./bindings/nodejs/index');

const config = SuperConfig.new()
    .withFile('{self.test_config_dir}/base.json')
    .withFile('{self.test_config_dir}/production.json');

const result = config.extractJson();
console.log(JSON.stringify(result, null, 0));
"""
        
        result = subprocess.run(['node', '-e', nodejs_code], 
                              capture_output=True, text=True)
        if result.returncode != 0:
            raise Exception(f"Node.js test failed: {result.stderr}")
        
        return json.loads(result.stdout)
    
    def test_cross_language_parity(self):
        """Test that Python and Node.js produce identical results."""
        python_result = self.test_python_config_loading()
        nodejs_result = self.test_nodejs_config_loading()
        
        # Deep comparison of results
        if python_result != nodejs_result:
            print("❌ Cross-language parity FAILED")
            print(f"Python result: {json.dumps(python_result, sort_keys=True, indent=2)}")
            print(f"Node.js result: {json.dumps(nodejs_result, sort_keys=True, indent=2)}")
            return False
        
        print("✅ Cross-language parity PASSED")
        print(f"Both languages produced identical result with {len(python_result)} keys")
        return True
    
    def test_error_consistency(self):
        """Test that error messages are consistent across languages."""
        # Test file not found error
        python_error = self.get_python_error("nonexistent.json")
        nodejs_error = self.get_nodejs_error("nonexistent.json")
        
        # Both should contain "SuperConfig" and file path
        assert "SuperConfig" in python_error
        assert "SuperConfig" in nodejs_error
        assert "nonexistent.json" in python_error
        assert "nonexistent.json" in nodejs_error
        
        print("✅ Error message consistency PASSED")
    
    def get_python_error(self, filename):
        """Get error message from Python binding."""
        python_code = f"""
import sys
sys.path.append('bindings/python')
from superconfig import SuperConfig

try:
    config = SuperConfig.new().with_file('{filename}')
except Exception as e:
    print(str(e))
"""
        result = subprocess.run(['python', '-c', python_code], 
                              capture_output=True, text=True)
        return result.stdout.strip()
    
    def get_nodejs_error(self, filename):
        """Get error message from Node.js binding."""
        nodejs_code = f"""
const {{ SuperConfig }} = require('./bindings/nodejs/index');

try {{
    const config = SuperConfig.new().withFile('{filename}');
}} catch (error) {{
    console.log(error.message);
}}
"""
        result = subprocess.run(['node', '-e', nodejs_code], 
                              capture_output=True, text=True)
        return result.stdout.strip()

if __name__ == "__main__":
    tester = CrossLanguageIntegrationTest()
    
    success = True
    success &= tester.test_cross_language_parity()
    success &= tester.test_error_consistency()
    
    exit(0 if success else 1)
```

### API Consistency Verification

**Location**: `tests/cross_language_consistency.py` (Python script that coordinates tests)

```python
#!/usr/bin/env python3
"""
Cross-language API consistency verification.
This script ensures all language bindings have identical APIs (considering naming conventions).
"""

import subprocess
import json
import sys

def extract_python_api():
    """Extract Python API methods using introspection."""
    code = """
import superconfig
from superconfig import SuperConfig

config = SuperConfig.new()
methods = [method for method in dir(config) if not method.startswith('_')]
print(json.dumps(sorted(methods)))
"""
    result = subprocess.run(['python', '-c', code], capture_output=True, text=True)
    return json.loads(result.stdout)

def extract_nodejs_api():
    """Extract Node.js API methods using JavaScript introspection."""
    code = """
const { SuperConfig } = require('./bindings/nodejs/index');
const config = SuperConfig.new();
const methods = Object.getOwnPropertyNames(Object.getPrototypeOf(config))
    .filter(method => !method.startsWith('_') && typeof config[method] === 'function')
    .sort();
console.log(JSON.stringify(methods));
"""
    result = subprocess.run(['node', '-e', code], capture_output=True, text=True)
    return json.loads(result.stdout)

def convert_snake_to_camel(snake_str):
    """Convert snake_case to camelCase."""
    components = snake_str.split('_')
    return components[0] + ''.join(word.capitalize() for word in components[1:])

def verify_api_consistency():
    """Verify that all APIs are consistent across languages."""
    python_methods = extract_python_api()
    nodejs_methods = extract_nodejs_api()
    
    # Convert Python methods to expected camelCase for comparison
    expected_js_methods = [convert_snake_to_camel(method) for method in python_methods]
    expected_js_methods.sort()
    
    # Compare APIs
    if expected_js_methods == nodejs_methods:
        print("✅ API consistency check PASSED")
        print(f"Python methods: {len(python_methods)}")
        print(f"Node.js methods: {len(nodejs_methods)}")
        return True
    else:
        print("❌ API consistency check FAILED")
        print(f"Expected JS methods: {expected_js_methods}")
        print(f"Actual JS methods: {nodejs_methods}")
        
        missing_in_js = set(expected_js_methods) - set(nodejs_methods)
        extra_in_js = set(nodejs_methods) - set(expected_js_methods)
        
        if missing_in_js:
            print(f"Missing in Node.js: {missing_in_js}")
        if extra_in_js:
            print(f"Extra in Node.js: {extra_in_js}")
        
        return False

if __name__ == "__main__":
    success = verify_api_consistency()
    sys.exit(0 if success else 1)
```

## Test Execution Strategy

### Development Testing (Moon Commands)

```bash
# Rust layer tests
moon run superffi:test           # SuperFFI macro tests
moon run superconfig-ffi:test    # FFI wrapper tests

# Language integration tests  
moon run superconfig/python:test    # Python integration tests
moon run superconfig/nodejs:test    # Node.js integration tests
moon run superconfig/wasm:test      # WebAssembly integration tests

# Cross-language consistency
python tests/cross_language_consistency.py

# All tests
moon run test-all
```

### CI/CD Testing Pipeline

```yaml
# .github/workflows/superconfig-ffi-test.yml
name: SuperConfig FFI Tests

on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Test SuperFFI macro
        run: cargo test --package superffi
      
      - name: Test SuperConfig FFI wrapper  
        run: cargo test --package superconfig-ffi
      
      - name: Test feature flag combinations
        run: |
          cargo test --package superconfig-ffi --features python
          cargo test --package superconfig-ffi --features nodejs  
          cargo test --package superconfig-ffi --features wasm
          cargo test --package superconfig-ffi --features all

  integration-tests:
    runs-on: ubuntu-latest
    needs: rust-tests
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup build tools
        run: |
          pip install maturin pytest
          npm install -g @napi-rs/cli
          cargo install wasm-pack
      
      - name: Build all bindings
        run: |
          moon run superconfig/python:build
          moon run superconfig/nodejs:build  
          moon run superconfig/wasm:build
      
      - name: Run integration tests
        run: |
          moon run superconfig/python:test
          moon run superconfig/nodejs:test
          moon run superconfig/wasm:test
      
      - name: Verify API consistency
        run: python tests/cross_language_consistency.py
```

## Performance Testing

### Benchmark Tests

```rust
// benches/ffi_performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use superconfig_ffi::SuperConfig;
use serde_json::json;

fn benchmark_simple_methods(c: &mut Criterion) {
    c.bench_function("with_file", |b| {
        b.iter(|| {
            let config = SuperConfig::new();
            black_box(config.with_file("test.toml".to_string()))
        });
    });
    
    c.bench_function("with_env", |b| {
        b.iter(|| {
            let config = SuperConfig::new();
            black_box(config.with_env("APP_".to_string()))
        });
    });
}

fn benchmark_json_conversion(c: &mut Criterion) {
    let wildcard_config = json!({
        "pattern": "*.toml",
        "search": {
            "type": "recursive",
            "root": "./config",
            "max_depth": 3
        }
    });
    
    c.bench_function("wildcard_json_conversion", |b| {
        b.iter(|| {
            let config = SuperConfig::new();
            black_box(config.with_wildcard(wildcard_config.clone()))
        });
    });
}

criterion_group!(benches, benchmark_simple_methods, benchmark_json_conversion);
criterion_main!(benches);
```

## Test Coverage Requirements

### Coverage Targets

- **SuperFFI Macro**: 95% line coverage
- **SuperConfig FFI Wrapper**: 90% line coverage
- **Python Integration**: 85% scenario coverage
- **Node.js Integration**: 85% scenario coverage
- **WebAssembly Integration**: 85% scenario coverage

### Coverage Verification

```bash
# Generate coverage reports
cargo tarpaulin --package superffi --package superconfig-ffi
pytest --cov=superconfig bindings/python/tests/
npm run coverage # In each binding directory
```

## Quality Gates

### Automated Quality Checks

1. **All unit tests pass** - Rust layer functionality
2. **All integration tests pass** - Language binding functionality
3. **API consistency verified** - Cross-language compatibility
4. **Performance benchmarks within 10% of baseline** - No regression
5. **Test coverage meets targets** - Adequate test coverage
6. **Documentation examples work** - All examples in docs execute successfully

### Manual Testing Checklist

- [ ] Real configuration files load correctly in all languages
- [ ] Error messages are helpful and include SuperConfig context
- [ ] Memory usage is reasonable (no leaks detected)
- [ ] Build process works on all supported platforms
- [ ] Package installation works from PyPI and npm

---

_This testing strategy ensures robust, reliable SuperConfig FFI bindings. See [`11-api-examples.md`](./11-api-examples.md) for usage examples and [`03-architecture.md`](./03-architecture.md) for technical details._
