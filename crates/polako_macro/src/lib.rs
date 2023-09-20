use proc_macro::TokenStream;
use constructivist::prelude::*;
use eml::Eml;
use syn::parse_macro_input;

implement_constructivism_macro!("polako");

mod eml;

#[proc_macro]
pub fn eml(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Eml);
    let stream = match input.build() {
        Err(e) => e.to_compile_error(),
        Ok(s) => s,
    };
    TokenStream::from(stream)
}
#[proc_macro]
pub fn blueprint(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as Eml);
    input.strict = true;
    let stream = match input.build() {
        Err(e) => e.to_compile_error(),
        Ok(s) => s,
    };
    TokenStream::from(stream)
}