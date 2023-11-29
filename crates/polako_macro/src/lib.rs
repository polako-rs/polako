use constructivist::prelude::*;
use derive::{DeriveBehavior, DeriveConstraint, DeriveElement, DeriveSignal};
use eml::Eml;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

implement_constructivism_macro!("polako", variant::Variant, eml::EmlContext);

mod derive;
mod eml;
mod exts;
mod hand;
mod variant;

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

#[proc_macro_derive(Constraint, attributes(param, prop))]
pub fn constraint_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(match DeriveConstraint::build_from_derive(input) {
        Ok(stream) => stream,
        Err(e) => e.to_compile_error(),
    })
}

#[proc_macro_derive(Behavior, attributes(param, prop))]
pub fn behaviour_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(match DeriveBehavior::build_from_derive(input) {
        Ok(stream) => stream,
        Err(e) => e.to_compile_error(),
    })
}

#[proc_macro_derive(Element, attributes(construct, param, prop, signals))]
pub fn element_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(match DeriveElement::build_from_derive(input) {
        Ok(stream) => stream,
        Err(e) => e.to_compile_error(),
    })
}

#[proc_macro_derive(Signal, attributes(construct, param, prop))]
pub fn signal_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(match DeriveSignal::build_from_derive(input) {
        Ok(stream) => stream,
        Err(e) => e.to_compile_error(),
    })
}
