use std::collections::HashMap;

use quote::{quote, format_ident, quote_spanned};
use proc_macro2::{token_stream, Ident, TokenStream, Span};

use syn::{parse::Parse, Expr, Token, token, braced, bracketed, Lit, spanned::Spanned, LitStr, parenthesized};

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
        let value = if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            input.parse()?
        } else {
            syn::parse2(quote! { true })?
        };
        Ok(EmlArgument { ident, value })
    }
}

pub enum EmlChild {
    Literal(LitStr),
    Node(EmlNode)
}

impl Parse for EmlChild {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Ok(lit) = input.parse::<Lit>() {
            if let Lit::Str(val) = lit {
                Ok(EmlChild::Literal(val.clone()))
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
    InstallView(Ident),
    View(Ident),
    Declared(Vec<EmlArgument>),
}

impl EmlArguments {
    pub fn is_view(&self) -> bool {
        match self {
            Self::View(_) => true,
            _ => false
        }
    }
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
                throw!(model, format!("EntityComponent {} already defined", model.to_string()));
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

    pub fn build(&self, cst: &TokenStream, eml: &TokenStream, as_root: bool, strict: bool) -> syn::Result<TokenStream> {
        let tag = &self.tag;
        let children = match &self.children {
            EmlChildren::Provided(ident) => quote!{ #ident },
            EmlChildren::Declared(children) => {
                let size = children.len();
                let mut chs = quote! { };
                for child in children.iter() {
                    chs = match child {
                        EmlChild::Literal(lit) => {
                            let assign = quote_spanned!{ lit.span()=> 
                                let _: Valid<()> = <<#tag as #cst::Construct>::Protocols as #cst::Singleton>::instance().push_text(world, &mut e_children, #lit);
                            };
                            quote! { #chs #assign }
                                
                        },
                        EmlChild::Node(ch) => {
                            let span = ch.tag.span();
                            let ch = ch.build(cst, eml, false, strict)?;
                            let assign = quote_spanned!{ span=>
                                let _: Valid<()> = <<#tag as #cst::Construct>::Protocols as #cst::Singleton>::instance().push_model(world, &mut e_children, e_child);
                            };
                            quote! { #chs 
                                let e_child = { #ch };
                                #assign 
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
        let model = match &self.args {
            // make model-view
            // build! {
            //     Div(model)
            // }
            EmlArguments::InstallView(view) => if strict && as_root {
                quote! {{
                    let e_view = #view.as_view();
                    let mut e_model = #view.as_model();
                    e_model.for_view = __root__;
                    world.entity_mut(#view.entity).insert((e_model, e_view));
                    (#view.into_base(), __root__)
               }}
            } else {
                throw!(tag, "InstallView only supported on the root element of build! macro.");
            },
            // model bypassing:
            // build! { 
            //     Div [
            //         Label {{ model }}
            //     ]
            // }
            EmlArguments::View(model) => if strict && !as_root {
                 quote! {{
                    let e_this = world.spawn_empty().id();
                    (#model.into_base(), e_this)
                }}
            
            } else {
                throw!(tag, "Model bypassing only supported for nont-root elements in build! macro.");
            },
            // just do nothing for the build! empty root
            EmlArguments::Declared(v) if v.is_empty() && strict && as_root => quote! {{
                let e_model = #eml::EntityComponent::new(__root__);
                (e_model, __root__)                
            }},
            EmlArguments::Declared(_) if strict && as_root => {
                throw!(tag, "Root element of the builder constructed by outer eml");
            },
            EmlArguments::Declared(args) => {
                let mut build = quote! { };
                for arg in args.iter() {
                    let ident = &arg.ident;
                    let value = &arg.value;
                    build = quote! { #build #ident: #value, };
                }
                build = quote! { #cst::construct!(#tag { #build }) };
                if let Some(model) = &self.model {
                    quote! {
                        {

                            world.entity_mut(#model.entity)
                                .insert(#build);
                                // .insert((
                                //     #eml::Model::<#tag>::new(#model.entity),
                                //     #eml::View::<#tag>::new(#model.entity)
                                // ))
                            (#model, #model.entity)
                        }
                    }
                } else if as_root {
                    quote! {
                        {
                            world.entity_mut(__root__).insert(#build);
                            (EntityComponent::<#tag>::new(__root__), __root__)
                        }
                    }
                } else {
                    quote! {
                        {
                            let e_model = EntityComponent::<#tag>::new(world.spawn(#build).id());
                            (e_model, e_model.entity)
                        }
                    }
                }
            }
        };
        Ok(quote_spanned! {self.tag.span()=>
            let (e_model, __this__) = #model;
            let e_content = { #children };
            <<#tag as #eml::Element>::Build as #eml::Build>::build(world, __this__, e_model, e_content);
            #eml::EntityComponent::<#tag>::new(__this__)
        })
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
        let args = if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            EmlArguments::InstallView(content.parse()?)
        } else if input.peek(token::Brace) {
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

pub struct EntityComponent {
    pub ident: Ident,
    pub ty: Ident,
}

pub struct Eml {
    pub span: Span,
    pub strict: bool,
    pub roots: Vec<EmlNode>
}

impl Parse for Eml {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut roots = vec![];
        let span = input.span();
        for root in input.parse_terminated(EmlNode::parse, Token![;])? {
            roots.push(root)
        }
        Ok(Eml { roots, span, strict: false })
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
        let mut root_ty = None;
        for (model, (tag, is_root)) in models.iter() {
            if *is_root {
                body = quote! { #body
                    let #model: #eml::EntityComponent<#tag> = #eml::EntityComponent::new(__root__);
                }
            } else {
                body = quote! { #body
                    let #model = world.spawn_empty().id();
                    let #model: #eml::EntityComponent<#tag> = #eml::EntityComponent::new(#model);
                }
            }
        }
        for root in self.roots.iter() {
            let build = root.build(&cst, &eml, true, self.strict)?;
            body = quote! { 
                #body
                #build;
            };
            root_ty = Some(root.tag.clone());
        }
        let Some(root_ty) = root_ty else {
            throw!(self.span, "Can't detect Eml exact type");
        };
        let body = quote!{ 
            #eml::Eml::<#root_ty>::new(move |world: &mut #bevy::World, __root__: #bevy::Entity| {
                let __this__ = __root__;
                #body
            })
        };
        Ok(if self.strict {
            quote! { #eml::Builder::new(#body) }
        } else {
            body
        })
    }
}