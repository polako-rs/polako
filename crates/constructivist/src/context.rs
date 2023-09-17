use std::{cell::RefCell, collections::HashMap};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub struct Context {
    prefix: &'static str,
    cache: RefCell<HashMap<&'static str, TokenStream>>
}

impl Context {
    pub fn new(prefix: &'static str) -> Self {
        Self { prefix, cache: RefCell::new(HashMap::new())  }
    }

    fn cache(&self, key: &'static str, value: TokenStream) -> TokenStream {
        self.cache.borrow_mut().insert(key, value.clone());
        value
    }

    pub fn path(&self, name: &'static str) -> TokenStream {
        if let Some(cached) = self.cache.borrow().get(name).cloned() {
            return cached
        }
        let prefix = self.prefix;
        let iprefix = format_ident!("{prefix}");
        let global = format_ident!("{name}");
        let local = format_ident!("{prefix}_{name}");
        let lib = if name == prefix {
            quote! { ::#iprefix }
        } else {
            quote! { ::#iprefix::#global }
        };
        let Some(manifest_path) = std::env::var_os("CARGO_MANIFEST_DIR")
            .map(std::path::PathBuf::from)
            .map(|mut path| { path.push("Cargo.toml"); path })
            else { return self.cache(name, lib) };
        let Ok(manifest) = std::fs::read_to_string(&manifest_path) else {
            return self.cache(name, lib);
        };
        let Ok(manifest) = toml::from_str::<toml::map::Map<String, toml::Value>>(&manifest) else {
            return self.cache(name, lib);
        };
    
        let Some(pkg) = manifest.get("package") else { return self.cache(name, lib) };
        let Some(pkg) = pkg.as_table() else { return self.cache(name, lib) };
        let Some(pkg) = pkg.get("name") else { return self.cache(name, lib) };
        let Some(pkg) = pkg.as_str() else { return self.cache(name, lib) };
        if pkg == &format!("{prefix}_{name}") {
            self.cache(name, quote!{ crate })
        } else if pkg.starts_with(&format!("{prefix}_mod_")) {
            self.cache(name, lib)
        } else if pkg.starts_with(&format!("{prefix}_")) {
            self.cache(name, quote! { ::#local })
        } else {
            self.cache(name, lib)
        }
    }
}