use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    Error, Expr, ExprArray, Ident, Token,
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
};

use crate::transform::transform_tokens;

/// Universal iteration macro supporting single items and arrays
///
/// # Examples
///
/// ```rust
/// use meta_rust::for_each;
///
/// // Single items - generates error, warn, info macros
/// for_each!([error, warn, info], |level| {
///     macro_rules! %{level} {
///         ($($arg:tt)*) => {
///             println!("[{}] {}", stringify!(%{level}).to_uppercase(), format!($($arg)*));
///         };
///     }
/// });
///
/// // Array items - generates status_GET(), status_POST() functions  
/// for_each!([["GET", 200], ["POST", 201]], |req| {
///     pub fn status_%{req[0]}() -> u16 {
///         %{req[1]}
///     }
/// });
/// ```
pub fn main(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ForEachInput);

    let mut generated = TokenStream2::new();

    for item in input.items {
        let expanded = replace_in_template(&input.template, &input.param, &item);
        generated.extend(expanded);
    }

    TokenStream::from(generated)
}

// The Magic Function - now uses %{param} syntax with transforms
fn replace_in_template(template: &TokenStream2, param: &str, item: &Item) -> TokenStream2 {
    // Strip outer braces first if present
    let template_str = template.to_string();
    let template_str = template_str.trim();
    let template_str = if template_str.starts_with('{') && template_str.ends_with('}') {
        // Remove outer braces and trim whitespace
        template_str[1..template_str.len() - 1].trim()
    } else {
        template_str
    };

    // Parse the cleaned template
    let cleaned_template: TokenStream2 = template_str.parse().unwrap_or_else(|_| template.clone());

    // Build params for transform_tokens
    let params = match item {
        Item::Array(values) => {
            // Create params for array indexing: param[0], param[1], etc.
            let mut array_params = vec![];
            for (index, val) in values.iter().enumerate() {
                array_params.push((format!("{param}[{index}]"), val.clone()));
            }
            // Also add the whole param for non-indexed references
            array_params.push((param.to_string(), format!("[{}]", values.join(", "))));
            array_params
        }
        Item::Single(single_value) => {
            // Single param
            vec![(param.to_string(), single_value.clone())]
        }
    };

    // Use the shared transform_tokens function
    transform_tokens(cleaned_template, &params)
}

// Input structures
struct ForEachInput {
    items: Vec<Item>,
    param: String,
    template: TokenStream2,
}

#[derive(Debug, Clone)]
enum Item {
    Single(String),
    Array(Vec<String>),
}

impl Parse for ForEachInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse array: [item1, item2, ...]
        let array: ExprArray = input.parse().map_err(|_| {
            Error::new_spanned(
                input.cursor().token_stream(),
                "Expected array like [item1, item2, ...] as first argument",
            )
        })?;

        // Fix ownership issue by checking empty first
        if array.elems.is_empty() {
            return Err(Error::new_spanned(&array, "Array cannot be empty"));
        }

        let mut items = Vec::new();
        for elem in array.elems {
            items.push(parse_item(elem)?);
        }

        input.parse::<Token![,]>()?;

        // Parse closure manually: |param| { template }
        input.parse::<Token![|]>()?;
        let param_ident: Ident = input.parse()?;
        input.parse::<Token![|]>()?;

        // Parse the rest as raw tokens (don't try to parse as structured Expr)
        let template: TokenStream2 = input.parse()?;

        let param = param_ident.to_string();

        Ok(ForEachInput {
            items,
            param,
            template,
        })
    }
}

fn parse_single_value(expr: &Expr) -> Result<String> {
    match expr {
        // Identifier: error, warn, info
        Expr::Path(path) if path.path.segments.len() == 1 => {
            Ok(path.path.segments[0].ident.to_string())
        }

        // String literal: "GET", "POST" - use raw value for identifiers
        Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) => Ok(lit_str.value()),

        // Number literal: 200, 201
        Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(lit_int),
            ..
        }) => Ok(lit_int.base10_digits().to_string()),

        _ => Err(Error::new_spanned(
            expr,
            "Expected identifier, string, or number",
        )),
    }
}

fn parse_item(expr: Expr) -> Result<Item> {
    match expr {
        // Try parsing as single value first
        ref single_expr @ (Expr::Path(_) | Expr::Lit(_)) => {
            Ok(Item::Single(parse_single_value(single_expr)?))
        }

        // Array: ["GET", 200], [error, "high"]
        Expr::Array(array) => {
            if array.elems.is_empty() {
                return Err(Error::new_spanned(&array, "Nested arrays cannot be empty"));
            }

            let mut values = Vec::new();
            for elem in array.elems {
                values.push(parse_single_value(&elem)?);
            }
            Ok(Item::Array(values))
        }

        _ => Err(Error::new_spanned(
            &expr,
            "Items must be identifiers, strings, numbers, or arrays of these",
        )),
    }
}
