use proc_macro2::{Ident, TokenStream};
use syn::{parse::Parse, DeriveInput, Type};
use quote::{quote, format_ident};

macro_rules! throw {
    ($loc:expr, $msg:expr) => {
        return Err(syn::Error::new($loc.span(), $msg));
    };
}

pub struct Element {
    pub ty: Ident,
    pub build: Ident
}

impl Element {
    pub fn from_derive(input: DeriveInput) -> syn::Result<Self>{
        let Some(build) = input.attrs.iter().filter(|a| a.path().is_ident("build")).next() else {
            throw!(input.ident, "#[build(func)] required for #[derive(Element)]");
        };
        let build = build.parse_args()?;
        let ty = input.ident.clone();
        Ok(Self { build, ty })
    }

    pub fn build(&self, cst: TokenStream, eml: TokenStream) -> syn::Result<TokenStream> {
        let func = &self.build;
        let ty = &self.ty;
        let build = format_ident!("Build{}Element", ty.to_string());
        let bevy = quote! { ::bevy::prelude };
        Ok(quote! { 
            pub struct #build;
            impl #eml::Build for #build {
                type Element = #ty;
                fn build(world: &mut #bevy::World, this: #eml::Model<Self::Element>, content: Vec<#bevy::Entity>) {
                    let mut func = #bevy::IntoSystem::into_system(#func).pipe(#eml::validate_builder::<#ty, _>);
                    func.initialize(world);
                    let eml = func.run((this, content), world);
                    eml.write(world, this.entity);
                    func.apply_deferred(world)
                }
            }
            impl #eml::Element for #ty {
                type Build = #build;
            }
        })
    }
}