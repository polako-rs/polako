use constructivist::{proc::{Value, ContextLike, Ref}, throw};
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::quote;
use syn::{
    braced,
    parse::Parse,
    spanned::Spanned,
    token::{Brace, Bracket, Paren},
    Expr, Ident, Token,
};

use crate::{hand::Hand, eml::EmlContext};

#[derive(Clone)]
pub enum Variant {
    Prop(Vec<Ident>),
    Color(Color),
    Hand(Hand),
    Expr(Expr),
}
impl Parse for Variant {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Paren) {
            Ok(Variant::Hand(input.parse()?))
        } else if input.peek(Token![#]) && !input.peek2(Bracket) {
            Ok(Variant::Color(input.parse()?))
        } else if input.peek(Brace) {
            let outer;
            braced!(outer in input);
            if outer.peek(Brace) {
                let inner;
                braced!(inner in outer);
                Ok(Variant::Prop(
                    inner
                        .parse_terminated(Ident::parse, Token![.])?
                        .into_iter()
                        .collect(),
                ))
            } else {
                Ok(Variant::Expr(outer.parse()?))
            }
        } else {
            Ok(Variant::Expr(input.parse()?))
        }
    }
}

impl ContextLike for EmlContext {
    fn path(&self, name: &'static str) -> TokenStream {
        self.context.path(name)
    }
}
impl Value for Variant {
    type Context = EmlContext;
    fn build(item: &Self, ctx: Ref<EmlContext>) -> syn::Result<TokenStream> {
        Ok(match item {
            Variant::Expr(e) => quote! { #e },
            Variant::Color(c) => c.build(ctx.clone())?,
            Variant::Hand(h) => h.build(ctx)?,
            Variant::Prop(_) => quote! {},
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    span: Span,
    value: usize,
    digits: u8,
}

impl Color {
    pub fn span(&self) -> Span {
        self.span.clone()
    }
    pub fn from_span(span: Span) -> Self {
        Self {
            span,
            value: 0,
            digits: 0,
        }
    }
    fn push_digit(&mut self, mut digit: usize) {
        if digit > 15 {
            digit = 15;
        }
        self.value <<= 4;
        self.value |= digit;
        self.digits += 1;
    }
    pub fn is_complete(&self) -> bool {
        self.digits >= 8
    }
    pub fn rgba(&self) -> syn::Result<(f32, f32, f32, f32)> {
        match self.digits {
            // rgb, alpha = 1.0
            3 => Ok((
                ((self.value & 0x0f00) >> 08) as f32 / 15.,
                ((self.value & 0x00f0) >> 04) as f32 / 15.,
                ((self.value & 0x000f) >> 00) as f32 / 15.,
                1.,
            )),

            // rgba
            4 => Ok((
                ((self.value & 0xf000) >> 12) as f32 / 15.,
                ((self.value & 0x0f00) >> 08) as f32 / 15.,
                ((self.value & 0x00f0) >> 04) as f32 / 15.,
                ((self.value & 0x000f) >> 00) as f32 / 15.,
            )),

            // RR/GG/BB, alpha = 1.0
            6 => Ok((
                ((self.value & 0x00ff0000) >> 16) as f32 / 255.,
                ((self.value & 0x0000ff00) >> 08) as f32 / 255.,
                ((self.value & 0x000000ff) >> 00) as f32 / 255.,
                1.,
            )),

            // rrggbbaa
            8 => Ok((
                ((self.value & 0xff000000) >> 24) as f32 / 255.,
                ((self.value & 0x00ff0000) >> 16) as f32 / 255.,
                ((self.value & 0x0000ff00) >> 08) as f32 / 255.,
                ((self.value & 0x000000ff) >> 00) as f32 / 255.,
            )),

            _ => {
                throw!(
                    self,
                    "Color is supposed to consists of 3, 4, 6 or 8 digits in total."
                );
            }
        }
    }
    pub fn build(&self, ctx: Ref<EmlContext>) -> syn::Result<TokenStream> {
        let (r, g, b, a) = self.rgba()?;
        let bevy = ctx.path("bevy");
        Ok(quote!(#bevy::prelude::Color::rgba(#r, #g, #b, #a)))
    }
}

impl Parse for Color {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let hash = input.parse::<Token![#]>()?;
        input.step(|cursor| {
            let mut rest = *cursor;
            let mut color = Color::from_span(hash.span());
            while let Some((tt, next)) = rest.token_tree() {
                if let TokenTree::Punct(_) = &tt {
                    return Ok((color, rest));
                }
                let repr = tt.to_string().to_lowercase();
                for ch in repr.chars().into_iter() {
                    match ch {
                        _ if color.is_complete() => {
                            throw!(
                                hash,
                                "Color is supposed to consists of 3, 4, 6 or 8 digits in total."
                            );
                        }
                        ' ' => {}
                        '0' => color.push_digit(0x0),
                        '1' => color.push_digit(0x1),
                        '2' => color.push_digit(0x2),
                        '3' => color.push_digit(0x3),
                        '4' => color.push_digit(0x4),
                        '5' => color.push_digit(0x5),
                        '6' => color.push_digit(0x6),
                        '7' => color.push_digit(0x7),
                        '8' => color.push_digit(0x8),
                        '9' => color.push_digit(0x9),
                        'a' => color.push_digit(0xa),
                        'b' => color.push_digit(0xb),
                        'c' => color.push_digit(0xc),
                        'd' => color.push_digit(0xd),
                        'e' => color.push_digit(0xe),
                        'f' => color.push_digit(0xf),
                        ch => {
                            throw!(input, "Unexpected Color digit: '{}'.", ch);
                        }
                    }
                }
                if color.is_complete() {
                    return Ok((color, next));
                } else {
                    rest = next;
                }
            }
            if ![3, 4, 6, 8].contains(&color.digits) {
                throw!(
                    hash,
                    "Color is supposed to consists of 3, 4, 6 or 8 digits in total."
                );
            }
            Ok((color, rest))
        })
    }
}