use std::collections::HashMap;

use constructivist::{proc::*, context::Context};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned, format_ident};

use syn::{bracketed, parse::Parse, spanned::Spanned, token::{self, Brace, Bracket}, Lit, LitStr, Token, Expr, braced};

macro_rules! throw {
    ($loc:expr, $msg:expr) => {
        return Err(syn::Error::new($loc.span(), $msg));
    };
}

pub trait ParamsExt {
    fn build_patch(
        &self,
        _: &EmlContext,
        tag: &Ident,
        this: &TokenStream,
        patch_empty: bool,
    ) -> syn::Result<TokenStream>;
    fn build_construct(
        &self,
        ctx: &EmlContext,
        tag: &Ident,
        flattern: bool,
    ) -> syn::Result<TokenStream>;
}

impl ParamsExt for Params {
    fn build_patch(
        &self,
        _: &EmlContext,
        tag: &Ident,
        this: &TokenStream,
        patch_empty: bool,
    ) -> syn::Result<TokenStream> {
        let mut body = quote! {};
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
    fn build_construct(
        &self,
        ctx: &EmlContext,
        tag: &Ident,
        flattern: bool,
    ) -> syn::Result<TokenStream> {
        let construct = Construct {
            flattern,
            ty: syn::parse2(quote! { #tag })?,
            params: self.clone(),
        };
        construct.build(&ctx.context)
    }
}

pub enum EmlPathPart {
    /// `hidden` in `.class[hidden]`
    Index(Ident),
    /// `color` in `.bind.color`
    Prop(Ident),
}
impl Parse for EmlPathPart {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Bracket) {
            let content;
            bracketed!(content in input);
            Ok(EmlPathPart::Index(content.parse()?))
        } else {
            Ok(EmlPathPart::Prop(input.parse()?))
        }
    }
}
pub struct EmlPath(Vec<EmlPathPart>);
impl Parse for EmlPath {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut parts = vec![];
        let mut dot = if input.peek(Token![.]) {
            Some(input.parse::<Token![.]>()?)
        } else {
            None
        };
        while let Ok(part) = input.parse() {
            parts.push(part);
            dot = if input.peek(Token![.]) {
                Some(input.parse::<Token![.]>()?)
            } else {
                None
            };
        }
        if dot.is_some() {
            parts.push(EmlPathPart::Prop(format_ident!("DOT_AUTOCOMPLETE_TOKEN", span = dot.span())));
        }
        if parts.is_empty() {
            throw!(input, "EmlPath should contain at least one part");
        }
        Ok(EmlPath(parts))
    }
}

pub enum EmlExpr {
    Prop(Vec<Ident>),
    Expr(Expr),
}

impl Parse for EmlExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Brace) {
            let outer;
            braced!(outer in input);
            if outer.peek(Brace) {
                let inner;
                braced!(inner in outer);
                Ok(EmlExpr::Prop(inner.parse_terminated(Ident::parse, Token![.])?.into_iter().collect()))
            } else {
                Ok(EmlExpr::Expr(outer.parse()?))
            }
        } else {
            Ok(EmlExpr::Expr(input.parse()?))
        }
    }
}

impl EmlExpr {
    pub fn build(&self, _: &EmlContext) -> syn::Result<TokenStream> {
        Ok(match self {
            EmlExpr::Expr(e) => quote! { #e },
            EmlExpr::Prop(_) => quote! { },
        })
    }
}

pub struct EmlParam {
    pub extension: Ident,
    pub path: EmlPath,
    pub value: Option<EmlExpr>,
}

impl Parse for EmlParam {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![.]>()?;
        let extension = input.parse()?;
        let path = input.parse()?;
        let value = if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(EmlParam { extension, path, value })
    }
}

impl EmlParam {
    pub fn build_extension(&self, ctx: &EmlContext, tag: &Ident, entity: &TokenStream) -> syn::Result<TokenStream> {
        let cst = ctx.constructivism();
        let ext_ident = &self.extension;
        let mut ext = quote! { 
            <<#tag as #cst::Construct>::Design as #cst::Singleton>::instance().#ext_ident()
        };
        for part in self.path.0.iter() {
            ext = match part {
                EmlPathPart::Prop(ident) => {
                    quote! { #ext.#ident() }
                },
                EmlPathPart::Index(ident) => {
                    let ident = ident.to_string();
                    quote! { #ext.at(#ident) }
                }
            }
        }
        // Ok(quote! { #ext; })
        if let Some(value) = &self.value {
            let value = value.build(ctx)?;
            let assign = quote_spanned!{ value.span()=>
                __ext__.assign(#entity, #value)
            };
            Ok(quote! {{
                let __ext__ = #ext;
                #assign;
            }})
        } else {
            Ok(quote! { #ext.declare(#entity); })
        }
    }
}

// pub struct EmplParams(Vec<EmlParam>)

pub struct EmlParams {
    common: Params,
    extended: Vec<EmlParam>,
}

impl Parse for EmlParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut common = vec![];
        let mut extended = vec![];
        while !input.is_empty() {
            if input.fork().parse::<EmlParam>().is_ok() {
                extended.push(input.parse()?);
            } else {
                common.push(input.parse()?);
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(EmlParams { extended, common: Params { items: common } })
    }
}

impl EmlParams {
    pub fn build_construct(&self, ctx: &EmlContext, tag: &Ident) -> syn::Result<TokenStream> {
        self.common.build_construct(ctx, tag, false)
    }

    pub fn build_extensions(&self, ctx: &EmlContext, tag: &Ident, entity: &TokenStream) -> syn::Result<TokenStream> {
        let mut out = quote! { };
        for param in self.extended.iter() {
            let ext = param.build_extension(ctx, tag, entity)?;
            out = quote! { #out #ext };
        }
        Ok(out)
    }

    pub fn braced(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        content.parse()
    }
    pub fn empty() -> Self {
        EmlParams { common: Params::empty(), extended: vec![] }
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
                                    <<#tag as #cst::Construct>::Design as #cst::Singleton>::instance()
                                        .push_text(world, &mut __content__, #lit);
                            };
                            quote! { #content #assign }
                        }
                        EmlChild::Node(ch) => {
                            let span = ch.tag.span();
                            let ct = ch.build(ctx, false)?;
                            let assign = quote_spanned! { span=>
                                let _: Implemented =
                                    <<#tag as #cst::Construct>::Design as #cst::Singleton>::instance()
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
    pub fn build(&self, ctx: &EmlContext, this: &TokenStream) -> syn::Result<TokenStream> {
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
        let construct = self.items.build_construct(ctx, &self.ident, false)?;
        let cst = ctx.path("constructivism");
        Ok(quote! {
            world.entity_mut(#this).insert(#cst::Flattern::flattern(#construct));
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
        let mut out = quote! {};
        for mixin in self.0.iter() {
            out = match mixin {
                EmlMixin::Patch(patch) => {
                    let patch = patch.build(ctx, this)?;
                    quote! { #out #patch }
                }
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
    Base {
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
            if &sup.to_string() != "Base" {
                throw!(sup, "Expected Base");
            }
            let overrides = if input.peek(token::Paren) {
                Params::parenthesized(input)?
            } else {
                Params::empty()
            };
            let mixins = input.parse()?;
            let children = input.parse()?;
            Ok(EmlRoot::Base {
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
            EmlRoot::Base { tag, .. } => tag.clone(),
        }
    }
    pub fn fetch_models(&self, models: &mut HashMap<Ident, (Ident, bool)>) -> syn::Result<()> {
        match self {
            EmlRoot::Element(node) => node.fetch_models(models, true),
            EmlRoot::Base {
                children: EmlContent::Declared(items),
                ..
            } => {
                for item in items.iter() {
                    if let EmlChild::Node(node) = item {
                        node.fetch_models(models, false)?
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    pub fn build(&self, ctx: &EmlContext) -> syn::Result<TokenStream> {
        match self {
            EmlRoot::Base {
                tag,
                overrides,
                mixins,
                children,
            } => {
                if !ctx.strict {
                    throw!(
                        tag,
                        "Tag::Base only available as root inside the build! macro."
                    );
                }
                self.build_super(ctx, tag, overrides, mixins, children)
            }
            EmlRoot::Element(node) => {
                if ctx.strict {
                    throw!(
                        node.tag,
                        "Only Tag::Base available as root inside the build! macro."
                    );
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

    fn build_super(
        &self,
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

        Ok(quote! {
            let __root_model__ = #eml::Model::<#tag>::new(__root__);
            #apply_patches;
            <<#tag as #cst::Construct>::Base as #eml::Element>::build_element(#build_content)
                .eml()
                .write(world, __root__);
            #apply_mixins
        })
    }
}

pub struct EmlNode {
    pub tag: Ident,
    pub model: Option<Ident>,
    pub args: EmlParams,
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

    pub fn build(&self, ctx: &EmlContext, as_root: bool) -> syn::Result<TokenStream> {
        let tag = &self.tag;
        let eml = &ctx.path("eml");
        let content = self.children.build(ctx, tag)?;
        let construct = self.args.build_construct(ctx, tag)?;
        let model = if let Some(model) = &self.model {
            quote! {{
                world.entity_mut(#model.entity).insert(#eml::IntoBundle::into_bundle(#construct));
                #model
            }}
        } else if as_root {
            quote! {{
                world.entity_mut(__root__).insert(#eml::IntoBundle::into_bundle(#construct));
                __root_model__
            }}
        } else {
            quote! {{
                let __entity__ = world.spawn(#eml::IntoBundle::into_bundle(#construct)).id();
                #eml::Model::<#tag>::new(__entity__)
            }}
        };
        let apply_mixins = self.mixins.build(ctx, &quote! { __model__.entity })?;
        let apply_extensions = self.args.build_extensions(ctx, tag, &quote!{ __entity__ })?;
        Ok(quote_spanned! {self.tag.span()=> {
            let __model__ = #model;
            let __content__ = #content;
            <#tag as #eml::Element>::build_element(__content__)
                .eml()
                .write(world, __model__.entity);
            {
                let __entity__ = world.entity_mut(__model__.entity);
                #apply_extensions
            }
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
            EmlParams::braced(input)?
        } else {
            EmlParams::empty()
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

        let ctx = EmlContext {
            context: Context::new("polako"),
            strict: self.strict,
        };
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
