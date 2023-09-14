use std::path::Component;

use element::Element;
use eml::Eml;
use proc_macro as pm;
use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::ItemImpl;
use syn::{parse_macro_input, DeriveInput};

const DEFAULT_CONSTRUCT_FIELD_LIMIT: u8 = 8;

use constructivist::derive::{ConstructMode, Constructable, Methods};
use constructivist::genlib;

mod eml;
mod element;

fn lib<S: AsRef<str>>(name: S) -> TokenStream {
    let name = name.as_ref();
    let global = format_ident!("{name}");
    let local = format_ident!("polako_{name}");
    let lib = quote! { ::polako::#global };
    let Some(manifest_path) = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .map(|mut path| { path.push("Cargo.toml"); path })
        else { return lib };
    let Ok(manifest) = std::fs::read_to_string(&manifest_path) else {
        return lib
    };
    let Ok(manifest) = toml::from_str::<toml::map::Map<String, toml::Value>>(&manifest) else {
        return lib
    };

    let Some(pkg) = manifest.get("package") else { return lib };
    let Some(pkg) = pkg.as_table() else { return lib };
    let Some(pkg) = pkg.get("name") else { return lib };
    let Some(pkg) = pkg.as_str() else { return lib };
    if pkg == &format!("polako_{name}") {
        quote!{ crate }
    } else if pkg.starts_with("polako_mod_") {
        lib
    } else if pkg.starts_with("polako_") {
        quote! { ::#local }
    } else {
        lib
    }
}

#[proc_macro_derive(Construct, attributes(extends, mixin, required, default))]
pub fn derive_construct_item(input: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let constructable = match Constructable::from_derive(input, ConstructMode::object()) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    let stream = match constructable.build(lib("constructivism")) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    pm::TokenStream::from(stream)
}
#[proc_macro_derive(Mixin, attributes(required, default))]
pub fn derive_mixin(input: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let constructable = match Constructable::from_derive(input, ConstructMode::mixin()) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    let stream = match constructable.build(lib("constructivism")) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    pm::TokenStream::from(stream)
}

#[proc_macro]
pub fn constructable(input: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(input as Constructable);
    let stream = match input.build(lib("constructivism")) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    pm::TokenStream::from(stream)
}
#[proc_macro_attribute]
pub fn construct_methods(_: pm::TokenStream, input: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(input as ItemImpl);
    let methods = match Methods::from_input(input) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(r) => r,
    };
    let stream = match methods.build(lib("constructivism")) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    pm::TokenStream::from(stream)
}

#[proc_macro]
pub fn implement_constructivism_core(_: pm::TokenStream) -> pm::TokenStream {
    pm::TokenStream::from(genlib::implement_constructivism_core(
        DEFAULT_CONSTRUCT_FIELD_LIMIT,
    ))
}

#[proc_macro]
pub fn implement_constructivism(_: pm::TokenStream) -> pm::TokenStream {
    pm::TokenStream::from(genlib::implement_constructivism(
        DEFAULT_CONSTRUCT_FIELD_LIMIT,
    ))
}

#[proc_macro]
pub fn eml(input: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(input as Eml);
    let cst = lib("constructivism");
    let eml = lib("eml");
    let stream = match input.build(cst, eml) {
        Err(e) => e.to_compile_error(),
        Ok(s) => s,
    };
    pm::TokenStream::from(stream)
}
#[proc_macro]
pub fn build(input: pm::TokenStream) -> pm::TokenStream {
    let mut input = parse_macro_input!(input as Eml);
    input.strict = true;
    let cst = lib("constructivism");
    let eml = lib("eml");
    let stream = match input.build(cst, eml) {
        Err(e) => e.to_compile_error(),
        Ok(s) => s,
    };
    pm::TokenStream::from(stream)
}

#[proc_macro_derive(Element, attributes(extends, mixin, required, default, build))]
pub fn derive_element(input: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let cst = lib("constructivism");
    let eml = lib("eml");
    let construct = match Constructable::from_derive(input.clone(), ConstructMode::object()) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(r) => r
    };
    let construct = match construct.build(cst.clone()) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(r) => r
    };
    let element = match Element::from_derive(input) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(r) => r
    };
    let element = match element.build(cst, eml) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(r) => r
    };

    pm::TokenStream::from(quote! { 
        #construct
        #element
    })
}