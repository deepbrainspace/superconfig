# Meta-Rust Enhanced Format Macro

## Overview

Extend rusttoolkit with an enhanced `format!` macro that supports case transformations and string manipulations while maintaining full backward compatibility with `std::format!`.

## Goals

1. **Drop-in replacement** for `std::format!` with enhanced capabilities
2. **Transform functions** like `%{param:title}`, `%{param:upper}`, etc.
3. **Backward compatibility** - existing format! code works unchanged
4. **Integration with for_each!** - eliminate paste dependency in logffi
5. **Community utility** - powerful string manipulation in macros

## Architecture

### Module Structure

```
src/
├── lib.rs              # Export format! proc macro
├── loops/              # Existing for_each!
│   ├── mod.rs
│   └── for_each.rs
└── transform/          # New enhanced format!
    ├── mod.rs          # Main format! macro implementation
    └── format.rs       # Format string processing and transforms
```

### Core Components

#### 1. Enhanced `format!` Macro (`src/lib.rs`)

```rust
/// Enhanced format! macro with transformation support
/// 
/// Supports all standard format! functionality plus meta transformations:
/// - `%{param:title}` - Title case
/// - `%{param:upper}` - UPPERCASE  
/// - `%{param:lower}` - lowercase
/// - `%{param:camel}` - CamelCase
/// - `%{param:snake}` - snake_case
/// - `%{param:kebab}` - kebab-case
#[proc_macro]
pub fn format(input: TokenStream) -> TokenStream {
    transform::format::main(input)
}
```

#### 2. Format Processor (`src/transform/format.rs`)

```rust
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub fn main(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    
    if contains_meta_patterns(&input_str) {
        // Process enhanced format with transformations
        process_enhanced_format(input)
    } else {
        // Pass through to standard format!
        pass_through_std_format(input)
    }
}

fn contains_meta_patterns(input: &str) -> bool {
    input.contains("%{") && input.contains("}")
}

fn process_enhanced_format(input: TokenStream) -> TokenStream {
    // Parse format string and parameters
    let parsed = parse_format_input(input);
    
    // Apply transformations to meta patterns
    let transformed = apply_meta_transformations(parsed);
    
    // Generate std::format! call with transformed values
    quote! { std::format!(#transformed) }.into()
}

fn pass_through_std_format(input: TokenStream) -> TokenStream {
    quote! { std::format!(#input) }.into()
}
```

#### 3. Transform Functions (`src/transform/format.rs`)

```rust
/// Apply case transformation to a string
fn apply_transform(value: &str, transform: &str) -> String {
    match transform {
        "title" => title_case(value),
        "upper" => value.to_uppercase(),
        "lower" => value.to_lowercase(), 
        "camel" => camel_case(value),
        "snake" => snake_case(value),
        "kebab" => kebab_case(value),
        "reverse" => value.chars().rev().collect(),
        "len" => value.len().to_string(),
        _ => {
            // For unknown transforms, just return original value
            // Could also emit compile error if desired
            value.to_string()
        }
    }
}

fn title_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
    }
}

fn camel_case(s: &str) -> String {
    s.split('_')
        .enumerate()
        .map(|(i, word)| {
            if i == 0 {
                word.to_lowercase()
            } else {
                title_case(word)
            }
        })
        .collect()
}

fn snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap());
    }
    result
}

fn kebab_case(s: &str) -> String {
    snake_case(s).replace('_', "-")
}
```

## Implementation Plan

### Phase 1: Core Infrastructure

- [ ] Create `src/transform/` module structure
- [ ] Implement basic `format!` passthrough functionality
- [ ] Add pattern detection (`contains_meta_patterns`)
- [ ] Set up basic parsing for enhanced format syntax

### Phase 2: Transform Engine

- [ ] Implement core transform functions (`title`, `upper`, `lower`)
- [ ] Add advanced transforms (`camel`, `snake`, `kebab`)
- [ ] Add utility transforms (`reverse`, `len`)
- [ ] Create transform function registry

### Phase 3: Format Processing

- [ ] Implement format string parsing
- [ ] Handle parameter extraction and transformation
- [ ] Generate proper `std::format!` calls
- [ ] Add error handling for malformed patterns

### Phase 4: Integration Testing

- [ ] Test backward compatibility with existing `format!` usage
- [ ] Test all transform functions individually
- [ ] Test complex format strings with multiple transformations
- [ ] Test integration with `for_each!` macro

### Phase 5: for_each! Integration

- [ ] Update `for_each.rs` to use enhanced `format!` internally
- [ ] Remove manual string replacement logic
- [ ] Simplify template processing
- [ ] Test logffi use cases

## Usage Examples

### 1. Backward Compatibility

```rust
use rusttoolkit::format;

// Works exactly like std::format!
let msg = format!("Hello {}", name);
let num = format!("Value: {:.2}", 3.14159);
```

### 2. Enhanced Transformations

```rust
use rusttoolkit::format;

// Title case transformation
let msg = format!("Hello %{name:title}!", name = "john"); 
// → "Hello John!"

// Multiple transformations
let api = format!("%{method:upper}_%{endpoint:snake}", 
    method = "GetUser", 
    endpoint = "UserProfile"
);
// → "GETUSER_user_profile"
```

### 3. LogFFI Integration

```rust
use rusttoolkit::{for_each, format};

for_each!([error, warn, info, debug, trace], |level| {
    macro_rules! %{level} {
        (target: $target:expr, $($arg:tt)*) => {
            println!("{}", format!("[%{level:upper}] %{target}: {}", 
                level = %{level}, 
                target = $target, 
                format!($($arg)*)
            ));
        };
        ($($arg:tt)*) => {
            %{level}!(target: module_path!(), $($arg)*)
        };
    }
});
```

### 4. Advanced Use Cases

```rust
// Dynamic function names
macro_rules! create_getter {
    ($field:ident) => {
        pub fn %{field:snake}_getter() -> String {
            format!("%{field:title} Value", field = stringify!($field))
        }
    };
}

create_getter!(UserName);
// Generates: pub fn user_name_getter() -> String { "UserName Value" }
```

## Benefits

### For Meta-Rust Users

1. **Enhanced macro capabilities** - powerful string manipulation
2. **No learning curve** - familiar `format!` syntax with additions
3. **Backward compatible** - existing code works unchanged
4. **Extensible** - easy to add new transform functions

### For LogFFI

1. **Remove paste dependency** - no more unmaintained external crates
2. **Cleaner code** - simpler macro generation
3. **Better error messages** - integrated with syn error handling
4. **Consistent syntax** - same `%{param:transform}` everywhere

### For Community

1. **Powerful utility** - addresses common macro string manipulation needs
2. **Well-tested** - comprehensive test coverage
3. **Documentation** - clear examples and use cases
4. **Maintenance** - actively maintained as part of rusttoolkit

## Testing Strategy

### Unit Tests

- [ ] Each transform function individually
- [ ] Pattern detection accuracy
- [ ] Error handling for malformed input
- [ ] Edge cases (empty strings, special characters)

### Integration Tests

- [ ] Complex format strings with multiple params
- [ ] Nested transformations
- [ ] Performance with large inputs
- [ ] Memory usage patterns

### Compatibility Tests

- [ ] All existing `std::format!` test cases
- [ ] Integration with other proc macros
- [ ] Error message quality
- [ ] Compile-time performance

## Success Criteria

1. **✅ Backward Compatible**: All existing `format!` code works unchanged
2. **✅ Transform Functions**: All planned transforms work correctly
3. **✅ LogFFI Integration**: Successfully removes paste dependency
4. **✅ Performance**: No significant slowdown vs `std::format!`
5. **✅ Error Handling**: Clear error messages for malformed patterns
6. **✅ Documentation**: Comprehensive examples and API docs
7. **✅ Test Coverage**: >95% code coverage with integration tests

## Future Enhancements

### Conditional Transforms

```rust
format!("%{level:if(error,🔥,📝)} %{msg:title}", 
    level = "error", 
    msg = "connection failed"
)
// → "🔥 Connection Failed"
```

### Custom Transform Functions

```rust
// Allow users to register custom transforms
register_transform!("reverse_title", |s| title_case(&s.chars().rev().collect::<String>()));
```

### Performance Optimizations

- Compile-time pattern analysis
- Caching for repeated transforms
- SIMD string operations for large inputs

This enhanced `format!` macro will make rusttoolkit a powerful meta-programming toolkit while solving the paste dependency issue for logffi and providing significant value to the Rust community.
