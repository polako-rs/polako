use std::collections::HashMap;

use quote::{quote, format_ident};
use proc_macro2::{token_stream, Ident, TokenStream};

use syn::{parse::Parse, Expr, Token, token, braced, bracketed, Lit};

macro_rules! throw {
    ($loc:expr, $msg:expr) => {
        return Err(syn::Error::new($loc.span(), $msg));
    };
}

pub struct EmlArgument {
    pub ident: Ident,
    pub value: Expr,
}

impl Parse for EmlArgument {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let value = input.parse()?;
        Ok(EmlArgument { ident, value })
    }
}

pub enum EmlChild {
    Literal(String),
    Node(EmlNode)
}

impl Parse for EmlChild {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Ok(lit) = input.parse::<Lit>() {
            if let Lit::Str(val) = lit {
                Ok(EmlChild::Literal(val.value()))
            } else {
                throw!(lit, "Only string literals supported");
            }
        } else {
            Ok(EmlChild::Node(input.parse()?))
        }
    }
}

pub enum EmlChildren {
    Provided(Ident),
    Declared(Vec<EmlChild>),
}

pub enum EmlArguments {
    View(Ident),
    Declared(Vec<EmlArgument>),
}

pub struct EmlNode {
    pub tag: Ident,
    pub model: Option<Ident>,
    pub args: EmlArguments,
    pub children: EmlChildren,
}

impl EmlNode {
    pub fn fetch_models(&self, models: &mut HashMap<Ident, (Ident, bool)>, root: bool) -> syn::Result<()> {
        if let Some(model) = self.model.clone() {
            if models.contains_key(&model) {
                throw!(model, format!("Model {} already defined", model.to_string()));
            }

            models.insert(model, (self.tag.clone(), root));
        }
        if let EmlChildren::Declared(children) = &self.children {
            for child in children.iter() {
                if let EmlChild::Node(node) = child {
                    node.fetch_models(models, false)?
                }
            }
        }
        Ok(())
    }

    pub fn build(&self, cst: &TokenStream, eml: &TokenStream, as_root: bool) -> TokenStream {
        let tag = &self.tag;
        match &self.args {
            EmlArguments::View(ident) => quote! {
                world.entity_mut(#ident.entity).insert(#ident.view()).id()
            },
            EmlArguments::Declared(args) => {
                let children = match &self.children {
                    EmlChildren::Provided(ident) => quote!{ #ident },
                    EmlChildren::Declared(children) => {
                        let size = children.len();
                        let mut chs = quote! { };
                        for child in children.iter() {
                            chs = match child {
                                EmlChild::Literal(lit) => quote! { #chs
                                    <<#tag as #eml::Element>::PushText as #eml::PushText>::push_text(world, &mut e_children, #lit);
                                },
                                EmlChild::Node(ch) => {
                                    let ch = ch.build(cst, eml, false);
                                    quote! { #chs
                                        e_children.push({ #ch });
                                    }
                                }
                            }
                        }
                        quote! { 
                            {
                                let mut e_children = ::std::vec::Vec::<_>::new();
                                e_children.reserve(#size);
                                #chs
                                e_children
                            }
                        }
                    }
                };
                let mut build = quote! { };
                for arg in args.iter() {
                    let ident = &arg.ident;
                    let value = &arg.value;
                    build = quote! { #build #ident: #value, };
                }
                let fetch_model = if let Some(model) = &self.model {
                    quote! {
                        {
                            world.entity_mut(#model.entity).insert(e_bundle);
                            #model
                        }
                    }
                } else if as_root {
                    quote! {
                        {
                            world.entity_mut(this).insert(e_bundle);
                            Model::<#tag>::new(this)
                        }
                    }
                } else {
                    quote! {
                        Model::<#tag>::new(world.spawn(e_bundle).id())
                    }
                };

                quote! {
                    let e_bundle = #cst::construct!(#tag { #build });
                    let e_model = #fetch_model;
                    let e_content = { #children };
                    <<#tag as #eml::Element>::Install as #eml::InstallElement>::install(world, e_model, e_content);
                    e_model.entity
                }
            }
        }
    }
}

impl Parse for EmlNode {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let tag = input.parse()?;
        let mut model = None;
        if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            model = Some(input.parse()?);
        }
        let args = if input.peek(token::Brace) {
            let content;
            braced!(content in input);
            if content.peek(token::Brace) {
                let content2;
                braced!(content2 in content);
                EmlArguments::View(content2.parse()?)
            } else {
                let mut args = vec![];
                for arg in content.parse_terminated(EmlArgument::parse, Token![,])? {
                    args.push(arg)
                }
                EmlArguments::Declared(args)
            }
        } else {
            EmlArguments::Declared(vec![])
        };
        let children = if input.peek(token::Bracket) {
            let content;
            bracketed!(content in input);
            if content.peek(token::Bracket) {
                let content2;
                bracketed!(content2 in content);
                EmlChildren::Provided(content2.parse()?)
            } else {
                let mut children = vec![];
                for child in content.parse_terminated(EmlChild::parse, Token![,])? {
                    children.push(child);
                }
                EmlChildren::Declared(children)
            }
        } else {
            EmlChildren::Declared(vec![])
        };
        Ok(EmlNode { tag, model, args, children })

    }
}

pub struct Model {
    pub ident: Ident,
    pub ty: Ident,
}

pub struct Eml {
    pub roots: Vec<EmlNode>
}

impl Parse for Eml {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut roots = vec![];
        for root in input.parse_terminated(EmlNode::parse, Token![;])? {
            roots.push(root)
        }
        Ok(Eml { roots })
    }
}

impl Eml {
    pub fn fetch_models(&self) -> syn::Result<HashMap<Ident, (Ident, bool)>> {
        let mut models = HashMap::new();
        for root in self.roots.iter() {
            root.fetch_models(&mut models, true)?;
        } 
        Ok(models)
    }
    pub fn build(&self, cst: TokenStream, eml: TokenStream) -> syn::Result<TokenStream> {
        let bevy = quote!{ ::bevy::prelude };
        let mut body = quote! { };
        let models = self.fetch_models()?;
        for (model, (tag, is_root)) in models.iter() {
            if *is_root {
                body = quote! { #body
                    let #model: #eml::Model<#tag> = #eml::Model::new(this);
                }
            } else {
                body = quote! { #body
                    let #model = world.spawn_empty().id();
                    let #model: #eml::Model<#tag> = #eml::Model::new(#model);
                }
            }
        }
        for root in self.roots.iter() {
            let build = root.build(&cst, &eml, true);
            body = quote! { 
                #body
                #build;
            }
        }
        Ok(quote!{ 
            #eml::Eml::new(|world: &mut #bevy::World, this: #bevy::Entity| {
                #body
            })
        })
    }
}