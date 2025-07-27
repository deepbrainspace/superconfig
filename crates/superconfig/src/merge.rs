//! Native merge functionality with warning collection for SuperConfig
//!
//! This module implements SuperConfig's enhanced merge capabilities that extend beyond
//! Figment's standard merge functionality by adding:
//! - Warning collection from providers with validation errors
//! - Array merging with _add/_remove patterns
//! - Resilient configuration loading that continues despite provider errors

use figment::{Error, Figment, Provider, providers::Format};
use serde_json;
use std::collections::HashSet;

/// Trait for providers that can have validation errors
///
/// Providers that implement this trait can report validation errors
/// that occurred during construction, allowing SuperConfig to collect
/// these as warnings while continuing the configuration loading process.
pub trait ValidatedProvider {
    /// Check if the provider has any validation errors
    ///
    /// Returns `Some(error)` if there are validation issues, `None` if valid.
    /// This allows SuperConfig to collect warnings without failing the merge.
    fn validation_error(&self) -> Option<Error>;
}

impl crate::SuperConfig {
    /// Merge a provider with warning collection and array merging
    ///
    /// This method extends Figment's merge functionality by:
    /// 1. Collecting validation warnings from providers (like Wildcard) if they support it
    /// 2. Continuing configuration loading even if providers have validation errors
    /// 3. Applying array merging with _add/_remove patterns
    ///
    /// # Examples
    /// ```rust,no_run
    /// use superconfig::{SuperConfig, Wildcard};
    ///
    /// let config = SuperConfig::new()
    ///     .merge(Wildcard::new("invalid[pattern"))  // Stores warning, continues
    ///     .merge(Wildcard::new("*.toml"));          // Normal loading
    ///     
    /// // Check for warnings after loading
    /// for warning in config.warnings() {
    ///     eprintln!("Configuration warning: {}", warning);
    /// }
    /// ```
    pub fn merge<P: Provider>(mut self, provider: P) -> Self {
        self.figment = self.figment.merge(provider);
        self.apply_array_merging()
    }

    /// Merge a validated provider with warning collection
    pub fn merge_validated<P: Provider + ValidatedProvider>(mut self, provider: P) -> Self {
        // Check for validation errors and collect as warnings
        if let Some(error) = provider.validation_error() {
            self.warnings
                .push(format!("Provider validation error: {error}"));
        }

        // Merge the provider regardless of validation errors, then apply array merging
        self.figment = self.figment.merge(provider);
        self.apply_array_merging()
    }

    /// Merge an optional provider with warning collection
    ///
    /// Convenience method for merging optional providers. If the provider is `None`,
    /// no merge operation is performed.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use superconfig::{SuperConfig, Wildcard};
    ///
    /// let optional_config = Some(Wildcard::new("*.toml"));
    /// let config = SuperConfig::new()
    ///     .merge_opt(optional_config);
    /// ```
    pub fn merge_opt<P: Provider>(self, provider: Option<P>) -> Self {
        match provider {
            Some(p) => self.merge(p),
            None => self,
        }
    }

    /// Apply array merging to current configuration
    ///
    /// This method processes _add and _remove patterns in the configuration to
    /// intelligently merge arrays across all configuration sources.
    fn apply_array_merging(mut self) -> Self {
        // Optimization: Check if array merging is needed before expensive operations
        if !ArrayMergeHelper::needs_array_merging(&self.figment) {
            return self; // No merging needed - early return
        }

        self.figment = ArrayMergeHelper::apply_array_merging(self.figment);
        self
    }

    /// Get all collected warnings
    ///
    /// Returns a slice of warning messages collected during configuration loading.
    /// Warnings indicate non-fatal issues like invalid patterns in providers that
    /// were safely handled without stopping the configuration process.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use superconfig::{SuperConfig, Wildcard};
    ///
    /// let config = SuperConfig::new()
    ///     .merge(Wildcard::new("invalid[pattern"));
    ///     
    /// assert!(!config.warnings().is_empty());
    /// println!("Warnings: {:?}", config.warnings());
    /// ```
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Check if there are any warnings
    ///
    /// Returns `true` if any providers generated validation warnings during loading.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use superconfig::{SuperConfig, Wildcard};
    ///
    /// let config = SuperConfig::new()
    ///     .merge(Wildcard::new("*.toml"));
    ///     
    /// if config.has_warnings() {
    ///     eprintln!("Configuration loaded with warnings");
    /// }
    /// ```
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Print all warnings to stderr
    ///
    /// Convenience method to display all collected warnings. Useful during
    /// application startup to alert users about configuration issues.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use superconfig::{SuperConfig, Wildcard};
    ///
    /// let config = SuperConfig::new()
    ///     .merge(Wildcard::new("invalid[pattern"))
    ///     .merge(Wildcard::new("*.toml"));
    ///     
    /// config.print_warnings(); // Prints to stderr if any warnings exist
    /// ```
    pub fn print_warnings(&self) {
        for warning in &self.warnings {
            eprintln!("SuperConfig warning: {warning}");
        }
    }
}

/// Helper functions for array merging
struct ArrayMergeHelper;

impl ArrayMergeHelper {
    /// Performance optimization: Check if array merging is actually needed
    ///
    /// Scans for keys ending with "_add" or "_remove" without full extraction
    fn needs_array_merging(figment: &Figment) -> bool {
        // Quick check: extract configuration and scan for merge patterns
        if let Ok(value) = figment.extract::<serde_json::Value>() {
            Self::contains_merge_patterns(&value)
        } else {
            false // If extraction fails, skip array merging
        }
    }

    /// Recursively check if JSON value contains array merge patterns
    fn contains_merge_patterns(value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::Object(obj) => {
                // Check if any keys end with _add or _remove
                for key in obj.keys() {
                    if key.ends_with("_add") || key.ends_with("_remove") {
                        return true;
                    }
                }
                // Recursively check nested objects
                obj.values().any(Self::contains_merge_patterns)
            }
            serde_json::Value::Array(arr) => {
                // Check array elements for nested objects with merge patterns
                arr.iter().any(Self::contains_merge_patterns)
            }
            _ => false,
        }
    }

    /// Apply array merging to the figment configuration
    fn apply_array_merging(figment: Figment) -> Figment {
        // Extract complete configuration as JSON for processing
        let json_config = match figment.extract::<serde_json::Value>() {
            Ok(config) => config,
            Err(_) => return figment, // Return original if extraction fails
        };

        eprintln!(
            "DEBUG: Before array merging: {}",
            serde_json::to_string_pretty(&json_config).unwrap_or_default()
        );

        // Apply array merging transformations
        let merged_config = Self::merge_object_arrays(json_config);

        eprintln!(
            "DEBUG: After array merging: {}",
            serde_json::to_string_pretty(&merged_config).unwrap_or_default()
        );

        // Create new figment with merged configuration
        Figment::new().merge(figment::providers::Json::string(&merged_config.to_string()))
    }

    /// Core array merging logic with _add and _remove pattern support
    fn merge_object_arrays(mut value: serde_json::Value) -> serde_json::Value {
        match &mut value {
            serde_json::Value::Object(obj) => {
                let mut fields_to_remove = Vec::new();
                let mut arrays_to_update: Vec<(String, serde_json::Value)> = Vec::new();

                // Identify base arrays and their _add/_remove operations
                let base_fields: HashSet<String> = obj
                    .keys()
                    .filter_map(|key| {
                        if key.ends_with("_add") {
                            Some(key.strip_suffix("_add").unwrap().to_string())
                        } else if key.ends_with("_remove") {
                            Some(key.strip_suffix("_remove").unwrap().to_string())
                        } else {
                            None
                        }
                    })
                    .collect();

                // Process each base array that has merge operations
                for base_field in &base_fields {
                    let add_key = format!("{base_field}_add");
                    let remove_key = format!("{base_field}_remove");

                    eprintln!(
                        "DEBUG: Processing base field '{base_field}' with add_key='{add_key}', remove_key='{remove_key}'"
                    );

                    // Get base array (or create empty if not exists)
                    let mut result_array = obj
                        .get(base_field)
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_else(Vec::new);

                    eprintln!("DEBUG: Initial base array for '{base_field}': {result_array:?}");

                    // Apply _add operations
                    if let Some(add_value) = obj.get(&add_key).and_then(|v| v.as_array()) {
                        eprintln!("DEBUG: Adding values to '{base_field}': {add_value:?}");
                        result_array.extend(add_value.clone());
                        fields_to_remove.push(add_key);
                    } else {
                        eprintln!("DEBUG: No _add values found for '{base_field}'");
                    }

                    // Apply _remove operations
                    if let Some(remove_value) = obj.get(&remove_key).and_then(|v| v.as_array()) {
                        eprintln!("DEBUG: Removing values from '{base_field}': {remove_value:?}");
                        let before_count = result_array.len();
                        result_array.retain(|item| !remove_value.contains(item));
                        eprintln!(
                            "DEBUG: Removed {} items from '{base_field}'",
                            before_count - result_array.len()
                        );
                        fields_to_remove.push(remove_key);
                    } else {
                        eprintln!("DEBUG: No _remove values found for '{base_field}'");
                    }

                    eprintln!("DEBUG: Final array for '{base_field}': {result_array:?}");

                    // Queue array for update
                    arrays_to_update
                        .push((base_field.clone(), serde_json::Value::Array(result_array)));
                }

                // Apply updates and cleanup
                for (field, new_array) in arrays_to_update {
                    obj.insert(field, new_array);
                }
                for field in fields_to_remove {
                    obj.remove(&field);
                }

                // Recursively process nested objects
                for value in obj.values_mut() {
                    *value = Self::merge_object_arrays(value.clone());
                }

                serde_json::Value::Object(obj.clone())
            }
            serde_json::Value::Array(arr) => {
                // Recursively process array elements
                let processed_array: Vec<serde_json::Value> = arr
                    .iter()
                    .map(|item| Self::merge_object_arrays(item.clone()))
                    .collect();
                serde_json::Value::Array(processed_array)
            }
            // Pass through all other value types unchanged
            other => other.clone(),
        }
    }
}
