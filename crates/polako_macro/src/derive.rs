use constructivist::{prelude::*, throw, derive::Props};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, format_ident};
use syn::{DeriveInput, Type, spanned::Spanned, parse_quote, Field, Data};

use crate::eml::EmlContext;

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

    pub fn build(&self, ctx: &EmlContext) -> syn::Result<TokenStream> {
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
        let ctx = EmlContext::new("polako");
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
    pub fn build(&self, ctx: &EmlContext) -> syn::Result<TokenStream> {
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
        let ctx = EmlContext::new("polako");
        input.build(&ctx)
    }
}


pub struct DeriveSignal {
    ident: Ident,
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
        let args = if input.props.is_empty() {
            None
        } else {
            Some((input, fields))
        };
        Ok(DeriveSignal { ident, args })
    }

    fn build(&self, ctx: &EmlContext) -> syn::Result<TokenStream> {
        let cst = ctx.constructivism();
        let flow = ctx.path("flow");
        let bevy = ctx.path("bevy");
        let ident = &self.ident;
        let descriptor = format_ident!("{}SignalDescriptor", ident);
        let mut out = quote! { };
        let (args_ty, args_body) = if let Some((args, fields)) = &self.args {
            let args_ty = &args.ty;
            let mut body = quote! { };
            for field in fields.iter() {
                let Some(ident) = &field.ident else {
                    throw!(field, "Only named fields supported");
                };
                body = quote! { #body #ident = args.#ident, };
            }
            (
                quote! { #args_ty },
                body,
            )
        } else {
            (
                quote!{ () },
                quote! { },
            )
        };
        Ok(quote! {
            impl #flow::Signal for #ident {
                type Event = Self;
                type Args = #args_ty;
                type Descriptor = #descriptor;
                fn filter(event: &Self::Event) -> Option<#bevy::prelude::Entity> {
                    Some(event.entity)
                }
            }
            pub struct #descriptor;
            impl  #cst::Singleton for #descriptor {
                fn instance() -> &'static Self {
                    &#descriptor
                }
            }
            impl #descriptor {
                pub fn emit(
                    &self,
                    world: &mut #bevy::World,
                    entity: #bevy::prelude::Entity,
                    args: <$name as #flow::Signal>::Args
                ) {
                    let event = Self {
                        entity,
                        #args_body
                    };
                    world.resource_mut::<#bevy::prelude::Events<Self::Event>>().send(event);
                }

                pub fn assign<'w, S: #bevy::ecs::SystemParam>(
                    &self,
                    entity: #bevy::ecs::world::EntityMut<'w>,
                    value: Hand<#ident, #flow::Handler<S>>,
                ) {
                    
                }
            }
        })

    }
}