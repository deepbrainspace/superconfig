//! Unified wildcard configuration provider using globset patterns
//!
//! The wildcard module provides powerful pattern-based configuration file discovery
//! and loading. It combines the flexibility of glob patterns with intelligent search
//! strategies and customizable merge ordering.
//!
//! # Core Components
//!
//! - **[`Wildcard`]**: The main provider implementing figment's Provider trait
//! - **[`WildcardBuilder`]**: Advanced builder pattern for complex configurations
//! - **[`SearchStrategy`]**: Defines how files are discovered (directory, recursive, etc.)
//! - **[`MergeOrder`]**: Controls the order in which multiple files are merged
//!
//! # Quick Start
//!
//! ```rust
//! use superconfig::Wildcard;
//! use figment::Figment;
//!
//! // Simple pattern-based loading
//! let figment = Figment::new()
//!     .merge(Wildcard::from_pattern("*.toml"));
//!
//! // Git-style hierarchical configuration
//! let figment = Figment::new()
//!     .merge(Wildcard::hierarchical("config", "myapp"));
//! ```
//!
//! # Pattern Examples
//!
//! ## Simple File Patterns
//! ```rust
//! use superconfig::Wildcard;
//!
//! // All TOML files in current directory
//! let provider = Wildcard::from_pattern("*.toml");
//!
//! // Specific file pattern
//! let provider = Wildcard::from_pattern("config.*");
//! ```
//!
//! ## Path-Based Patterns
//! ```rust
//! use superconfig::Wildcard;
//!
//! // Specific directory
//! let provider = Wildcard::from_pattern("./config/*.yaml");
//!
//! // User configuration directory
//! let provider = Wildcard::from_pattern("~/.config/myapp/*.toml");
//! ```
//!
//! ## Recursive Patterns
//! ```rust
//! use superconfig::Wildcard;
//!
//! // Recursive search from current directory
//! let provider = Wildcard::from_pattern("**/*.json");
//!
//! // Recursive search from specific directory
//! let provider = Wildcard::from_pattern("./config/**/*.yaml");
//! ```
//!
//! ## Multi-Directory Patterns (Brace Expansion)
//! ```rust
//! use superconfig::Wildcard;
//!
//! // Search multiple directories
//! let provider = Wildcard::from_pattern("{./config,~/.config}/*.toml");
//!
//! // Complex multi-directory patterns
//! let provider = Wildcard::from_pattern("{/etc,~/.config,./config}/myapp/*.yaml");
//! ```
//!
//! # Advanced Configuration
//!
//! ## Custom Search Strategies
//! ```rust
//! use superconfig::{Wildcard, SearchStrategy};
//! use std::path::PathBuf;
//!
//! let provider = Wildcard::from_pattern("*.toml")
//!     .with_search_strategy(SearchStrategy::Recursive {
//!         roots: vec![PathBuf::from("./config")],
//!         max_depth: Some(3),
//!     });
//! ```
//!
//! ## Custom Merge Ordering
//! ```rust
//! use superconfig::{Wildcard, MergeOrder};
//!
//! let provider = Wildcard::from_patterns(&["*.toml", "*.yaml"])
//!     .with_merge_order(MergeOrder::Custom(vec![
//!         "base.*".to_string(),        // Base configs first (lowest priority)
//!         "env-*.toml".to_string(),     // Environment configs
//!         "local.*".to_string(),        // Local overrides last (highest priority)
//!     ]));
//! ```
//!
//! ## Builder Pattern for Complex Scenarios
//! ```rust
//! use superconfig::{WildcardBuilder, SearchStrategy, MergeOrder};
//! use std::path::PathBuf;
//!
//! let provider = WildcardBuilder::new()
//!     .pattern("*.toml")?
//!     .pattern("*.yaml")?
//!     .recursive(&[PathBuf::from("./config")], Some(2))
//!     .custom_priority(&["base.*", "env-*", "local.*"])
//!     .build()?;
//! # Ok::<(), figment::Error>(())
//! ```
//!
//! # Convenience Constructors
//!
//! ## Git-Style Hierarchical Discovery
//! ```rust
//! use superconfig::Wildcard;
//!
//! // Searches common config locations hierarchically
//! let provider = Wildcard::hierarchical("config", "myapp");
//! // Equivalent to searching:
//! // - ~/.config/myapp/*.{toml,yaml,yml,json}
//! // - ~/.myapp/*.{toml,yaml,yml,json}  
//! // - ./config/*.{toml,yaml,yml,json}
//! // - **/config.{toml,yaml,yml,json}
//! ```
//!
//! ## XDG Base Directory Support
//! ```rust
//! use superconfig::Wildcard;
//!
//! // Follows XDG Base Directory specification
//! let provider = Wildcard::xdg("myapp");
//! // Searches:
//! // - ~/.config/myapp/*
//! // - ~/.local/share/myapp/*
//! // - /etc/myapp/*
//! ```
//!
//! ## Development Environment Optimized
//! ```rust
//! use superconfig::Wildcard;
//!
//! // Optimized for development with proper priority ordering
//! let provider = Wildcard::development("config");
//! // Priority order:
//! // 1. config/base.* (lowest priority)
//! // 2. config/env-*.*
//! // 3. config/local.* (highest priority)
//! ```
//!
//! # Integration Examples
//!
//! ## With Figment
//! ```rust
//! use superconfig::Wildcard;
//! use figment::{Figment, providers::Env};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Config {
//!     #[serde(default = "default_database_url")]
//!     database_url: String,
//!     #[serde(default)]
//!     port: u16,
//! }
//!
//! fn default_database_url() -> String {
//!     "postgres://localhost".to_string()
//! }
//!
//! let figment = Figment::new()
//!     .merge(Wildcard::hierarchical("config", "myapp"))
//!     .merge(Env::prefixed("MYAPP_"));
//!
//! let config: Config = figment.extract()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Direct Usage
//! ```rust
//! use superconfig::Wildcard;
//! use figment::Provider;
//!
//! let provider = Wildcard::from_pattern("*.toml");
//! let data = provider.data()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Performance Considerations
//!
//! - **Alphabetical sorting**: O(n log n) - fastest and most predictable
//! - **Size/time-based sorting**: O(n log n) + O(n) for metadata access
//! - **Custom pattern sorting**: O(n log n) + O(n × m) where m is number of patterns
//! - **Recursive search**: Can be slow on deep trees - use `max_depth` to limit
//! - **Directory search**: Fast for flat structures
//!
//! # Error Handling
//!
//! - Invalid glob patterns return `globset::Error`
//! - Non-existent directories are silently skipped
//! - Unparseable configuration files are silently skipped
//! - Permission errors during file traversal are logged but don't fail the operation

pub mod builder;
pub mod core;
pub mod discovery;
pub mod parsing;
pub mod sorting;

// Re-export the main types for convenience
pub use builder::WildcardBuilder;
pub use core::Wildcard;
pub use discovery::SearchStrategy;
pub use sorting::MergeOrder;

// Re-export figment error for consistency
pub use figment::Error;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use figment::{Figment, Provider};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_wildcard_provider_integration() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create test configuration files
        let config_path = dir_path.join("config.toml");
        let override_path = dir_path.join("override.toml");

        fs::write(
            &config_path,
            r#"
[database]
host = "localhost"
port = 5432

[app]
name = "test"
debug = true
"#,
        )
        .unwrap();

        fs::write(
            &override_path,
            r#"
[database]
port = 5433

[app]
debug = false
"#,
        )
        .unwrap();

        // Ensure files are fully written by opening and syncing them
        use std::fs::OpenOptions;
        use std::io::Write;
        
        {
            let mut file = OpenOptions::new().append(true).open(&config_path).unwrap();
            file.flush().unwrap();
            file.sync_all().unwrap();
        }
        
        {
            let mut file = OpenOptions::new().append(true).open(&override_path).unwrap();
            file.flush().unwrap();
            file.sync_all().unwrap();
        }

        // Change to temp directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir_path).unwrap();

        // Test the provider
        let provider = Wildcard::from_pattern("*.toml").with_merge_order(MergeOrder::Alphabetical);

        let data = provider.data().unwrap();

        // Restore directory - handle error gracefully
        if let Err(e) = std::env::set_current_dir(&original_dir) {
            eprintln!("Warning: Could not restore original directory {original_dir:?}: {e}");
        }

        // Should have merged the configurations
        assert!(!data.is_empty());
    }

    #[test]
    fn test_figment_integration() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create test files
        fs::write(
            dir_path.join("base.toml"),
            r#"
port = 8080
debug = true
"#,
        )
        .unwrap();

        fs::write(
            dir_path.join("local.toml"),
            r#"
debug = false
"#,
        )
        .unwrap();

        // Change directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir_path).unwrap();

        // Test with Figment
        let figment = Figment::new().merge(Wildcard::from_pattern("*.toml"));

        let result = figment.extract::<figment::value::Value>();

        // Restore directory - handle error gracefully
        if let Err(e) = std::env::set_current_dir(&original_dir) {
            eprintln!("Warning: Could not restore original directory {original_dir:?}: {e}");
            // Try to set to a safe fallback directory
            if let Err(e2) = std::env::set_current_dir("/tmp") {
                eprintln!("Warning: Could not set fallback directory: {e2}");
            }
        }

        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_integration() {
        let provider = WildcardBuilder::new()
            .pattern("*.toml")
            .unwrap()
            .pattern("*.yaml")
            .unwrap()
            .alphabetical()
            .current_directory()
            .build()
            .unwrap();

        // Should be able to call provider methods
        assert_eq!(provider.patterns().len(), 2);
        assert!(matches!(
            provider.search_strategy(),
            SearchStrategy::Current
        ));
        assert!(matches!(provider.merge_order(), MergeOrder::Alphabetical));
    }

    #[test]
    fn test_convenience_constructors() {
        // Test hierarchical constructor
        let hierarchical = Wildcard::hierarchical("config", "myapp");
        assert!(hierarchical.patterns().len() > 10); // Should have many patterns

        // Test XDG constructor
        let xdg = Wildcard::xdg("myapp");
        assert!(xdg.patterns().iter().any(|p| p.contains("~/.config")));

        // Test development constructor
        let dev = Wildcard::development("config");
        assert!(matches!(dev.merge_order(), MergeOrder::Custom(_)));
    }
}
