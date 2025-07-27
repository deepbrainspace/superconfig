//! Integration tests for SuperConfig functionality
//!
//! These tests validate the complete SuperConfig feature set including:
//! - SuperConfig builder patterns
//! - Extension trait functionality  
//! - Provider integration
//! - Array merging capabilities
//! - Format detection and caching

use figment::Figment;
use serde::{Deserialize, Serialize};
use serial_test::serial;
use std::env;
use std::fs;
use superconfig::{SuperConfig, Wildcard};
use tempfile::TempDir;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct TestConfig {
    #[serde(default)]
    host: String,
    #[serde(default)]
    port: u16,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    database: DatabaseConfig,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct DatabaseConfig {
    #[serde(default)]
    url: String,
    #[serde(default)]
    timeout: u32,
    #[serde(default)]
    allowed_origins: Vec<String>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            features: vec![],
            database: DatabaseConfig::default(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://localhost".to_string(),
            timeout: 30,
            allowed_origins: vec![],
        }
    }
}

#[test]
fn test_superconfig_builder_basic() {
    let config = SuperConfig::new().with_defaults(TestConfig::default());

    let result: TestConfig = config.extract().expect("Failed to extract config");
    assert_eq!(result.host, "localhost");
    assert_eq!(result.port, 8080);
}

#[test]
fn test_superconfig_deref_compatibility() {
    let super_config = SuperConfig::new().with_defaults(TestConfig::default());

    // Test that SuperConfig can be used as a Figment via Deref
    let _figment_ref: &Figment = &super_config;

    // Test method calls that should work via Deref
    let profiles: Vec<_> = super_config.profiles().collect();
    assert!(!profiles.is_empty());
}

#[test]
fn test_superconfig_native_methods() -> Result<(), Box<dyn std::error::Error>> {
    // Create temp files for testing
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("config.json");

    fs::write(
        &config_path,
        r#"{"host": "example.com", "features": ["auth"]}"#,
    )?;

    // Test SuperConfig native methods
    let config = SuperConfig::new().with_file(&config_path);

    let extracted: TestConfig = config.extract()?;
    assert_eq!(extracted.host, "example.com");

    // Test built-in access methods
    let json_output = config.as_json()?;
    assert!(json_output.contains("example.com"));

    let yaml_output = config.as_yaml()?;
    assert!(yaml_output.contains("example.com"));

    // Test convenience methods
    let host = config.get_string("host")?;
    assert_eq!(host, "example.com");

    let features = config.get_array::<String>("features")?;
    assert_eq!(features, vec!["auth"]);

    assert!(config.has_key("host")?);
    assert!(!config.has_key("nonexistent")?);

    Ok(())
}

#[test]
fn test_superconfig_with_defaults_and_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("config.toml");

    fs::write(
        &config_path,
        r#"
host = "toml.example.com"
port = 9090

[database]
url = "mysql://localhost"
timeout = 60
"#,
    )?;

    // Test SuperConfig with defaults and file loading
    let config = SuperConfig::new()
        .with_defaults(TestConfig::default())
        .with_file(&config_path);

    let extracted: TestConfig = config.extract()?;
    assert_eq!(extracted.host, "toml.example.com");
    assert_eq!(extracted.port, 9090);
    assert_eq!(extracted.database.url, "mysql://localhost");

    // Test access methods
    let host = config.get_string("host")?;
    assert_eq!(host, "toml.example.com");

    let has_database = config.has_key("database")?;
    assert!(has_database);

    let keys = config.keys()?;
    assert!(keys.contains(&"host".to_string()));
    assert!(keys.contains(&"database".to_string()));

    Ok(())
}

#[test]
fn test_array_merging_functionality() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    // Create base config
    let base_config = temp_dir.path().join("base.json");
    fs::write(
        &base_config,
        r#"
{
    "features": ["auth", "logging"],
    "database": {
        "allowed_origins": ["https://app.com", "https://admin.com"]
    }
}
"#,
    )?;

    // Create override config with array operations
    let override_config = temp_dir.path().join("override.json");
    fs::write(
        &override_config,
        r#"
{
    "features_add": ["metrics", "tracing"],
    "features_remove": ["logging"],
    "database": {
        "allowed_origins_add": ["https://api.com"],
        "allowed_origins_remove": ["https://admin.com"]
    }
}
"#,
    )?;

    let config = SuperConfig::new()
        .with_file(&base_config)
        .with_file(&override_config);

    let result: TestConfig = config.extract()?;

    // Verify array merging
    assert!(result.features.contains(&"auth".to_string()));
    assert!(result.features.contains(&"metrics".to_string()));
    assert!(result.features.contains(&"tracing".to_string()));
    assert!(!result.features.contains(&"logging".to_string())); // Should be removed

    assert!(
        result
            .database
            .allowed_origins
            .contains(&"https://app.com".to_string())
    );
    assert!(
        result
            .database
            .allowed_origins
            .contains(&"https://api.com".to_string())
    );
    assert!(
        !result
            .database
            .allowed_origins
            .contains(&"https://admin.com".to_string())
    ); // Should be removed

    Ok(())
}

#[test]
#[serial] // Prevent concurrent env var modification
fn test_nested_environment_variables() -> Result<(), Box<dyn std::error::Error>> {
    // SAFETY: Environment variable modification is unsafe in Rust 2024 due to potential
    // race conditions in multi-threaded environments. However, this is the standard
    // approach for testing environment variable functionality. We mitigate the risk by:
    // 1. Using #[serial] to ensure tests run sequentially, not concurrently
    // 2. Using test-specific prefixes (TEST_*) to avoid conflicts with real env vars
    // 3. Cleaning up all variables at the end of the test
    // 4. This is isolated test code, not production code
    unsafe {
        std::env::set_var("TEST_HOST", "env.example.com");
        std::env::set_var("TEST_PORT", "3000");
        std::env::set_var("TEST_FEATURES", r#"["cache", "redis"]"#);
        std::env::set_var("TEST_DATABASE_URL", "redis://localhost");
        std::env::set_var("TEST_DATABASE_TIMEOUT", "120");
    }

    let config = SuperConfig::new()
        .with_defaults(TestConfig::default())
        .with_env("TEST_");

    let result: TestConfig = config.extract()?;

    assert_eq!(result.host, "env.example.com");
    assert_eq!(result.port, 3000);
    assert_eq!(result.features, vec!["cache", "redis"]);
    assert_eq!(result.database.url, "redis://localhost");
    assert_eq!(result.database.timeout, 120);

    // Clean up test environment variables
    unsafe {
        std::env::remove_var("TEST_HOST");
        std::env::remove_var("TEST_PORT");
        std::env::remove_var("TEST_FEATURES");
        std::env::remove_var("TEST_DATABASE_URL");
        std::env::remove_var("TEST_DATABASE_TIMEOUT");
    }

    Ok(())
}

#[test]
fn test_format_detection() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    // Test JSON detection
    let json_file = temp_dir.path().join("config.json");
    fs::write(&json_file, r#"{"host": "json.example.com"}"#)?;

    // Test TOML detection
    let toml_file = temp_dir.path().join("config.toml");
    fs::write(&toml_file, r#"host = "toml.example.com""#)?;

    // Test YAML detection
    let yaml_file = temp_dir.path().join("config.yaml");
    fs::write(&yaml_file, r#"host: yaml.example.com"#)?;

    // Test unknown extension (should try all formats)
    let unknown_file = temp_dir.path().join("config.cfg");
    fs::write(&unknown_file, r#"host = "cfg.example.com""#)?; // TOML content

    let json_config: TestConfig = SuperConfig::new().with_file(&json_file).extract()?;
    assert_eq!(json_config.host, "json.example.com");

    let toml_config: TestConfig = SuperConfig::new().with_file(&toml_file).extract()?;
    assert_eq!(toml_config.host, "toml.example.com");

    let yaml_config: TestConfig = SuperConfig::new().with_file(&yaml_file).extract()?;
    assert_eq!(yaml_config.host, "yaml.example.com");

    let unknown_config: TestConfig = SuperConfig::new().with_file(&unknown_file).extract()?;
    assert_eq!(unknown_config.host, "cfg.example.com");

    Ok(())
}

#[test]
fn test_empty_value_filtering() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct FilterConfig {
        #[serde(default)]
        enabled: bool,
        #[serde(default)]
        name: String,
        #[serde(default)]
        count: u32,
        #[serde(default)]
        items: Vec<String>,
    }

    let temp_dir = TempDir::new()?;
    let base_config = temp_dir.path().join("base.json");
    fs::write(
        &base_config,
        r#"
{
    "enabled": true,
    "name": "production",
    "count": 42,
    "items": ["item1", "item2"]
}
"#,
    )?;

    // Create CLI-like provider with empty values that should be filtered
    let cli_values = FilterConfig {
        enabled: false,       // meaningful falsy value - should be preserved
        name: "".to_string(), // empty string - should be filtered
        count: 0,             // meaningful zero - should be preserved
        items: Vec::new(),    // empty array - should be filtered
    };

    let config = SuperConfig::new()
        .with_file(&base_config)
        .with_cli_opt(Some(cli_values));

    let result: FilterConfig = config.extract()?;

    // enabled should be overridden to false (meaningful falsy value preserved)
    assert!(!result.enabled);

    // name should keep base config value (empty string filtered out)
    assert_eq!(result.name, "production");

    // count should be overridden to 0 (meaningful zero preserved)
    assert_eq!(result.count, 0);

    // items should keep base config value (empty array filtered out)
    assert_eq!(result.items, vec!["item1", "item2"]);

    Ok(())
}

#[test]
#[serial] // Prevent concurrent env var modification
fn test_hierarchical_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    // Create system-level config
    let system_dir = temp_dir.path().join(".config").join("testapp");
    fs::create_dir_all(&system_dir)?;
    let system_config = system_dir.join("testapp.toml");
    fs::write(
        &system_config,
        r#"
host = "system.example.com"
port = 8080
[database]
timeout = 30
allowed_origins = ["https://system.com"]
"#,
    )?;

    // Create user-level config
    let user_dir = temp_dir.path().join(".testapp");
    fs::create_dir_all(&user_dir)?;
    let user_config = user_dir.join("testapp.toml");
    fs::write(
        &user_config,
        r#"
port = 9090
[database] 
allowed_origins_add = ["https://user.com"]
"#,
    )?;

    // Create project-level config
    let project_config = temp_dir.path().join("testapp.toml");
    fs::write(
        &project_config,
        r#"
host = "project.example.com"
[database]
allowed_origins_remove = ["https://system.com"]
allowed_origins_add = ["https://project.com"]
"#,
    )?;

    // Update HOME to point to our temp directory for testing
    let original_home = env::var("HOME").unwrap_or_default();
    // SAFETY: Temporarily modifying HOME env var for hierarchical config testing
    // This is safe because we restore it immediately after the test
    unsafe {
        std::env::set_var("HOME", temp_dir.path());
    }

    // Debug: List what files we actually created
    println!("Files created:");
    println!(
        "System: {:?} - exists: {}",
        system_config,
        system_config.exists()
    );
    println!("User: {:?} - exists: {}", user_config, user_config.exists());
    println!(
        "Project: {:?} - exists: {}",
        project_config,
        project_config.exists()
    );

    // Change to temp directory to simulate project directory
    let original_dir = env::current_dir()?;
    env::set_current_dir(temp_dir.path())?;

    println!("Current dir: {:?}", env::current_dir()?);
    println!("HOME set to: {:?}", env::var("HOME"));

    let wildcard_provider = Wildcard::hierarchical("config", "testapp");
    let discovered_files = wildcard_provider.discover_files();
    println!("Discovered files in merge order: {discovered_files:?}");
    println!("Merge order strategy: {:?}", wildcard_provider.merge_order());
    
    let config = SuperConfig::new()
        .with_defaults(TestConfig::default()) // Add defaults first
        .with_hierarchical_config("testapp");
    
    println!("Config warnings: {:?}", config.warnings());

    // Note: Hierarchical configuration currently has profile handling issues
    // The provider correctly loads files but data structure needs architectural fixes
    // For now, test basic functionality without complex array merging

    let result: TestConfig = config.extract()?;

    // Debug: Print the actual result
    println!("Final result: {result:?}");

    // Test that hierarchical merging works correctly (system -> user -> project priority)
    assert_eq!(result.host, "project.example.com"); // Project config overrides system
    assert_eq!(result.port, 9090); // User config overrides system (no project override)
    assert_eq!(result.database.timeout, 30); // From system config (no overrides)

    // Test complex array merging across hierarchy (system -> user -> project)
    let origins = &result.database.allowed_origins;
    assert!(origins.contains(&"https://user.com".to_string())); // Added by user config
    assert!(origins.contains(&"https://project.com".to_string())); // Added by project config  
    assert!(!origins.contains(&"https://system.com".to_string())); // Removed by project config

    // Restore environment
    // SAFETY: Restoring the original HOME environment variable
    unsafe {
        if original_home.is_empty() {
            std::env::remove_var("HOME");
        } else {
            std::env::set_var("HOME", original_home);
        }
    }
    env::set_current_dir(original_dir)?;

    Ok(())
}

#[test]
#[serial] // Prevent concurrent env var modification
fn test_sequential_array_merging_order() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    // Create system-level config
    let system_dir = temp_dir.path().join(".config").join("testapp");
    fs::create_dir_all(&system_dir)?;
    let system_config = system_dir.join("testapp.toml");
    fs::write(
        &system_config,
        r#"
[database]
allowed_origins = ["https://system.com", "https://common.com"]
"#,
    )?;

    // Create user-level config that adds first, then a subsequent file removes
    let user_dir = temp_dir.path().join(".testapp");
    fs::create_dir_all(&user_dir)?;
    let user_config = user_dir.join("testapp.toml");
    fs::write(
        &user_config,
        r#"
[database] 
allowed_origins_add = ["https://user-added.com"]
"#,
    )?;

    // Create project-level config that removes what was added, then adds something new
    let project_config = temp_dir.path().join("testapp.toml");
    fs::write(
        &project_config,
        r#"
[database]
allowed_origins_remove = ["https://system.com"]
allowed_origins_add = ["https://project-final.com"]
"#,
    )?;

    // Update HOME to point to our temp directory for testing
    let original_home = env::var("HOME").unwrap_or_default();
    unsafe {
        std::env::set_var("HOME", temp_dir.path());
    }

    // Change to temp directory to simulate project directory
    let original_dir = env::current_dir()?;
    env::set_current_dir(temp_dir.path())?;

    let config = SuperConfig::new()
        .with_defaults(TestConfig::default())
        .with_hierarchical_config("testapp");

    let result: TestConfig = config.extract()?;

    // Expected sequence:
    // 1. Start: ["https://system.com", "https://common.com"] 
    // 2. After user: ["https://system.com", "https://common.com", "https://user-added.com"]
    // 3. After project remove: ["https://common.com", "https://user-added.com"] 
    // 4. After project add: ["https://common.com", "https://user-added.com", "https://project-final.com"]

    let origins = &result.database.allowed_origins;
    println!("Final allowed_origins: {origins:?}");
    
    // Verify the sequential processing worked correctly
    assert!(origins.contains(&"https://common.com".to_string()), "Should preserve common.com");
    assert!(origins.contains(&"https://user-added.com".to_string()), "Should preserve user addition");
    assert!(origins.contains(&"https://project-final.com".to_string()), "Should include project addition");
    assert!(!origins.contains(&"https://system.com".to_string()), "Should have removed system.com");
    
    // Verify expected final state
    assert_eq!(origins.len(), 3, "Should have exactly 3 origins");

    // Restore environment
    unsafe {
        if original_home.is_empty() {
            std::env::remove_var("HOME");
        } else {
            std::env::set_var("HOME", original_home);
        }
    }
    env::set_current_dir(original_dir)?;

    Ok(())
}

#[test]
#[serial] // Prevent concurrent env var modification  
fn test_complex_sequential_array_operations() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    // Create system-level config
    let system_dir = temp_dir.path().join(".config").join("testapp");
    fs::create_dir_all(&system_dir)?;
    let system_config = system_dir.join("testapp.toml");
    fs::write(
        &system_config,
        r#"
[database]
allowed_origins = ["A", "B", "C"]
"#,
    )?;

    // User config: remove B, add D
    let user_dir = temp_dir.path().join(".testapp");
    fs::create_dir_all(&user_dir)?;
    let user_config = user_dir.join("testapp.toml");
    fs::write(
        &user_config,
        r#"
[database] 
allowed_origins_remove = ["B"]
allowed_origins_add = ["D"]
"#,
    )?;

    // Project config: remove D (what user just added), add E  
    let project_config = temp_dir.path().join("testapp.toml");
    fs::write(
        &project_config,
        r#"
[database]
allowed_origins_remove = ["D"]
allowed_origins_add = ["E"]
"#,
    )?;

    // Update HOME to point to our temp directory for testing
    let original_home = env::var("HOME").unwrap_or_default();
    unsafe {
        std::env::set_var("HOME", temp_dir.path());
    }

    // Change to temp directory to simulate project directory
    let original_dir = env::current_dir()?;
    env::set_current_dir(temp_dir.path())?;

    let config = SuperConfig::new()
        .with_defaults(TestConfig::default())
        .with_hierarchical_config("testapp");

    let result: TestConfig = config.extract()?;

    // Expected sequence:
    // 1. System: ["A", "B", "C"] 
    // 2. User: Remove B, add D → ["A", "C", "D"]
    // 3. Project: Remove D, add E → ["A", "C", "E"]

    let origins = &result.database.allowed_origins;
    println!("Complex sequence final allowed_origins: {origins:?}");
    
    // With correct sequential processing:
    assert!(origins.contains(&"A".to_string()), "Should preserve A");
    assert!(origins.contains(&"C".to_string()), "Should preserve C");
    assert!(origins.contains(&"E".to_string()), "Should include E (project addition)");
    assert!(!origins.contains(&"B".to_string()), "Should have removed B (user removal)");
    assert!(!origins.contains(&"D".to_string()), "Should have removed D (project removal after user addition)");
    
    // Should have exactly A, C, E
    assert_eq!(origins.len(), 3, "Should have exactly 3 origins");

    // Restore environment
    unsafe {
        if original_home.is_empty() {
            std::env::remove_var("HOME");
        } else {
            std::env::set_var("HOME", original_home);
        }
    }
    env::set_current_dir(original_dir)?;

    Ok(())
}

#[test]
fn test_conversion_methods() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("config.json");

    fs::write(
        &config_path,
        r#"
{
    "host": "conversion.example.com",
    "port": 8080,
    "database": {
        "url": "postgres://localhost",
        "timeout": 60
    }
}
"#,
    )?;

    let figment = SuperConfig::new().with_file(&config_path);

    // Test format conversions
    let json_str = figment.as_json()?;
    assert!(json_str.contains("conversion.example.com"));
    assert!(json_str.contains("8080"));

    let yaml_str = figment.as_yaml()?;
    assert!(yaml_str.contains("conversion.example.com"));

    let toml_str = figment.as_toml()?;
    assert!(toml_str.contains("conversion.example.com"));

    // Test debug output
    let debug_str = figment.debug_config()?;
    assert!(debug_str.contains("SuperConfig Debug"));
    assert!(debug_str.contains("conversion.example.com"));

    let sources = figment.debug_sources();
    assert!(!sources.is_empty());

    Ok(())
}
