use constructivist::prelude::*;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::DeriveInput;

pub struct DeriveConstraint {
    pub ident: Ident,
    pub segment: DeriveSegment,
}

impl DeriveConstraint {
    pub fn from_derive(input: DeriveInput) -> syn::Result<Self> {
        Ok(Self {
            ident: input.ident.clone(),
            segment: DeriveSegment::from_derive(input)?,
        })
    }

    pub fn build(&self, ctx: &Context) -> syn::Result<TokenStream> {
        let segment = self.segment.build(ctx)?;
        let ident = &self.ident;
        let eml = ctx.path("eml");
        Ok(quote! {
            #segment
            impl #eml::IntoBundle for #ident {
                type Output = ();
                fn into_bundle(self) -> Self::Output {
                    ()
                }
            }
            impl #eml::Constraint for #ident { }
        })
    }

    pub fn build_from_derive(input: DeriveInput) -> syn::Result<TokenStream> {
        let input = Self::from_derive(input)?;
        let ctx = Context::new("polako");
        input.build(&ctx)
    }
}

pub struct DeriveBehaviour {
    ident: Ident,
    segment: DeriveSegment,
}

impl DeriveBehaviour {
    pub fn from_derive(input: DeriveInput) -> syn::Result<Self> {
        Ok(Self {
            ident: input.ident.clone(),
            segment: DeriveSegment::from_derive(input)?,
        })
    }
    pub fn build(&self, ctx: &Context) -> syn::Result<TokenStream> {
        let segment = self.segment.build(ctx)?;
        let ident = &self.ident;
        let eml = ctx.path("eml");
        Ok(quote! {
            #segment
            impl #eml::IntoBundle for #ident {
                type Output = Self;
                fn into_bundle(self) -> Self::Output {
                    self
                }
            }
            impl #eml::Behaviour for #ident {

            }
        })
    }
    pub fn build_from_derive(input: DeriveInput) -> syn::Result<TokenStream> {
        let input = Self::from_derive(input)?;
        let ctx = Context::new("polako");
        input.build(&ctx)
    }
}
