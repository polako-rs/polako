use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    bracketed, parenthesized, parse::Parse, spanned::Spanned, Attribute, Data, DeriveInput, Expr,
    FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, ReturnType, Token, Type,
};
use crate::context::Context;
use crate::exts::TypeExt;
use crate::throw;

enum ParamType {
    Single(Type),
    Union(Vec<Param>),
}
impl Parse for ParamType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Bracket) {
            let content;
            bracketed!(content in input);
            let params = content.parse_terminated(Param::parse, Token![,])?;
            Ok(ParamType::Union(params.into_iter().collect()))
        } else {
            Ok(ParamType::Single(input.parse()?))
        }
    }
}

enum ParamDefault {
    None,
    Default,
    Custom(Expr),
}
struct Param {
    name: Ident,
    ty: ParamType,
    default: ParamDefault,
}

impl Parse for Param {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        let mut default = ParamDefault::None;
        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            default = ParamDefault::Custom(input.parse()?);
        }
        Ok(Param { name, ty, default })
    }
}

pub enum ConstructMode {
    Mixin,
    Construct {
        extends: Option<Type>,
        mixins: Vec<Type>,
    },
}

impl ConstructMode {
    pub fn mixin() -> Self {
        ConstructMode::Mixin
    }
    pub fn object() -> Self {
        ConstructMode::Construct {
            extends: None,
            mixins: vec![],
        }
    }
    pub fn is_mixin(&self) -> bool {
        match self {
            ConstructMode::Mixin => true,
            _ => false,
        }
    }
    pub fn is_object(&self) -> bool {
        match self {
            ConstructMode::Construct { .. } => true,
            _ => false,
        }
    }
    fn set_extends(&mut self, ty: Type) -> Result<(), syn::Error> {
        match self {
            ConstructMode::Construct { extends, .. } => {
                *extends = Some(ty);
                Ok(())
            }
            _ => {
                throw!(
                    ty,
                    "set_extends(..) available only for ConstructMode::Construct"
                );
            }
        }
    }
    fn push_mixin(&mut self, ty: Type) -> Result<(), syn::Error> {
        match self {
            ConstructMode::Construct { mixins, .. } => {
                // throw!(ty, format!("adding mixin for {:?}", ty.to_token_stream()));
                mixins.push(ty);
                Ok(())
            }
            _ => {
                throw!(
                    ty,
                    "push_mixin(..) available only for ConstructMode::Construct"
                );
            }
        }
    }
}

pub struct Constructable {
    ty: Type,
    params: Vec<Param>,
    body: Option<Expr>,
    mode: ConstructMode,
}

impl Parse for Constructable {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ty: Type = input.parse()?;
        let extends = if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        let mode = ConstructMode::Construct {
            extends,
            mixins: vec![],
        };
        let content;
        parenthesized!(content in input);
        let params = content.parse_terminated(Param::parse, Token![,])?;
        let params = params.into_iter().collect();
        let body = Some(input.parse()?);
        Ok(Constructable {
            ty,
            params,
            body,
            mode,
        })
    }
}

impl Constructable {
    pub fn build(&self, ctx: &Context) -> syn::Result<TokenStream> {
        let ty = &self.ty;
        let lib = ctx.path("constructivism");
        let type_ident = ty.as_ident()?;
        let mod_ident = format_ident!(
            // slider_construct
            "{}_construct",
            type_ident.to_string().to_lowercase()
        );
        let mut type_params = quote! {}; // slider_construct::min, slider_construct::max, slider_construct::val,
        let mut type_params_deconstruct = quote! {}; // slider_construct::min(min), slider_construct::max(max), slider_construct::val(val),
        let mut param_values = quote! {}; // min, max, val,
        let mut impls = quote! {};
        let mut fields = quote! {};
        let mut fields_new = quote! {};
        for param in self.params.iter() {
            let ParamType::Single(param_ty) = &param.ty else {
                throw!(ty, "Union params not supported yet.");
            };
            let ident = &param.name;
            param_values = quote! { #param_values #ident, };
            type_params = quote! { #type_params #mod_ident::#ident, };
            type_params_deconstruct =
                quote! { #type_params_deconstruct #mod_ident::#ident(mut #ident), };
            fields = quote! { #fields
                #[allow(unused_variables)]
                pub #ident: #lib::Param<#ident, #param_ty>,
            };
            fields_new = quote! { #fields_new #ident: #lib::Param(::std::marker::PhantomData), };
            let default = match &param.default {
                ParamDefault::Custom(default) => {
                    quote! {
                        impl Default for #ident {
                            fn default() -> Self {
                                #ident(#default)
                            }
                        }
                    }
                }
                ParamDefault::Default => {
                    quote! {
                        impl Default for #ident {
                            fn default() -> Self {
                                #ident(Default::default())
                            }
                        }
                    }
                }
                ParamDefault::None => {
                    quote! {}
                }
            };
            impls = quote! { #impls
                #default
                #[allow(non_camel_case_types)]
                pub struct #ident(pub #param_ty);
                impl<T: Into<#param_ty>> From<T> for #ident {
                    fn from(__value__: T) -> Self {
                        #ident(__value__.into())
                    }
                }
                impl #lib::AsField for #ident {
                    fn as_field() -> #lib::Field<Self> {
                        #lib::Field::new()
                    }
                }
                impl #lib::New<#param_ty> for #ident {
                    fn new(from: #param_ty) -> #ident {
                        #ident(from)
                    }
                }
            };
        }
        let construct = if let Some(expr) = &self.body {
            expr.clone()
        } else {
            syn::parse2(quote! {
                Self { #param_values }
            })
            .unwrap()
        };

        let object = if let ConstructMode::Construct { extends, mixins } = &self.mode {
            let inheritance = if let Some(extends) = extends {
                quote! { (#type_ident, <#extends as #lib::Construct>::Inheritance) }
            } else {
                quote! { (#type_ident, ()) }
            };
            let extends = if let Some(extends) = extends {
                quote! { #extends }
            } else {
                quote! { () }
            };

            let mut mixed_params = quote! {};
            let mut expanded_params = quote! { <Self::Extends as #lib::Construct>::ExpandedParams };
            let mut hierarchy = quote! { <Self::Extends as #lib::Construct>::NestedComponents };
            let mut deconstruct = quote! {};
            let mut construct = quote! { <Self::Extends as #lib::Construct>::construct(rest) };
            for mixin in mixins.iter().rev() {
                let mixin_params =
                    format_ident!("{}_params", mixin.as_ident()?.to_string().to_lowercase());
                if mixed_params.is_empty() {
                    mixed_params = quote! { <#mixin as #lib::ConstructItem>::Params, };
                    deconstruct = quote! { #mixin_params };
                } else {
                    mixed_params =
                        quote! {  #lib::Mix<<#mixin as #lib::ConstructItem>::Params, #mixed_params> };
                    deconstruct = quote! { (#mixin_params, #deconstruct) };
                }
                expanded_params =
                    quote! { #lib::Mix<<#mixin as #lib::ConstructItem>::Params, #expanded_params> };
                construct = quote! { ( <#mixin as #lib::ConstructItem>::construct_item(#mixin_params), #construct ) };
                hierarchy = quote! { (#mixin, #hierarchy) };
            }
            let mixed_params = if mixed_params.is_empty() {
                quote! { (#type_params) }
            } else {
                quote! { #lib::Mix<(#type_params), #mixed_params> }
            };
            let deconstruct = if deconstruct.is_empty() {
                quote! { self_params }
            } else {
                quote! { (self_params, #deconstruct) }
            };
            let construct = quote! {
                (
                    <Self as #lib::ConstructItem>::construct_item(self_params),
                    #construct
                )
            };
            quote! {
                impl #lib::Construct for #type_ident {
                    type Extends = #extends;
                    type Fields = #mod_ident::Fields;
                    type Protocols = #mod_ident::Protocols;
                    type MixedParams = (#mixed_params);
                    type NestedComponents = (Self, #hierarchy);
                    type ExpandedParams = #lib::Mix<(#type_params), #expanded_params>;
                    type Components = <Self::NestedComponents as #lib::Flattern>::Output;
                    type Inheritance = #inheritance;


                    fn construct<P, const I: u8>(params: P) -> Self::NestedComponents where P: #lib::ExtractParams<
                        I, Self::MixedParams,
                        Value = <Self::MixedParams as #lib::Extractable>::Output,
                        Rest = <<<Self::Extends as #lib::Construct>::ExpandedParams as #lib::Extractable>::Input as #lib::AsParams>::Defined
                    > {
                        let (#deconstruct, rest) = params.extract_params();
                        #construct
                    }
                }
            }
        } else {
            quote! {}
        };
        let mixin = if self.mode.is_mixin() {
            quote! {
                impl #lib::Mixin for #type_ident {
                    type Fields<T: #lib::Singleton + 'static> = #mod_ident::Fields<T>;
                    type Protocols<T: #lib::Singleton + 'static> = #mod_ident::Protocols<T>;
                }
            }
        } else {
            quote! {}
        };
        let decls = match &self.mode {
            ConstructMode::Construct { extends, mixins } => {
                let extends = if let Some(extends) = extends {
                    quote! { #extends }
                } else {
                    quote! { () }
                };
                let mut deref_fields = quote! { <#extends as #lib::Construct>::Fields };
                let mut deref_protocols = quote! { <#extends as #lib::Construct>::Protocols };
                for mixin in mixins.iter() {
                    deref_fields = quote! { <#mixin as #lib::Mixin>::Fields<#deref_fields> };
                    deref_protocols = quote! { <#mixin as #lib::Mixin>::Protocols<#deref_protocols> };
                }

                quote! {
                    pub struct Fields {
                        #fields
                    }

                    pub struct Protocols;
                    impl #lib::Singleton for Fields {
                        fn instance() -> &'static Self {
                            &Fields {
                                #fields_new
                            }
                        }
                    }
                    impl #lib::Singleton for Protocols {
                        fn instance() -> &'static Self {
                            &Protocols
                        }
                    }
                    impl ::std::ops::Deref for Fields {
                        type Target = #deref_fields;
                        fn deref(&self) -> &Self::Target {
                            <#deref_fields as #lib::Singleton>::instance()
                        }
                    }
                    impl #lib::Protocols<#ty> for Protocols { }
                    impl ::std::ops::Deref for Protocols {
                        type Target = #deref_protocols;
                        fn deref(&self) -> &Self::Target {
                            <#deref_protocols as #lib::Singleton>::instance()
                        }
                    }

                }
            }
            ConstructMode::Mixin => quote! {
                pub struct Fields<T: #lib::Singleton> {
                    #fields
                    __base__: ::std::marker::PhantomData<T>,
                }
                pub struct Protocols<T: #lib::Singleton>(
                    ::std::marker::PhantomData<T>
                );
                impl<T: #lib::Singleton> #lib::Singleton for Fields<T> {
                    fn instance() -> &'static Self {
                        &Fields {
                            #fields_new
                            __base__: ::std::marker::PhantomData,
                        }
                    }
                }
                impl<T: #lib::Singleton> #lib::Singleton for Protocols<T> {
                    fn instance() -> &'static Self {
                        &Protocols(::std::marker::PhantomData)
                    }
                }
                impl<T: #lib::Singleton + 'static> std::ops::Deref for Fields<T> {
                    type Target = T;
                    fn deref(&self) -> &Self::Target {
                        T::instance()
                    }
                }
                impl<T: #lib::Singleton + 'static> std::ops::Deref for Protocols<T> {
                    type Target = T;
                    fn deref(&self) -> &Self::Target {
                        T::instance()
                    }
                }
            },
        };
        Ok(quote! {
            mod #mod_ident {
                use super::*;
                #decls
                #impls
            }
            impl #lib::ConstructItem for #type_ident {
                type Params = ( #type_params );
                fn construct_item(params: Self::Params) -> Self {
                    let (#type_params_deconstruct) = params;
                    #construct
                }
            }
            #object
            #mixin
        })
    }

    pub fn from_derive(input: DeriveInput, mut mode: ConstructMode) -> Result<Self, syn::Error> {
        if input.generics.params.len() > 0 {
            throw!(
                input.ident,
                "#[derive(Construct)] doesn't support generics yet."
            );
        }
        let ident = input.ident.clone(); // Slider
        let ty = syn::parse2(quote! { #ident }).unwrap();
        if let Some(extends) = input.attrs.iter().find(|a| a.path().is_ident("extend")) {
            if !mode.is_object() {
                throw!(
                    extends,
                    "#[extend(..) only supported by #[derive(Construct)]."
                );
            }
            mode.set_extends(extends.parse_args()?)?
        }
        if let Some(mixin) = input.attrs.iter().find(|a| a.path().is_ident("mix")) {
            // throw!(mixin, "found mixin");
            if !mode.is_object() {
                throw!(mixin, "#[mix(..) only supported by #[derive(Construct)].");
            }
            // mixin.meta.
            mixin.parse_nested_meta(|meta| {
                mode.push_mixin(syn::parse2(meta.path.into_token_stream())?)
                // for mixin in meta.input.parse_terminated(Type::parse, Token![,])?.iter() {
                //     throw!(mixin, "adding mixin");
                // }
                // Ok(())
            })?;
        }

        let Data::Struct(input) = input.data else {
            throw!(input.ident, "#[derive(Construct)] only supports named structs. You can use `constructable!` for complex cases.");
        };
        let mut params = vec![];
        for field in input.fields.iter() {
            let ty = ParamType::Single(field.ty.clone());
            let Some(name) = field.ident.clone() else {
                throw!(field, "#[derive(Construct)] only supports named structs. You can use `constructable!` for complex cases.");
            };
            let default = if field.attrs.iter().any(|a| a.path().is_ident("required")) {
                ParamDefault::None
            } else if let Some(default) = field.attrs.iter().find(|a| a.path().is_ident("default"))
            {
                let Ok(expr) = default.parse_args::<Expr>() else {
                    throw!(name, "Invalid expression for #[default(expr)].");
                };
                ParamDefault::Custom(expr)
            } else {
                ParamDefault::Default
            };
            params.push(Param { ty, name, default });
        }
        let body = None;
        Ok(Constructable {
            ty,
            params,
            body,
            mode,
        })
    }
}

#[derive(PartialEq)]
pub enum MethodKind {
    Static,
}

pub struct Argument {
    pub attrs: Vec<Attribute>,
    pub pat: TokenStream,
    pub ty: Type,
}

pub struct Method {
    pub ident: Ident,
    pub kind: MethodKind,
    pub input: Vec<Argument>,
    pub output: Type,
    pub attrs: Vec<Attribute>,
}
pub struct Protocols {
    pub ty: Type,
    pub input: ItemImpl,
    pub protocols: Vec<Method>,
}

impl Protocols {
    pub fn from_input(input: ItemImpl) -> syn::Result<Self> {
        let ty = *input.self_ty.clone();
        let mut protocols = vec![];
        for item in input.items.iter() {
            let ImplItem::Fn(ImplItemFn { sig, attrs, .. }) = item else {
                throw!(item, "Only fn $method(...) supported");
            };
            let ident = sig.ident.clone();
            let kind = MethodKind::Static;
            let mut input = vec![];
            for arg in sig.inputs.iter() {
                match arg {
                    FnArg::Receiver(this) => {
                        throw!(this, "Only static protocols supported yet");
                    }
                    FnArg::Typed(arg) => {
                        let pat = &arg.pat;
                        let ty = *arg.ty.clone();
                        input.push(Argument {
                            ty,
                            pat: quote! { #pat },
                            attrs: arg.attrs.clone(),
                        });
                    }
                }
            }
            let output = match &sig.output {
                ReturnType::Default => syn::parse2(quote! { () }).unwrap(),
                ReturnType::Type(_, ty) => *ty.clone(),
            };
            protocols.push(Method {
                ident,
                kind,
                input,
                output,
                attrs: attrs.clone(),
            })
        }

        Ok(Self { ty, protocols, input })
    }

    pub fn build(&self, _lib: TokenStream) -> syn::Result<TokenStream> {
        let ty = &self.ty;
        let mod_ident = format_ident!(
            "{}_construct",
            self.ty.as_ident()?.to_string().to_lowercase()
        );
        let mut protocols = quote! {};
        for method in self.protocols.iter() {
            let ident = &method.ident;
            let mut args_pass = quote! {};
            let mut args_typed = quote! {};
            if method.kind != MethodKind::Static {
                throw!(ident, "Only static protocols supported yet");
            }
            for arg in method.input.iter() {
                let pat = &arg.pat;
                let ty = &arg.ty;
                for attr in arg.attrs.iter() {
                    args_typed = quote! { #args_typed #attr }
                }
                args_typed = quote! { #args_typed #pat: #ty, };
                args_pass = quote! { #args_pass #pat, };
            }
            let output = &method.output;
            for attr in method.attrs.iter() {
                protocols = quote! { #protocols #attr }
            }
            protocols = quote! { #protocols
                pub fn #ident(&self, #args_typed) -> #output {
                    <#ty>::#ident(#args_pass)
                }
            };
        }
        let input = &self.input;
        Ok(quote! {
            #input
            impl #mod_ident::Protocols {
                #protocols
            }
        })
    }
}
