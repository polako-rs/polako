#[macro_export]
macro_rules! implement_constructivism_macro {
    ($prefix:literal, $size:literal) => {

        use proc_macro as pm;
        use syn::{parse_macro_input, DeriveInput};
        use constructivist::prelude::*;

        const DEFAULT_CONSTRUCT_FIELD_LIMIT: u8 = $size;



        #[proc_macro_derive(Construct, attributes(extend, mix, required, default))]
        pub fn derive_construct_item(input: pm::TokenStream) -> pm::TokenStream {
            let input = parse_macro_input!(input as DeriveInput);
            let constructable = match Constructable::from_derive(input, ConstructMode::object()) {
                Err(e) => return pm::TokenStream::from(e.to_compile_error()),
                Ok(c) => c,
            };
            let stream = match constructable.build(&Context::new($prefix)) {
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
            let stream = match constructable.build(&Context::new($prefix)) {
                Err(e) => return pm::TokenStream::from(e.to_compile_error()),
                Ok(c) => c,
            };
            pm::TokenStream::from(stream)
        }

        #[proc_macro]
        pub fn constructable(input: pm::TokenStream) -> pm::TokenStream {
            let input = parse_macro_input!(input as Constructable);
            let stream = match input.build(&Context::new($prefix)) {
                Err(e) => return pm::TokenStream::from(e.to_compile_error()),
                Ok(c) => c,
            };
            pm::TokenStream::from(stream)
        }

        #[proc_macro]
        pub fn construct(input: pm::TokenStream) -> pm::TokenStream {
            let cst = parse_macro_input!(input as Construct);
            let ctx = Context::new($prefix);
            pm::TokenStream::from(match cst.build(&ctx) {
                Ok(r) => r,
                Err(e) => e.to_compile_error()
            })
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

    };
}