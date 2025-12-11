mod impls;

use proc_macro::TokenStream;
use syn::{parse_macro_input, Error};

// Supports debug attribute macro
#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn custom_debug_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    impls::derive_impl(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
