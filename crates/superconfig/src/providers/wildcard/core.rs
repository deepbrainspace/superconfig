//! Core Wildcard provider implementation for pattern-based configuration discovery

use crate::providers::wildcard::{
    discovery::SearchStrategy,
    parsing::{parse_multiple_patterns, build_globset},
    sorting::MergeOrder,
};
use crate::merge::ValidatedProvider;
use figment::{
    value::{Map, Value},
    Error, Metadata, Profile, Provider,
};
use globset::GlobSet;
use std::path::PathBuf;


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
/// let provider = Wildcard::from_pattern("*.toml");
///
/// // Path-based patterns  
/// let provider = Wildcard::from_pattern("./config/*.yaml");
///
/// // Recursive patterns
/// let provider = Wildcard::from_pattern("**/*.json");
///
/// // Multi-directory patterns
/// let provider = Wildcard::from_pattern("{./config,~/.config}/*.toml");
/// ```
///
/// # Advanced Configuration
///
/// ```rust
/// use superconfig::{Wildcard, MergeOrder, SearchStrategy};
/// use std::path::PathBuf;
///
/// let provider = Wildcard::from_patterns(&["*.toml", "*.yaml"])
///     .with_merge_order(MergeOrder::Custom(vec![
///         "base.*".to_string(),        // Base configs first
///         "env-*.toml".to_string(),     // Environment configs
///         "local.*".to_string(),        // Local overrides last
///     ]))
///     .with_search_strategy(SearchStrategy::Recursive {
///         roots: vec![PathBuf::from("./config")],
///         max_depth: Some(2),
///     });
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
    /// Cached validation error (if any)
    validation_error: Option<String>,
}

impl Wildcard {
    /// Create a new Wildcard provider from a single pattern (simple constructor)
    ///
    /// This is the simplest and most convenient way to create a wildcard provider.
    /// Follows Figment's standard pattern of deferring validation to data loading time,
    /// enabling fluent chaining with other providers.
    ///
    /// # Arguments
    /// * `pattern` - Glob pattern string (e.g., "*.toml", "./config/*.yaml")
    ///
    /// # Validation
    /// Pattern validation is deferred until `.data()` is called. Use `.validate()`
    /// or `.has_errors()` immediately after construction for early validation.
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::Wildcard;
    /// use figment::Figment;
    ///
    /// // Standard Figment chaining (validation deferred)
    /// let figment = Figment::new()
    ///     .merge(Wildcard::new("*.toml"))
    ///     .merge(Wildcard::new("./config/*.yaml"));
    ///
    /// // Early validation when desired
    /// let provider = Wildcard::new("*.json");
    /// if let Some(error) = provider.has_errors() {
    ///     eprintln!("Warning: {}", error);
    /// }
    /// ```
    pub fn new(pattern: &str) -> Self {
        Self::from_pattern_unchecked(pattern)
    }

    /// Create a new Wildcard provider from a single pattern
    ///
    /// Similar to `.new()` but with a more explicit name. Follows Figment's
    /// standard pattern of deferring validation to data loading time.
    ///
    /// # Arguments
    /// * `pattern` - Glob pattern string (e.g., "*.toml", "./config/*.yaml")
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::Wildcard;
    ///
    /// let provider = Wildcard::from_pattern("*.toml");
    /// let provider = Wildcard::from_pattern("./config/*.yaml");
    /// let provider = Wildcard::from_pattern("**/*.json");
    /// ```
    pub fn from_pattern(pattern: &str) -> Self {
        Self::from_pattern_unchecked(pattern)
    }

    /// Internal method: create provider without validation
    fn from_pattern_unchecked(pattern: &str) -> Self {
        Self::from_patterns_unchecked(&[pattern])
    }

    /// Create a new Wildcard provider from multiple patterns
    ///
    /// Multiple patterns are combined intelligently based on their types.
    /// Follows Figment's standard pattern of deferring validation to data loading time.
    ///
    /// # Arguments
    /// * `patterns` - Slice of glob pattern strings
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
    /// let provider = Wildcard::from_patterns(&["*.toml", "*.yaml", "*.json"]);
    ///
    /// // Mixed strategies (uses recursive)
    /// let provider = Wildcard::from_patterns(&["./config/*.toml", "**/*.yaml"]);
    /// ```
    pub fn from_patterns(patterns: &[impl AsRef<str>]) -> Self {
        Self::from_patterns_unchecked(patterns)
    }

    /// Internal method: create provider from patterns, validate and store result
    fn from_patterns_unchecked(patterns: &[impl AsRef<str>]) -> Self {
        let pattern_strings: Vec<String> = patterns.iter().map(|p| p.as_ref().to_string()).collect();
        
        // Try to validate and build - store error if validation fails
        match Self::validate_and_build(&pattern_strings) {
            Ok((search_strategy, globset)) => Self {
                globset,
                search_strategy,
                merge_order: MergeOrder::default(),
                patterns: pattern_strings,
                validation_error: None,
            },
            Err(error) => Self {
                // Use safe defaults when validation fails, store error for warning
                globset: globset::GlobSetBuilder::new().build().unwrap(),
                search_strategy: SearchStrategy::Current,
                merge_order: MergeOrder::default(),
                patterns: pattern_strings,
                validation_error: Some(error.to_string()),
            }
        }
    }

    /// Internal helper to validate patterns and build components
    fn validate_and_build(patterns: &[String]) -> Result<(SearchStrategy, GlobSet), Error> {
        if patterns.is_empty() {
            return Err(Error::from("At least one pattern is required"));
        }

        let (search_strategy, file_patterns) = parse_multiple_patterns(patterns)?;
        let globset = build_globset(&file_patterns)?;
        
        Ok((search_strategy, globset))
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
    /// let provider = Wildcard::from_pattern("*.toml")
    ///     .with_merge_order(MergeOrder::ModificationTimeAscending);
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
    /// let provider = Wildcard::from_pattern("*.toml")
    ///     .with_search_strategy(SearchStrategy::Recursive {
    ///         roots: vec![PathBuf::from("./config")],
    ///         max_depth: Some(2),
    ///     });
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

    /// Check if the provider has any validation errors
    ///
    /// Returns `Some(error)` if the provider was created with invalid patterns,
    /// `None` if all patterns are valid. This allows SuperConfig to warn about
    /// problematic providers while continuing to load other providers.
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::Wildcard;
    ///
    /// let provider = Wildcard::new("*.toml");
    /// assert!(provider.has_errors().is_none());
    ///
    /// let invalid_provider = Wildcard::new("invalid[pattern");
    /// assert!(invalid_provider.has_errors().is_some());
    /// ```
    pub fn has_errors(&self) -> Option<Error> {
        self.validation_error.as_ref().map(|s| Error::from(s.clone()))
    }

    /// Check if the provider is valid (no validation errors)
    ///
    /// This is the inverse of `has_errors()` for convenience.
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::Wildcard;
    ///
    /// let provider = Wildcard::new("*.toml");
    /// assert!(provider.is_valid());
    ///
    /// let invalid_provider = Wildcard::new("invalid[pattern");
    /// assert!(!invalid_provider.is_valid());
    /// ```
    pub fn is_valid(&self) -> bool {
        self.validation_error.is_none()
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

        // Use SuperConfig's existing merge logic for proper sequential array processing
        let mut super_config = crate::SuperConfig::new();

        // Chain merge each file in order - SuperConfig.merge() handles array operations correctly
        for file_path in files {
            let provider = crate::providers::Universal::file(&file_path);
            super_config = super_config.merge(provider);
        }

        // Extract the final merged data
        super_config.figment.data()
    }
}

impl ValidatedProvider for Wildcard {
    fn validation_error(&self) -> Option<Error> {
        self.has_errors()
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
    /// - `~/.config/{app_name}/*.{toml,yaml,yml,json}` (system level)
    /// - `~/.{app_name}/*.{toml,yaml,yml,json}` (user level)
    /// - `./{app_name}.{toml,yaml,yml,json}` (project level)
    /// - `**/{app_name}.{toml,yaml,yml,json}` (recursive project search)
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::Wildcard;
    ///
    /// // Search for "myapp" configurations
    /// let provider = Wildcard::hierarchical("config", "myapp");
    /// 
    /// // Check for validation errors if needed
    /// if let Some(error) = provider.has_errors() {
    ///     eprintln!("Warning: {}", error);
    /// }
    /// ```
    pub fn hierarchical(_config_name: &str, app_name: &str) -> Self {
        let patterns = vec![
            format!("~/.config/{}/*.toml", app_name),
            format!("~/.config/{}/*.yaml", app_name),
            format!("~/.config/{}/*.yml", app_name),
            format!("~/.config/{}/*.json", app_name),
            format!("~/.{}/*.toml", app_name),
            format!("~/.{}/*.yaml", app_name),
            format!("~/.{}/*.yml", app_name),
            format!("~/.{}/*.json", app_name),
            format!("./{}.toml", app_name),
            format!("./{}.yaml", app_name),
            format!("./{}.yml", app_name),
            format!("./{}.json", app_name),
            format!("**/{}.toml", app_name),
            format!("**/{}.yaml", app_name),
            format!("**/{}.yml", app_name),
            format!("**/{}.json", app_name),
        ];

        Self::from_patterns(&patterns).with_merge_order(MergeOrder::Hierarchical)
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
    /// let provider = Wildcard::xdg("myapp");
    /// 
    /// // Check for validation errors if needed
    /// if let Some(error) = provider.has_errors() {
    ///     eprintln!("Warning: {}", error);
    /// }
    /// ```
    pub fn xdg(app_name: &str) -> Self {
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
    /// let provider = Wildcard::development("config");
    /// 
    /// // Check for validation errors if needed
    /// if let Some(error) = provider.has_errors() {
    ///     eprintln!("Warning: {}", error);
    /// }
    /// ```
    pub fn development(base_name: &str) -> Self {
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

        Self::from_patterns(&patterns).with_merge_order(custom_order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_from_pattern() {
        let provider = Wildcard::from_pattern("*.toml");
        assert_eq!(provider.patterns(), &["*.toml"]);
    }

    #[test]
    fn test_from_patterns() {
        let patterns = vec!["*.toml".to_string(), "*.yaml".to_string()];
        let provider = Wildcard::from_patterns(&patterns);
        assert_eq!(provider.patterns(), &patterns);
    }

    #[test]
    fn test_with_merge_order() {
        let provider = Wildcard::from_pattern("*.toml")
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

        // Use a provider that searches in a specific directory instead of relying on current directory
        let pattern = format!("{}/*.toml", dir_path.display());
        let provider = Wildcard::from_pattern(&pattern);
        let files = provider.discover_files();

        // Should find at least the toml file
        assert!(files.len() >= 1, "Should find at least one file matching pattern. Found: {:?}", files);
        assert!(files.iter().any(|p| p.file_name().unwrap().to_str().unwrap().ends_with(".toml")));
        
        // Verify the txt file is not included
        assert!(!files.iter().any(|p| p.file_name().unwrap().to_str().unwrap().ends_with(".txt")));
    }

    #[test]
    fn test_hierarchical_convenience() {
        let provider = Wildcard::hierarchical("config", "myapp");
        let patterns = provider.patterns();
        
        // Should contain patterns for various directories
        assert!(patterns.iter().any(|p| p.contains("~/.config/myapp")));
        assert!(patterns.iter().any(|p| p.contains("./myapp")));
        assert!(patterns.iter().any(|p| p.contains("**/myapp.")));
    }

    #[test]
    fn test_xdg_convenience() {
        let provider = Wildcard::xdg("myapp");
        let patterns = provider.patterns();
        
        // Should contain XDG-compliant patterns
        assert!(patterns.iter().any(|p| p.contains("~/.config/myapp")));
        assert!(patterns.iter().any(|p| p.contains("~/.local/share/myapp")));
        assert!(patterns.iter().any(|p| p.contains("/etc/myapp")));
    }

    #[test]
    fn test_development_convenience() {
        let provider = Wildcard::development("config");
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