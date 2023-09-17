use constructivist::implement_constructivism_macro;
implement_constructivism_macro! { "polako", 16 }

mod eml;
use eml::Eml;

#[proc_macro]
pub fn eml(input: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(input as Eml);
    let stream = match input.build() {
        Err(e) => e.to_compile_error(),
        Ok(s) => s,
    };
    pm::TokenStream::from(stream)
}
#[proc_macro]
pub fn blueprint(input: pm::TokenStream) -> pm::TokenStream {
    let mut input = parse_macro_input!(input as Eml);
    input.strict = true;
    let stream = match input.build() {
        Err(e) => e.to_compile_error(),
        Ok(s) => s,
    };
    pm::TokenStream::from(stream)
}