# SuperConfig Examples

This directory contains practical examples demonstrating SuperConfig features, especially the new verbosity system for configuration debugging.

## 🚀 Verbosity System Examples

### Running the Examples

#### 1. Guardy Usage Example (`guardy_usage.rs`)

Demonstrates how to integrate SuperConfig verbosity in a CLI application like guardy.

```bash
# Run with different verbosity levels:
cargo run --example guardy_usage           # Silent mode (no debug output)
cargo run --example guardy_usage -- -v     # Info mode (basic progress)
cargo run --example guardy_usage -- -vv    # Debug mode (detailed steps)
cargo run --example guardy_usage -- -vvv   # Trace mode (full introspection)
```

**Example Output (Debug mode `-vv`):**

```
=== Guardy Configuration Loading ===
Verbosity level: debug

CONFIG: Set verbosity level to: debug
CONFIG: [1/] Loading embedded default configuration
CONFIG: Embedded config: 6 lines, 51 characters
CONFIG: [2/] Loading hierarchical config for: guardy
CONFIG: Checking hierarchical config paths:
CONFIG:   - /etc/guardy/config.toml ✗
CONFIG:   - /etc/guardy.toml ✗
CONFIG:   - /home/user/.config/guardy/config.toml ✗
CONFIG:   - ./guardy.toml ✗
CONFIG:   - ./config/guardy.toml ✗
CONFIG: [3/] Loading environment variables with prefix: GUARDY_ (ignore empty)
CONFIG: [3/] No environment variables found with prefix: GUARDY_ ✗
CONFIG: [4/] No CLI arguments provided ✗
CONFIG: Extracting final configuration
CONFIG: Configuration extraction successful ✓

Configuration loaded successfully!
Scanner mode: standard
Log level: info
```

### Understanding Verbosity Levels

- **Silent (0)**: No debug output - production mode
- **Info (-v)**: Basic loading progress - shows what's happening
- **Debug (-vv)**: Detailed steps with success/failure - troubleshooting mode
- **Trace (-vvv)**: Full introspection with actual values - deep debugging

### Testing with Environment Variables

You can test environment variable loading by setting them before running:

```bash
# Set some test environment variables
export GUARDY_SCANNER_MODE="strict"
export GUARDY_LOG_LEVEL="debug"

# Run with debug verbosity to see them being loaded
cargo run --example guardy_usage -- -vv

# Clean up
unset GUARDY_SCANNER_MODE GUARDY_LOG_LEVEL
```

## 🔧 Integration in Your CLI Application

### Step 1: Add Verbosity to Your CLI Args

```rust
use clap::Parser;
use superconfig::{SuperConfig, VerbosityLevel};

#[derive(Parser)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    
    // ... other CLI args
}
```

### Step 2: Use Verbosity in Configuration Loading

```rust
let mut config = SuperConfig::new()
    .with_verbosity(VerbosityLevel::from_cli_args(cli.verbose))
    .with_defaults_string(DEFAULT_CONFIG)
    .with_hierarchical_config("myapp")
    .with_env_ignore_empty("MYAPP_")
    .with_cli_opt(Some(cli_overrides));

let final_config: MyConfig = config.extract()?;
```

### Step 3: Help Users Debug Configuration Issues

When users report configuration problems:

```bash
# Basic troubleshooting
myapp -v

# Detailed debugging  
myapp -vv

# Full configuration introspection (shows actual values)
myapp -vvv
```

## 🛡️ Security Features

### Automatic Sensitive Data Masking

Environment variables containing these keywords are automatically masked in trace output:

- `password`
- `secret`
- `token`
- `key`

Example:

```bash
export MYAPP_DATABASE_PASSWORD="secret123"
export MYAPP_API_TOKEN="abc123" 

# When running with -vvv, these will show as:
# MYAPP_DATABASE_PASSWORD=***MASKED***
# MYAPP_API_TOKEN=***MASKED***
```

## 📚 More Examples

Want to see more examples? Consider contributing:

1. **Database Connection Example**: Show how to debug database configuration loading
2. **Microservice Example**: Multi-service configuration with shared and service-specific configs
3. **Development vs Production**: Environment-specific configuration patterns

## 🤝 Contributing Examples

To add a new example:

1. Create a new `.rs` file in this directory
2. Use `SuperConfig` with verbosity to demonstrate specific features
3. Test with all verbosity levels (silent, -v, -vv, -vvv)
4. Document the example in this README
5. Include practical use cases that solve real problems

Examples should be:

- **Practical**: Solve real-world configuration problems
- **Educational**: Show best practices and common patterns
- **Testable**: Include clear instructions for running and testing
- **Documented**: Explain what the example demonstrates and why it's useful
