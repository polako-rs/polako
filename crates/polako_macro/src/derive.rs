use constructivist::{prelude::*, throw};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, format_ident};
use syn::{DeriveInput, spanned::Spanned, Field, Data, Attribute, parse::Parse, Token, Type, parse_quote};

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

pub struct DeriveBehavior {
    ident: Ident,
    segment: DeriveSegment,
    signals: Vec<AssignedSignal>,
}

impl DeriveBehavior {
    pub fn from_derive(input: DeriveInput) -> syn::Result<Self> {
        Ok(Self {
            ident: input.ident.clone(),
            signals: AssignedSignal::from_derive(&input)?,
            segment: DeriveSegment::from_derive(input)?,
        })
    }
    pub fn mod_ident(&self) -> syn::Result<Ident> {
        Ok(Ident::new(&format!("{}_behavior", self.ident.to_string().to_lowercase()), self.ident.span()))
    }
    pub fn build(&self, ctx: &EmlContext) -> syn::Result<TokenStream> {
        let cst = ctx.constructivism();
        let eml = ctx.path("eml");
        let segment = self.segment.build(ctx)?;
        let ident = &self.ident;
        let mod_ident = self.mod_ident()?;
        let mut signals = quote! {};
        for signal in self.signals.iter() {
            let signal = signal.build_getter(ctx)?;
            signals = quote! { #signals #signal }
        }
        Ok(quote! {
            #segment
            impl #eml::IntoBundle for #ident {
                type Output = Self;
                fn into_bundle(self) -> Self::Output {
                    self
                }
            }
            impl #eml::Behavior for #ident {
                type Signals<T: #cst::Singleton + 'static> = #mod_ident::Signals<T>;
            }

            mod #mod_ident {
                use super::*;
                pub struct Signals<T>(::std::marker::PhantomData<T>);
                impl<T: #cst::Singleton + 'static> ::std::ops::Deref for Signals<T> {
                    type Target = T;
                    fn deref(&self) -> &Self::Target {
                        T::instance()
                    }
                }
                impl<T: #cst::Singleton + 'static> #cst::Singleton for Signals<T> {
                    fn instance() -> &'static Signals<T> {
                        &Signals(::std::marker::PhantomData)
                    }
                }
                impl<T: #cst::Singleton + 'static> Signals<T> {
                    #signals
                }

            }
        })
    }
    pub fn build_from_derive(input: DeriveInput) -> syn::Result<TokenStream> {
        let input = Self::from_derive(input)?;
        let ctx = EmlContext::new("polako");
        input.build(&ctx)
    }
}

pub struct AssignedSignal {
    pub docs: Vec<Attribute>,
    pub ident: Ident,
    pub ty: Type,
}

impl AssignedSignal {
    fn parse_terminated(input: syn::parse::ParseStream) -> syn::Result<Vec<Self>> {
        Ok(input.parse_terminated(AssignedSignal::parse, Token![,])?.into_iter().collect())
    }

    pub fn from_derive(input: &DeriveInput) -> syn::Result<Vec<Self>> {
        Ok(if let Some(attr) = input.attrs
            .iter()
            .find(|a| a.path().is_ident("signals"))
        {
            attr
                .parse_args_with(AssignedSignal::parse_terminated)?
                .into_iter()
                .collect()
        } else {
            vec![]
        })

    }

    pub fn build_getter(&self, ctx: &EmlContext) -> syn::Result<TokenStream> {
        let cst = ctx.constructivism();
        let flow = ctx.path("flow");
        let ident = &self.ident;
        let ty = &self.ty;
        Ok(quote! {
            pub fn #ident(&self) -> &'static <#ty as #flow::Signal>::Descriptor {
                <<#ty as #flow::Signal>::Descriptor as #cst::Singleton>::instance()
            }
        })
    }
}

impl Parse for AssignedSignal {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let docs = Attribute::parse_outer(input)?
            .into_iter()
            .filter(|a| a.path().is_ident("doc"))
            .collect();
        let ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        Ok(AssignedSignal { docs, ident, ty })
    }
}

pub struct DeriveElement {
    pub ident: Ident,
    pub construct: DeriveConstruct,
    pub signals: Vec<AssignedSignal>,
}

impl DeriveElement {
    pub fn mod_ident(&self) -> syn::Result<Ident> {
        Ok(Ident::new(&format!("{}_element", self.ident.to_string().to_lowercase()), self.ident.span()))
    }

    pub fn from_derive(input: DeriveInput) -> syn::Result<Self> {
        let signals = AssignedSignal::from_derive(&input)?;
        Ok(Self {
            ident: input.ident.clone(),
            construct: DeriveConstruct::from_derive(input)?,
            signals,
        })
    }
    pub fn build(&self, ctx: &EmlContext) -> syn::Result<TokenStream> {
        let cst = ctx.constructivism();
        let eml = ctx.path("eml");
        let construct = self.construct.build(&ctx.context)?;
        let ident = &self.ident;
        let mod_construct = self.construct.mod_ident()?;
        let mod_element = self.mod_ident()?;
        let mut signals = quote! {};
        let design = self.construct.design_ident()?;
        for signal in self.signals.iter() {
            let signal = signal.build_getter(ctx)?;
            signals = quote! { #signals #signal };
        }
        let base = &self.construct.sequence.next;
        let mut signals_base = quote! { <#base as #eml::Element>::Signals };
        for seg in self.construct.sequence.segments.iter() {
            signals_base = quote! { <#seg as #eml::Behavior>::Signals<#signals_base> }
        };
        Ok(quote! {
            #construct
            impl ::bevy::ecs::component::Component for #ident {
                type Storage = ::bevy::ecs::component::TableStorage;
            }
            impl #eml::Element for #ident {
                type Signals = #mod_element::Signals;

            }

            impl #mod_construct::Props<#cst::Describe> {
                #signals
            }


            impl #design {
                pub fn on(&self) -> &'static #mod_element::Signals {
                    &#mod_element::Signals
                }
            }

            mod #mod_element {
                use super::*;
                pub struct Signals;
                impl #cst::Singleton for Signals {
                    fn instance() -> &'static Signals {
                        &Signals
                    }
                }
                impl ::std::ops::Deref for Signals {
                    type Target = #signals_base;
                    fn deref(&self) -> &Self::Target {
                        <#signals_base as #cst::Singleton>::instance()
                    }
                }
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
        let args = format_ident!("{}Signal", ident);
        input.ty = parse_quote!(#args);
        input.sequence.this = input.ty.clone();
        input.props.retain(|p| &p.ident.to_string() != "entity");
        input.params.retain(|p| &p.name.to_string() != "entity");
        let args = if input.props.is_empty() {
            None
        } else {
            Some((input, fields))
        };
        Ok(DeriveSignal { ident, args })
    }

    pub fn build(&self, ctx: &EmlContext) -> syn::Result<TokenStream> {
        let cst = ctx.constructivism();
        let flow = ctx.path("flow");
        let ident = &self.ident;
        let descriptor = format_ident!("{}Descriptor", ident);
        let (args_ty, args_body, args_def) = if let Some((args, fields)) = &self.args {
            let args_ty = &args.ty;
            let mut body = quote! { };
            let mut def = quote! { };
            for field in fields.iter() {
                let Some(ident) = &field.ident else {
                    throw!(field, "Only named fields supported");
                };
                let ty = &field.ty;
                body = quote! { #body #ident: args.#ident, };
                def = quote! { #def #ident: #ty, };
            }
            (
                quote! { #args_ty },
                body,
                def,
            )
        } else {
            (
                quote!{ () },
                quote! { },
                quote! { },
            )
        };
        let impl_args = if args_def.is_empty() {
            quote! { }
        } else {
            let args_construct = self.args.as_ref().unwrap().0.build(&ctx.context)?;
            quote! {
                pub struct #args_ty {
                    #args_def
                }
                #args_construct
            }
        };
        Ok(quote! {
            impl #flow::Signal for #ident {
                type Event = Self;
                type Args = #args_ty;
                type Descriptor = #descriptor;
                fn filter(event: &Self::Event) -> Option<::bevy::prelude::Entity> {
                    Some(event.entity)
                }
            }
            impl ::bevy::prelude::Event for #ident {

            }
            #impl_args
            pub struct #descriptor;
            impl  #cst::Singleton for #descriptor {
                fn instance() -> &'static Self {
                    &#descriptor
                }
            }
            impl #descriptor {
                pub fn emit(
                    &self,
                    world: &mut ::bevy::prelude::World,
                    entity: ::bevy::prelude::Entity,
                    args: <#ident as #flow::Signal>::Args
                ) {
                    let event = #ident {
                        entity,
                        #args_body
                    };
                    world.resource_mut::<::bevy::prelude::Events<#ident>>().send(event);
                }

                pub fn assign<'w, S: ::bevy::ecs::system::SystemParam>(
                    &self,
                    entity: &mut ::bevy::ecs::world::EntityMut<'w>,
                    value: #flow::Hand<#ident, S>,
                ) {
                    
                }
            }
        })

    }

    pub fn build_from_derive(input: DeriveInput) -> syn::Result<TokenStream> {
        let input = Self::from_derive(input)?;
        let ctx = EmlContext::new("polako");
        input.build(&ctx)
    }
}