use std::collections::HashMap;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use constructivist::{construct::*, context::Context};

use syn::{
    bracketed, parse::Parse, spanned::Spanned, token, Lit, LitStr,
    Token,
};

macro_rules! throw {
    ($loc:expr, $msg:expr) => {
        return Err(syn::Error::new($loc.span(), $msg));
    };
}

pub trait EmlParams {
    fn build_patch(&self, _: &EmlContext, tag: &Ident, this: &TokenStream, patch_empty: bool) -> syn::Result<TokenStream>;
    fn build_construct(&self, ctx: &EmlContext, tag: &Ident) -> syn::Result<TokenStream>;
}

impl EmlParams for Params {
    fn build_patch(&self, _: &EmlContext, tag: &Ident, this: &TokenStream, patch_empty: bool) -> syn::Result<TokenStream> {
        let mut body = quote! { };
        if !patch_empty && self.items.is_empty() {
            return Ok(body);
        }
        for arg in self.items.iter() {
            let ident = &arg.ident;
            let value = &arg.value;
            body = quote! { #body
                __component__.#ident = #value.into();
            }
        }
        Ok(quote! {{
            let mut __entity__ = world.entity_mut(#this);
            if !__entity__.contains::<#tag>() {
                __entity__.insert(#tag::default());
            }
            let mut __component__ = __entity__.get_mut::<#tag>().unwrap();
            #body
        }})
    }
    fn build_construct(&self, ctx: &EmlContext, tag: &Ident) -> syn::Result<TokenStream> {
        let construct = Construct {
            ty: syn::parse2(quote! { #tag })?,
            flattern: true,
            params: self.clone()
        };
        construct.build(&ctx.context)
    }
}

pub enum EmlChild {
    Literal(LitStr),
    Node(EmlNode),
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

pub enum EmlContent {
    Provided(Ident),
    Declared(Vec<EmlChild>),
}

impl EmlContent {
    pub fn build(&self, ctx: &EmlContext, tag: &Ident) -> syn::Result<TokenStream> {
        let cst = &ctx.path("constructivism");
        Ok(match self {
            EmlContent::Provided(ident) => quote! { #ident },
            EmlContent::Declared(children) => {
                let size = children.len();
                let mut content = quote! {};
                for child in children.iter() {
                    content = match child {
                        EmlChild::Literal(lit) => {
                            let assign = quote_spanned! { lit.span()=>
                                let _: Implemented =
                                    <<#tag as #cst::Construct>::Protocols as #cst::Singleton>::instance()
                                        .push_text(world, &mut __content__, #lit);
                            };
                            quote! { #content #assign }
                        }
                        EmlChild::Node(ch) => {
                            let span = ch.tag.span();
                            let ct = ch.build(ctx, false)?;
                            let assign = quote_spanned! { span=>
                                let _: Implemented = 
                                    <<#tag as #cst::Construct>::Protocols as #cst::Singleton>::instance()
                                        .push_content(world, &mut __content__, __content_item__);
                            };
                            quote! { #content
                                let __content_item__ = { #ct };
                                #assign
                            }
                        }
                    }
                }
                quote! {
                    {
                        let mut __content__ = ::std::vec::Vec::<_>::new();
                        __content__.reserve(#size);
                        #content
                        __content__
                    }
                }
            }
        })

    }
}

impl Parse for EmlContent {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(token::Bracket) {
            let content;
            bracketed!(content in input);
            if content.peek(token::Bracket) {
                let content2;
                bracketed!(content2 in content);
                EmlContent::Provided(content2.parse()?)
            } else {
                let mut children = vec![];
                for child in content.parse_terminated(EmlChild::parse, Token![,])? {
                    children.push(child);
                }
                EmlContent::Declared(children)
            }
        } else {
            EmlContent::Declared(vec![])
        })
    }
}

// Patch component on current entity, everythinng after + in
// Div + Style(width: Val::Percent(100.))
pub struct EmlPatch {
    pub ident: Ident,
    pub items: Params,
}

impl Parse for EmlPatch {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let items = Params::parenthesized(input)?;
        Ok(EmlPatch { ident, items })
    }
}

impl EmlPatch {
    pub fn build(&self, ctx: &EmlContext, this: &TokenStream) -> syn::Result<TokenStream>{
        self.items.build_patch(ctx, &self.ident, this, true)
    }
}

// Add new component to the current entity, everything after ++ in
// Div + Style
pub struct EmlComponent {
    pub ident: Ident,
    pub items: Params,
}

impl Parse for EmlComponent {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        let items = if input.peek(token::Brace) {
            Params::braced(input)?
        } else {
            Params::empty()
        };
        Ok(EmlComponent { ident, items })
    }
}

impl EmlComponent {
    pub fn build(&self, ctx: &EmlContext, this: &TokenStream) -> syn::Result<TokenStream> {
        let construct = self.items.build_construct(ctx, &self.ident)?;
        Ok(quote! {
            world.entity_mut(#this).insert(#construct);
        })
    }
}

pub enum EmlMixin {
    Patch(EmlPatch),
    Component(EmlComponent),
}

impl Parse for EmlMixin {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.fork().parse::<EmlPatch>().is_ok() {
            Ok(EmlMixin::Patch(input.parse()?))
        } else if input.fork().parse::<EmlComponent>().is_ok() {
            Ok(EmlMixin::Component(input.parse()?))
        } else {
            throw!(input.span(), "Unexpected input");
        }
    }
}

pub struct EmlMixins(pub Vec<EmlMixin>);
impl Parse for EmlMixins {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut mixins = vec![];
        while input.peek(Token![+]) {
            input.parse::<Token![+]>()?;
            mixins.push(input.parse()?);
        }
        Ok(EmlMixins(mixins))
    }
}

impl EmlMixins {
    pub fn build(&self, ctx: &EmlContext, this: &TokenStream) -> syn::Result<TokenStream> {
        let mut out = quote! { };
        for mixin in self.0.iter() {
            out = match mixin {
                EmlMixin::Patch(patch) => {
                    let patch = patch.build(ctx, this)?;
                    quote! { #out #patch }
                },
                EmlMixin::Component(component) => {
                    let construct = component.build(ctx, this)?;
                    quote! { #out #construct }
                }
            };
        }
        Ok(out)
    }
}

pub enum EmlRoot {
    Element(EmlNode),
    Super {
        tag: Ident,
        overrides: Params,
        mixins: EmlMixins,
        children: EmlContent,
    },
}

impl Parse for EmlRoot {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Ident) && input.peek2(Token![:]) && input.peek3(Token![:]) {
            let tag = input.parse::<Ident>()?;
            input.parse::<Token![:]>()?;
            input.parse::<Token![:]>()?;
            let sup = input.parse::<Ident>()?;
            if &sup.to_string() != "Super" {
                throw!(sup, "Expected Super");
            }
            let overrides = if input.peek(token::Paren) {
                Params::parenthesized(input)?            
            } else {
                Params::empty()
            };
            let mixins = input.parse()?;
            let children = input.parse()?;
            Ok(EmlRoot::Super {
                tag,
                overrides,
                mixins,
                children,
            })
        } else {
            // throw!(node_input, format!("parsing node {}",node_input.to_string()) );
            Ok(EmlRoot::Element(input.parse()?))
        }
    }
}

pub struct EmlContext {
    context: Context,
    strict: bool,
}

impl std::ops::Deref for EmlContext {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl EmlRoot {
    pub fn tag(&self) -> Ident {
        match self {
            EmlRoot::Element(elem) => elem.tag.clone(),
            EmlRoot::Super { tag, .. } => tag.clone(),
        }
    }
    pub fn fetch_models(
        &self,
        models: &mut HashMap<Ident, (Ident, bool)>,
    ) -> syn::Result<()> {
        match self {
            EmlRoot::Element(node) => node.fetch_models(models, true),
            EmlRoot::Super { children: EmlContent::Declared(items), .. } => {
                for item in items.iter() {
                    if let EmlChild::Node(node) = item {
                        node.fetch_models(models, false)?
                    }
                }
                Ok(())
            },
            _ => Ok(())
        }
    }
    pub fn build(
        &self,
        ctx: &EmlContext,
    ) -> syn::Result<TokenStream> {
        match self {
            EmlRoot::Super { tag, overrides, mixins, children } => {
                if !ctx.strict {
                    throw!(tag, "Tag::Super only available as root inside the build! macro.");
                }
                self.build_super(ctx, tag, overrides, mixins, children)
            },
            EmlRoot::Element(node) => {
                if ctx.strict {
                    throw!(node.tag, "Only Tag::Super available as root inside the build! macro.");
                }
                let eml = &ctx.path("eml");
                let body = node.build(ctx, true)?;
                let tag = &node.tag;
                Ok(quote! { 
                    let __root_model__ = #eml::Model::<#tag>::new(__root__);
                    #body
                })
            }
        }
    }

    fn build_super( &self,
        ctx: &EmlContext,
        tag: &Ident,
        overrides: &Params,
        mixins: &EmlMixins,
        content: &EmlContent,
    ) -> syn::Result<TokenStream> {
        let eml = &ctx.path("eml");
        let cst = &ctx.path("constructivism");
        let build_content = content.build(ctx, tag)?;
        let apply_patches = overrides.build_patch(ctx, tag, &quote! { __root__ }, false)?;
        let apply_mixins = mixins.build(ctx, &quote! { __root__ })?;

        Ok(quote!{
            let __root_model__ = #eml::Model::<#tag>::new(__root__);
            #apply_patches;
            <<#tag as #cst::Construct>::Extends as #eml::Element>::build_element(#build_content)
                .eml()
                .write(world, __root__);
            #apply_mixins
        })
    }
}

pub struct EmlNode {
    pub tag: Ident,
    pub model: Option<Ident>,
    pub args: Params,
    pub mixins: EmlMixins,
    pub children: EmlContent,
}

impl EmlNode {
    pub fn fetch_models(
        &self,
        models: &mut HashMap<Ident, (Ident, bool)>,
        root: bool,
    ) -> syn::Result<()> {
        if let Some(model) = self.model.clone() {
            if models.contains_key(&model) {
                throw!(
                    model,
                    format!("Model {} already defined", model.to_string())
                );
            }

            models.insert(model, (self.tag.clone(), root));
        }
        if let EmlContent::Declared(children) = &self.children {
            for child in children.iter() {
                if let EmlChild::Node(node) = child {
                    node.fetch_models(models, false)?
                }
            }
        }
        Ok(())
    }

    pub fn build(
        &self,
        ctx: &EmlContext,
        as_root: bool,
    ) -> syn::Result<TokenStream> {
        let tag = &self.tag;
        let eml = &ctx.path("eml");
        let content = self.children.build(ctx, tag)?;
        let construct = self.args.build_construct(ctx, tag)?;
        let model = if let Some(model) = &self.model {
            quote! {{
                world.entity_mut(#model.entity).insert(#construct);
                #model
            }}
        } else if as_root {
            quote! {{
                world.entity_mut(__root__).insert(#construct);
                __root_model__
            }}
        } else {
            quote! {{
                let __entity__ = world.spawn(#construct).id();
                #eml::Model::<#tag>::new(__entity__)
            }}
        };
        let apply_mixins = self.mixins.build(ctx, &quote! { __model__.entity })?;
        Ok(quote_spanned! {self.tag.span()=> {
            let __model__ = #model;
            let __content__ = #content;
            <#tag as #eml::Element>::build_element(__content__)
                .eml()
                .write(world, __model__.entity);
            #apply_mixins
            __model__
        }})
    }
}

impl Parse for EmlNode {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let tag: Ident = input.parse()?;
        let mut model: Option<Ident> = None;
        if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            model = Some(input.parse()?);
        }
        let args = if input.peek(token::Brace) {
            Params::braced(input)?
        } else {
            Params::empty()
        };
        let mixins = input.parse()?;
        let children = input.parse()?;
        Ok(EmlNode {
            tag,
            model,
            args,
            mixins,
            children,
        })
    }
}

pub struct Eml {
    pub span: Span,
    pub strict: bool,
    pub roots: Vec<EmlRoot>,
}

impl Parse for Eml {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut roots = vec![];
        let span = input.span();
        for root in input.parse_terminated(EmlRoot::parse, Token![;])? {
            roots.push(root);
        }
        Ok(Eml {
            roots,
            span,
            strict: false,
        })
    }
}

impl Eml {
    pub fn fetch_models(&self) -> syn::Result<HashMap<Ident, (Ident, bool)>> {
        let mut models = HashMap::new();
        for root in self.roots.iter() {
            root.fetch_models(&mut models)?;
        }
        Ok(models)
    }
    pub fn build(&self) -> syn::Result<TokenStream> {
        let bevy = quote! { ::bevy::prelude };
        let mut body = quote! {};
        let models = self.fetch_models()?;
        let mut root_ty = None;
        
        let ctx = EmlContext { context: Context::new("polako"), strict: self.strict };
        let eml = ctx.path("eml");
        for (model, (tag, is_root)) in models.iter() {
            if *is_root {
                body = quote! { #body
                    let #model: #eml::Model<#tag> = #eml::Model::new(__root__);
                }
            } else {
                body = quote! { #body
                    let #model = world.spawn_empty().id();
                    let #model: #eml::Model<#tag> = #eml::Model::new(#model);
                }
            }
        }
        for root in self.roots.iter() {
            let build = root.build(&ctx)?;
            body = quote! {
                #body
                #build;
            };
            root_ty = Some(root.tag());
        }
        let Some(root_ty) = root_ty else {
            throw!(self.span, "Can't detect Eml exact type");
        };
        let body = quote! {
            #eml::Eml::<#root_ty>::new(move |world: &mut #bevy::World, __root__: #bevy::Entity| {
                let __this__ = __root__;
                #body
            })
        };
        Ok(if self.strict {
            quote! { #eml::Blueprint::new(#body) }
        } else {
            body
        })
    }
}
