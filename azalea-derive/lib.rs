//! # azalea-derive
//!
//! Derive macros used by Azalea

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Derive macro to implement StaticHandler and LocalStaticHandler for a Service.
///
/// ```rust
/// #[derive(azalea_derive::StaticHandler)]
/// pub struct Service {}
/// ```
#[proc_macro_derive(StaticHandler)]
pub fn derive_static_handler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        azalea_service::impl_static_handler!(#name);
        azalea_service::impl_local_static_handler!(#name);
    };

    TokenStream::from(expanded)
}
