//! # Verbosity System for Configuration Debugging
//!
//! This module provides a comprehensive verbosity system for debugging configuration loading
//! issues. Similar to CLI tools like `curl -v`, `git --verbose`, or `rsync -vvv`, SuperConfig
//! supports multiple verbosity levels to help troubleshoot configuration problems step-by-step.
//!
//! ## Why Verbosity Matters
//!
//! Configuration loading often fails silently or with cryptic error messages. Users need to
//! understand:
//! - Which configuration sources are being checked
//! - What values are being loaded from each source
//! - Where conflicts or overrides are happening
//! - Why certain values aren't being applied
//!
//! The verbosity system makes this transparent.
//!
//! ## Verbosity Levels
//!
//! SuperConfig provides 4 verbosity levels, following CLI tool conventions:
//!
//! - **Silent (0)**: No debug output - production mode
//! - **Info (1)**: Basic loading progress - shows major steps  
//! - **Debug (2)**: Detailed steps with success/failure indicators - troubleshooting mode
//! - **Trace (3)**: Full introspection with actual configuration values - deep debugging
//!
//! ## Basic Usage
//!
//! Enable verbosity during configuration building:
//!
//! ```rust,no_run
//! use superconfig::{SuperConfig, VerbosityLevel};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize, Serialize, Default)]
//! struct AppConfig {
//!     name: String,
//!     port: u16,
//! }
//!
//! let config = SuperConfig::new()
//!     .with_verbosity(VerbosityLevel::Debug)  // Enable debug output
//!     .with_file("config.toml")
//!     .with_env("APP_");
//!
//! let result: AppConfig = config.extract()?;
//! # Ok::<(), figment::Error>(())
//! ```
//!
//! ## CLI Integration Pattern
//!
//! Integrate with CLI argument parsing for user-friendly debugging:
//!
//! ```rust,no_run
//! use superconfig::{SuperConfig, VerbosityLevel};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize, Serialize, Default)]
//! struct AppConfig {
//!     database_url: String,
//!     log_level: String,
//! }
//!
//! // Simulate CLI argument parsing (normally from clap, structopt, etc.)
//! fn parse_cli_verbosity() -> u8 {
//!     let args: Vec<String> = std::env::args().collect();
//!     args.iter()
//!         .skip(1)
//!         .filter(|arg| arg.starts_with("-v"))
//!         .map(|arg| arg.matches('v').count() as u8)
//!         .max()
//!         .unwrap_or(0)
//! }
//!
//! let verbose_count = parse_cli_verbosity();
//! let verbosity = VerbosityLevel::from_cli_args(verbose_count);
//!
//! let config = SuperConfig::new()
//!     .with_verbosity(verbosity)
//!     .with_hierarchical_config("myapp")
//!     .with_env("APP_");
//!
//! let app_config: AppConfig = config.extract()?;
//! # Ok::<(), figment::Error>(())
//! ```
//!
//! ## Verbosity Level Examples
//!
//! ### Silent Mode (Production)
//! ```bash
//! myapp  # No verbosity flags, no debug output
//! ```
//!
//! ### Info Mode (-v)
//! ```bash
//! myapp -v
//! ```
//! **Output:**
//! ```text
//! CONFIG: Loading hierarchical config for: myapp
//! CONFIG: Loading environment variables with prefix: APP_
//! CONFIG: Extracting final configuration
//! CONFIG: Configuration extraction successful ✓
//! ```
//!
//! ### Debug Mode (-vv)  
//! ```bash
//! myapp -vv
//! ```
//! **Output:**
//! ```text
//! CONFIG: [1/4] Loading hierarchical config for: myapp
//! CONFIG: Checking hierarchical config paths:
//! CONFIG:   - /etc/myapp/config.toml ✗
//! CONFIG:   - ~/.config/myapp/config.toml ✓
//! CONFIG: [2/4] Loading environment variables with prefix: APP_
//! CONFIG: Found 3 environment variables ✓
//! CONFIG: [3/4] No CLI arguments provided ✗
//! CONFIG: [4/4] Extracting final configuration
//! CONFIG: Configuration extraction successful ✓
//! ```
//!
//! ### Trace Mode (-vvv)
//! ```bash
//! myapp -vvv
//! ```
//! **Output:**
//! ```text
//! CONFIG: [1/4] Loading hierarchical config for: myapp
//! CONFIG: Checking hierarchical config paths:
//! CONFIG:   - /etc/myapp/config.toml ✗
//! CONFIG:   - ~/.config/myapp/config.toml ✓
//! CONFIG:     database_url = "postgresql://localhost/dev"
//! CONFIG:     log_level = "debug"
//! CONFIG: [2/4] Loading environment variables with prefix: APP_
//! CONFIG: Found 3 environment variables ✓
//! CONFIG:     APP_DATABASE_URL = "postgresql://prod.db/myapp"
//! CONFIG:     APP_LOG_LEVEL = "info"
//! CONFIG:     APP_DEBUG_PASSWORD = ***MASKED***
//! CONFIG: [3/4] No CLI arguments provided ✗
//! CONFIG: [4/4] Extracting final configuration
//! CONFIG: Final merged configuration:
//! {
//!   "database_url": "postgresql://prod.db/myapp",
//!   "log_level": "info"
//! }
//! CONFIG: Configuration extraction successful ✓
//! ```
//!
//! ## Security Features
//!
//! The verbosity system automatically masks sensitive data in trace output.
//! Environment variables containing these keywords are masked:
//! - `password`
//! - `secret`
//! - `token`
//! - `key`
//!
//! ```rust,no_run
//! use superconfig::{SuperConfig, VerbosityLevel};
//!
//! // Environment variables with sensitive keywords are automatically masked
//! // Example: APP_DATABASE_PASSWORD=***MASKED*** in trace output
//! // Example: APP_API_TOKEN=***MASKED*** in trace output
//!
//! let config = SuperConfig::new()
//!     .with_verbosity(VerbosityLevel::Trace)  // Will mask sensitive values
//!     .with_env("APP_");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Collecting Debug Messages
//!
//! Access debug messages programmatically for custom logging or analysis:
//!
//! ```rust,no_run
//! use superconfig::{SuperConfig, VerbosityLevel};
//!
//! let config = SuperConfig::new()
//!     .with_verbosity(VerbosityLevel::Debug)
//!     .with_file("config.toml")
//!     .with_env("APP_");
//!
//! // Messages are collected regardless of display verbosity
//! let debug_messages = config.debug_messages();
//! println!("Collected {} debug messages", debug_messages.len());
//!
//! // Filter messages by level
//! let error_messages = config.debug_messages_at_level(VerbosityLevel::Info);
//! for msg in error_messages {
//!     println!("Info: {}", msg.message);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::time::Instant;

/// Verbosity levels for configuration debugging
///
/// Controls the amount of debug information displayed during configuration loading.
/// Higher levels include all information from lower levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum VerbosityLevel {
    /// No debug output (default behavior)
    #[default]
    Silent = 0,

    /// Basic configuration loading progress (-v)
    /// Shows which providers are being loaded and final success/failure
    Info = 1,

    /// Detailed step-by-step information (-vv)
    /// Shows file discovery, individual provider results, and merge operations
    Debug = 2,

    /// Full introspection with configuration values (-vvv)
    /// Shows actual configuration values at each step and final merged result
    Trace = 3,
}

impl VerbosityLevel {
    /// Create verbosity level from CLI argument count
    ///
    /// Typically used with clap's `action = ArgAction::Count` to convert
    /// the number of -v flags into appropriate verbosity level.
    ///
    /// # Examples
    /// ```rust
    /// use superconfig::VerbosityLevel;
    ///
    /// assert_eq!(VerbosityLevel::from_cli_args(0), VerbosityLevel::Silent);
    /// assert_eq!(VerbosityLevel::from_cli_args(1), VerbosityLevel::Info);
    /// assert_eq!(VerbosityLevel::from_cli_args(2), VerbosityLevel::Debug);
    /// assert_eq!(VerbosityLevel::from_cli_args(3), VerbosityLevel::Trace);
    /// assert_eq!(VerbosityLevel::from_cli_args(5), VerbosityLevel::Trace); // Caps at Trace
    /// ```
    pub fn from_cli_args(verbosity_count: u8) -> Self {
        match verbosity_count {
            0 => VerbosityLevel::Silent,
            1 => VerbosityLevel::Info,
            2 => VerbosityLevel::Debug,
            _ => VerbosityLevel::Trace, // 3+ all map to Trace
        }
    }

    /// Check if current level should display messages at the given level
    pub fn should_display(&self, message_level: VerbosityLevel) -> bool {
        *self >= message_level
    }

    /// Get a human-readable description of the verbosity level
    pub fn description(&self) -> &'static str {
        match self {
            VerbosityLevel::Silent => "Silent (no debug output)",
            VerbosityLevel::Info => "Info (basic loading progress)",
            VerbosityLevel::Debug => "Debug (detailed step-by-step)",
            VerbosityLevel::Trace => "Trace (full introspection with values)",
        }
    }
}

impl std::fmt::Display for VerbosityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerbosityLevel::Silent => write!(f, "silent"),
            VerbosityLevel::Info => write!(f, "info"),
            VerbosityLevel::Debug => write!(f, "debug"),
            VerbosityLevel::Trace => write!(f, "trace"),
        }
    }
}

/// A debug message captured during configuration loading
#[derive(Debug, Clone)]
pub struct DebugMessage {
    /// The verbosity level this message is intended for
    pub level: VerbosityLevel,
    /// The provider or component that generated this message
    pub provider: String,
    /// The debug message content
    pub message: String,
    /// When this message was created
    pub timestamp: Instant,
    /// Optional step number for ordered operations
    pub step: Option<usize>,
    /// Whether this represents a success (✓) or failure (✗)
    pub success: Option<bool>,
}

impl DebugMessage {
    /// Create a new debug message
    pub fn new(level: VerbosityLevel, provider: &str, message: &str) -> Self {
        Self {
            level,
            provider: provider.to_string(),
            message: message.to_string(),
            timestamp: Instant::now(),
            step: None,
            success: None,
        }
    }

    /// Create a debug message with step information
    pub fn with_step(mut self, step: usize) -> Self {
        self.step = Some(step);
        self
    }

    /// Create a debug message with success/failure indication
    pub fn with_success(mut self, success: bool) -> Self {
        self.success = Some(success);
        self
    }

    /// Format the message for display
    pub fn format(&self) -> String {
        let prefix = match self.level {
            VerbosityLevel::Silent => "",
            VerbosityLevel::Info => "CONFIG: ",
            VerbosityLevel::Debug => "CONFIG: ",
            VerbosityLevel::Trace => "CONFIG: ",
        };

        let step_prefix = if let Some(step) = self.step {
            format!("[{step}/] ")
        } else {
            String::new()
        };

        let success_suffix = match self.success {
            Some(true) => " ✓",
            Some(false) => " ✗",
            None => "",
        };

        format!(
            "{}{}{}{}",
            prefix, step_prefix, self.message, success_suffix
        )
    }
}

/// Helper trait for collecting debug messages during configuration operations
pub trait DebugCollector {
    /// Add a debug message
    fn debug(&self, level: VerbosityLevel, provider: &str, message: &str);

    /// Add a debug message with step information
    fn debug_step(&self, level: VerbosityLevel, provider: &str, step: usize, message: &str);

    /// Add a debug message with success/failure indication
    fn debug_result(&self, level: VerbosityLevel, provider: &str, message: &str, success: bool);

    /// Add a debug message with both step and success information
    fn debug_step_result(
        &self,
        level: VerbosityLevel,
        provider: &str,
        step: usize,
        message: &str,
        success: bool,
    );
}
