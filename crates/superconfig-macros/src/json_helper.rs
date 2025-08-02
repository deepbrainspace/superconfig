//! Implementation of the `#[generate_json_helper]` procedural macro with bidirectional support

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    FnArg, Ident, ItemFn, Pat, ReturnType, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// Supported JSON conversion directions
#[derive(Debug, Clone, PartialEq)]
enum JsonDirection {
    Incoming, // Generate _from_json method
    Outgoing, // Generate _as_json method
    Both,     // Generate unified _json method
    Auto,     // Auto-detect based on method signature
}

/// Arguments for the generate_json_helper macro
struct JsonHelperArgs {
    directions: Vec<JsonDirection>,
    handle_mode: bool,
}

impl Parse for JsonHelperArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(JsonHelperArgs {
                directions: vec![JsonDirection::Auto],
                handle_mode: false,
            });
        }

        let mut directions = Vec::new();
        let mut handle_mode = false;

        loop {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "in" | "incoming" => directions.push(JsonDirection::Incoming),
                "out" | "outgoing" => directions.push(JsonDirection::Outgoing),
                "auto" => directions.push(JsonDirection::Auto),
                "bidirectional" => directions.push(JsonDirection::Both), // Legacy support
                "handle_mode" => handle_mode = true,
                _ => {
                    return Err(syn::Error::new_spanned(
                        ident,
                        "Expected 'in', 'out', 'incoming', 'outgoing', 'auto', 'bidirectional', or 'handle_mode'",
                    ));
                }
            };

            if input.is_empty() {
                break;
            }

            input.parse::<Token![,]>()?;

            if input.is_empty() {
                break;
            }
        }

        // Convert "in,out" to Both
        if directions.len() == 2
            && directions.contains(&JsonDirection::Incoming)
            && directions.contains(&JsonDirection::Outgoing)
        {
            directions = vec![JsonDirection::Both];
        }

        Ok(JsonHelperArgs {
            directions,
            handle_mode,
        })
    }
}

/// Check if a type is an Arc<T>
fn is_arc_type(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                segment.ident == "Arc"
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Check if a type is considered "complex" (needs JSON serialization)
fn is_complex_type(ty: &Type) -> bool {
    match ty {
        // Primitive types that can cross FFI boundaries directly
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                match segment.ident.to_string().as_str() {
                    "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32"
                    | "i64" | "i128" | "isize" | "f32" | "f64" | "bool" | "String" | "str" => false,
                    // ConfigHandle serializes as just a u64, so it's simple for FFI
                    "ConfigHandle" => false,
                    _ => true, // All other types are considered complex
                }
            } else {
                true
            }
        }
        Type::Reference(type_ref) => {
            // &str and &ConfigHandle<T> are simple, other references are complex
            match type_ref.elem.as_ref() {
                Type::Path(type_path) => {
                    if let Some(segment) = type_path.path.segments.last() {
                        match segment.ident.to_string().as_str() {
                            "str" | "ConfigHandle" => false, // These are simple
                            _ => true,                       // Other references are complex
                        }
                    } else {
                        true
                    }
                }
                _ => true,
            }
        }
        _ => true, // Tuples, arrays, etc. are complex
    }
}

/// Auto-detect which directions are needed based on method signature
fn auto_detect_directions(input_fn: &ItemFn) -> Vec<JsonDirection> {
    let has_complex_params = input_fn.sig.inputs.iter().any(|param| {
        if let FnArg::Typed(pat_type) = param {
            is_complex_type(&pat_type.ty)
        } else {
            false // self parameter
        }
    });

    let has_complex_return = match &input_fn.sig.output {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => {
            // Check if it's Result<ComplexType, _> or just ComplexType
            match ty.as_ref() {
                Type::Path(type_path) => {
                    if let Some(segment) = type_path.path.segments.last() {
                        if segment.ident == "Result" {
                            // Check the Ok type in Result<T, E>
                            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                                if let Some(syn::GenericArgument::Type(ok_type)) = args.args.first()
                                {
                                    is_complex_type(ok_type)
                                } else {
                                    true
                                }
                            } else {
                                true
                            }
                        } else {
                            is_complex_type(ty)
                        }
                    } else {
                        true
                    }
                }
                _ => is_complex_type(ty),
            }
        }
    };

    match (has_complex_params, has_complex_return) {
        (true, true) => vec![JsonDirection::Both],
        (true, false) => vec![JsonDirection::Incoming],
        (false, true) => vec![JsonDirection::Outgoing],
        (false, false) => vec![], // No JSON methods needed - FFI can handle simple types directly
    }
}

/// Implementation of the `generate_json_helper` procedural macro
pub fn generate_json_helper_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as JsonHelperArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    // Check if using auto-detection
    let is_auto_mode = args.directions.contains(&JsonDirection::Auto);
    let handle_mode = args.handle_mode;

    // Determine which directions to generate
    let directions = if is_auto_mode {
        auto_detect_directions(&input_fn)
    } else {
        args.directions
    };

    // Check if we're trying to use JSON helpers for simple types only
    if directions.is_empty() && is_auto_mode {
        // Auto-detected that no JSON helpers are needed
        return TokenStream::from(quote! { #input_fn });
    }

    // Check for explicit usage on simple types
    if !is_auto_mode {
        let auto_detected = auto_detect_directions(&input_fn);
        if auto_detected.is_empty() {
            return syn::Error::new_spanned(
                &input_fn.sig.ident,
                "Cannot use #[generate_json_helper] on methods with only simple parameters and simple return types. \
                 FFI can handle simple types (i32, String, bool, etc.) directly without JSON serialization. \
                 This macro is only needed for complex types (structs, enums, Vec, etc.).\n\n\
                 Consider removing this macro or using complex types that require JSON serialization."
            ).to_compile_error().into();
        }
    }

    // Extract function components
    let fn_name = &input_fn.sig.ident;
    let params = &input_fn.sig.inputs;
    let generics = &input_fn.sig.generics;
    let vis = &input_fn.vis;

    // Create modified generics with Serialize constraint for JSON methods
    let mut json_generics = generics.clone();
    for param in &mut json_generics.params {
        if let syn::GenericParam::Type(type_param) = param {
            // Add Serialize bound to each type parameter
            type_param.bounds.push(syn::parse_quote!(serde::Serialize));
        }
    }

    // Extract parameter names for method calls
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

    let mut generated_methods = Vec::new();

    // Handle different direction combinations
    if directions.contains(&JsonDirection::Both) {
        // Generate unified bidirectional method
        let json_unified_name = format_ident!("{}_json", fn_name);

        // Separate self, simple params, and complex params
        let self_param = params
            .iter()
            .find(|param| matches!(param, FnArg::Receiver(_)));

        let (simple_params, complex_params): (Vec<_>, Vec<_>) = params
            .iter()
            .filter_map(|param| {
                if let FnArg::Typed(pat_type) = param {
                    Some((param, pat_type, is_complex_type(&pat_type.ty)))
                } else {
                    None
                }
            })
            .partition(|(_, _, is_complex)| !is_complex);

        let self_param_tokens = if let Some(self_param) = self_param {
            quote! { #self_param, }
        } else {
            quote! {}
        };

        let simple_params_tokens: Vec<_> =
            simple_params.iter().map(|(param, _, _)| param).collect();
        let simple_param_sigs = if !simple_params_tokens.is_empty() {
            quote! { #(#simple_params_tokens),*, }
        } else {
            quote! {}
        };

        // Generate deserialization for complex parameters
        let complex_param_deserializations: Vec<_> = complex_params.iter().enumerate().map(|(i, (param, pat_type, _))| {
            let param_name = if let Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                &pat_ident.ident
            } else {
                panic!("Unsupported parameter pattern");
            };
            let param_type = &pat_type.ty;
            let field_name = param_name.to_string();
            let param_index = i + 1;

            let _param_metadata = param; // Could be used for attributes

            quote! {
                let #param_name: #param_type = match params_json.get(#field_name) {
                    Some(value) => match serde_json::from_value(value.clone()) {
                        Ok(deserialized) => deserialized,
                        Err(e) => return serde_json::to_string(&serde_json::json!({
                            "success": false,
                            "error": format!("Failed to deserialize parameter {} '{}' (type: {}): {}",
                                #param_index, #field_name, stringify!(#param_type), e)
                        })).unwrap(),
                    },
                    None => return serde_json::to_string(&serde_json::json!({
                        "success": false,
                        "error": format!("Missing required parameter {} '{}' (type: {})",
                            #param_index, #field_name, stringify!(#param_type))
                    })).unwrap(),
                };
            }
        }).collect();

        let all_param_names: Vec<_> = params
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

        generated_methods.push(quote! {
            #vis fn #json_unified_name(#self_param_tokens #simple_param_sigs json_params: &str) -> String {
                // Parse the incoming JSON
                let params_json: serde_json::Value = match serde_json::from_str(json_params) {
                    Ok(json) => json,
                    Err(e) => return serde_json::to_string(&serde_json::json!({
                        "success": false,
                        "error": format!("Invalid JSON: {}", e)
                    })).unwrap(),
                };

                // Deserialize complex parameters
                #(#complex_param_deserializations)*

                // Call the original method with all parameters
                match self.#fn_name(#(#all_param_names),*) {
                    Ok(result) => {
                        // Serialize the result
                        match serde_json::to_value(&result) {
                            Ok(serialized) => serde_json::to_string(&serde_json::json!({
                                "success": true,
                                "data": serialized
                            })).unwrap(),
                            Err(_) => serde_json::to_string(&serde_json::json!({
                                "success": true
                            })).unwrap(),
                        }
                    },
                    Err(e) => serde_json::to_string(&serde_json::json!({
                        "success": false,
                        "error": e.to_string()
                    })).unwrap(),
                }
            }
        });
    } else {
        // Generate separate methods for single-direction cases

        // Generate _as_json method (outgoing) if needed
        if directions.contains(&JsonDirection::Outgoing) {
            let json_out_name = format_ident!("{}_as_json", fn_name);

            // Handle mode: only return success/error, don't serialize result
            let method_call = if handle_mode {
                match &input_fn.sig.output {
                    ReturnType::Default => {
                        quote! {
                            let _result = self.#fn_name(#(#param_names),*);
                            serde_json::to_string(&serde_json::json!({"success": true})).unwrap()
                        }
                    }
                    ReturnType::Type(_, ty) => {
                        match ty.as_ref() {
                            Type::Path(type_path) => {
                                if let Some(segment) = type_path.path.segments.last() {
                                    if segment.ident == "Result" {
                                        // Result type - handle Ok/Err
                                        quote! {
                                            match self.#fn_name(#(#param_names),*) {
                                                Ok(_) => serde_json::to_string(&serde_json::json!({"success": true})).unwrap(),
                                                Err(e) => serde_json::to_string(&serde_json::json!({
                                                    "success": false,
                                                    "error": e.to_string()
                                                })).unwrap(),
                                            }
                                        }
                                    } else {
                                        // Non-Result type
                                        quote! {
                                            let _result = self.#fn_name(#(#param_names),*);
                                            serde_json::to_string(&serde_json::json!({"success": true})).unwrap()
                                        }
                                    }
                                } else {
                                    // Fallback for non-Result
                                    quote! {
                                        let _result = self.#fn_name(#(#param_names),*);
                                        serde_json::to_string(&serde_json::json!({"success": true})).unwrap()
                                    }
                                }
                            }
                            _ => {
                                // Fallback for non-Result
                                quote! {
                                    let _result = self.#fn_name(#(#param_names),*);
                                    serde_json::to_string(&serde_json::json!({"success": true})).unwrap()
                                }
                            }
                        }
                    }
                }
            } else {
                // Normal mode: serialize the actual result
                match &input_fn.sig.output {
                    ReturnType::Default => {
                        quote! {
                            let result = self.#fn_name(#(#param_names),*);
                            match serde_json::to_value(&result) {
                                Ok(serialized) => serde_json::to_string(&serde_json::json!({
                                    "success": true,
                                    "data": serialized
                                })).unwrap(),
                                Err(_) => serde_json::to_string(&serde_json::json!({
                                    "success": true
                                })).unwrap(),
                            }
                        }
                    }
                    ReturnType::Type(_, ty) => {
                        match ty.as_ref() {
                            Type::Path(type_path) => {
                                if let Some(segment) = type_path.path.segments.last() {
                                    if segment.ident == "Result" {
                                        // Result type - handle Ok/Err
                                        // Check if Result contains Arc<T> and handle dereferencing
                                        let serialize_expr = if let Type::Path(path) = ty.as_ref() {
                                            if let Some(segment) = path.path.segments.last() {
                                                if segment.ident == "Result" {
                                                    // Check if the Ok type is Arc<T>
                                                    if let syn::PathArguments::AngleBracketed(
                                                        args,
                                                    ) = &segment.arguments
                                                    {
                                                        if let Some(syn::GenericArgument::Type(
                                                            ok_type,
                                                        )) = args.args.first()
                                                        {
                                                            if is_arc_type(ok_type) {
                                                                quote! { &*result }
                                                            } else {
                                                                quote! { &result }
                                                            }
                                                        } else {
                                                            quote! { &result }
                                                        }
                                                    } else {
                                                        quote! { &result }
                                                    }
                                                } else {
                                                    quote! { &result }
                                                }
                                            } else {
                                                quote! { &result }
                                            }
                                        } else {
                                            quote! { &result }
                                        };

                                        quote! {
                                            match self.#fn_name(#(#param_names),*) {
                                                Ok(result) => {
                                                    match serde_json::to_value(#serialize_expr) {
                                                        Ok(serialized) => serde_json::to_string(&serde_json::json!({
                                                            "success": true,
                                                            "data": serialized
                                                        })).unwrap(),
                                                        Err(_) => serde_json::to_string(&serde_json::json!({
                                                            "success": true
                                                        })).unwrap(),
                                                    }
                                                },
                                                Err(e) => serde_json::to_string(&serde_json::json!({
                                                    "success": false,
                                                    "error": e.to_string()
                                                })).unwrap(),
                                            }
                                        }
                                    } else {
                                        // Non-Result type - direct serialization
                                        let serialize_expr = if is_arc_type(ty.as_ref()) {
                                            quote! { &*result }
                                        } else {
                                            quote! { &result }
                                        };

                                        quote! {
                                            let result = self.#fn_name(#(#param_names),*);
                                            match serde_json::to_value(#serialize_expr) {
                                                Ok(serialized) => serde_json::to_string(&serde_json::json!({
                                                    "success": true,
                                                    "data": serialized
                                                })).unwrap(),
                                                Err(_) => serde_json::to_string(&serde_json::json!({
                                                    "success": true
                                                })).unwrap(),
                                            }
                                        }
                                    }
                                } else {
                                    // Fallback for non-Result
                                    let serialize_expr = if is_arc_type(ty.as_ref()) {
                                        quote! { &*result }
                                    } else {
                                        quote! { &result }
                                    };

                                    quote! {
                                        let result = self.#fn_name(#(#param_names),*);
                                        match serde_json::to_value(#serialize_expr) {
                                            Ok(serialized) => serde_json::to_string(&serde_json::json!({
                                                "success": true,
                                                "data": serialized
                                            })).unwrap(),
                                            Err(_) => serde_json::to_string(&serde_json::json!({
                                                "success": true
                                            })).unwrap(),
                                        }
                                    }
                                }
                            }
                            _ => {
                                // Fallback for non-Result - assume non-Arc for unknown types
                                quote! {
                                    let result = self.#fn_name(#(#param_names),*);
                                    match serde_json::to_value(&result) {
                                        Ok(serialized) => serde_json::to_string(&serde_json::json!({
                                            "success": true,
                                            "data": serialized
                                        })).unwrap(),
                                        Err(_) => serde_json::to_string(&serde_json::json!({
                                            "success": true
                                        })).unwrap(),
                                    }
                                }
                            }
                        }
                    }
                } // Close the else block
            };

            let base_method_name = fn_name.to_string();
            let doc_content = if handle_mode {
                format!(
                    "JSON wrapper for `{base_method_name}` method (handle-based architecture)\n\nReturns simple success/error JSON responses optimized for handle-based FFI clients:\n- Success: `{{\"success\": true}}`\n- Error: `{{\"success\": false, \"error\": \"message\"}}`\n\nThis method does not serialize the actual result data, making it suitable for\nhandle-based architectures where clients only need to know if the operation succeeded."
                )
            } else {
                format!(
                    "JSON wrapper for `{base_method_name}` method (FFI compatibility)\n\nReturns detailed JSON responses with serialized result data:\n- Success: `{{\"success\": true, \"data\": <serialized_result>}}`\n- Error: `{{\"success\": false, \"error\": \"message\"}}`\n\nThis method serializes the actual result data for clients that need the full response."
                )
            };

            generated_methods.push(quote! {
                #[doc = #doc_content]
                #vis fn #json_out_name #json_generics (#params) -> String {
                    #method_call
                }
            });
        }

        // Generate _from_json method (incoming) if needed
        if directions.contains(&JsonDirection::Incoming) {
            let json_in_name = format_ident!("{}_from_json", fn_name);

            // Separate self, simple params, and complex params
            let self_param = params
                .iter()
                .find(|param| matches!(param, FnArg::Receiver(_)));

            let (simple_params, complex_params): (Vec<_>, Vec<_>) = params
                .iter()
                .filter_map(|param| {
                    if let FnArg::Typed(pat_type) = param {
                        Some((param, pat_type, is_complex_type(&pat_type.ty)))
                    } else {
                        None
                    }
                })
                .partition(|(_, _, is_complex)| !is_complex);

            let self_param_tokens = if let Some(self_param) = self_param {
                quote! { #self_param, }
            } else {
                quote! {}
            };

            let simple_params_tokens: Vec<_> =
                simple_params.iter().map(|(param, _, _)| param).collect();
            let simple_param_sigs = if !simple_params_tokens.is_empty() {
                quote! { #(#simple_params_tokens),*, }
            } else {
                quote! {}
            };

            // Generate deserialization for complex parameters
            let complex_param_deserializations: Vec<_> = complex_params.iter().enumerate().map(|(i, (param, pat_type, _))| {
                let param_name = if let Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                    &pat_ident.ident
                } else {
                    panic!("Unsupported parameter pattern");
                };
                let param_type = &pat_type.ty;
                let field_name = param_name.to_string();
                let param_index = i + 1;

                let _param_metadata = param; // Could be used for attributes

                quote! {
                    let #param_name: #param_type = match params_json.get(#field_name) {
                        Some(value) => match serde_json::from_value(value.clone()) {
                            Ok(deserialized) => deserialized,
                            Err(e) => return serde_json::to_string(&serde_json::json!({
                                "success": false,
                                "error": format!("Failed to deserialize parameter {} '{}' (type: {}): {}",
                                    #param_index, #field_name, stringify!(#param_type), e)
                            })).unwrap(),
                        },
                        None => return serde_json::to_string(&serde_json::json!({
                            "success": false,
                            "error": format!("Missing required parameter {} '{}' (type: {})",
                                #param_index, #field_name, stringify!(#param_type))
                        })).unwrap(),
                    };
                }
            }).collect();

            let all_param_names: Vec<_> = params
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

            generated_methods.push(quote! {
                #vis fn #json_in_name(#self_param_tokens #simple_param_sigs json_params: &str) -> String {
                    // Parse the incoming JSON
                    let params_json: serde_json::Value = match serde_json::from_str(json_params) {
                        Ok(json) => json,
                        Err(e) => return serde_json::to_string(&serde_json::json!({
                            "success": false,
                            "error": format!("Invalid JSON: {}", e)
                        })).unwrap(),
                    };

                    // Deserialize complex parameters
                    #(#complex_param_deserializations)*

                    // Call the original method with all parameters
                    match self.#fn_name(#(#all_param_names),*) {
                        Ok(result) => {
                            // Try to serialize the result
                            match serde_json::to_value(&result) {
                                Ok(serialized) => serde_json::to_string(&serde_json::json!({
                                    "success": true,
                                    "data": serialized
                                })).unwrap(),
                                Err(_) => serde_json::to_string(&serde_json::json!({
                                    "success": true
                                })).unwrap(),
                            }
                        },
                        Err(e) => serde_json::to_string(&serde_json::json!({
                            "success": false,
                            "error": e.to_string()
                        })).unwrap(),
                    }
                }
            });
        }
    }

    let expanded: TokenStream2 = quote! {
        // Original method (unchanged)
        #input_fn

        // Generated JSON helper methods
        #(#generated_methods)*
    };

    TokenStream::from(expanded)
}
