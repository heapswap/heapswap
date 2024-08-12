#![allow(unused)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};



#[proc_macro_derive(ProtoExt)]
pub fn derive_proto_ext(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Only inject the field if it's a struct
    if let Data::Struct(ref mut struct_data) = input.data {
        if let Fields::Named(ref mut fields) = struct_data.fields {
            fields.named.push(
                syn::parse_quote! {
                    #[prost(skip)]
                    _string_cache: Option<String>
                }
            );
        }
    }

    let expanded = quote! {
        #input
        
        impl ::subfield_macro_proto_ext::ProtoExt for #name {
            fn _set_string_cache(&mut self, s: &str) {
                self._string_cache = Some(s.to_string());
            }
            
            fn _get_string_cache(&self) -> &str {
                self._string_cache.as_ref().unwrap()
            }
        }
        
        impl Default for #name {
            fn default() -> Self {
                Self {
                    _string_cache: None,
                    ..Default::default()
                }
            }
        }
    };

    TokenStream::from(expanded)
}
