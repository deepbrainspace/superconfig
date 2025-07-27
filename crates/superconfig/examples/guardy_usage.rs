//! Example showing how to use SuperConfig verbosity in a CLI application like guardy
//!
//! Run with different verbosity levels:
//! cargo run --example guardy_usage           # Silent mode
//! cargo run --example guardy_usage -- -v     # Info mode  
//! cargo run --example guardy_usage -- -vv    # Debug mode
//! cargo run --example guardy_usage -- -vvv   # Trace mode

use serde::{Deserialize, Serialize};
use superconfig::{SuperConfig, VerbosityLevel};

#[derive(Debug, Deserialize, Serialize, Default)]
struct GuardyConfig {
    scanner: ScannerConfig,
    log: LogConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct ScannerConfig {
    mode: String,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            mode: "standard".to_string(),
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

#[derive(Debug, Deserialize, Serialize)]
struct CliOverrides {
    scanner: Option<CliScannerConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CliScannerConfig {
    mode: Option<String>,
}

const DEFAULT_CONFIG: &str = r#"
[scanner]
mode = "standard"

[log]
level = "info"
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate parsing CLI arguments for verbosity
    let args: Vec<String> = std::env::args().collect();
    let verbosity_count = args
        .iter()
        .skip(1)
        .filter(|arg| arg.starts_with("-v"))
        .map(|arg| arg.matches('v').count() as u8)
        .max()
        .unwrap_or(0);

    println!("=== Guardy Configuration Loading ===");
    println!(
        "Verbosity level: {}",
        VerbosityLevel::from_cli_args(verbosity_count)
    );
    println!();

    // CLI overrides (normally parsed by clap)
    // For this example, we'll pass None to avoid serialization issues
    let cli_overrides: Option<CliOverrides> = None;

    // This is how your guardy client would use SuperConfig with verbosity
    let config = SuperConfig::new()
        .with_verbosity(VerbosityLevel::from_cli_args(verbosity_count)) // Set based on -v, -vv, -vvv
        .with_defaults_string(DEFAULT_CONFIG) // 1. Defaults (lowest)
        .with_hierarchical_config("guardy") // 2. Hierarchical: system→user→project
        .with_env_ignore_empty("GUARDY_") // 3. Environment variables (with empty filtering)
        .with_cli_opt(cli_overrides); // 4. CLI (highest priority)

    // Extract the final configuration
    let final_config: GuardyConfig = config.extract()?;

    if verbosity_count == 0 {
        println!("Configuration loaded successfully!");
        println!("Scanner mode: {}", final_config.scanner.mode);
        println!("Log level: {}", final_config.log.level);
    }

    Ok(())
}
