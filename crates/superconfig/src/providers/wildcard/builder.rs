//! Builder pattern for advanced Wildcard provider configuration

use crate::providers::wildcard::{
    core::Wildcard,
    discovery::SearchStrategy,
    sorting::MergeOrder,
};
use figment::Error;
use std::path::PathBuf;

/// Advanced builder for configuring Wildcard providers
///
/// The WildcardBuilder provides a fluent interface for complex wildcard
/// configurations. Use this when you need fine-grained control over
/// search strategies, merge ordering, and pattern combinations.
///
/// # Examples
///
/// ```rust
/// use superconfig::{WildcardBuilder, MergeOrder, SearchStrategy};
/// use std::path::PathBuf;
///
/// # fn main() -> Result<(), figment::Error> {
/// let provider = WildcardBuilder::new()
///     .pattern("*.toml")?
///     .pattern("*.yaml")?
///     .search_strategy(SearchStrategy::Recursive {
///         roots: vec![PathBuf::from("./config")],
///         max_depth: Some(3),
///     })
///     .merge_order(MergeOrder::Custom(vec![
///         "base.*".to_string(),
///         "env-*.toml".to_string(),
///         "local.*".to_string(),
///     ]))
///     .build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Default)]
pub struct WildcardBuilder {
    patterns: Vec<String>,
    search_strategy: Option<SearchStrategy>,
    merge_order: Option<MergeOrder>,
}

impl WildcardBuilder {
    /// Create a new empty builder
    ///
    /// Use this to start building a complex Wildcard configuration
    /// from scratch with full control over all parameters.
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::WildcardBuilder;
    ///
    /// let builder = WildcardBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a glob pattern to the builder
    ///
    /// Multiple patterns can be added and will be combined intelligently
    /// based on their types. Each pattern is validated when added.
    ///
    /// # Arguments
    /// * `pattern` - Glob pattern string to add
    ///
    /// # Returns
    /// The builder for method chaining
    ///
    /// # Errors
    /// Returns `figment::Error` if the pattern is invalid
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::WildcardBuilder;
    ///
    /// # fn main() -> Result<(), figment::Error> {
    /// let builder = WildcardBuilder::new()
    ///     .pattern("*.toml")?
    ///     .pattern("config/*.yaml")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn pattern(mut self, pattern: &str) -> Result<Self, Error> {
        // Validate the pattern by attempting to compile it
        globset::Glob::new(pattern).map_err(|e| Error::from(format!("Invalid pattern '{}': {}", pattern, e)))?;
        self.patterns.push(pattern.to_string());
        Ok(self)
    }

    /// Add multiple patterns at once
    ///
    /// Convenience method for adding multiple patterns in a single call.
    /// All patterns are validated before any are added.
    ///
    /// # Arguments
    /// * `patterns` - Iterator of pattern strings to add
    ///
    /// # Returns
    /// The builder for method chaining
    ///
    /// # Errors
    /// Returns `figment::Error` if any pattern is invalid
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::WildcardBuilder;
    ///
    /// # fn main() -> Result<(), figment::Error> {
    /// let patterns = ["*.toml", "*.yaml", "config/*.json"];
    /// let builder = WildcardBuilder::new()
    ///     .patterns(&patterns)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn patterns<I, S>(mut self, patterns: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let pattern_strings: Vec<String> = patterns
            .into_iter()
            .map(|s| s.as_ref().to_string())
            .collect();

        // Validate all patterns first
        for pattern in &pattern_strings {
            globset::Glob::new(pattern).map_err(|e| Error::from(format!("Invalid pattern '{}': {}", pattern, e)))?;
        }

        // If all patterns are valid, add them
        self.patterns.extend(pattern_strings);
        Ok(self)
    }

    /// Set the search strategy
    ///
    /// Override the automatically determined search strategy with a custom one.
    /// This gives you full control over how files are discovered.
    ///
    /// # Arguments
    /// * `strategy` - The search strategy to use
    ///
    /// # Returns
    /// The builder for method chaining
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::{WildcardBuilder, SearchStrategy};
    /// use std::path::PathBuf;
    ///
    /// # fn main() -> Result<(), figment::Error> {
    /// let builder = WildcardBuilder::new()
    ///     .pattern("*.toml")?
    ///     .search_strategy(SearchStrategy::Recursive {
    ///         roots: vec![PathBuf::from("./config")],
    ///         max_depth: Some(2),
    ///     });
    /// # Ok(())
    /// # }
    /// ```
    pub fn search_strategy(mut self, strategy: SearchStrategy) -> Self {
        self.search_strategy = Some(strategy);
        self
    }

    /// Set the merge order strategy
    ///
    /// Control how multiple discovered files are ordered and merged.
    /// Later files in the order have higher priority.
    ///
    /// # Arguments
    /// * `order` - The merge order strategy to use
    ///
    /// # Returns
    /// The builder for method chaining
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::{WildcardBuilder, MergeOrder};
    ///
    /// # fn main() -> Result<(), figment::Error> {
    /// let builder = WildcardBuilder::new()
    ///     .pattern("*.toml")?
    ///     .merge_order(MergeOrder::ModificationTimeAscending);
    /// # Ok(())
    /// # }
    /// ```
    pub fn merge_order(mut self, order: MergeOrder) -> Self {
        self.merge_order = Some(order);
        self
    }

    /// Build the final Wildcard provider
    ///
    /// Construct the Wildcard provider with all configured options.
    /// This consumes the builder and validates the final configuration.
    ///
    /// # Returns
    /// A configured Wildcard provider ready for use
    ///
    /// # Errors
    /// Returns `GlobError` if:
    /// - No patterns have been added
    /// - Pattern compilation fails
    /// - Search strategy conflicts with patterns
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::WildcardBuilder;
    ///
    /// let provider = WildcardBuilder::new()
    ///     .pattern("*.toml")?
    ///     .build()?;
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn build(self) -> Result<Wildcard, figment::Error> {
        if self.patterns.is_empty() {
            return Err(figment::Error::from("At least one pattern is required"));
        }

        let mut wildcard = Wildcard::from_patterns(&self.patterns)?;

        if let Some(strategy) = self.search_strategy {
            wildcard = wildcard.with_search_strategy(strategy);
        }

        if let Some(order) = self.merge_order {
            wildcard = wildcard.with_merge_order(order);
        }

        Ok(wildcard)
    }
}

// Convenience builder methods for common configurations
impl WildcardBuilder {
    /// Configure for current directory search
    ///
    /// Sets up the builder to search only the current directory
    /// with the given patterns. Equivalent to using SearchStrategy::Current.
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::WildcardBuilder;
    ///
    /// let provider = WildcardBuilder::new()
    ///     .pattern("*.toml")?
    ///     .current_directory()
    ///     .build()?;
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn current_directory(self) -> Self {
        self.search_strategy(SearchStrategy::Current)
    }

    /// Configure for specific directories
    ///
    /// Sets up the builder to search only the specified directories
    /// (non-recursive). Files in subdirectories will not be found.
    ///
    /// # Arguments
    /// * `directories` - Directories to search
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::WildcardBuilder;
    /// use std::path::PathBuf;
    ///
    /// let provider = WildcardBuilder::new()
    ///     .pattern("*.toml")?
    ///     .directories(&[PathBuf::from("./config"), PathBuf::from("~/.config")])
    ///     .build()?;
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn directories(self, directories: &[PathBuf]) -> Self {
        self.search_strategy(SearchStrategy::Directories(directories.to_vec()))
    }

    /// Configure for recursive search
    ///
    /// Sets up the builder to search recursively from the specified
    /// root directories with optional depth limiting.
    ///
    /// # Arguments
    /// * `roots` - Root directories to start search from
    /// * `max_depth` - Optional maximum depth to search (None = unlimited)
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::WildcardBuilder;
    /// use std::path::PathBuf;
    ///
    /// let provider = WildcardBuilder::new()
    ///     .pattern("*.toml")?
    ///     .recursive(&[PathBuf::from("./config")], Some(3))
    ///     .build()?;
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn recursive(self, roots: &[PathBuf], max_depth: Option<usize>) -> Self {
        self.search_strategy(SearchStrategy::Recursive {
            roots: roots.to_vec(),
            max_depth,
        })
    }

    /// Configure alphabetical merge order
    ///
    /// Files will be merged in alphabetical order by filename.
    /// This is the default behavior and provides predictable results.
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::WildcardBuilder;
    ///
    /// let provider = WildcardBuilder::new()
    ///     .pattern("*.toml")?
    ///     .alphabetical()
    ///     .build()?;
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn alphabetical(self) -> Self {
        self.merge_order(MergeOrder::Alphabetical)
    }

    /// Configure reverse alphabetical merge order
    ///
    /// Files will be merged in reverse alphabetical order by filename.
    /// Useful when higher-numbered configs should have lower priority.
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::WildcardBuilder;
    ///
    /// let provider = WildcardBuilder::new()
    ///     .pattern("*.toml")?
    ///     .reverse_alphabetical()
    ///     .build()?;
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn reverse_alphabetical(self) -> Self {
        self.merge_order(MergeOrder::Reverse)
    }

    /// Configure modification time merge order
    ///
    /// Files will be merged by modification time, with older files
    /// having lower priority. Recent changes will override older ones.
    ///
    /// # Arguments
    /// * `newest_first` - If true, newest files have lowest priority
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::WildcardBuilder;
    ///
    /// // Newest files override (most common)
    /// let provider = WildcardBuilder::new()
    ///     .pattern("*.toml")?
    ///     .by_modification_time(false)
    ///     .build()?;
    ///
    /// // Oldest files override (preserve established config)
    /// let provider = WildcardBuilder::new()
    ///     .pattern("*.toml")?
    ///     .by_modification_time(true)
    ///     .build()?;
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn by_modification_time(self, newest_first: bool) -> Self {
        let order = if newest_first {
            MergeOrder::ModificationTimeDescending
        } else {
            MergeOrder::ModificationTimeAscending
        };
        self.merge_order(order)
    }

    /// Configure file size merge order
    ///
    /// Files will be merged by file size. Typically smaller files
    /// contain base configurations while larger files contain overrides.
    ///
    /// # Arguments
    /// * `largest_first` - If true, largest files have lowest priority
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::WildcardBuilder;
    ///
    /// // Larger files override (common for detailed configs)
    /// let provider = WildcardBuilder::new()
    ///     .pattern("*.toml")?
    ///     .by_file_size(false)
    ///     .build()?;
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn by_file_size(self, largest_first: bool) -> Self {
        let order = if largest_first {
            MergeOrder::SizeDescending
        } else {
            MergeOrder::SizeAscending
        };
        self.merge_order(order)
    }

    /// Configure custom priority merge order
    ///
    /// Files will be merged according to custom pattern-based priorities.
    /// Files matching earlier patterns have lower priority.
    ///
    /// # Arguments
    /// * `priority_patterns` - Patterns in priority order (low to high)
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::WildcardBuilder;
    ///
    /// let provider = WildcardBuilder::new()
    ///     .pattern("*.toml")?
    ///     .custom_priority(&[
    ///         "base.*",        // Lowest priority
    ///         "env-*.toml",    // Medium priority
    ///         "local.*",       // Highest priority
    ///     ])
    ///     .build()?;
    /// # Ok::<(), figment::Error>(())
    /// ```
    pub fn custom_priority<I, S>(self, priority_patterns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let patterns: Vec<String> = priority_patterns
            .into_iter()
            .map(|s| s.as_ref().to_string())
            .collect();
        self.merge_order(MergeOrder::Custom(patterns))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let provider = WildcardBuilder::new()
            .pattern("*.toml")
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(provider.patterns(), &["*.toml"]);
    }

    #[test]
    fn test_builder_multiple_patterns() {
        let patterns = ["*.toml", "*.yaml"];
        let provider = WildcardBuilder::new()
            .patterns(&patterns)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(provider.patterns().len(), 2);
        assert!(provider.patterns().contains(&"*.toml".to_string()));
        assert!(provider.patterns().contains(&"*.yaml".to_string()));
    }

    #[test]
    fn test_builder_with_search_strategy() {
        let provider = WildcardBuilder::new()
            .pattern("*.toml")
            .unwrap()
            .current_directory()
            .build()
            .unwrap();

        match provider.search_strategy() {
            SearchStrategy::Current => {},
            _ => panic!("Expected Current search strategy"),
        }
    }

    #[test]
    fn test_builder_with_merge_order() {
        let provider = WildcardBuilder::new()
            .pattern("*.toml")
            .unwrap()
            .reverse_alphabetical()
            .build()
            .unwrap();

        match provider.merge_order() {
            MergeOrder::Reverse => {},
            _ => panic!("Expected Reverse merge order"),
        }
    }

    #[test]
    fn test_builder_recursive() {
        let roots = vec![PathBuf::from("./config")];
        let provider = WildcardBuilder::new()
            .pattern("*.toml")
            .unwrap()
            .recursive(&roots, Some(3))
            .build()
            .unwrap();

        match provider.search_strategy() {
            SearchStrategy::Recursive { roots: r, max_depth } => {
                assert_eq!(r.len(), 1);
                assert_eq!(r[0], PathBuf::from("./config"));
                assert_eq!(*max_depth, Some(3));
            },
            _ => panic!("Expected Recursive search strategy"),
        }
    }

    #[test]
    fn test_builder_custom_priority() {
        let patterns = ["base.*", "local.*"];
        let provider = WildcardBuilder::new()
            .pattern("*.toml")
            .unwrap()
            .custom_priority(&patterns)
            .build()
            .unwrap();

        match provider.merge_order() {
            MergeOrder::Custom(priorities) => {
                assert_eq!(priorities.len(), 2);
                assert_eq!(priorities[0], "base.*");
                assert_eq!(priorities[1], "local.*");
            },
            _ => panic!("Expected Custom merge order"),
        }
    }

    #[test]
    fn test_builder_invalid_pattern() {
        let result = WildcardBuilder::new()
            .pattern("[invalid")
            .unwrap_err();

        // Should get a glob error for invalid pattern
        // The actual error message will contain "invalid" or "bracket" or similar
        let error_str = result.to_string();
        assert!(error_str.contains("invalid") || error_str.contains("bracket") || error_str.contains("Pattern"));
    }

    #[test]
    fn test_builder_no_patterns() {
        let result = WildcardBuilder::new()
            .build()
            .unwrap_err();

        assert!(result.to_string().contains("At least one pattern is required"));
    }
}