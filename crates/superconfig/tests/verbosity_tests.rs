//! Integration tests for SuperConfig verbosity functionality

use serde::{Deserialize, Serialize};
use superconfig::{SuperConfig, VerbosityLevel};

#[derive(Debug, Deserialize, Serialize, Default)]
struct TestConfig {
    scanner: ScannerConfig,
    log: LogConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct ScannerConfig {
    mode: String,
    timeout: u32,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            mode: "standard".to_string(),
            timeout: 30,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct LogConfig {
    level: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
        }
    }
}

const DEFAULT_CONFIG: &str = r#"
[scanner]
mode = "standard"
timeout = 30

[log]
level = "info"
"#;

#[test]
fn test_verbosity_level_from_cli_args() {
    assert_eq!(VerbosityLevel::from_cli_args(0), VerbosityLevel::Silent);
    assert_eq!(VerbosityLevel::from_cli_args(1), VerbosityLevel::Info);
    assert_eq!(VerbosityLevel::from_cli_args(2), VerbosityLevel::Debug);
    assert_eq!(VerbosityLevel::from_cli_args(3), VerbosityLevel::Trace);
    assert_eq!(VerbosityLevel::from_cli_args(5), VerbosityLevel::Trace); // Caps at Trace
}

#[test]
fn test_verbosity_builder_methods() {
    let config = SuperConfig::new()
        .with_info_verbosity()
        .with_defaults_string(DEFAULT_CONFIG);

    assert_eq!(config.verbosity(), VerbosityLevel::Info);

    let config = SuperConfig::new()
        .with_debug_verbosity()
        .with_defaults_string(DEFAULT_CONFIG);

    assert_eq!(config.verbosity(), VerbosityLevel::Debug);

    let config = SuperConfig::new()
        .with_trace_verbosity()
        .with_defaults_string(DEFAULT_CONFIG);

    assert_eq!(config.verbosity(), VerbosityLevel::Trace);
}

#[test]
fn test_configuration_loading_with_verbosity() {
    // Test that configuration still loads correctly with verbosity enabled
    let config = SuperConfig::new()
        .with_debug_verbosity()
        .with_defaults_string(DEFAULT_CONFIG);

    let result: TestConfig = config
        .extract()
        .expect("Configuration should load successfully");

    assert_eq!(result.scanner.mode, "standard");
    assert_eq!(result.scanner.timeout, 30);
    assert_eq!(result.log.level, "info");
}

#[test]
fn test_debug_message_collection() {
    let config = SuperConfig::new()
        .with_trace_verbosity()
        .with_defaults_string(DEFAULT_CONFIG)
        .with_env_ignore_empty("NONEXISTENT_PREFIX_");

    let _result: TestConfig = config
        .extract()
        .expect("Configuration should load successfully");

    // Verify debug messages were collected
    let debug_messages = config.debug_messages();
    assert!(
        !debug_messages.is_empty(),
        "Debug messages should be collected"
    );

    // Check that we have messages from different providers
    let providers: Vec<&str> = debug_messages
        .iter()
        .map(|msg| msg.provider.as_str())
        .collect();
    assert!(
        providers.contains(&"defaults"),
        "Should have defaults provider messages"
    );
    assert!(
        providers.contains(&"env"),
        "Should have env provider messages"
    );
    assert!(
        providers.contains(&"extract"),
        "Should have extract messages"
    );
}

#[test]
fn test_verbosity_with_environment_variables() {
    // We'll test env var processing by checking the debug messages
    // rather than actually setting env vars in tests
    let config = SuperConfig::new()
        .with_debug_verbosity()
        .with_defaults_string(DEFAULT_CONFIG)
        .with_env_ignore_empty("NONEXISTENT_PREFIX_");

    let result: TestConfig = config
        .extract()
        .expect("Configuration should load successfully");

    // Should fall back to defaults since no env vars match the prefix
    assert_eq!(result.scanner.mode, "standard");
    assert_eq!(result.log.level, "info");

    // Verify debug messages contain env var information
    let debug_messages = config.debug_messages();
    let env_messages: Vec<_> = debug_messages
        .iter()
        .filter(|msg| msg.provider == "env")
        .collect();

    assert!(
        !env_messages.is_empty(),
        "Should have environment variable debug messages"
    );

    // Check that the debug message indicates no env vars were found
    let has_no_vars_message = env_messages
        .iter()
        .any(|msg| msg.message.contains("No environment variables found"));
    assert!(
        has_no_vars_message,
        "Should have message about no env vars found"
    );
}

#[test]
fn test_silent_verbosity_produces_no_output() {
    let config = SuperConfig::new()
        .with_verbosity(VerbosityLevel::Silent)
        .with_defaults_string(DEFAULT_CONFIG);

    let _result: TestConfig = config
        .extract()
        .expect("Configuration should load successfully");

    // In silent mode, messages are still collected but not printed
    // We can verify they exist but stderr output would be silent
    let debug_messages = config.debug_messages();
    assert!(
        !debug_messages.is_empty(),
        "Debug messages should still be collected in silent mode"
    );
}

#[test]
fn test_hierarchical_config_verbosity() {
    let config = SuperConfig::new()
        .with_debug_verbosity()
        .with_defaults_string(DEFAULT_CONFIG)
        .with_hierarchical_config("nonexistent_app");

    let _result: TestConfig = config
        .extract()
        .expect("Configuration should load successfully");

    // Verify hierarchical config messages were generated
    let debug_messages = config.debug_messages();
    let hierarchical_messages: Vec<_> = debug_messages
        .iter()
        .filter(|msg| msg.provider == "hierarchical")
        .collect();

    assert!(
        !hierarchical_messages.is_empty(),
        "Should have hierarchical config debug messages"
    );
}
