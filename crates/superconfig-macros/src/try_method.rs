//! Implementation of the `#[generate_try_method]` procedural macro with generic error handling

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, Pat, ReturnType, Type, parse_macro_input};

/// Check if the method returns a Result type
fn returns_result(input_fn: &ItemFn) -> bool {
    match &input_fn.sig.output {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Path(type_path) => {
                if let Some(segment) = type_path.path.segments.last() {
                    segment.ident == "Result"
                } else {
                    false
                }
            }
            _ => false,
        },
    }
}

/// Extract the Ok type from Result<T, E>
fn extract_ok_type(input_fn: &ItemFn) -> Option<Type> {
    if let ReturnType::Type(_, ty) = &input_fn.sig.output {
        if let Type::Path(type_path) = ty.as_ref() {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "Result" {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(ok_type)) = args.args.first() {
                            return Some(ok_type.clone());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Implementation of the `generate_try_method` procedural macro
pub fn generate_try_method_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    // Only generate try method for Result-returning methods
    if !returns_result(&input_fn) {
        // For non-Result methods, just return the original method unchanged
        return TokenStream::from(quote! { #input_fn });
    }

    // Extract function components
    let fn_name = &input_fn.sig.ident;
    let try_fn_name = format_ident!("try_{}", fn_name);
    let vis = &input_fn.vis;
    let params = &input_fn.sig.inputs;

    // Extract parameter names for the method call
    let param_names: Vec<_> = params
        .iter()
        .filter_map(|param| {
            if let FnArg::Typed(pat_type) = param {
                if let Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                    Some(&pat_ident.ident)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    // Determine return type for try method by extracting T from Result<T, E>
    let try_return_type = match extract_ok_type(&input_fn) {
        Some(ok_type) => quote! { #ok_type },
        None => quote! { Self }, // Fallback
    };

    // Use a universal approach that works for any struct/error type
    let final_error_handling = quote! {
        let self_clone = self.clone();
        match self_clone.#fn_name(#(#param_names.clone()),*) {
            Ok(result) => result,
            Err(_e) => {
                // For now, just return self on error
                // Individual structs can override this by providing their own try_* methods
                self
            }
        }
    };

    let expanded: TokenStream2 = quote! {
        // Original method (unchanged)
        #input_fn

        // Generated try_* method - only for Result-returning methods
        #vis fn #try_fn_name(#params) -> #try_return_type {
            #final_error_handling
        }
    };

    TokenStream::from(expanded)
}
