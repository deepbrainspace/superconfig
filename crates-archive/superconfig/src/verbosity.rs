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
//! use superconfig::{SuperConfig, verbosity};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize, Serialize, Default)]
//! struct AppConfig {
//!     name: String,
//!     port: u16,
//! }
//!
//! let config = SuperConfig::new()
//!     .with_verbosity(verbosity::DEBUG)  // Enable debug output
//!     .with_file("config.toml")
//!     .with_env("APP_");
//!
//! let result: AppConfig = config.extract()?;
//! # Ok::<(), figment::Error>(())
//! ```

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// No debug output (default behavior)
pub const SILENT: u8 = 0;

/// Basic configuration loading progress (-v)
/// Shows which providers are being loaded and final success/failure
pub const INFO: u8 = 1;

/// Detailed step-by-step information (-vv)
/// Shows file discovery, individual provider results, and merge operations
pub const DEBUG: u8 = 2;

/// Full introspection with configuration values (-vvv)
/// Shows actual configuration values at each step and final merged result
pub const TRACE: u8 = 3;

/// Type alias for verbosity level - now using u8 for FFI compatibility
pub type VerbosityLevel = u8;

/// Debug message structure for collecting configuration loading information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugMessage {
    /// Verbosity level of this message
    pub level: u8,
    /// Provider or component that generated this message
    pub provider: String,
    /// The actual debug message
    pub message: String,
    /// Timestamp when message was created (not serialized for FFI compatibility)
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
    /// Optional step number for ordered operations
    pub step: Option<usize>,
    /// Optional success/failure indication
    pub success: Option<bool>,
}

impl DebugMessage {
    /// Create a new debug message
    pub fn new(level: u8, provider: &str, message: &str) -> Self {
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
            SILENT => "",
            INFO => "CONFIG: ",
            DEBUG => "CONFIG: ",
            TRACE => "CONFIG: ",
            _ => "CONFIG: ",
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
    fn debug(&self, level: u8, provider: &str, message: &str);

    /// Add a debug message with step information
    fn debug_step(&self, level: u8, provider: &str, step: usize, message: &str);

    /// Add a debug message with success/failure indication
    fn debug_result(&self, level: u8, provider: &str, message: &str, success: bool);

    /// Add a debug message with both step and success information
    fn debug_step_result(
        &self,
        level: u8,
        provider: &str,
        step: usize,
        message: &str,
        success: bool,
    );
}
