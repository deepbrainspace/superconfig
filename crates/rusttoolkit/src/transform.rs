use heck::{ToKebabCase, ToLowerCamelCase, ToPascalCase, ToSnakeCase, ToTitleCase};
use proc_macro2::TokenStream;
use quote::quote;

/// Transforms a string value according to the specified transform type
pub fn apply_transform(value: &str, transform: &str) -> String {
    match transform {
        "snake" => value.to_snake_case(),
        "camel" => value.to_lower_camel_case(),
        "kebab" => value.to_kebab_case(),
        "pascal" => value.to_pascal_case(),
        "title" => value.to_title_case(),
        "upper" => value.to_uppercase(),
        "lower" => value.to_lowercase(),
        "reverse" => value.chars().rev().collect(),
        "len" => value.len().to_string(),
        _ => value.to_string(), // Unknown transform, return as-is
    }
}

/// Scans a token stream for %{param:transform} patterns and returns transformed token stream
pub fn transform_tokens(tokens: TokenStream, params: &[(String, String)]) -> TokenStream {
    let token_string = tokens.to_string();
    let mut result = token_string.clone();

    // Process %{param:transform} patterns
    for (param_name, param_value) in params {
        // Find all patterns like %{param_name:...}
        let pattern_prefix = format!("%{{{param_name}:");

        while let Some(start) = result.find(&pattern_prefix) {
            // Find the closing }
            if let Some(end) = result[start..].find('}') {
                let full_pattern = &result[start..start + end + 1];
                let transform_start = pattern_prefix.len();
                let transform = &full_pattern[transform_start..full_pattern.len() - 1];

                // Apply the transform
                let transformed = apply_transform(param_value, transform);
                result = result.replace(full_pattern, &transformed);
            } else {
                break;
            }
        }

        // Also handle %{param} without transform
        let simple_pattern = format!("%{{{param_name}}}");
        result = result.replace(&simple_pattern, param_value);
    }

    // Parse back to TokenStream
    result.parse().unwrap_or_else(|_| {
        quote! { compile_error!("Failed to parse transformed tokens") }
    })
}
