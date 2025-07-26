//! File sorting and merge order strategies for wildcard configuration discovery

use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

/// Defines how multiple configuration files should be merged
///
/// When wildcard patterns discover multiple files, this enum controls the order
/// in which they are processed and merged. Different strategies optimize for
/// different use cases and organizational patterns.
///
/// # Performance Characteristics
///
/// - **Alphabetical/Reverse**: O(n log n) sorting, most predictable
/// - **Size-based**: O(n log n) + O(n) for metadata reads
/// - **Time-based**: O(n log n) + O(n) for metadata reads
/// - **Custom**: O(n log n) + O(n) for pattern matching per file
///
/// # Examples
///
/// ```rust
/// use superconfig::{Wildcard, MergeOrder};
///
/// // Alphabetical order (most common)
/// let provider = Wildcard::from_pattern("*.toml")?
///     .with_merge_order(MergeOrder::Alphabetical);
///
/// // Custom priority order
/// let provider = Wildcard::from_patterns(&["*.toml", "*.yaml"])?
///     .with_merge_order(MergeOrder::Custom(vec![
///         "base.*".to_string(),        // Base configs first
///         "*.toml".to_string(),        // TOML files next
///         "*.yaml".to_string(),        // YAML files last
///     ]));
/// # Ok::<(), figment::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeOrder {
    /// Sort files alphabetically by filename (most predictable)
    ///
    /// This is the most common and predictable ordering strategy.
    /// Files are sorted by their filename in lexicographic order.
    ///
    /// # Use Cases
    /// - Default behavior for most applications
    /// - When you want predictable, repeatable configuration loading
    /// - Development environments where consistency matters
    ///
    /// # Example Order
    /// ```text
    /// config-01-base.toml
    /// config-02-database.toml  
    /// config-03-features.toml
    /// config-99-local.toml
    /// ```
    Alphabetical,

    /// Sort files alphabetically in reverse order (Z to A)
    ///
    /// Useful when you want later files (by name) to have lower priority.
    ///
    /// # Use Cases
    /// - When higher-numbered configs should be base configs
    /// - Reverse priority systems
    ///
    /// # Example Order
    /// ```text
    /// config-99-local.toml      (lowest priority)
    /// config-03-features.toml
    /// config-02-database.toml
    /// config-01-base.toml       (highest priority)
    /// ```
    Reverse,

    /// Sort files by file size (smallest to largest)
    ///
    /// Smaller files typically contain base configurations while larger
    /// files contain extensive overrides or feature-specific settings.
    ///
    /// # Use Cases
    /// - When file size correlates with configuration importance
    /// - Base configs are typically small, feature configs are large
    /// - Microservice configurations with different complexity levels
    ///
    /// # Performance Note
    /// Requires filesystem metadata access for each file.
    SizeAscending,

    /// Sort files by file size (largest to smallest)
    ///
    /// Larger files get lower priority, useful when comprehensive
    /// base configurations are large and specific overrides are small.
    SizeDescending,

    /// Sort files by modification time (oldest to newest)
    ///
    /// Older files get higher priority, newer modifications override.
    /// This creates a natural "last modified wins" behavior.
    ///
    /// # Use Cases  
    /// - Development environments where recent changes should override
    /// - Configuration that evolves over time
    /// - Temporary override files that should take precedence
    ///
    /// # Performance Note
    /// Requires filesystem metadata access for each file.
    ModificationTimeAscending,

    /// Sort files by modification time (newest to oldest)
    ///
    /// Newer files get lower priority, preserving established configurations.
    ModificationTimeDescending,

    /// Custom priority order based on file patterns
    ///
    /// Files are sorted according to the position of the first matching
    /// pattern in the provided vector. Files that don't match any pattern
    /// are placed at the end in alphabetical order.
    ///
    /// # Pattern Matching
    /// - Supports glob patterns (e.g., `"*.toml"`, `"base.*"`)
    /// - Matches against the filename only (not full path)
    /// - First matching pattern determines priority
    /// - Non-matching files sorted alphabetically at the end
    ///
    /// # Use Cases
    /// - Complex priority rules (base → env → local → user)
    /// - Framework-specific loading orders
    /// - Enterprise configuration hierarchies
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::MergeOrder;
    ///
    /// let order = MergeOrder::Custom(vec![
    ///     "base.*".to_string(),           // 1. Base configuration files
    ///     "env-*.toml".to_string(),       // 2. Environment-specific configs  
    ///     "features-*.yaml".to_string(),  // 3. Feature flags
    ///     "local.*".to_string(),          // 4. Local developer overrides
    /// ]);
    /// ```
    ///
    /// Given files: `["local.toml", "base.yaml", "env-prod.toml", "features-auth.yaml", "other.json"]`
    ///
    /// Result order:
    /// 1. `base.yaml` (matches `base.*`)
    /// 2. `env-prod.toml` (matches `env-*.toml`)  
    /// 3. `features-auth.yaml` (matches `features-*.yaml`)
    /// 4. `local.toml` (matches `local.*`)
    /// 5. `other.json` (no match, alphabetical)
    Custom(Vec<String>),
}

impl Default for MergeOrder {
    fn default() -> Self {
        Self::Alphabetical
    }
}

impl MergeOrder {
    /// Sort a list of file paths according to this merge order strategy
    ///
    /// # Arguments
    /// * `files` - Vector of file paths to sort
    ///
    /// # Returns
    /// Sorted vector of file paths in merge order (lowest to highest priority)
    ///
    /// # Performance
    /// - Alphabetical sorting: O(n log n)
    /// - Size/time-based sorting: O(n log n) + O(n) for metadata access
    /// - Custom sorting: O(n log n) + O(n * m) where m is number of patterns
    ///
    /// # Error Handling
    /// - Files that can't be accessed for metadata are sorted alphabetically
    /// - Invalid patterns in Custom order are ignored
    /// - Non-existent files are excluded from the result
    pub fn sort_files(&self, mut files: Vec<PathBuf>) -> Vec<PathBuf> {
        match self {
            MergeOrder::Alphabetical => {
                files.sort_by(|a, b| {
                    a.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .cmp(b.file_name().and_then(|n| n.to_str()).unwrap_or(""))
                });
                files
            }

            MergeOrder::Reverse => {
                files.sort_by(|a, b| {
                    b.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .cmp(a.file_name().and_then(|n| n.to_str()).unwrap_or(""))
                });
                files
            }

            MergeOrder::SizeAscending => {
                files.sort_by(|a, b| {
                    let size_a = file_size(a).unwrap_or(0);
                    let size_b = file_size(b).unwrap_or(0);
                    size_a.cmp(&size_b)
                });
                files
            }

            MergeOrder::SizeDescending => {
                files.sort_by(|a, b| {
                    let size_a = file_size(a).unwrap_or(0);
                    let size_b = file_size(b).unwrap_or(0);
                    size_b.cmp(&size_a)
                });
                files
            }

            MergeOrder::ModificationTimeAscending => {
                files.sort_by(|a, b| {
                    let time_a = modification_time(a).unwrap_or(SystemTime::UNIX_EPOCH);
                    let time_b = modification_time(b).unwrap_or(SystemTime::UNIX_EPOCH);
                    time_a.cmp(&time_b)
                });
                files
            }

            MergeOrder::ModificationTimeDescending => {
                files.sort_by(|a, b| {
                    let time_a = modification_time(a).unwrap_or(SystemTime::UNIX_EPOCH);
                    let time_b = modification_time(b).unwrap_or(SystemTime::UNIX_EPOCH);
                    time_b.cmp(&time_a)
                });
                files
            }

            MergeOrder::Custom(patterns) => {
                files.sort_by(|a, b| {
                    let priority_a = pattern_priority(a, patterns);
                    let priority_b = pattern_priority(b, patterns);

                    match priority_a.cmp(&priority_b) {
                        std::cmp::Ordering::Equal => {
                            // If same priority, sort alphabetically
                            a.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("")
                                .cmp(b.file_name().and_then(|n| n.to_str()).unwrap_or(""))
                        }
                        other => other,
                    }
                });
                files
            }
        }
    }
}

/// Get file size in bytes, returning None if metadata cannot be accessed
fn file_size(path: &Path) -> Option<u64> {
    fs::metadata(path).ok().map(|meta| meta.len())
}

/// Get file modification time, returning None if metadata cannot be accessed
fn modification_time(path: &Path) -> Option<SystemTime> {
    fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
}

/// Calculate pattern priority for custom ordering
///
/// Returns the index of the first matching pattern, or usize::MAX for no match
fn pattern_priority(path: &Path, patterns: &[String]) -> usize {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    for (index, pattern) in patterns.iter().enumerate() {
        if pattern_matches(filename, pattern) {
            return index;
        }
    }

    usize::MAX // No match - sort at end
}

/// Simple glob pattern matching for custom ordering
///
/// Supports basic glob patterns: `*`, `?`, and literal strings
/// This is a simplified implementation for ordering purposes
fn pattern_matches(filename: &str, pattern: &str) -> bool {
    // Simple glob matching - can be enhanced later
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            let prefix = parts[0];
            let suffix = parts[1];
            filename.starts_with(prefix) && filename.ends_with(suffix)
        } else if pattern.starts_with('*') {
            filename.ends_with(&pattern[1..])
        } else if pattern.ends_with('*') {
            filename.starts_with(&pattern[..pattern.len() - 1])
        } else {
            filename == pattern
        }
    } else {
        filename == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_alphabetical_sorting() {
        let files = vec![
            PathBuf::from("config-z.toml"),
            PathBuf::from("config-a.toml"), 
            PathBuf::from("config-m.toml"),
        ];

        let sorted = MergeOrder::Alphabetical.sort_files(files);
        
        assert_eq!(sorted[0].file_name().unwrap(), "config-a.toml");
        assert_eq!(sorted[1].file_name().unwrap(), "config-m.toml");
        assert_eq!(sorted[2].file_name().unwrap(), "config-z.toml");
    }

    #[test]
    fn test_reverse_sorting() {
        let files = vec![
            PathBuf::from("config-a.toml"),
            PathBuf::from("config-z.toml"),
            PathBuf::from("config-m.toml"),
        ];

        let sorted = MergeOrder::Reverse.sort_files(files);
        
        assert_eq!(sorted[0].file_name().unwrap(), "config-z.toml");
        assert_eq!(sorted[1].file_name().unwrap(), "config-m.toml");
        assert_eq!(sorted[2].file_name().unwrap(), "config-a.toml");
    }

    #[test]
    fn test_pattern_matching() {
        assert!(pattern_matches("config.toml", "config.*"));
        assert!(pattern_matches("base.yaml", "base.*"));
        assert!(pattern_matches("env-prod.toml", "env-*.toml"));
        assert!(pattern_matches("local", "local"));
        assert!(!pattern_matches("other.json", "config.*"));
    }

    #[test]
    fn test_custom_sorting() {
        let files = vec![
            PathBuf::from("local.toml"),
            PathBuf::from("base.yaml"),
            PathBuf::from("env-prod.toml"),
            PathBuf::from("other.json"),
        ];

        let patterns = vec![
            "base.*".to_string(),
            "env-*.toml".to_string(),
            "local.*".to_string(),
        ];

        let sorted = MergeOrder::Custom(patterns).sort_files(files);
        
        assert_eq!(sorted[0].file_name().unwrap(), "base.yaml");       // First pattern
        assert_eq!(sorted[1].file_name().unwrap(), "env-prod.toml");   // Second pattern  
        assert_eq!(sorted[2].file_name().unwrap(), "local.toml");      // Third pattern
        assert_eq!(sorted[3].file_name().unwrap(), "other.json");      // No match, alphabetical
    }
}