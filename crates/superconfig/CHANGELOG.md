# Changelog

All notable changes to the SuperConfig crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-07-27

### Added
- **Core Configuration System**: Advanced configuration management built on Figment with 100% compatibility
- **Fluent Builder API**: Chainable methods including `.with_file()`, `.with_env()`, `.with_hierarchical_config()`, `.with_defaults()`
- **Wildcard Configuration Provider**: Pattern-based discovery with glob support and intelligent sorting
- **Comprehensive Verbosity System**: CLI-style debugging with multiple verbosity levels for configuration troubleshooting
- **Enhanced Environment Variable Processing**: JSON parsing, automatic nesting, and smart type detection
- **Array Merging Capabilities**: Intelligent composition with `_add`/`_remove` patterns across all configuration sources
- **Resilient Loading System**: Continues loading when some configs fail, collecting warnings instead of crashing
- **Smart Format Detection**: Content-based parsing with intelligent caching and fallback strategies
- **Access and Export Methods**: `.as_json()`, `.as_yaml()`, `.get_string()`, `.has_key()`, `.debug_config()`
- **Advanced Debugging Tools**: Built-in introspection, source tracking, validation, and warning collection
- **Missing Optional Methods**: Added `with_defaults_string()` for embedded config strings and other convenience methods

### Features
- **Core Features**: Basic configuration management (`core` feature flag)
- **Provider System**: Extended configuration providers (`providers` feature flag)
- **Production Optimizations**: Lazy loading, modification time caching, and optimized data structures

### Dependencies
- Built on Figment 0.10.19 with JSON, TOML, YAML, and environment variable support
- Serde 1.0 for serialization/deserialization
- Additional utilities: `toml`, `serde_yml`, `lazy_static`, `dirs`, `anyhow`, `serde_json`, `globset`, `walkdir`

### Documentation
- Comprehensive README with feature documentation and examples
- Inline documentation with rustdoc and Ayu theme
- Example usage patterns and integration guides
- Debugging and troubleshooting documentation
- Clean, focused documentation without marketing content

### Security
- Industry-standard security auditing with GitHub Actions
- Cargo deny checks for license compliance and security vulnerabilities
- Comprehensive CI/CD pipeline with security validation
- MPL-2.0 license compatibility resolved

### Performance
- Optimized CI with efficient build→test flow
- Moon-based build system for consistent toolchain management
- Intelligent caching strategies for build artifacts

### Initial Release
- MIT License for maximum adoption
- GitHub repository setup with comprehensive CI/CD
- Moon workspace configuration with Proto toolchain management
- Rust 1.86.0 toolchain compatibility