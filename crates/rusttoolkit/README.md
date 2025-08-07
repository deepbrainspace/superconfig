# rusttoolkit

[![Crates.io](https://img.shields.io/crates/v/rusttoolkit.svg)](https://crates.io/crates/rusttoolkit)
[![Documentation](https://docs.rs/rusttoolkit/badge.svg)](https://docs.rs/rusttoolkit)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/deepbrain/superconfig/tree/main/crates/rusttoolkit)

Rust meta-programming toolkit with advanced code generation macros.

## Features

- **`for_each!` macro**: Eliminate repetitive code patterns with powerful iteration
- **Macro generation**: Generate other macros dynamically (perfect for logging systems)
- **Multiple item types**: Identifiers, strings, numbers, and arrays
- **Array indexing**: Access array elements with `%{param[0]}`, `%{param[1]}`, etc.
- **Clean syntax**: Uses `%{param}` to avoid conflicts with Rust syntax
- **Comprehensive**: Handles all common code generation scenarios

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
rusttoolkit = "0.1.0"
```

## Examples - All Supported Scenarios

### 1. Single Items (Identifiers) - Generate Functions

```rust
use rusttoolkit::for_each;

for_each!([error, warn, info], |level| {
    pub fn %{level}(msg: &str) {
        println!("[{}] {}", stringify!(%{level}).to_uppercase(), msg);
    }
});

// Generates:
// pub fn error(msg: &str) { ... }
// pub fn warn(msg: &str) { ... } 
// pub fn info(msg: &str) { ... }

// Usage:
error("Something went wrong");     // Prints: [ERROR] Something went wrong
warn("This is a warning");         // Prints: [WARN] This is a warning
info("Just informing you");        // Prints: [INFO] Just informing you
```

### 2. String Items - HTTP Methods

```rust
for_each!(["GET", "POST", "PUT"], |method| {
    pub fn handle_%{method}() -> &'static str {
        "%{method}"
    }
});

// Generates:
// pub fn handle_GET() -> &'static str { "GET" }
// pub fn handle_POST() -> &'static str { "POST" }
// pub fn handle_PUT() -> &'static str { "PUT" }

// Usage:
handle_GET()    // Returns: "GET"
handle_POST()   // Returns: "POST"
handle_PUT()    // Returns: "PUT"
```

### 3. Number Items - Status Codes

```rust
for_each!([200, 404, 500], |code| {
    pub fn status_%{code}() -> u16 { 
        %{code} 
    }
});

// Generates:
// pub fn status_200() -> u16 { 200 }
// pub fn status_404() -> u16 { 404 }
// pub fn status_500() -> u16 { 500 }

// Usage:
status_200()    // Returns: 200
status_404()    // Returns: 404
status_500()    // Returns: 500
```

### 4. Array Items with Indexing

```rust
for_each!([["GET", 200], ["POST", 201]], |req| {
    pub fn status_%{req[0]}() -> u16 {
        %{req[1]}
    }
});

// Generates:
// pub fn status_GET() -> u16 { 200 }
// pub fn status_POST() -> u16 { 201 }

// Usage:
status_GET()    // Returns: 200
status_POST()   // Returns: 201
```

### 5. Mixed Types in Same Array

```rust
for_each!([error, "GET", 200], |item| {
    pub fn mixed_%{item}() -> &'static str { 
        stringify!(%{item}) 
    }
});

// Generates:
// pub fn mixed_error() -> &'static str { "error" }
// pub fn mixed_GET() -> &'static str { "GET" }
// pub fn mixed_200() -> &'static str { "200" }

// Usage:
mixed_error()   // Returns: "error"
mixed_GET()     // Returns: "GET" 
mixed_200()     // Returns: "200"
```

### 6. Multiple Parameter References

```rust
for_each!([debug, info], |level| {
    pub fn %{level}_log_%{level}() -> &'static str { 
        concat!(stringify!(%{level}), "_", stringify!(%{level}))
    }
});

// Generates:
// pub fn debug_log_debug() -> &'static str { "debug_debug" }
// pub fn info_log_info() -> &'static str { "info_info" }

// Usage:
debug_log_debug()   // Returns: "debug_debug"
info_log_info()     // Returns: "info_info"
```

### 7. Complex Array Indexing

```rust
for_each!([["users", "GET", "/api/users"], ["posts", "POST", "/api/posts"]], |route| {
    pub fn %{route[0]}_%{route[1]}() -> &'static str { 
        "%{route[2]}" 
    }
});

// Generates:
// pub fn users_GET() -> &'static str { "/api/users" }
// pub fn posts_POST() -> &'static str { "/api/posts" }

// Usage:
users_GET()     // Returns: "/api/users"
posts_POST()    // Returns: "/api/posts"
```

### 8. Macro Generation - LogFFI Use Case

```rust
for_each!([error, warn, info], |level| {
    macro_rules! %{level}_log {
        ($msg:expr) => {
            format!("[{}] {}", stringify!(%{level}).to_uppercase(), $msg)
        };
    }
});

// Generates:
// macro_rules! error_log { ... }
// macro_rules! warn_log { ... }
// macro_rules! info_log { ... }

// Usage:
error_log!("failed")    // Returns: "[ERROR] failed"
warn_log!("warning")    // Returns: "[WARN] warning"
info_log!("started")    // Returns: "[INFO] started"
```

### 9. Advanced Macro Generation

```rust
for_each!([["create", "user"], ["delete", "post"]], |action| {
    macro_rules! %{action[0]}_%{action[1]}_macro {
        ($id:expr) => {
            format!("{}_{}_action: {}", 
                stringify!(%{action[0]}), 
                stringify!(%{action[1]}), 
                $id)
        };
    }
});

// Generates:
// macro_rules! create_user_macro { ... }
// macro_rules! delete_post_macro { ... }

// Usage:
create_user_macro!(123)     // Returns: "create_user_action: 123"
delete_post_macro!(456)     // Returns: "delete_post_action: 456"
```

## Supported Item Types

- **Identifiers**: `error`, `warn`, `info`
- **Strings**: `"GET"`, `"POST"`, `"PUT"`
- **Numbers**: `200`, `404`, `500`
- **Arrays**: `["GET", 200]`, `["POST", 201]`
- **Mixed**: `[error, "GET", 200]`

## Template Syntax - `%{}` Notation

The `%{}` notation is our special template syntax that gets replaced during macro expansion:

- `%{param}` - Replace with single item value
- `%{param[0]}` - Replace with first array element
- `%{param[1]}` - Replace with second array element
- Multiple references supported in same template

**Why `%{}`?** We chose this syntax to avoid conflicts with Rust's native syntax:

- Doesn't conflict with Rust 2024's prefix identifier parsing (`#identifier`)
- Visually distinct from standard Rust syntax
- Allows natural-looking templates that remain readable
- Supports complex expressions like `%{param[0]}` for array indexing

## Use Cases

- **Logging systems**: Generate level-specific macros
- **HTTP handlers**: Create method-specific functions
- **API endpoints**: Generate route handlers
- **Configuration**: Create type-safe config accessors
- **Testing**: Generate test functions for different scenarios

## Requirements

- Rust 2024 edition or later
- Stable Rust (no nightly features required)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
