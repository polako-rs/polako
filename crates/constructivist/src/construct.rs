use proc_macro2::{Ident, TokenStream};
use syn::{parse::Parse, Type, Expr, Token, braced, parenthesized};

use quote::quote;

use crate::{throw, context::Context};

#[derive(Clone)]
pub struct Param {
    pub ident: Ident,
    pub value: Expr,
}

impl Parse for Param {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut value = None;
        if input.peek(Token![!]) {
            input.parse::<Token![!]>()?;
            value = Some(syn::parse2(quote! { false })?);
        }
        let ident = input.parse()?;
        if value.is_none() && input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            value = Some(input.parse()?);
        }
        if value.is_none() && input.is_empty() {
            value = Some(syn::parse2(quote! { true })?);
        }
        if value.is_none() {
            throw!(input, "Unexpected param input");
        }
        Ok(Param { ident, value: value.unwrap()})
    }
}
//         let param: &$crate::Param<_, _> = &$fields.$f;
//         let field = param.field();
//         let value = $params.field(&field).define(param.value($e.into()));
//         let $params = $params + value;
impl Param {
    pub fn build(&self, ctx: &Context) -> syn::Result<TokenStream> {
        let ident = &self.ident;
        let value = &self.value;
        let lib = ctx.path("constructivism");
        Ok(quote! {
            let param: &#lib::Param<_, _> = &fields.#ident;
            let field = param.field();
            let value = params.field(&field).define(param.value((#value).into()));
            let params = params + value;
        })
    }
}

#[derive(Clone)]
pub struct Params {
    pub items: Vec<Param>
}

impl Parse for Params {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Params { 
            items: input.parse_terminated(Param::parse, Token![,])?.into_iter().collect()
        })
    }
}

impl Params {
    pub fn build(&self, ctx: &Context) -> syn::Result<TokenStream> {
        let mut out = quote! { };
        for param in self.items.iter() {
            let param = param.build(ctx)?;
            out = quote! { #out #param }
        }
        Ok(out)
    }
    pub fn empty() -> Self {
        Params { items: vec![] }
    }
    pub fn parenthesized(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        content.parse()
    }

    pub fn braced(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        content.parse()
    }
}
pub struct Construct {
    pub ty: Type,
    pub flattern: bool,
    pub params: Params,
}

impl Parse for Construct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ty = input.parse()?;
        let mut flattern = true;
        if input.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            flattern = false;
        }
        let params = Params::braced(input)?;
        Ok(Construct { ty, flattern, params })
    }
}

// #[macro_export]
// macro_rules! construct {
//     ($t:ty { $($rest:tt)* } ) => {
//         {
//             use $crate::traits::*;
//             type Fields = <$t as $crate::Construct>::Fields;
//             let fields = <<$t as $crate::Construct>::Fields as $crate::Singleton>::instance();
//             let params = <<$t as $crate::Construct>::ExpandedParams as $crate::Extractable>::as_params();
//
//             // body here, see Param::build(..)
//
//             let defined_params = params.defined();
//             <$t as $crate::Construct>::construct(defined_params).flattern()
//         }
//     };
// }
impl Construct {
    pub fn build(&self, ctx: &Context) -> syn::Result<TokenStream> {
        let lib = ctx.path("constructivism");
        let ty = &self.ty;
        let flattern = if self.flattern {
            quote! { .flattern() }
        } else {
            quote! { }
        };
        let body = self.params.build(ctx)?;
        Ok(quote! {{
            use #lib::traits::*;
            let fields = <<#ty as #lib::Construct>::Fields as #lib::Singleton>::instance();
            let params = <<#ty as #lib::Construct>::ExpandedParams as #lib::Extractable>::as_params();
            #body
            let defined_params = params.defined();
            <#ty as #lib::Construct>::construct(defined_params)#flattern
        }})
    }
}