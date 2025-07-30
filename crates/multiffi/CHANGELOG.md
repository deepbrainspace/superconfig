# Changelog

All notable changes to MultiFFI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-07-30

### Changed

- **BREAKING**: Renamed crate from `superffi` to `multiffi`
- **BREAKING**: Renamed macro from `#[superffi]` to `#[multiffi]`
- **BREAKING**: Package name changed from `superffi` to `multiffi`
- Updated version to 0.2.0 to reflect the breaking rename from SuperFFI

### Migration Guide

To migrate from SuperFFI to MultiFFI:

1. Update your `Cargo.toml`:
   ```toml
   # Before
   superffi = { version = "0.1", features = ["python", "nodejs", "wasm"] }

   # After
   multiffi = { version = "0.1", features = ["python", "nodejs", "wasm"] }
   ```

2. Update your imports:
   ```rust
   // Before
   use superffi::superffi;

   // After
   use multiffi::multiffi;
   ```

3. Update your macro usage:
   ```rust
   // Before
   #[superffi]

   // After
   #[multiffi]
   ```

## [0.1.2] - 2025-07-29 (SuperFFI - Legacy)

### Added

- docs.rs configuration for comprehensive documentation builds
- Crate-specific README badges for better discoverability
- Workspace overview table in main README

### Changed

- Documentation now builds with all features enabled on docs.rs
- Enhanced README structure with proper badge organization

### Fixed

- docs.rs documentation generation for all feature combinations
- Missing documentation links on crates.io

## [0.1.1] - 2025-07-29

### Added

- Automatic naming conversion for JavaScript targets (WASM)
- Comprehensive test suite with 6 test scenarios covering edge cases
- Conditional compilation for feature-gated functions to eliminate warnings

### Changed

- WASM bindings now use camelCase naming for JavaScript consistency
- Node.js bindings rely on native NAPI camelCase conversion (no manual conversion needed)
- Python bindings preserve snake_case for Pythonic APIs

### Technical Details

- Added `convert_to_camel_case()` function with edge case handling
- Added `create_camel_case_ident()` helper for AST manipulation
- Tests organized in conditional module (`#[cfg(feature = "wasm")]`)
- Zero compiler warnings across all feature combinations

## [0.1.0] - 2025-07-28

### Added

- Initial MultiFFI procedural macro implementation
- Support for Python bindings via PyO3 (`#[pyo3::pyclass]`, `#[pyo3::pymethods]`)
- Support for Node.js bindings via NAPI (`#[napi::napi]`)
- Support for WebAssembly bindings via wasm-bindgen (`#[wasm_bindgen]`)
- Feature flags for selective target compilation (`python`, `nodejs`, `wasm`, `all`)
- Comprehensive documentation with usage examples
- Support for structs, impl blocks, and standalone functions
- Zero-cost abstractions - only generates code for enabled features
