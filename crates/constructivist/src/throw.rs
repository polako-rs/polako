#[macro_export]
macro_rules! throw {
    ($loc:expr, $msg:literal) => {
        return Err(syn::Error::new($loc.span(), $msg));
    };
    ($loc:expr, $msg:literal, $($arg:expr),*) => {
        return Err(syn::Error::new($loc.span(), format!($msg, $(arg),*)));
    };
}
