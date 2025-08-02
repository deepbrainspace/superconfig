# SuperConfig V2: Deployment and Packaging

## Overview

This document outlines the complete deployment strategy for SuperConfig V2, covering build systems, distribution channels, CI/CD pipelines, and multi-platform deployment. The strategy ensures reliable delivery across all target platforms while maintaining performance and compatibility.

## Build Architecture

### Moon Monorepo Build System

#### Workspace Configuration

```yaml
# moon.yml - Root workspace configuration
$schema: https://moonrepo.dev/schemas/project.json

workspace:
  extends: 'https://raw.githubusercontent.com/moonrepo/moon/master/crates/config/templates/workspace.yml'
  vcs:
    manager: 'git'
    defaultBranch: 'main'

projects:
  - 'crates/*'

docker:
  scaffold:
    include: ['crates/**/*']

runner:
  archiveOutputs: true
  cacheLifetime: '7 days'

vcs:
  hooks:
    pre-push: ['moon run :lint', 'moon run :test', 'moon run :build-release']
```

#### Project-Specific Build Configurations

```yaml
# crates/superconfig/moon.yml
$schema: https://moonrepo.dev/schemas/project.json

type: 'library'
language: 'rust'
platform: 'system'

dependsOn:
  # No dependencies - core crate

fileGroups:
  sources: ['src/**/*']
  tests: ['tests/**/*', 'benches/**/*']
  configs: ['Cargo.toml', '*.toml']

tasks:
  build:
    command: 'cargo build --lib'
    inputs: ['@globs(sources)', '@globs(configs)']
    outputs: ['target/debug/libsuperconfig.*']

  build-release:
    command: 'cargo build --lib --release'
    inputs: ['@globs(sources)', '@globs(configs)']
    outputs: ['target/release/libsuperconfig.*']

  test:
    command: 'cargo test --lib --all-features'
    inputs: ['@globs(sources)', '@globs(tests)', '@globs(configs)']
    deps: ['build']

  bench:
    command: 'cargo bench'
    inputs: ['@globs(sources)', '@globs(tests)', '@globs(configs)']
    deps: ['build-release']

  doc:
    command: 'cargo doc --no-deps --all-features'
    inputs: ['@globs(sources)', '@globs(configs)']
    outputs: ['target/doc/**/*']

  package:
    command: 'cargo package --allow-dirty'
    inputs: ['@globs(sources)', '@globs(configs)', 'README.md', 'LICENSE*']
    outputs: ['target/package/superconfig-*.crate']
    deps: ['build-release', 'test', 'clippy', 'fmt-check']

  publish:
    command: 'cargo publish'
    inputs: ['@globs(sources)', '@globs(configs)', 'README.md', 'LICENSE*']
    deps: ['package']
    options:
      cache: false
```

```yaml
# crates/superconfig-py/moon.yml
$schema: https://moonrepo.dev/schemas/project.json

type: 'library'
language: 'rust'
platform: 'system'

dependsOn:
  - superconfig

fileGroups:
  sources: ['src/**/*', 'python/**/*']
  tests: ['tests/**/*', 'python/tests/**/*']
  configs: ['Cargo.toml', 'pyproject.toml', '*.toml']

tasks:
  build:
    command: 'cargo build --lib'
    inputs: ['@globs(sources)', '@globs(configs)']
    outputs: ['target/debug/libsuperconfig_py.*']
    deps: ['^:build']

  build-release:
    command: 'cargo build --lib --release'
    inputs: ['@globs(sources)', '@globs(configs)']
    outputs: ['target/release/libsuperconfig_py.*']
    deps: ['^:build-release']

  build-python:
    command: 'maturin build --release'
    inputs: ['@globs(sources)', '@globs(configs)']
    outputs: ['target/wheels/*.whl']
    deps: ['build-release']

  test:
    command: 'cargo test --lib'
    inputs: ['@globs(sources)', '@globs(tests)', '@globs(configs)']
    deps: ['build', '^:test']

  test-python:
    command: 'maturin develop && python -m pytest python/tests/'
    inputs: ['@globs(sources)', '@globs(tests)', '@globs(configs)']
    deps: ['build']

  package-python:
    command: 'maturin build --release --out dist/'
    inputs: ['@globs(sources)', '@globs(configs)']
    outputs: ['dist/*.whl']
    deps: ['build-release', 'test', 'test-python']

  publish-python:
    command: 'maturin publish --username __token__ --password $PYPI_TOKEN'
    inputs: ['@globs(sources)', '@globs(configs)']
    deps: ['package-python']
    options:
      cache: false
```

```yaml
# crates/superconfig-napi/moon.yml
$schema: https://moonrepo.dev/schemas/project.json

type: 'library'
language: 'rust'
platform: 'node'

dependsOn:
  - superconfig

fileGroups:
  sources: ['src/**/*', 'js/**/*', 'index.js', '*.d.ts']
  tests: ['tests/**/*', 'js/test/**/*']
  configs: ['Cargo.toml', 'package.json', '*.toml', '*.json']

tasks:
  build:
    command: 'napi build --platform'
    inputs: ['@globs(sources)', '@globs(configs)']
    outputs: ['*.node', 'index.js', '*.d.ts']
    deps: ['^:build']

  build-release:
    command: 'napi build --platform --release'
    inputs: ['@globs(sources)', '@globs(configs)']
    outputs: ['*.node', 'index.js', '*.d.ts']
    deps: ['^:build-release']

  test:
    command: 'npm test'
    inputs: ['@globs(sources)', '@globs(tests)', '@globs(configs)']
    deps: ['build', '^:test']

  package:
    command: 'npm pack'
    inputs: ['@globs(sources)', '@globs(configs)', 'README.md', 'LICENSE*']
    outputs: ['*.tgz']
    deps: ['build-release', 'test']

  publish:
    command: 'npm publish --access public'
    inputs: ['@globs(sources)', '@globs(configs)']
    deps: ['package']
    options:
      cache: false
```

### Cross-Platform Build Matrix

#### Rust Core Targets

```yaml
# .github/workflows/build-matrix.yml
name: Build Matrix

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  MOON_BASE: origin/main

jobs:
  build-rust:
    name: Build Rust (${{ matrix.target }})
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          # Linux targets
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            cross: false
          - target: x86_64-unknown-linux-musl
            os: ubuntu-latest
            cross: true
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
            cross: true
          
          # macOS targets
          - target: x86_64-apple-darwin
            os: macos-latest
            cross: false
          - target: aarch64-apple-darwin
            os: macos-latest
            cross: false
          
          # Windows targets
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            cross: false
          - target: i686-pc-windows-msvc
            os: windows-latest
            cross: false
          
          # WebAssembly
          - target: wasm32-unknown-unknown
            os: ubuntu-latest
            cross: false

    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Moon
        uses: ./.github/actions/setup-moon

      - name: Setup Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Setup cross compilation
        if: matrix.cross
        run: |
          cargo install cross --git https://github.com/cross-rs/cross

      - name: Build superconfig core
        run: |
          if [ "${{ matrix.cross }}" = "true" ]; then
            cross build --target ${{ matrix.target }} --release
          else
            cargo build --target ${{ matrix.target }} --release
          fi
        working-directory: crates/superconfig

      - name: Run tests (native only)
        if: "!matrix.cross && matrix.target != 'wasm32-unknown-unknown'"
        run: moon run superconfig:test

      - name: Package artifacts
        run: |
          mkdir -p artifacts/${{ matrix.target }}
          cp target/${{ matrix.target }}/release/libsuperconfig* artifacts/${{ matrix.target }}/ || true
          cp target/${{ matrix.target }}/release/superconfig.* artifacts/${{ matrix.target }}/ || true

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: superconfig-${{ matrix.target }}
          path: artifacts/${{ matrix.target }}/*
```

#### Python Wheels Build

```yaml
# .github/workflows/python-wheels.yml
name: Python Wheels

on:
  push:
    tags: ['v*']
  workflow_dispatch:

jobs:
  build-wheels:
    name: Build wheel for ${{ matrix.python }}-${{ matrix.platform_id }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          # Linux x86_64
          - os: ubuntu-latest
            platform_id: linux_x86_64
            python: cp38
          - os: ubuntu-latest
            platform_id: linux_x86_64
            python: cp39
          - os: ubuntu-latest
            platform_id: linux_x86_64
            python: cp310
          - os: ubuntu-latest
            platform_id: linux_x86_64
            python: cp311
          - os: ubuntu-latest
            platform_id: linux_x86_64
            python: cp312
          
          # Linux aarch64
          - os: ubuntu-latest
            platform_id: linux_aarch64
            python: cp38
          - os: ubuntu-latest
            platform_id: linux_aarch64
            python: cp39
          - os: ubuntu-latest
            platform_id: linux_aarch64
            python: cp310
          - os: ubuntu-latest
            platform_id: linux_aarch64
            python: cp311
          - os: ubuntu-latest
            platform_id: linux_aarch64
            python: cp312
          
          # macOS x86_64
          - os: macos-latest
            platform_id: macosx_x86_64
            python: cp38
          - os: macos-latest
            platform_id: macosx_x86_64
            python: cp39
          - os: macos-latest
            platform_id: macosx_x86_64
            python: cp310
          - os: macos-latest
            platform_id: macosx_x86_64
            python: cp311
          - os: macos-latest
            platform_id: macosx_x86_64
            python: cp312
          
          # macOS arm64
          - os: macos-latest
            platform_id: macosx_arm64
            python: cp38
          - os: macos-latest
            platform_id: macosx_arm64
            python: cp39
          - os: macos-latest
            platform_id: macosx_arm64
            python: cp310
          - os: macos-latest
            platform_id: macosx_arm64
            python: cp311
          - os: macos-latest
            platform_id: macosx_arm64
            python: cp312
          
          # Windows x86_64
          - os: windows-latest
            platform_id: win_amd64
            python: cp38
          - os: windows-latest
            platform_id: win_amd64
            python: cp39
          - os: windows-latest
            platform_id: win_amd64
            python: cp310
          - os: windows-latest
            platform_id: win_amd64
            python: cp311
          - os: windows-latest
            platform_id: win_amd64
            python: cp312

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Moon
        uses: ./.github/actions/setup-moon

      - name: Build wheels
        uses: PyO3/maturin-action@v1
        with:
          target: ${{ matrix.platform_id }}
          args: --release --out dist --find-interpreter
          sccache: 'true'
          manylinux: auto
          working-directory: crates/superconfig-py

      - name: Upload wheels
        uses: actions/upload-artifact@v4
        with:
          name: wheels-${{ matrix.python }}-${{ matrix.platform_id }}
          path: crates/superconfig-py/dist/*.whl
```

#### Node.js Native Modules

```yaml
# .github/workflows/nodejs-binaries.yml
name: Node.js Native Modules

on:
  push:
    tags: ['v*']
  workflow_dispatch:

jobs:
  build:
    name: Build ${{ matrix.settings.target }}
    runs-on: ${{ matrix.settings.host }}
    strategy:
      fail-fast: false
      matrix:
        settings:
          - host: macos-latest
            target: x86_64-apple-darwin
            build: yarn build --target x86_64-apple-darwin
          - host: macos-latest
            target: aarch64-apple-darwin
            build: yarn build --target aarch64-apple-darwin
          - host: windows-latest
            target: x86_64-pc-windows-msvc
            build: yarn build --target x86_64-pc-windows-msvc
          - host: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            build: yarn build --target x86_64-unknown-linux-gnu
          - host: ubuntu-latest
            target: x86_64-unknown-linux-musl
            build: yarn build --target x86_64-unknown-linux-musl
          - host: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            build: yarn build --target aarch64-unknown-linux-gnu --use-cross
          - host: ubuntu-latest
            target: aarch64-unknown-linux-musl
            build: yarn build --target aarch64-unknown-linux-musl --use-cross

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Moon
        uses: ./.github/actions/setup-moon

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: yarn
          cache-dependency-path: crates/superconfig-napi/yarn.lock

      - name: Install dependencies
        run: yarn install --frozen-lockfile
        working-directory: crates/superconfig-napi

      - name: Build native module
        run: ${{ matrix.settings.build }}
        working-directory: crates/superconfig-napi

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: bindings-${{ matrix.settings.target }}
          path: crates/superconfig-napi/*.node
```

## Package Distribution

### Rust Crates (Crates.io)

#### Core Package Metadata

```toml
# crates/superconfig/Cargo.toml
[package]
name = "superconfig"
version = "2.0.0"
edition = "2021"
rust-version = "1.70"

description = "High-performance configuration management with zero-copy access and multi-language bindings"
documentation = "https://docs.rs/superconfig"
repository = "https://github.com/deepbrainspace/superconfig"
homepage = "https://superconfig.dev"
readme = "README.md"
license = "MIT OR Apache-2.0"

keywords = ["config", "configuration", "settings", "environment", "performance"]
categories = ["config", "development-tools", "parsing"]

include = [
  "src/**/*",
  "tests/**/*",
  "benches/**/*",
  "README.md",
  "LICENSE-*",
  "CHANGELOG.md",
]

[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]

[features]
default = ["file-providers", "env-provider", "hot-reload"]

# Core features
file-providers = ["serde", "toml", "serde_json", "serde_yaml"]
env-provider = []
hot-reload = ["notify", "crossbeam-channel"]

# Format support
toml-support = ["toml"]
json-support = ["serde_json"]
yaml-support = ["serde_yaml"]
ron-support = ["ron"]

# Performance features
simd = ["wide"]
rayon = ["dep:rayon"]

# Validation
validation = ["jsonschema"]

# Development and testing
v1-compat = [] # V1 compatibility layer
test-utils = []

[dependencies]
# Core dependencies (minimal)
thiserror = "1.0"
parking_lot = "0.12"

# Optional dependencies
serde = { version = "1.0", features = ["derive"], optional = true }
toml = { version = "0.8", optional = true }
serde_json = { version = "1.0", optional = true }
serde_yaml = { version = "0.9", optional = true }
ron = { version = "0.8", optional = true }

# Hot reload
notify = { version = "6.0", optional = true }
crossbeam-channel = { version = "0.5", optional = true }

# Performance
wide = { version = "0.7", optional = true }
rayon = { version = "1.7", optional = true }

# Validation
jsonschema = { version = "0.17", optional = true }

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
proptest = "1.0"
tempfile = "3.0"
tokio = { version = "1.0", features = ["rt", "macros"] }

[[bench]]
name = "core_performance"
harness = false

[[bench]]
name = "handle_access"
harness = false

[[bench]]
name = "provider_loading"
harness = false
```

#### Publication Automation

```bash
#!/bin/bash
# .moon/scripts/publish.sh

set -euo pipefail

PROJECT="${1:-}"
VERSION="${2:-}"
DRY_RUN="${3:-}"

if [[ -z "$PROJECT" || -z "$VERSION" ]]; then
    echo "Usage: $0 <project> <version> [--yes]"
    exit 1
fi

echo "Publishing $PROJECT v$VERSION"

cd "crates/$PROJECT"

# Validate version matches Cargo.toml
CARGO_VERSION=$(grep '^version =' Cargo.toml | cut -d'"' -f2)
if [[ "$CARGO_VERSION" != "$VERSION" ]]; then
    echo "Error: Version mismatch. Cargo.toml has $CARGO_VERSION, requested $VERSION"
    exit 1
fi

# Run pre-publish checks
echo "Running pre-publish checks..."
moon run "$PROJECT:clippy"
moon run "$PROJECT:fmt-check"
moon run "$PROJECT:test"
moon run "$PROJECT:doc"

# Package and verify
echo "Packaging..."
cargo package

# Dry run first
echo "Performing dry run..."
cargo publish --dry-run

if [[ "$DRY_RUN" != "--yes" ]]; then
    read -p "Proceed with publish? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted"
        exit 1
    fi
fi

# Actual publish
echo "Publishing to crates.io..."
cargo publish

# Create git tag
git tag -a "v$VERSION" -m "Release $PROJECT v$VERSION"
git push origin "v$VERSION"

echo "✅ Successfully published $PROJECT v$VERSION"
```

### Python Packages (PyPI)

#### Python Package Metadata

```toml
# crates/superconfig-py/pyproject.toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "superconfig"
description = "High-performance configuration management with zero-copy access"
authors = [
  { name = "DeepBrain Team", email = "team@deepbrain.space" },
]
license = { text = "MIT OR Apache-2.0" }
readme = "README.md"
homepage = "https://superconfig.dev"
repository = "https://github.com/deepbrainspace/superconfig"
documentation = "https://docs.superconfig.dev/python"

requires-python = ">=3.8"
dynamic = ["version"]

keywords = ["config", "configuration", "settings", "performance"]
classifiers = [
  "Development Status :: 5 - Production/Stable",
  "Intended Audience :: Developers",
  "License :: OSI Approved :: MIT License",
  "License :: OSI Approved :: Apache Software License",
  "Operating System :: OS Independent",
  "Programming Language :: Python :: 3",
  "Programming Language :: Python :: 3.8",
  "Programming Language :: Python :: 3.9",
  "Programming Language :: Python :: 3.10",
  "Programming Language :: Python :: 3.11",
  "Programming Language :: Python :: 3.12",
  "Programming Language :: Rust",
  "Topic :: Software Development :: Libraries :: Python Modules",
  "Topic :: System :: Systems Administration",
  "Typing :: Typed",
]

dependencies = [
  "typing-extensions>=4.0.0; python_version<'3.10'",
]

[project.optional-dependencies]
dev = [
  "pytest>=7.0",
  "pytest-asyncio>=0.20",
  "pytest-benchmark>=4.0",
  "mypy>=1.0",
  "black>=22.0",
  "ruff>=0.1.0",
]

validation = [
  "jsonschema>=4.0",
  "pydantic>=2.0",
]

[project.urls]
Homepage = "https://superconfig.dev"
Documentation = "https://docs.superconfig.dev/python"
Repository = "https://github.com/deepbrainspace/superconfig"
"Bug Tracker" = "https://github.com/deepbrainspace/superconfig/issues"
Changelog = "https://github.com/deepbrainspace/superconfig/blob/main/CHANGELOG.md"

[tool.maturin]
features = ["pyo3/extension-module"]
module-name = "superconfig._native"
python-source = "python"

[tool.pytest.ini_options]
testpaths = ["python/tests"]
python_files = "test_*.py"
python_functions = "test_*"
addopts = "-v --tb=short"

[tool.mypy]
python_version = "3.8"
warn_return_any = true
warn_unused_configs = true
disallow_untyped_defs = true
no_implicit_optional = true

[tool.black]
line-length = 88
target-version = ["py38", "py39", "py310", "py311", "py312"]

[tool.ruff]
target-version = "py38"
line-length = 88
```

#### PyPI Publishing

```bash
#!/bin/bash
# .moon/scripts/publish-python.sh

set -euo pipefail

VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
    echo "Usage: $0 <version>"
    exit 1
fi

cd crates/superconfig-py

echo "Publishing superconfig-py v$VERSION to PyPI"

# Update version in Cargo.toml
sed -i "s/^version = .*/version = \"$VERSION\"/" Cargo.toml

# Build wheels for all platforms
echo "Building wheels..."
maturin build --release --out dist/

# Upload to PyPI
echo "Uploading to PyPI..."
maturin publish --username __token__ --password "$PYPI_TOKEN"

echo "✅ Successfully published superconfig v$VERSION to PyPI"
```

### Node.js Packages (NPM)

#### NPM Package Metadata

```json
{
  "name": "@superconfig/core",
  "version": "2.0.0",
  "description": "High-performance configuration management with zero-copy access",
  "main": "index.js",
  "types": "index.d.ts",
  "repository": {
    "type": "git",
    "url": "https://github.com/deepbrainspace/superconfig.git",
    "directory": "crates/superconfig-napi"
  },
  "homepage": "https://superconfig.dev",
  "bugs": {
    "url": "https://github.com/deepbrainspace/superconfig/issues"
  },
  "license": "MIT OR Apache-2.0",
  "keywords": [
    "config",
    "configuration",
    "settings",
    "performance",
    "rust",
    "native",
    "napi"
  ],
  "engines": {
    "node": ">= 14"
  },
  "files": [
    "index.js",
    "index.d.ts",
    "*.node",
    "README.md",
    "LICENSE*"
  ],
  "napi": {
    "name": "superconfig",
    "triples": {
      "defaults": true,
      "additional": [
        "x86_64-unknown-linux-musl",
        "aarch64-apple-darwin",
        "aarch64-unknown-linux-gnu",
        "aarch64-pc-windows-msvc",
        "armv7-unknown-linux-gnueabihf"
      ]
    }
  },
  "scripts": {
    "artifacts": "napi artifacts",
    "bench": "node -r @swc-node/register benchmark/bench.ts",
    "build": "napi build --platform --release",
    "build:debug": "napi build --platform",
    "prepublishOnly": "napi prepublish -t npm",
    "test": "vitest run",
    "test:watch": "vitest",
    "version": "napi version"
  },
  "devDependencies": {
    "@napi-rs/cli": "^2.16.0",
    "@swc-node/register": "^1.6.0",
    "@types/node": "^20.0.0",
    "typescript": "^5.0.0",
    "vitest": "^1.0.0"
  },
  "packageManager": "yarn@4.0.0",
  "publishConfig": {
    "registry": "https://registry.npmjs.org/",
    "access": "public"
  }
}
```

#### NPM Publishing

```bash
#!/bin/bash
# .moon/scripts/publish-nodejs.sh

set -euo pipefail

VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
    echo "Usage: $0 <version>"
    exit 1
fi

cd crates/superconfig-napi

echo "Publishing @superconfig/core v$VERSION to NPM"

# Update version in package.json
npm version "$VERSION" --no-git-tag-version

# Build for all platforms
echo "Building native modules..."
yarn build

# Package
echo "Packaging..."
yarn napi prepublish -t npm

# Publish
echo "Publishing to NPM..."
npm publish --access public

echo "✅ Successfully published @superconfig/core v$VERSION to NPM"
```

## Container Distribution

### Docker Images

#### Multi-Architecture Dockerfile

```dockerfile
# Dockerfile
FROM --platform=$BUILDPLATFORM rust:1.75 AS builder

ARG TARGETPLATFORM
ARG BUILDPLATFORM

# Install cross-compilation dependencies
RUN apt-get update && apt-get install -y \
    gcc-aarch64-linux-gnu \
    gcc-x86-64-linux-gnu \
    gcc-i686-linux-gnu \
    && rm -rf /var/lib/apt/lists/*

# Set up cross-compilation environment
RUN case "$TARGETPLATFORM" in \
    "linux/amd64") echo "x86_64-unknown-linux-gnu" > /tmp/target.txt ;; \
    "linux/arm64") echo "aarch64-unknown-linux-gnu" > /tmp/target.txt ;; \
    "linux/arm/v7") echo "armv7-unknown-linux-gnueabihf" > /tmp/target.txt ;; \
    *) echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
    esac

RUN TARGET=$(cat /tmp/target.txt) && rustup target add $TARGET

WORKDIR /workspace

# Copy Moon configuration
COPY moon.yml .moon/ ./
COPY .moon/ ./.moon/

# Copy source code
COPY crates/ ./crates/

# Build for target platform
RUN TARGET=$(cat /tmp/target.txt) && \
    case "$TARGET" in \
    "aarch64-unknown-linux-gnu") \
        export CC=aarch64-linux-gnu-gcc && \
        export AR=aarch64-linux-gnu-ar ;; \
    "armv7-unknown-linux-gnueabihf") \
        export CC=arm-linux-gnueabihf-gcc && \
        export AR=arm-linux-gnueabihf-ar ;; \
    esac && \
    cargo build --target $TARGET --release --manifest-path crates/superconfig/Cargo.toml

# Runtime image
FROM --platform=$TARGETPLATFORM debian:bookworm-slim

ARG TARGETPLATFORM

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
RUN case "$TARGETPLATFORM" in \
    "linux/amd64") echo "x86_64-unknown-linux-gnu" > /tmp/target.txt ;; \
    "linux/arm64") echo "aarch64-unknown-linux-gnu" > /tmp/target.txt ;; \
    "linux/arm/v7") echo "armv7-unknown-linux-gnueabihf" > /tmp/target.txt ;; \
    esac

COPY --from=builder /workspace/target/$(cat /tmp/target.txt)/release/libsuperconfig.* /usr/local/lib/

# Create non-root user
RUN useradd -r -s /bin/false superconfig

USER superconfig

LABEL org.opencontainers.image.title="SuperConfig"
LABEL org.opencontainers.image.description="High-performance configuration management library"
LABEL org.opencontainers.image.url="https://superconfig.dev"
LABEL org.opencontainers.image.source="https://github.com/deepbrainspace/superconfig"
LABEL org.opencontainers.image.vendor="DeepBrain"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD echo 'SuperConfig container is healthy'

CMD ["echo", "SuperConfig library container"]
```

#### GitHub Container Registry

```yaml
# .github/workflows/docker.yml
name: Docker Images

on:
  push:
    tags: ['v*']
  workflow_dispatch:

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  build-and-push:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=ref,event=branch
            type=ref,event=pr
            type=semver,pattern={{version}}
            type=semver,pattern={{major}}.{{minor}}
            type=semver,pattern={{major}}

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          platforms: linux/amd64,linux/arm64,linux/arm/v7
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

## Release Automation

### Semantic Release Configuration

```json
{
  "extends": [
    "@semantic-release/commit-analyzer",
    "@semantic-release/release-notes-generator",
    "@semantic-release/changelog",
    "@semantic-release/github"
  ],
  "branches": [
    "main",
    {
      "name": "beta",
      "prerelease": true
    }
  ],
  "plugins": [
    [
      "@semantic-release/commit-analyzer",
      {
        "preset": "conventionalcommits",
        "releaseRules": [
          {
            "type": "feat",
            "scope": "superconfig",
            "release": "minor"
          },
          {
            "type": "feat",
            "scope": "superconfig-py",
            "release": "minor"
          },
          {
            "type": "feat",
            "scope": "superconfig-napi",
            "release": "minor"
          },
          {
            "type": "fix",
            "release": "patch"
          },
          {
            "type": "perf",
            "release": "patch"
          },
          {
            "type": "docs",
            "release": false
          }
        ]
      }
    ],
    [
      "@semantic-release/release-notes-generator",
      {
        "preset": "conventionalcommits",
        "presetConfig": {
          "types": [
            {
              "type": "feat",
              "section": "Features"
            },
            {
              "type": "fix",
              "section": "Bug Fixes"
            },
            {
              "type": "perf",
              "section": "Performance Improvements"
            },
            {
              "type": "docs",
              "section": "Documentation",
              "hidden": true
            }
          ]
        }
      }
    ],
    [
      "@semantic-release/changelog",
      {
        "changelogFile": "CHANGELOG.md",
        "changelogTitle": "# SuperConfig Changelog"
      }
    ],
    [
      "@semantic-release/exec",
      {
        "prepareCmd": ".moon/scripts/prepare-release.sh ${nextRelease.version}",
        "publishCmd": ".moon/scripts/publish-all.sh ${nextRelease.version}"
      }
    ],
    [
      "@semantic-release/github",
      {
        "assets": [
          {
            "path": "dist/*.whl",
            "label": "Python wheels"
          },
          {
            "path": "dist/*.node",
            "label": "Node.js native modules"
          },
          {
            "path": "target/package/*.crate",
            "label": "Rust crates"
          }
        ]
      }
    ]
  ]
}
```

### Release Scripts

```bash
#!/bin/bash
# .moon/scripts/prepare-release.sh

set -euo pipefail

VERSION="$1"

echo "Preparing release v$VERSION"

# Update version in all Cargo.toml files
find crates -name "Cargo.toml" -exec sed -i "s/^version = .*/version = \"$VERSION\"/" {} \;

# Update version in package.json files
find crates -name "package.json" -exec sed -i "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" {} \;

# Update dependencies between crates
sed -i "s/superconfig = { version = \".*\", path/superconfig = { version = \"$VERSION\", path/" crates/*/Cargo.toml

# Generate documentation
moon run superconfig:doc

# Run full test suite
moon run :test
moon run :clippy
moon run :fmt-check

echo "✅ Release v$VERSION prepared"
```

```bash
#!/bin/bash
# .moon/scripts/publish-all.sh

set -euo pipefail

VERSION="$1"

echo "Publishing all packages for v$VERSION"

# Publish in dependency order
.moon/scripts/publish.sh superconfig "$VERSION" --yes

# Wait for crates.io to update
sleep 30

.moon/scripts/publish-python.sh "$VERSION"
.moon/scripts/publish-nodejs.sh "$VERSION"

echo "✅ All packages published for v$VERSION"
```

## Monitoring and Analytics

### Download Analytics

```yaml
# .github/workflows/analytics.yml
name: Package Analytics

on:
  schedule:
    - cron: '0 6 * * *'  # Daily at 6 AM UTC
  workflow_dispatch:

jobs:
  collect-metrics:
    runs-on: ubuntu-latest
    steps:
      - name: Collect crates.io metrics
        run: |
          CRATE_DATA=$(curl -s https://crates.io/api/v1/crates/superconfig)
          DOWNLOADS=$(echo "$CRATE_DATA" | jq '.crate.downloads')
          RECENT_DOWNLOADS=$(echo "$CRATE_DATA" | jq '.crate.recent_downloads')
          
          echo "superconfig_downloads_total{package=\"rust\"} $DOWNLOADS" >> metrics.txt
          echo "superconfig_downloads_recent{package=\"rust\"} $RECENT_DOWNLOADS" >> metrics.txt

      - name: Collect PyPI metrics
        run: |
          PYPI_DATA=$(curl -s https://pypistats.org/api/packages/superconfig/recent)
          DOWNLOADS=$(echo "$PYPI_DATA" | jq '.data.last_month')
          
          echo "superconfig_downloads_recent{package=\"python\"} $DOWNLOADS" >> metrics.txt

      - name: Collect NPM metrics
        run: |
          NPM_DATA=$(curl -s https://api.npmjs.org/downloads/point/last-month/@superconfig/core)
          DOWNLOADS=$(echo "$NPM_DATA" | jq '.downloads')
          
          echo "superconfig_downloads_recent{package=\"nodejs\"} $DOWNLOADS" >> metrics.txt

      - name: Send to monitoring
        run: |
          curl -X POST "${{ secrets.METRICS_ENDPOINT }}" \
            -H "Authorization: Bearer ${{ secrets.METRICS_TOKEN }}" \
            --data-binary @metrics.txt
```

### Performance Monitoring

```rust
// Performance telemetry collection
#[cfg(feature = "telemetry")]
mod telemetry {
    use std::time::Instant;
    
    pub struct PerformanceCollector {
        metrics: Vec<Metric>,
    }
    
    impl PerformanceCollector {
        pub fn record_handle_access(&mut self, duration: Duration, key: &str) {
            self.metrics.push(Metric {
                name: "handle_access_duration_ns".to_string(),
                value: duration.as_nanos() as f64,
                labels: vec![("key".to_string(), key.to_string())],
                timestamp: Instant::now(),
            });
        }
        
        pub fn record_config_load(&mut self, duration: Duration, source: &str) {
            self.metrics.push(Metric {
                name: "config_load_duration_ms".to_string(),
                value: duration.as_millis() as f64,
                labels: vec![("source".to_string(), source.to_string())],
                timestamp: Instant::now(),
            });
        }
    }
}
```

This comprehensive deployment and packaging strategy ensures reliable, automated distribution of SuperConfig V2 across all target platforms while maintaining high quality and performance standards.
