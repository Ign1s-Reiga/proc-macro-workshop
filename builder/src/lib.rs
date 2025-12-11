mod impls;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn bulder_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    impls::derive_impl(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
