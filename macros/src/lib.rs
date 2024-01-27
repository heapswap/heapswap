// lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data};

/*

sled_zero_copy is a macro that  allows

	#[derive(FromBytes, FromZeroes, AsBytes, Unaligned)]
	#[repr(C)]
	struct Key {
		a: U64<BigEndian>,
	}

to be written as

	#[sled_zero_copy]
	struct Key {
		a: U64<BigEndian>,
	}

*/
#[proc_macro_attribute]
pub fn sled_zero_copy(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let vis = &input.vis;

    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => panic!("SledZeroCopy can only be derived for structs"),
    };

    let field_names = fields.iter().map(|f| &f.ident);
    let field_types = fields.iter().map(|f| &f.ty);

    let expanded = quote! {
        #[derive(zerocopy_derive::FromBytes, zerocopy_derive::FromZeroes, zerocopy_derive::AsBytes, zerocopy_derive::Unaligned)]
        #[repr(C)]
        #vis struct #name {
            #(#field_names: #field_types),*
        }
    };

    TokenStream::from(expanded)
}