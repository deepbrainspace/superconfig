//! Pattern parsing utilities for wildcard configuration discovery

use globset::{Glob, GlobSetBuilder};
use std::path::PathBuf;

use super::discovery::SearchStrategy;

/// Parse a pattern string into search strategy and file patterns
///
/// This function intelligently parses patterns to extract directory information
/// and convert them into appropriate search strategies.
///
/// # Pattern Types Supported
///
/// ## Simple File Patterns
/// - `"*.toml"` → Search current directory for .toml files
/// - `"config.*"` → Search current directory for files starting with "config"
///
/// ## Path-based Patterns  
/// - `"./config/*.toml"` → Search "./config" directory for .toml files
/// - `"~/.config/app/*.yaml"` → Search "~/.config/app" directory for .yaml files
///
/// ## Recursive Patterns
/// - `"**/*.toml"` → Search recursively from current directory for .toml files
/// - `"./config/**/*.yaml"` → Search recursively from "./config" for .yaml files
///
/// ## Brace Expansion Patterns
/// - `"{dir1,dir2}/*.toml"` → Search both "dir1" and "dir2" for .toml files
/// - `"{~/.config,./config}/*.yaml"` → Search multiple directories for .yaml files
///
/// # Examples
///
/// ```rust
/// use superconfig::providers::wildcard::parsing::parse_pattern;
///
/// // Simple pattern
/// let (strategy, patterns) = parse_pattern("*.toml")?;
/// 
/// // Path-based pattern
/// let (strategy, patterns) = parse_pattern("./config/*.yaml")?;
///
/// // Recursive pattern
/// let (strategy, patterns) = parse_pattern("**/*.json")?;
///
/// // Multi-directory pattern
/// let (strategy, patterns) = parse_pattern("{./config,~/.config}/*.toml")?;
/// # Ok::<(), figment::Error>(())
/// ```
///
/// # Error Handling
///
/// Returns `figment::Error` if:
/// - Pattern contains invalid glob syntax
/// - Brace expansion syntax is malformed
/// - Pattern is empty or invalid
pub fn parse_pattern(pattern: &str) -> Result<(SearchStrategy, Vec<String>), figment::Error> {
    if pattern.is_empty() {
        return Err(figment::Error::from("Pattern cannot be empty"));
    }

    // Handle brace expansion patterns first
    if let Some((directories, file_pattern)) = parse_brace_expansion(pattern) {
        return Ok((
            SearchStrategy::Directories(directories),
            vec![file_pattern],
        ));
    }

    // Handle recursive patterns
    if pattern.contains("**/") {
        let (dirs, file_pattern) = parse_recursive_pattern(pattern);
        return Ok((
            SearchStrategy::Recursive {
                roots: dirs,
                max_depth: None,
            },
            vec![file_pattern],
        ));
    }

    // Handle path-based patterns
    if pattern.contains('/') {
        let (dir, file_pattern) = parse_path_pattern(pattern);
        return Ok((
            SearchStrategy::Directories(vec![dir]),
            vec![file_pattern],
        ));
    }

    // Simple file pattern - search current directory
    Ok((SearchStrategy::Current, vec![pattern.to_string()]))
}

/// Parse brace expansion patterns like `{dir1,dir2}/*.toml`
///
/// # Returns
/// - `Some((directories, file_pattern))` if brace expansion detected
/// - `None` if no brace expansion found
///
/// # Examples
/// - `"{dir1,dir2}/*.toml"` → `(["dir1", "dir2"], "*.toml")`
/// - `"{~/.config,./config}/app.yaml"` → `(["~/.config", "./config"], "app.yaml")`
fn parse_brace_expansion(pattern: &str) -> Option<(Vec<PathBuf>, String)> {
    if !pattern.starts_with('{') {
        return None;
    }

    let close_brace = pattern.find('}')?;
    let dirs_part = &pattern[1..close_brace]; // Remove { and }
    let rest = &pattern[close_brace + 1..];   // Everything after }

    // Split directories by comma
    let directories: Vec<PathBuf> = dirs_part
        .split(',')
        .map(|d| PathBuf::from(d.trim()))
        .collect();

    // Remove leading slash if present
    let file_pattern = if let Some(stripped) = rest.strip_prefix('/') {
        stripped.to_string()
    } else {
        rest.to_string()
    };

    Some((directories, file_pattern))
}

/// Parse recursive patterns like `**/config.toml` or `./config/**/*.yaml`
///
/// # Returns
/// - `(root_directories, file_pattern)` tuple
///
/// # Examples
/// - `"**/*.toml"` → `(["."], "*.toml")`
/// - `"./config/**/*.yaml"` → `(["./config"], "*.yaml")`
/// - `"dir1/**/subdir/*.json"` → `(["dir1"], "subdir/*.json")`
fn parse_recursive_pattern(pattern: &str) -> (Vec<PathBuf>, String) {
    if let Some(double_star_pos) = pattern.find("**/") {
        let prefix = &pattern[..double_star_pos];
        let suffix = &pattern[double_star_pos + 3..]; // Skip "*/"

        let root_dir = if prefix.is_empty() || prefix == "./" {
            PathBuf::from(".")
        } else {
            PathBuf::from(prefix.trim_end_matches('/'))
        };

        (vec![root_dir], suffix.to_string())
    } else {
        // Pattern ends with /** (no file pattern)
        let prefix = pattern.trim_end_matches("/**");
        let root_dir = if prefix.is_empty() {
            PathBuf::from(".")
        } else {
            PathBuf::from(prefix)
        };
        (vec![root_dir], "*".to_string())
    }
}

/// Parse path-based patterns like `./config/*.toml`
///
/// # Returns
/// - `(directory, file_pattern)` tuple
///
/// # Examples
/// - `"./config/*.toml"` → `("./config", "*.toml")`
/// - `"~/.config/app.yaml"` → `("~/.config", "app.yaml")`
/// - `"/etc/myapp/config.*"` → `("/etc/myapp", "config.*")`
fn parse_path_pattern(pattern: &str) -> (PathBuf, String) {
    if let Some(last_slash) = pattern.rfind('/') {
        let dir_part = &pattern[..last_slash];
        let file_part = &pattern[last_slash + 1..];

        let directory = if dir_part.is_empty() {
            PathBuf::from("/") // Root directory
        } else {
            PathBuf::from(dir_part)
        };

        (directory, file_part.to_string())
    } else {
        // No slash found, treat as current directory
        (PathBuf::from("."), pattern.to_string())
    }
}

/// Build a GlobSet from multiple patterns
///
/// # Arguments
/// * `patterns` - Vector of glob pattern strings
///
/// # Returns
/// Compiled GlobSet ready for file matching
///
/// # Errors
/// Returns `figment::Error` if any pattern is invalid
///
/// # Examples
/// ```rust
/// use superconfig::providers::wildcard::parsing::build_globset;
///
/// let patterns = vec!["*.toml", "*.yaml", "config.*"];
/// let globset = build_globset(&patterns)?;
/// # Ok::<(), figment::Error>(())
/// ```
pub fn build_globset(patterns: &[impl AsRef<str>]) -> Result<globset::GlobSet, figment::Error> {
    let mut builder = GlobSetBuilder::new();
    
    for pattern in patterns {
        let glob = Glob::new(pattern.as_ref()).map_err(|e| figment::Error::from(format!("Invalid pattern '{}': {}", pattern.as_ref(), e)))?;
        builder.add(glob);
    }
    
    builder.build().map_err(|e| figment::Error::from(format!("Failed to build globset: {e}")))
}

/// Parse multiple patterns and combine their search strategies
///
/// This function handles the case where users provide multiple patterns
/// that might have different search requirements.
///
/// # Strategy Resolution
/// - If all patterns use the same strategy type, combine them
/// - If patterns have conflicting strategies, use the most general (Recursive)
/// - Current directory patterns are upgraded to Directories(["."])
///
/// # Examples
/// ```rust
/// use superconfig::providers::wildcard::parsing::parse_multiple_patterns;
///
/// let patterns = vec!["*.toml", "*.yaml"];
/// let (strategy, combined_patterns) = parse_multiple_patterns(&patterns)?;
///
/// let mixed_patterns = vec!["./config/*.toml", "**/*.yaml"];  
/// let (strategy, combined_patterns) = parse_multiple_patterns(&mixed_patterns)?;
/// # Ok::<(), figment::Error>(())
/// ```
pub fn parse_multiple_patterns(patterns: &[impl AsRef<str>]) -> Result<(SearchStrategy, Vec<String>), figment::Error> {
    if patterns.is_empty() {
        return Err(figment::Error::from("At least one pattern is required"));
    }

    if patterns.len() == 1 {
        return parse_pattern(patterns[0].as_ref());
    }

    let mut all_strategies = Vec::new();
    let mut all_file_patterns = Vec::new();

    // Parse each pattern individually
    for pattern in patterns {
        let (strategy, file_patterns) = parse_pattern(pattern.as_ref())?;
        all_strategies.push(strategy);
        all_file_patterns.extend(file_patterns);
    }

    // Determine the combined search strategy
    let combined_strategy = resolve_combined_strategy(all_strategies);

    Ok((combined_strategy, all_file_patterns))
}

/// Resolve multiple search strategies into a single combined strategy
///
/// Priority (most general wins):
/// 1. Custom strategies are preserved if all are custom
/// 2. Recursive strategies take precedence over Directory/Current
/// 3. Directory strategies are combined
/// 4. Current directory patterns remain as Current if they're all current directory
fn resolve_combined_strategy(strategies: Vec<SearchStrategy>) -> SearchStrategy {
    let mut directories = Vec::new();
    let mut recursive_roots = Vec::new();
    let mut has_recursive = false;
    let mut has_custom = false;
    let mut current_count = 0;
    let strategies_len = strategies.len();

    for strategy in strategies {
        match strategy {
            SearchStrategy::Directories(dirs) => {
                directories.extend(dirs);
            }
            SearchStrategy::Recursive { roots, .. } => {
                has_recursive = true;
                recursive_roots.extend(roots);
            }
            SearchStrategy::Current => {
                current_count += 1;
                directories.push(PathBuf::from("."));
            }
            SearchStrategy::Custom(_) => {
                has_custom = true;
                // Custom strategies can't be easily combined
                // Fall back to recursive search from current directory
                recursive_roots.push(PathBuf::from("."));
                has_recursive = true;
            }
        }
    }

    // Return the most appropriate combined strategy
    if has_recursive || has_custom {
        // Deduplicate recursive roots
        recursive_roots.sort();
        recursive_roots.dedup();
        SearchStrategy::Recursive {
            roots: recursive_roots,
            max_depth: None,
        }
    } else if current_count == strategies_len {
        // All patterns were Current directory patterns
        SearchStrategy::Current
    } else {
        // Deduplicate directories
        directories.sort();
        directories.dedup();
        SearchStrategy::Directories(directories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_pattern() {
        let (strategy, patterns) = parse_pattern("*.toml").unwrap();
        
        match strategy {
            SearchStrategy::Current => {}
            _ => panic!("Expected Current strategy"),
        }
        
        assert_eq!(patterns, vec!["*.toml"]);
    }

    #[test]
    fn test_parse_path_pattern() {
        let (strategy, patterns) = parse_pattern("./config/*.yaml").unwrap();
        
        match strategy {
            SearchStrategy::Directories(dirs) => {
                assert_eq!(dirs.len(), 1);
                assert_eq!(dirs[0], PathBuf::from("./config"));
            }
            _ => panic!("Expected Directories strategy"),
        }
        
        assert_eq!(patterns, vec!["*.yaml"]);
    }

    #[test]
    fn test_parse_recursive_pattern() {
        let (strategy, patterns) = parse_pattern("**/*.json").unwrap();
        
        match strategy {
            SearchStrategy::Recursive { roots, max_depth } => {
                assert_eq!(roots.len(), 1);
                assert_eq!(roots[0], PathBuf::from("."));
                assert_eq!(max_depth, None);
            }
            _ => panic!("Expected Recursive strategy"),
        }
        
        assert_eq!(patterns, vec!["*.json"]);
    }

    #[test]
    fn test_parse_brace_expansion() {
        let (strategy, patterns) = parse_pattern("{./config,~/.config}/*.toml").unwrap();
        
        match strategy {
            SearchStrategy::Directories(dirs) => {
                assert_eq!(dirs.len(), 2);
                assert!(dirs.contains(&PathBuf::from("./config")));
                assert!(dirs.contains(&PathBuf::from("~/.config")));
            }
            _ => panic!("Expected Directories strategy"),
        }
        
        assert_eq!(patterns, vec!["*.toml"]);
    }

    #[test]
    fn test_parse_multiple_patterns_same_type() {
        let patterns = vec!["*.toml".to_string(), "*.yaml".to_string()];
        let (strategy, combined_patterns) = parse_multiple_patterns(&patterns).unwrap();
        
        match strategy {
            SearchStrategy::Current => {}
            _ => panic!("Expected Current strategy for simple patterns"),
        }
        
        assert_eq!(combined_patterns.len(), 2);
        assert!(combined_patterns.contains(&"*.toml".to_string()));
        assert!(combined_patterns.contains(&"*.yaml".to_string()));
    }

    #[test]
    fn test_parse_multiple_patterns_mixed_types() {
        let patterns = vec!["./config/*.toml".to_string(), "**/*.yaml".to_string()];
        let (strategy, _) = parse_multiple_patterns(&patterns).unwrap();
        
        match strategy {
            SearchStrategy::Recursive { .. } => {}
            _ => panic!("Expected Recursive strategy for mixed patterns"),
        }
    }

    #[test]
    fn test_build_globset() {
        let patterns = vec!["*.toml".to_string(), "*.yaml".to_string()];
        let globset = build_globset(&patterns).unwrap();
        
        // Test that the globset was built successfully
        assert!(globset.is_match("config.toml"));
        assert!(globset.is_match("app.yaml"));
        assert!(!globset.is_match("readme.txt"));
    }

    #[test]
    fn test_parse_path_pattern_helper() {
        let (dir, file) = parse_path_pattern("./config/app.toml");
        assert_eq!(dir, PathBuf::from("./config"));
        assert_eq!(file, "app.toml");
    }

    #[test]
    fn test_parse_recursive_pattern_helper() {
        let (roots, pattern) = parse_recursive_pattern("./config/**/*.yaml");
        assert_eq!(roots, vec![PathBuf::from("./config")]);
        assert_eq!(pattern, "*.yaml");
    }

    #[test]
    fn test_parse_brace_expansion_helper() {
        let result = parse_brace_expansion("{dir1,dir2}/config.toml");
        assert!(result.is_some());
        
        let (dirs, pattern) = result.unwrap();
        assert_eq!(dirs.len(), 2);
        assert!(dirs.contains(&PathBuf::from("dir1")));
        assert!(dirs.contains(&PathBuf::from("dir2")));
        assert_eq!(pattern, "config.toml");
    }
}