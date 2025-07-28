use proc_macro::TokenStream;
use quote::quote;
use syn::{Item, ItemFn, ItemImpl, ItemStruct, parse_macro_input};

/// A macro that generates FFI bindings for multiple languages.
///
/// The client just writes normal Rust code with #[superffi] and gets
/// bindings for all enabled target languages automatically.
///
/// # Example
/// ```ignore
/// #[superffi]
/// pub struct Config {
///     data: String,
/// }
///
/// #[superffi]
/// impl Config {
///     pub fn new(data: String) -> Self {
///         Self { data }
///     }
///     
///     pub fn get_data(&self) -> String {
///         self.data.clone()
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn superffi(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_item = parse_macro_input!(input as Item);

    match input_item {
        Item::Struct(item_struct) => generate_struct_bindings(item_struct),
        Item::Impl(item_impl) => generate_impl_bindings(item_impl),
        Item::Fn(item_fn) => generate_fn_bindings(item_fn),
        _ => syn::Error::new_spanned(
            &input_item,
            "superffi can only be applied to structs, impls, or functions",
        )
        .to_compile_error()
        .into(),
    }
}

#[allow(unused_variables)]
fn generate_struct_bindings(item_struct: ItemStruct) -> TokenStream {
    let struct_name = &item_struct.ident;
    let struct_fields = &item_struct.fields;

    // Only make output mutable if we actually have features that will extend it
    #[cfg(any(feature = "python", feature = "nodejs", feature = "wasm"))]
    let mut output = quote! { #item_struct };

    #[cfg(not(any(feature = "python", feature = "nodejs", feature = "wasm")))]
    let output = quote! { #item_struct };

    // Generate Python bindings
    #[cfg(feature = "python")]
    {
        output.extend(quote! {
            #[pyo3::pyclass]
            #[derive(Clone)]
            pub struct #struct_name #struct_fields
        });
    }

    // Generate Node.js bindings
    #[cfg(feature = "nodejs")]
    {
        output.extend(quote! {
            #[napi::napi(object)]
            #[derive(Clone)]
            pub struct #struct_name #struct_fields
        });
    }

    // Generate WASM bindings
    #[cfg(feature = "wasm")]
    {
        output.extend(quote! {
            #[wasm_bindgen::prelude::wasm_bindgen]
            #[derive(Clone)]
            pub struct #struct_name #struct_fields
        });
    }

    output.into()
}

#[allow(unused_variables)]
fn generate_impl_bindings(item_impl: ItemImpl) -> TokenStream {
    let struct_type = &item_impl.self_ty;
    let impl_items = &item_impl.items;

    // Only make output mutable if we actually have features that will extend it
    #[cfg(any(feature = "python", feature = "nodejs", feature = "wasm"))]
    let mut output = quote! { #item_impl };

    #[cfg(not(any(feature = "python", feature = "nodejs", feature = "wasm")))]
    let output = quote! { #item_impl };

    // Generate Python method bindings
    #[cfg(feature = "python")]
    {
        output.extend(quote! {
            #[pyo3::pymethods]
            impl #struct_type {
                #(#impl_items)*
            }
        });
    }

    // Generate Node.js method bindings
    #[cfg(feature = "nodejs")]
    {
        output.extend(quote! {
            #[napi::napi]
            impl #struct_type {
                #(
                    #[napi::napi]
                    #impl_items
                )*
            }
        });
    }

    // Generate WASM method bindings
    #[cfg(feature = "wasm")]
    {
        output.extend(quote! {
            #[wasm_bindgen::prelude::wasm_bindgen]
            impl #struct_type {
                #(
                    #[wasm_bindgen::prelude::wasm_bindgen]
                    #impl_items
                )*
            }
        });
    }

    output.into()
}

fn generate_fn_bindings(item_fn: ItemFn) -> TokenStream {
    // Only make output mutable if we actually have features that will extend it
    #[cfg(any(feature = "python", feature = "nodejs", feature = "wasm"))]
    let mut output = quote! { #item_fn };

    #[cfg(not(any(feature = "python", feature = "nodejs", feature = "wasm")))]
    let output = quote! { #item_fn };

    // Generate Python function binding
    #[cfg(feature = "python")]
    {
        output.extend(quote! {
            #[pyo3::pyfunction]
            #item_fn
        });
    }

    // Generate Node.js function binding
    #[cfg(feature = "nodejs")]
    {
        output.extend(quote! {
            #[napi::napi]
            #item_fn
        });
    }

    // Generate WASM function binding
    #[cfg(feature = "wasm")]
    {
        output.extend(quote! {
            #[wasm_bindgen::prelude::wasm_bindgen]
            #item_fn
        });
    }

    output.into()
}
