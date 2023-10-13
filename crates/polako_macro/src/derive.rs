use constructivist::{prelude::*, throw, derive::Props};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, format_ident};
use syn::{DeriveInput, Type, spanned::Spanned, parse_quote, Field, Data};

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


pub struct DeriveSignal {
    ty: Type,
    args: Option<(DeriveConstruct, Vec<Field>)>,
}



impl DeriveSignal {
    pub fn from_derive(input: DeriveInput) -> syn::Result<Self> {
        let ident = input.ident.clone();
        let Data::Struct(data) = &input.data else {
            throw!(ident, "#[derive(Signal)] available only for structs with named fields.")
        };
        let fields = data.fields
            .iter()
            .filter(|f| f.ident.is_some() && &f.ident.as_ref().unwrap().to_string() != "entity")
            .cloned()
            .collect();
        let mut input = DeriveConstruct::from_derive(input)?;
        if !input.props.iter().any(|p| &p.ident.to_string() == "entity") {
            throw!(input.ty, "Missing required `entity` field");
        }
        input.props.retain(|p| &p.ident.to_string() != "entity");
        input.params.retain(|p| &p.name.to_string() != "entity");
        let ty = input.ty.clone();
        let signal_args = format_ident!("{}SignalArgs", ident);
        input.ty = parse_quote!(quote!{ #signal_args });
        let args = if input.props.is_empty() {
            None
        } else {
            Some((input, fields))
        };
        Ok(DeriveSignal { ty, args })
    }

    // fn build(&self, ctx: &Context) -> syn::Result<TokenStream> {
    //     let mut out = quote! { };
    //     let args_ty = if let Some(args) = &self.args {
            
    //         let argsy_ty = args.ty;
    //         quote! { #args_ty }
    //     } else {
    //         quote!{ () }
    //     }

    // }
}