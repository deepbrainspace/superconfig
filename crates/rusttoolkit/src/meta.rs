use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, parse::Parse, Token, Ident, LitStr};

use crate::transform::transform_tokens;

// meta!(method = "getUserData") {
//     fn %{method:snake}_handler() {} 
// };
struct MetaInput {
    params: syn::punctuated::Punctuated<MetaParam, Token![,]>,
    body: TokenStream2,
}

struct MetaParam {
    name: Ident,
    _eq: Token![=],
    value: LitStr,
}

impl Parse for MetaParam {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(MetaParam {
            name: input.parse()?,
            _eq: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl Parse for MetaInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse (param = "value", ...) { body }
        let content;
        syn::parenthesized!(content in input);
        let params = content.parse_terminated(MetaParam::parse, Token![,])?;
        
        // The rest of the input stream is the body (everything after the parentheses)
        let body: TokenStream2 = input.parse()?;
        
        Ok(MetaInput { params, body })
    }
}

pub fn main(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as MetaInput);
    
    // Convert params to vec of (name, value) tuples
    let params: Vec<(String, String)> = parsed.params
        .into_iter()
        .map(|p| (p.name.to_string(), p.value.value()))
        .collect();
    
    // Transform the body with the provided parameters
    let transformed = transform_tokens(parsed.body, &params);
    
    quote! { #transformed }.into()
}