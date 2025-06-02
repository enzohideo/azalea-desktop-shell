use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(StaticHandler)]
pub fn derive_static_handler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        azalea_service::impl_static_handler!(#name);
    };

    TokenStream::from(expanded)
}
