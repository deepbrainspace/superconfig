# Phase 4: Build & Publishing Integration

**Status**: ⏳ PENDING\
**Estimated Duration**: 2-3 hours\
**Dependencies**: Phase 3 Complete

## Overview

Phase 4 establishes the complete build system and CI/CD pipeline for multi-language package distribution. This phase creates the bindings directory structure, Moon task configurations, and GitHub Actions workflows needed to publish SuperConfig FFI packages to PyPI and npm registries.

## Deliverables

### 🎯 **Core Objectives**

1. **Create bindings directory structure** with proper Moon project configurations
2. **Implement Moon task hierarchy** for check → build → test → publish pipeline
3. **Configure GitHub Actions workflow** for automated multi-language releases
4. **Set up package configurations** for Python (PyPI) and Node.js/WASM (npm)

### 📋 **Implementation Tasks**

#### Task 1: Bindings Directory Structure (30 minutes)

**Create Directory Layout**:

```
crates/superconfig-ffi/
├── src/lib.rs                    # Phase 2 & 3 implementation
├── Cargo.toml                    # Feature flags for python/nodejs/wasm
├── moon.yml                      # Rust development tasks
└── bindings/
    ├── python/
    │   ├── moon.yml              # Project: superconfig/python
    │   ├── setup.py              # Maturin configuration
    │   ├── pyproject.toml        # Modern Python packaging
    │   ├── tests/
    │   │   └── test_integration.py
    │   └── superconfig/
    │       └── __init__.py       # Python entry point
    ├── nodejs/
    │   ├── moon.yml              # Project: superconfig/nodejs  
    │   ├── package.json          # npm package configuration
    │   ├── index.js              # JavaScript entry point
    │   └── tests/
    │       └── integration.test.js
    └── wasm/
        ├── moon.yml              # Project: superconfig/wasm
        ├── package.json          # WASM package configuration
        ├── webpack.config.js     # Bundling configuration
        └── tests/
            └── integration.test.js
```

#### Task 2: Python Binding Configuration (45 minutes)

**Create `bindings/python/moon.yml`**:

```yaml
# Project: superconfig/python
language: 'python'
type: 'library'

tasks:
  check:
    command: 'cargo check --manifest-path ../../Cargo.toml --features python'
    inputs: ['../../src/**/*', '../../Cargo.toml']
    
  build:
    command: 'maturin build --manifest-path ../../Cargo.toml --features python --release'
    inputs: ['../../src/**/*', '../../Cargo.toml', '**/*']
    outputs: ['dist/*.whl']
    deps: ['check']
    
  test:
    command: 'python -m pytest tests/ -v'
    inputs: ['tests/**/*', 'dist/*.whl']
    deps: ['build']
    
  publish:
    command: 'twine upload dist/*'
    deps: ['test']
    env:
      TWINE_PASSWORD: '$PYPI_TOKEN'
```

**Create `bindings/python/pyproject.toml`**:

```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "superconfig"
description = "Powerful configuration management for Python applications"
readme = "README.md"
authors = [
  { name = "DeepBrain Team", email = "team@deepbrain.space" },
]
license = { text = "MIT OR Apache-2.0" }
keywords = ["configuration", "config", "settings", "toml", "json", "yaml"]
classifiers = [
  "Development Status :: 4 - Beta",
  "Intended Audience :: Developers",
  "License :: OSI Approved :: MIT License",
  "License :: OSI Approved :: Apache Software License",
  "Programming Language :: Python :: 3",
  "Programming Language :: Python :: 3.8",
  "Programming Language :: Python :: 3.9",
  "Programming Language :: Python :: 3.10",
  "Programming Language :: Python :: 3.11",
  "Programming Language :: Python :: 3.12",
  "Programming Language :: Rust",
  "Topic :: Software Development :: Libraries :: Python Modules",
  "Topic :: System :: Systems Administration",
]
requires-python = ">=3.8"
dependencies = []

[project.urls]
Homepage = "https://github.com/deepbrainspace/superconfig"
Repository = "https://github.com/deepbrainspace/superconfig"
Documentation = "https://docs.superconfig.dev"

[tool.maturin]
features = ["python"]
module-name = "superconfig._superconfig"
python-source = "superconfig"

[tool.pytest.ini_options]
testpaths = ["tests"]
python_files = ["test_*.py"]
```

**Create `bindings/python/superconfig/__init__.py`**:

```python
"""
SuperConfig - Powerful configuration management for Python applications.

This package provides a Python interface to the SuperConfig Rust library,
offering high-performance configuration loading and merging capabilities.
"""

from ._superconfig import SuperConfig

__version__ = "0.1.0"
__all__ = ["SuperConfig"]

# Re-export the main class for easy importing
SuperConfig = SuperConfig
```

**Create `bindings/python/tests/test_integration.py`**:

```python
import pytest
import json
from superconfig import SuperConfig

def test_basic_functionality():
    """Test basic SuperConfig functionality through Python FFI."""
    config = SuperConfig.new()
    assert config is not None

def test_with_file_method():
    """Test file-based configuration loading."""
    config = SuperConfig.new()
    
    # This should handle the case where file doesn't exist gracefully
    try:
        result = config.with_file("nonexistent.toml")
        # If it succeeds, we got a new config instance
        assert result is not None
    except Exception as e:
        # If it fails, we should get a meaningful error message
        assert "Failed to load file" in str(e) or "No such file" in str(e)

def test_json_parameter_handling():
    """Test complex type handling through JSON parameters."""
    config = SuperConfig.new()
    
    wildcard_config = {
        "pattern": "*.toml",
        "search": {"type": "current"}
    }
    
    try:
        result = config.with_wildcard(wildcard_config)
        assert result is not None
    except Exception as e:
        # Should get SuperConfig-specific error messages
        assert "SuperConfig" in str(e)

def test_error_handling():
    """Test that error messages are properly propagated."""
    config = SuperConfig.new()
    
    # Test invalid JSON parameter
    invalid_wildcard = {
        "search": {"type": "current"}
        # Missing required 'pattern' field
    }
    
    with pytest.raises(Exception) as exc_info:
        config.with_wildcard(invalid_wildcard)
    
    error_message = str(exc_info.value)
    assert "pattern" in error_message.lower()

def test_extract_json():
    """Test JSON extraction functionality."""
    config = SuperConfig.new()
    
    try:
        json_result = config.extract_json()
        # Should return valid JSON data
        assert isinstance(json_result, (dict, list, str, int, float, bool))
    except Exception as e:
        # If extraction fails, should get meaningful error
        assert "SuperConfig" in str(e)
```

#### Task 3: Node.js Binding Configuration (45 minutes)

**Create `bindings/nodejs/moon.yml`**:

```yaml
# Project: superconfig/nodejs
language: 'javascript'
type: 'library'

tasks:
  check:
    command: 'cargo check --manifest-path ../../Cargo.toml --features nodejs'
    inputs: ['../../src/**/*', '../../Cargo.toml']
    
  build:
    command: 'napi build --manifest-path ../../Cargo.toml --features nodejs --platform --release'
    inputs: ['../../src/**/*', '../../Cargo.toml', '**/*']
    outputs: ['lib/']
    deps: ['check']
    
  test:
    command: 'npm test'
    inputs: ['tests/**/*', 'lib/**/*']
    deps: ['build']
    
  publish:
    command: 'npm publish'
    deps: ['test']
    env:
      NPM_TOKEN: '$NPM_TOKEN'
```

**Create `bindings/nodejs/package.json`**:

```json
{
  "name": "superconfig",
  "version": "0.1.0",
  "description": "Powerful configuration management for Node.js applications",
  "main": "index.js",
  "types": "index.d.ts",
  "files": [
    "index.js",
    "index.d.ts",
    "lib/"
  ],
  "scripts": {
    "test": "jest tests/",
    "build": "napi build --manifest-path ../../Cargo.toml --features nodejs --platform --release"
  },
  "keywords": [
    "configuration",
    "config",
    "settings",
    "toml",
    "json",
    "yaml",
    "rust",
    "native"
  ],
  "author": "DeepBrain Team <team@deepbrain.space>",
  "license": "MIT OR Apache-2.0",
  "repository": {
    "type": "git",
    "url": "https://github.com/deepbrainspace/superconfig.git"
  },
  "homepage": "https://docs.superconfig.dev",
  "engines": {
    "node": ">=16"
  },
  "devDependencies": {
    "jest": "^29.0.0",
    "@types/node": "^20.0.0"
  },
  "napi": {
    "name": "superconfig",
    "triples": {
      "defaults": true
    }
  }
}
```

**Create `bindings/nodejs/index.js`**:

```javascript
/**
 * SuperConfig - Powerful configuration management for Node.js applications.
 * 
 * This package provides a Node.js interface to the SuperConfig Rust library,
 * offering high-performance configuration loading and merging capabilities.
 */

const { SuperConfig } = require('./lib/superconfig.node');

module.exports = {
  SuperConfig
};
```

**Create `bindings/nodejs/index.d.ts`**:

```typescript
/**
 * SuperConfig TypeScript definitions
 */

export interface WildcardConfig {
  pattern: string;
  search?: {
    type: 'current' | 'recursive' | 'directories';
    root?: string;
    directories?: string[];
    max_depth?: number;
  };
  merge_order?: {
    type: 'alphabetical' | 'reverse_alphabetical' | 'custom' | 'modification_time';
    patterns?: string[];
  };
}

export declare class SuperConfig {
  /**
   * Create a new SuperConfig instance
   */
  static new(): SuperConfig;
  
  /**
   * Load configuration from a file
   */
  withFile(path: string): SuperConfig;
  
  /**
   * Load configuration from environment variables
   */
  withEnv(prefix: string): SuperConfig;
  
  /**
   * Configure wildcard file discovery
   */
  withWildcard(config: WildcardConfig): SuperConfig;
  
  /**
   * Set debug mode
   */
  setDebug(debug: boolean): SuperConfig;
  
  /**
   * Extract configuration as JSON
   */
  extractJson(): any;
  
  /**
   * Find configuration value by path
   */
  find(path: string): any | null;
  
  /**
   * Get configuration metadata
   */
  getMetadata(): any;
}
```

**Create `bindings/nodejs/tests/integration.test.js`**:

```javascript
const { SuperConfig } = require('../index');

describe('SuperConfig Node.js Integration', () => {
  test('basic functionality', () => {
    const config = SuperConfig.new();
    expect(config).toBeDefined();
  });
  
  test('with_file method', () => {
    const config = SuperConfig.new();
    
    // Test with non-existent file
    expect(() => {
      config.withFile('nonexistent.toml');
    }).toThrow();
  });
  
  test('wildcard configuration', () => {
    const config = SuperConfig.new();
    
    const wildcardConfig = {
      pattern: '*.toml',
      search: { type: 'current' }
    };
    
    expect(() => {
      const result = config.withWildcard(wildcardConfig);
      expect(result).toBeDefined();
    }).not.toThrow();
  });
  
  test('error handling', () => {
    const config = SuperConfig.new();
    
    // Test invalid wildcard config (missing pattern)
    const invalidConfig = {
      search: { type: 'current' }
    };
    
    expect(() => {
      config.withWildcard(invalidConfig);
    }).toThrow(/pattern/i);
  });
  
  test('json extraction', () => {
    const config = SuperConfig.new();
    
    expect(() => {
      const result = config.extractJson();
      expect(typeof result).toBe('object');
    }).not.toThrow();
  });
});
```

#### Task 4: WebAssembly Binding Configuration (45 minutes)

**Create `bindings/wasm/moon.yml`**:

```yaml
# Project: superconfig/wasm
language: 'javascript'
type: 'library'

tasks:
  check:
    command: 'cargo check --manifest-path ../../Cargo.toml --features wasm'
    inputs: ['../../src/**/*', '../../Cargo.toml']
    
  build:
    command: 'wasm-pack build --manifest-path ../../Cargo.toml --features wasm --target web --out-dir bindings/wasm/pkg'
    inputs: ['../../src/**/*', '../../Cargo.toml', '**/*']
    outputs: ['pkg/']
    deps: ['check']
    
  package:
    command: 'npm run build'
    inputs: ['pkg/**/*', 'webpack.config.js']
    outputs: ['dist/']
    deps: ['build']
    
  test:
    command: 'npm test'
    inputs: ['tests/**/*', 'dist/**/*']
    deps: ['package']
    
  publish:
    command: 'npm publish'
    deps: ['test']
    env:
      NPM_TOKEN: '$NPM_TOKEN'
```

**Create `bindings/wasm/package.json`**:

```json
{
  "name": "superconfig-wasm",
  "version": "0.1.0",
  "description": "Powerful configuration management for WebAssembly applications",
  "main": "dist/superconfig.js",
  "types": "dist/superconfig.d.ts",
  "files": [
    "dist/"
  ],
  "scripts": {
    "build": "webpack --mode=production",
    "test": "jest tests/",
    "dev": "webpack --mode=development --watch"
  },
  "keywords": [
    "configuration",
    "config",
    "settings",
    "wasm",
    "webassembly",
    "rust",
    "browser"
  ],
  "author": "DeepBrain Team <team@deepbrain.space>",
  "license": "MIT OR Apache-2.0",
  "repository": {
    "type": "git",
    "url": "https://github.com/deepbrainspace/superconfig.git"
  },
  "homepage": "https://docs.superconfig.dev",
  "devDependencies": {
    "webpack": "^5.0.0",
    "webpack-cli": "^5.0.0",
    "jest": "^29.0.0",
    "jest-environment-jsdom": "^29.0.0"
  }
}
```

**Create `bindings/wasm/webpack.config.js`**:

```javascript
const path = require('path');

module.exports = {
  entry: './pkg/superconfig.js',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'superconfig.js',
    library: 'SuperConfig',
    libraryTarget: 'umd',
    globalObject: 'this'
  },
  mode: 'production',
  experiments: {
    asyncWebAssembly: true
  }
};
```

#### Task 5: Workspace Integration (30 minutes)

**Update Root `moon.yml`**:

```yaml
# Root workspace moon.yml - SuperConfig FFI orchestration
tasks:
  # Rust development tasks
  test-rust:
    deps: ['superconfig-ffi:test', 'superffi:test']
    
  lint-rust:
    deps: ['superconfig-ffi:lint', 'superffi:lint']
    
  # Multi-language FFI tasks
  check-ffi:
    deps: ['superconfig/python:check', 'superconfig/nodejs:check', 'superconfig/wasm:check']
    
  build-ffi:
    deps: ['superconfig/python:build', 'superconfig/nodejs:build', 'superconfig/wasm:build']
    
  test-ffi:
    deps: ['superconfig/python:test', 'superconfig/nodejs:test', 'superconfig/wasm:test']
    
  publish-ffi:
    deps: ['superconfig/python:publish', 'superconfig/nodejs:publish', 'superconfig/wasm:publish']
    
  # Combined workflows
  test-all:
    deps: ['test-rust', 'test-ffi']
    
  ci-check:
    deps: ['lint-rust', 'check-ffi', 'test-all', 'build-ffi']
    
  release:
    deps: ['ci-check', 'publish-ffi']
```

#### Task 6: GitHub Actions Workflow (30 minutes)

**Create `.github/workflows/superconfig-ffi-release.yml`**:

```yaml
name: SuperConfig FFI Release

on:
  push:
    tags: ['superconfig-ffi-v*']

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Moon
        run: |
          curl -fsSL https://moonrepo.dev/install/moon.sh | bash
          echo "$HOME/.moon/bin" >> $GITHUB_PATH
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      
      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'
      
      - name: Install build tools
        run: |
          pip install maturin twine pytest
          npm install -g @napi-rs/cli
          cargo install wasm-pack
      
      - name: Run CI checks
        run: moon run ci-check
      
      - name: Publish packages
        run: moon run publish-ffi
        env:
          PYPI_TOKEN: ${{ secrets.PYPI_TOKEN }}
          NPM_TOKEN: ${{ secrets.NPM_TOKEN }}
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
      
      - name: Create GitHub Release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: ${{ github.ref }}
          release_name: SuperConfig FFI ${{ github.ref }}
          body: |
            SuperConfig multi-language FFI bindings release.
            
            ## Packages Published
            - Python: `pip install superconfig`
            - Node.js: `npm install superconfig`
            - WebAssembly: `npm install superconfig-wasm`
          draft: false
          prerelease: false
```

**Create `.github/workflows/superconfig-ffi-ci.yml`**:

```yaml
name: SuperConfig FFI CI

on:
  push:
    branches: [main, develop]
    paths: ['crates/superconfig-ffi/**', 'crates/superffi/**']
  pull_request:
    branches: [main]
    paths: ['crates/superconfig-ffi/**', 'crates/superffi/**']

jobs:
  ci:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Moon
        run: |
          curl -fsSL https://moonrepo.dev/install/moon.sh | bash
          echo "$HOME/.moon/bin" >> $GITHUB_PATH
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      
      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - name: Install build tools
        run: |
          pip install maturin pytest
          npm install -g @napi-rs/cli
          cargo install wasm-pack
      
      - name: Run CI checks (no publish)
        run: moon run ci-check
```

## Development Workflow

### **Local Development Commands**

```bash
# Check all FFI targets compile
moon run check-ffi

# Build all language packages
moon run build-ffi

# Test all language bindings
moon run test-ffi

# Full CI simulation
moon run ci-check
```

### **Release Process**

```bash
# 1. Ensure all tests pass
moon run ci-check

# 2. Tag release
git tag superconfig-ffi-v0.1.0
git push --tags

# 3. GitHub Actions automatically:
#    - Runs ci-check
#    - Publishes to PyPI and npm
#    - Creates GitHub release
```

### **User Installation**

```bash
# Python users
pip install superconfig

# Node.js users  
npm install superconfig

# WebAssembly users
npm install superconfig-wasm
```

## Success Metrics

### **Completion Criteria**

- [ ] All binding projects configured with proper Moon tasks
- [ ] GitHub Actions workflow builds and publishes successfully
- [ ] Package configurations valid for PyPI and npm
- [ ] Integration tests pass for all target languages
- [ ] CI/CD pipeline can publish packages automatically

### **Quality Targets**

- [ ] Build process reproducible across platforms
- [ ] Package metadata complete and accurate
- [ ] Documentation includes installation and usage instructions
- [ ] Error handling works consistently across all build steps

### **User Experience**

- [ ] Packages installable via standard package managers
- [ ] APIs work identically across all target languages
- [ ] Error messages consistent and helpful
- [ ] Performance acceptable for typical use cases

## Next Steps After Completion

1. **Documentation**: Create comprehensive API documentation for all languages
2. **Examples**: Build example projects showing real-world usage
3. **Community**: Announce packages to relevant language communities
4. **Feedback**: Collect user feedback and iterate on API design
5. **Performance**: Profile and optimize for production workloads

---

_Completes the SuperFFI implementation pipeline. See [`testing-strategy.md`](./testing-strategy.md) for comprehensive testing approach._
