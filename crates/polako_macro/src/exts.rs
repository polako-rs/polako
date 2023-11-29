use crate::eml::EmlContext;
use constructivist::throw;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseBuffer},
    spanned::Spanned,
    token, Expr, Lit, LitStr, Token,
};

pub trait PeekBindDirection {
    fn peek_bind_direction(&self) -> bool;
}

impl<'a> PeekBindDirection for ParseBuffer<'a> {
    fn peek_bind_direction(&self) -> bool {
        self.peek(Token![=>])
            || self.peek(Token![<=])
            || (self.peek(Token![<=]) && self.peek2(Token![=>]))
    }
}

pub enum PropMapKind {
    None,
    Call,
    Mult,
    Div,
    Add,
    Sub,
}

pub trait PeekPropMap {
    fn peek_prop_map(&self) -> PropMapKind;
}

impl<'a> PeekPropMap for ParseBuffer<'a> {
    fn peek_prop_map(&self) -> PropMapKind {
        if self.peek(syn::Ident) && self.peek2(token::Paren) {
            PropMapKind::Call
        } else if self.peek(syn::Ident) && self.peek2(Token![*]) {
            PropMapKind::Mult
        } else if self.peek(syn::Ident) && self.peek2(Token![/]) {
            PropMapKind::Div
        } else if self.peek(syn::Ident) && self.peek2(Token![+]) {
            PropMapKind::Add
        } else if self.peek(syn::Ident) && self.peek2(Token![-]) {
            PropMapKind::Sub
        } else {
            PropMapKind::None
        }
    }
}

pub trait ParsePropMap {
    fn parse_prop_map(&self, path: &mut Vec<Ident>) -> syn::Result<Option<BindMap>>;
}

impl<'a> ParsePropMap for ParseBuffer<'a> {
    fn parse_prop_map(&self, path: &mut Vec<Ident>) -> syn::Result<Option<BindMap>> {
        Ok(match self.peek_prop_map() {
            PropMapKind::Call => Some(self.parse()?),
            PropMapKind::Mult | PropMapKind::Div | PropMapKind::Add | PropMapKind::Sub => {
                path.push(self.parse()?);
                Some(self.parse()?)
            }
            PropMapKind::None => None,
        })
    }
}

pub enum BindMap {
    Format(LitStr, Vec<Lit>),
    Custom(TokenStream),
}

impl BindMap {
    pub fn build(&self, _: &EmlContext) -> syn::Result<TokenStream> {
        Ok(match self {
            BindMap::Format(f, args) => {
                let mut fargs = quote! { #f, s, };
                for arg in args.iter() {
                    fargs = quote! { #fargs #arg, };
                }
                quote! { |s| format!(#fargs)  }
            }
            BindMap::Custom(c) => c.clone(),
        })
    }
    pub fn span(&self) -> Span {
        match self {
            BindMap::Format(f, _) => f.span(),
            BindMap::Custom(c) => c.span(),
        }
    }
}

impl Parse for BindMap {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // parse time.elapsed_seconds * 0.5
        //                            ------
        if input.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            let expr = input.parse::<Expr>()?;
            return Ok(BindMap::Custom(quote! {
                |v| v * #expr
            }));
        }

        // parse time.elapsed_seconds / 0.5
        //                            ------
        if input.peek(Token![/]) {
            input.parse::<Token![/]>()?;
            let expr = input.parse::<Expr>()?;
            return Ok(BindMap::Custom(quote! {
                |v| v / #expr
            }));
        }

        // parse time.elapsed_seconds + 0.5
        //                            ------
        if input.peek(Token![+]) {
            input.parse::<Token![+]>()?;
            let expr = input.parse::<Expr>()?;
            return Ok(BindMap::Custom(quote! {
                |v| v + #expr
            }));
        }

        // parse time.elapsed_seconds - 0.5
        //                            ------
        if input.peek(Token![-]) {
            input.parse::<Token![-]>()?;
            let expr = input.parse::<Expr>()?;
            return Ok(BindMap::Custom(quote! {
                |v| v - #expr
            }));
        }

        // parse time.elapsed_seconds.fmt("{}")
        //                            ---------
        let ident = input.parse::<Ident>()?;
        if &ident.to_string() != "fmt" {
            throw!(ident, "Expected fmt ident");
        }
        let content;
        parenthesized!(content in input);
        let fmt = content.parse()?;
        let mut args = vec![];
        if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
        }
        while !content.is_empty() {
            args.push(content.parse()?);
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }
        Ok(BindMap::Format(fmt, args))
    }
}
