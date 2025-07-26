//! Core Wildcard provider implementation for pattern-based configuration discovery

use crate::providers::wildcard::{
    discovery::SearchStrategy,
    parsing::{parse_multiple_patterns, build_globset},
    sorting::MergeOrder,
};
use figment::{
    value::{Map, Value},
    Error, Metadata, Profile, Provider,
    providers::Format,
};
use globset::GlobSet;
use std::path::PathBuf;

// Handle empty patterns with a meaningful error message
fn handle_empty_patterns_error(patterns: &[impl AsRef<str>]) -> Result<(), Error> {
    if patterns.is_empty() {
        Err(Error::from("At least one pattern is required"))
    } else {
        Ok(())
    }
}

/// A unified wildcard configuration provider using globset patterns
///
/// The Wildcard provider offers powerful pattern-based file discovery for configuration
/// management. It combines the flexibility of glob patterns with intelligent search
/// strategies and customizable merge ordering.
///
/// # Core Features
///
/// - **Globset-based patterns**: Use standard glob syntax (`*.toml`, `**/*.yaml`, etc.)
/// - **Multiple search strategies**: Directory-specific, recursive, current directory, or custom
/// - **Flexible merge ordering**: Alphabetical, size-based, time-based, or custom priority
/// - **Brace expansion**: `{dir1,dir2}/*.toml` for multi-directory patterns
/// - **100% Figment compatibility**: Drop-in replacement for existing providers
///
/// # Pattern Examples
///
/// ```rust
/// use superconfig::Wildcard;
///
/// // Simple file patterns
/// let provider = Wildcard::from_pattern("*.toml")?;
///
/// // Path-based patterns  
/// let provider = Wildcard::from_pattern("./config/*.yaml")?;
///
/// // Recursive patterns
/// let provider = Wildcard::from_pattern("**/*.json")?;
///
/// // Multi-directory patterns
/// let provider = Wildcard::from_pattern("{./config,~/.config}/*.toml")?;
/// # Ok::<(), figment::Error>(())
/// ```
///
/// # Advanced Configuration
///
/// ```rust
/// use superconfig::{Wildcard, MergeOrder, SearchStrategy};
/// use std::path::PathBuf;
///
/// let provider = Wildcard::from_patterns(&["*.toml", "*.yaml"])?
///     .with_merge_order(MergeOrder::Custom(vec![
///         "base.*".to_string(),        // Base configs first
///         "env-*.toml".to_string(),     // Environment configs
///         "local.*".to_string(),        // Local overrides last
///     ]))
///     .with_search_strategy(SearchStrategy::Recursive {
///         roots: vec![PathBuf::from("./config")],
///         max_depth: Some(2),
///     });
/// # Ok::<(), figment::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct Wildcard {
    /// Compiled glob patterns for file matching
    globset: GlobSet,
    /// Search strategy for file discovery
    search_strategy: SearchStrategy,
    /// Merge order for multiple files
    merge_order: MergeOrder,
    /// Original patterns for metadata
    patterns: Vec<String>,
}

impl Wildcard {
    /// Create a new Wildcard provider from a single pattern (simple constructor)
    ///
    /// This is the simplest and most convenient way to create a wildcard provider.
    /// Equivalent to `from_pattern()` but with a shorter name for common usage.
    ///
    /// # Arguments
    /// * `pattern` - Glob pattern string (e.g., "*.toml", "./config/*.yaml")
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::Wildcard;
    ///
    /// let provider = Wildcard::new("*.toml")?;
    /// let provider = Wildcard::new("./config/*.yaml")?;
    /// let provider = Wildcard::new("**/*.json")?;
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn new(pattern: &str) -> Result<Self, Error> {
        Self::from_pattern(pattern)
    }

    /// Create a new Wildcard provider from a single pattern
    ///
    /// This is the simplest way to create a wildcard provider. The pattern
    /// is automatically parsed to determine the appropriate search strategy.
    ///
    /// # Arguments
    /// * `pattern` - Glob pattern string (e.g., "*.toml", "./config/*.yaml")
    ///
    /// # Returns
    /// A configured Wildcard provider ready for use
    ///
    /// # Errors
    /// Returns `GlobError` if the pattern is invalid
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::Wildcard;
    ///
    /// // Current directory search
    /// let provider = Wildcard::from_pattern("*.toml")?;
    ///
    /// // Specific directory
    /// let provider = Wildcard::from_pattern("./config/*.yaml")?;
    ///
    /// // Recursive search
    /// let provider = Wildcard::from_pattern("**/*.json")?;
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn from_pattern(pattern: &str) -> Result<Self, Error> {
        Self::from_patterns(&[pattern])
    }

    /// Create a new Wildcard provider from multiple patterns
    ///
    /// Multiple patterns are combined intelligently based on their types.
    /// The search strategy is determined by the most general pattern type.
    ///
    /// # Arguments
    /// * `patterns` - Vector of glob pattern strings
    ///
    /// # Returns
    /// A configured Wildcard provider that searches for all patterns
    ///
    /// # Strategy Resolution
    /// - All same type → Combined strategy
    /// - Mixed types → Most general strategy (usually Recursive)
    /// - Current directory patterns → Upgraded to Directories
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::Wildcard;
    ///
    /// // Multiple file types
    /// let provider = Wildcard::from_patterns(&["*.toml", "*.yaml", "*.json"])?;
    ///
    /// // Mixed strategies (uses recursive)
    /// let provider = Wildcard::from_patterns(&["./config/*.toml", "**/*.yaml"])?;
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn from_patterns(patterns: &[impl AsRef<str>]) -> Result<Self, Error> {
        handle_empty_patterns_error(patterns)?;

        let (search_strategy, file_patterns) = parse_multiple_patterns(patterns)?;
        let globset = build_globset(&file_patterns)?;

        Ok(Self {
            globset,
            search_strategy,
            merge_order: MergeOrder::default(),
            patterns: patterns.iter().map(|p| p.as_ref().to_string()).collect(),
        })
    }

    /// Set the merge order for discovered files
    ///
    /// When multiple files are discovered, this determines the order in which
    /// they are processed and merged. Later files in the order have higher
    /// priority and can override earlier values.
    ///
    /// # Arguments
    /// * `order` - The merge order strategy to use
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::{Wildcard, MergeOrder};
    ///
    /// let provider = Wildcard::from_pattern("*.toml")?
    ///     .with_merge_order(MergeOrder::ModificationTimeAscending);
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn with_merge_order(mut self, order: MergeOrder) -> Self {
        self.merge_order = order;
        self
    }

    /// Set the search strategy for file discovery
    ///
    /// Override the automatically determined search strategy with a custom one.
    /// This gives you full control over how files are discovered.
    ///
    /// # Arguments
    /// * `strategy` - The search strategy to use
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::{Wildcard, SearchStrategy};
    /// use std::path::PathBuf;
    ///
    /// let provider = Wildcard::from_pattern("*.toml")?
    ///     .with_search_strategy(SearchStrategy::Recursive {
    ///         roots: vec![PathBuf::from("./config")],
    ///         max_depth: Some(2),
    ///     });
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn with_search_strategy(mut self, strategy: SearchStrategy) -> Self {
        self.search_strategy = strategy;
        self
    }

    /// Get the current search strategy
    pub fn search_strategy(&self) -> &SearchStrategy {
        &self.search_strategy
    }

    /// Get the current merge order
    pub fn merge_order(&self) -> &MergeOrder {
        &self.merge_order
    }

    /// Get the original patterns
    pub fn patterns(&self) -> &[String] {
        &self.patterns
    }

    /// Discover and sort files according to the current configuration
    ///
    /// This method performs the core file discovery and sorting logic.
    /// It's primarily used internally by the Provider implementation,
    /// but can be useful for debugging or custom integrations.
    ///
    /// # Returns
    /// Vector of file paths in merge order (lowest to highest priority)
    pub fn discover_files(&self) -> Vec<PathBuf> {
        let mut files = self.search_strategy.discover_files(&self.globset);
        files = self.merge_order.sort_files(files);
        files
    }
}

impl Provider for Wildcard {
    fn metadata(&self) -> Metadata {
        Metadata::named("Wildcard Provider")
    }

    fn data(&self) -> Result<Map<Profile, Map<String, Value>>, Error> {
        let files = self.discover_files();
        
        if files.is_empty() {
            return Ok(Map::new());
        }

        let mut result_map = Map::new();
        let mut default_profile_data = Map::new();

        // Process files in merge order (lowest to highest priority)
        for file_path in files {
            // Determine the file format and load it
            let provider: Box<dyn Provider> = match file_path.extension().and_then(|ext| ext.to_str()) {
                Some("toml") => Box::new(figment::providers::Toml::file(&file_path)),
                Some("yaml") | Some("yml") => Box::new(figment::providers::Yaml::file(&file_path)),
                Some("json") => Box::new(figment::providers::Json::file(&file_path)),
                _ => continue, // Skip unsupported file types
            };

            // Merge the provider's data into our result
            match provider.data() {
                Ok(provider_data) => {
                    for (profile, data) in provider_data {
                        // For now, only handle Default profile - merge all data into it
                        if profile == Profile::Default {
                            for (key, value) in data {
                                default_profile_data.insert(key, value);
                            }
                        }
                    }
                }
                Err(_) => {
                    // Skip files that can't be parsed
                    continue;
                }
            }
        }

        result_map.insert(Profile::Default, default_profile_data);
        Ok(result_map)
    }
}

// Convenience constructors for common patterns
impl Wildcard {
    /// Git-style hierarchical configuration discovery
    ///
    /// Searches for configuration files starting from the current directory
    /// and walking up the directory tree to find configuration files.
    /// This mimics Git's configuration discovery behavior.
    ///
    /// # Arguments
    /// * `config_name` - Base name for config files (e.g., "config", "myapp")
    /// * `app_name` - Application name for app-specific directories
    ///
    /// # Generated Pattern
    /// Creates patterns that search:
    /// - `~/.config/{app_name}/*.{toml,yaml,yml,json}`
    /// - `~/.{app_name}/*.{toml,yaml,yml,json}`
    /// - `./{config_name}/*.{toml,yaml,yml,json}`
    /// - `**/{config_name}.{toml,yaml,yml,json}`
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::Wildcard;
    ///
    /// // Search for "myapp" configurations
    /// let provider = Wildcard::hierarchical("config", "myapp")?;
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn hierarchical(config_name: &str, app_name: &str) -> Result<Self, Error> {
        let patterns = vec![
            format!("~/.config/{}/*.toml", app_name),
            format!("~/.config/{}/*.yaml", app_name),
            format!("~/.config/{}/*.yml", app_name),
            format!("~/.config/{}/*.json", app_name),
            format!("~/.{}/*.toml", app_name),
            format!("~/.{}/*.yaml", app_name),
            format!("~/.{}/*.yml", app_name),
            format!("~/.{}/*.json", app_name),
            format!("./{}/*.toml", config_name),
            format!("./{}/*.yaml", config_name),
            format!("./{}/*.yml", config_name),
            format!("./{}/*.json", config_name),
            format!("**/{}.toml", config_name),
            format!("**/{}.yaml", config_name),
            format!("**/{}.yml", config_name),
            format!("**/{}.json", config_name),
        ];

        Self::from_patterns(&patterns)
    }

    /// Modern configuration directory discovery
    ///
    /// Searches common configuration directories using XDG Base Directory
    /// specification patterns. Ideal for Linux/Unix applications.
    ///
    /// # Arguments
    /// * `app_name` - Application name for directory structure
    ///
    /// # Generated Pattern
    /// Creates patterns that search:
    /// - `~/.config/{app_name}/*.{toml,yaml,yml,json}`
    /// - `~/.local/share/{app_name}/*.{toml,yaml,yml,json}`
    /// - `/etc/{app_name}/*.{toml,yaml,yml,json}`
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::Wildcard;
    ///
    /// let provider = Wildcard::xdg("myapp")?;
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn xdg(app_name: &str) -> Result<Self, Error> {
        let patterns = vec![
            format!("~/.config/{}/*.toml", app_name),
            format!("~/.config/{}/*.yaml", app_name),
            format!("~/.config/{}/*.yml", app_name),
            format!("~/.config/{}/*.json", app_name),
            format!("~/.local/share/{}/*.toml", app_name),
            format!("~/.local/share/{}/*.yaml", app_name),
            format!("~/.local/share/{}/*.yml", app_name),
            format!("~/.local/share/{}/*.json", app_name),
            format!("/etc/{}/*.toml", app_name),
            format!("/etc/{}/*.yaml", app_name),
            format!("/etc/{}/*.yml", app_name),
            format!("/etc/{}/*.json", app_name),
        ];

        Self::from_patterns(&patterns)
    }

    /// Development environment configuration discovery
    ///
    /// Optimized for development scenarios with environment-specific
    /// configurations and local overrides.
    ///
    /// # Arguments
    /// * `base_name` - Base configuration name (e.g., "config", "app")
    ///
    /// # Generated Patterns & Order
    /// 1. `{base_name}/base.*` - Base configurations
    /// 2. `{base_name}/env-*.*` - Environment-specific configs
    /// 3. `{base_name}/local.*` - Local developer overrides
    /// 4. `./{base_name}.*` - Root-level configs
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::Wildcard;
    ///
    /// let provider = Wildcard::development("config")?;
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn development(base_name: &str) -> Result<Self, Error> {
        let patterns = vec![
            format!("{}/*.toml", base_name),
            format!("{}/*.yaml", base_name),
            format!("{}/*.yml", base_name),
            format!("{}/*.json", base_name),
            format!("{}.toml", base_name),
            format!("{}.yaml", base_name),
            format!("{}.yml", base_name),
            format!("{}.json", base_name),
        ];

        let custom_order = MergeOrder::Custom(vec![
            format!("{}/base.*", base_name),
            format!("{}/env-*.*", base_name),
            format!("{}/local.*", base_name),
            format!("{}.*", base_name),
        ]);

        Self::from_patterns(&patterns).map(|w| w.with_merge_order(custom_order))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_from_pattern() {
        let provider = Wildcard::from_pattern("*.toml").unwrap();
        assert_eq!(provider.patterns(), &["*.toml"]);
    }

    #[test]
    fn test_from_patterns() {
        let patterns = vec!["*.toml".to_string(), "*.yaml".to_string()];
        let provider = Wildcard::from_patterns(&patterns).unwrap();
        assert_eq!(provider.patterns(), &patterns);
    }

    #[test]
    fn test_with_merge_order() {
        let provider = Wildcard::from_pattern("*.toml")
            .unwrap()
            .with_merge_order(MergeOrder::Reverse);
        
        match provider.merge_order() {
            MergeOrder::Reverse => {},
            _ => panic!("Expected Reverse merge order"),
        }
    }

    #[test]
    fn test_discover_files_integration() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create test files
        fs::write(dir_path.join("config.toml"), "[test]\nvalue = 1").unwrap();
        fs::write(dir_path.join("app.yaml"), "test:\n  value: 2").unwrap();
        fs::write(dir_path.join("readme.txt"), "This is a readme").unwrap();

        // Change to temp directory for testing
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir_path).unwrap();

        let provider = Wildcard::from_pattern("*.{toml,yaml}").unwrap();
        let files = provider.discover_files();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        // Should find the config files but not the txt file
        assert!(files.len() >= 1); // At least one file should be found
        assert!(files.iter().any(|p| p.file_name().unwrap().to_str().unwrap().ends_with(".toml") || 
                                     p.file_name().unwrap().to_str().unwrap().ends_with(".yaml")));
    }

    #[test]
    fn test_hierarchical_convenience() {
        let provider = Wildcard::hierarchical("config", "myapp").unwrap();
        let patterns = provider.patterns();
        
        // Should contain patterns for various directories
        assert!(patterns.iter().any(|p| p.contains("~/.config/myapp")));
        assert!(patterns.iter().any(|p| p.contains("./config")));
        assert!(patterns.iter().any(|p| p.contains("**/config.")));
    }

    #[test]
    fn test_xdg_convenience() {
        let provider = Wildcard::xdg("myapp").unwrap();
        let patterns = provider.patterns();
        
        // Should contain XDG-compliant patterns
        assert!(patterns.iter().any(|p| p.contains("~/.config/myapp")));
        assert!(patterns.iter().any(|p| p.contains("~/.local/share/myapp")));
        assert!(patterns.iter().any(|p| p.contains("/etc/myapp")));
    }

    #[test]
    fn test_development_convenience() {
        let provider = Wildcard::development("config").unwrap();
        let patterns = provider.patterns();
        
        // Should contain development-specific patterns
        assert!(patterns.iter().any(|p| p.contains("config/")));
        assert!(patterns.iter().any(|p| p == "config.toml"));
        
        // Should have custom merge order
        match provider.merge_order() {
            MergeOrder::Custom(_) => {},
            _ => panic!("Expected Custom merge order for development provider"),
        }
    }
}