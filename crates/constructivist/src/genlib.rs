use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn implement_constructivism_core(max_size: u8) -> TokenStream {
    let extract_field_impls = impl_all_extract_field(max_size);
    let add_to_params = impl_all_add_to_params(max_size);
    let defined = impl_all_defined(max_size);
    let extracts = impl_all_extracts(max_size);
    let mixed = impl_all_mixed(max_size);
    let as_params = impl_all_as_params(max_size);
    let flattern = impl_all_flattern(max_size);
    let contains = impl_all_contains(16);
    quote! {
        #extract_field_impls
        #add_to_params
        #defined
        #extracts
        #as_params
        #mixed
        #flattern
        #contains
    }
}

pub fn implement_constructivism(_: u8) -> TokenStream {
    let source = include_str!("../../constructivism/src/core.rs");
    let source = source
        .lines()
        .filter(|l| !l.contains("@constructivist-no-expose"))
        .collect::<Vec<_>>()
        .join("\n");
    let Ok(core) = TokenStream::from_str(source.as_str()) else {
        return quote! { compile_error! ("Coudn't parse constructivism::core")}
    };
    quote! {
        #core
    }
}

fn impl_all_extract_field(max_size: u8) -> TokenStream {
    let mut out = quote! {};
    for size in 1..max_size + 1 {
        for idx in 0..size {
            let impl_extract = impl_extract_field(idx, size);
            out = quote! { #out #impl_extract }
        }
    }
    out
}
fn impl_all_add_to_params(max_size: u8) -> TokenStream {
    let mut out = quote! {};
    for size in 1..max_size + 1 {
        for idx in 0..size {
            let impl_add_to_params = impl_add_to_params(idx, size);
            out = quote! { #out #impl_add_to_params }
        }
    }
    out
}
fn impl_all_defined(max_size: u8) -> TokenStream {
    let mut out = quote! {};
    for size in 1..max_size + 1 {
        let defined = impl_defined(size);
        out = quote! { #out #defined }
    }
    out
}
fn impl_all_extracts(max_size: u8) -> TokenStream {
    let mut out = quote! {};
    for size in 1..max_size + 1 {
        let extractable = impl_extractable(size);
        out = quote! { #out #extractable };
        for defined in 0..size + 1 {
            let extract = impl_extract(defined, size);
            out = quote! { #out #extract };
        }
    }
    out
}
fn impl_all_mixed(max_size: u8) -> TokenStream {
    let mut out = quote! {};
    for size in 1..max_size + 1 {
        for left in 0..size + 1 {
            let right = size - left;
            let mixed = impl_mixed(left, right);
            out = quote! { #out #mixed };
        }
    }
    out
}
/// ```ignore
/// impl<T0, T1> AsParams for (D<0, T0>, D<1, T1>) {
/// type Undefined = (U<0, T0>, U<1, T1>);
///     fn as_params() -> Params<Self::Undefined> {
///         Params((
///             U::<0, T0>(PhantomData),
///             U::<1, T1>(PhantomData)
///         ))
///     }
/// }
/// ```
fn impl_all_as_params(max_size: u8) -> TokenStream {
    let mut out = quote! {};
    for size in 0..max_size + 1 {
        let mut ts = quote! {};
        let mut ds = quote! {};
        let mut us = quote! {};
        let mut ps = quote! {};
        for i in 0..size {
            let ti = format_ident!("T{i}");
            ts = quote! { #ts #ti, };
            ds = quote! { #ds D<#i, #ti>, };
            us = quote! { #us U<#i, #ti>, };
            ps = quote! { #ps U::<#i, #ti>(::std::marker::PhantomData), }
        }
        out = quote! { #out
            impl<#ts> AsParams for (#ds) {
                type Undefined = Params<(#us)>;
                type Defined = Params<(#ds)>;
                fn as_params() -> Self::Undefined {
                    Params(( #ps ))
                }
            }
        }
    }
    out
}

fn impl_all_flattern(max_depth: u8) -> TokenStream {
    let mut out = quote! {};
    for depth in 1..max_depth + 1 {
        let mut ts = quote! {};
        let mut cstr = quote! {};
        let mut ns = quote! { () };
        let mut vs = quote! {};
        let mut dcs = quote! { _ };
        for i in 0..depth {
            let ti = format_ident!("T{i}");
            let pi = format_ident!("p{i}");
            let tr = format_ident!("T{}", depth - i - 1);
            let pr = format_ident!("p{}", depth - i - 1);
            cstr = quote! { #cstr #ti: ConstructItem, };
            ts = if i < depth - 1 {
                quote! { #ts #ti, }
            } else {
                quote! { #ts #ti }
            };
            vs = if i < depth - 1 {
                quote! { #vs #pi, }
            } else {
                quote! { #vs #pi }
            };
            // ts = quote! { #ts #ti, };
            ns = quote! { (#tr, #ns) };
            dcs = quote! { (#pr, #dcs) };
        }
        out = quote! { #out
            impl<#cstr> Flattern for #ns {
                type Output = (#ts);
                fn flattern(self) -> Self::Output {
                    let #dcs = self;
                    ( #vs )
                }
            }
        }
    }
    out
}

fn impl_all_contains(max_size: u8) -> TokenStream {
    let mut out = quote! { };
    for size in 1..max_size + 1 {
        let contains = impl_contains(size);
        out = quote! { #out #contains }
    }
    out
}

/// Generates single ExtractField trait implementation.
/// `impl_extract_field(1, 3) will generate this:
/// ```ignore
/// impl<T1, A0, A1: A<1, T1>, A2> ExtractField<F<1, T1>, T1> for Params<(A0, A1, A2)> {
///     fn field(&self, _: &Field<T1>) -> F<1, T1> {
///         F::<1, T1>(PhantomData)
///     }
/// }
/// ```
fn impl_extract_field(idx: u8, size: u8) -> TokenStream {
    let ti = format_ident!("T{idx}");
    let fi = quote! { F<#idx, #ti> };
    let mut gin = quote! {};
    let mut gout = quote! {};
    for i in 0..size {
        let ai = format_ident!("A{i}");
        if i == idx {
            gin = quote! { #gin #ai: A<#i, #ti>, }
        } else {
            gin = quote! { #gin #ai,}
        }
        gout = quote! { #gout #ai, };
    }

    quote! {
        impl<#ti, #gin> ExtractField<#fi, #ti> for Params<(#gout)> {
            fn field(&self, _: &Field<#ti>) -> #fi {
                F::<#idx, #ti>(PhantomData)
            }
        }
    }
}

/// Generates single std::ops::Add implementation for Params of size `size`
/// and param at `idx` position. `impl_add_to_params(1, 4)` will generate this:
/// ```ignore
//       #gin                                              #pundef
/// impl<T1, A0, A2, A3> std::ops::Add<D<1, T1>> for Params<(A0, U<1, T1>, A2, A3)> {
//                           #pout
///     type Output = Params<(A0, D<1, T1>, A2, A3)>;
///     fn add(self, rhs: D<1, T1>) -> Self::Output {
//               #dcs
///         let (p0, _, p2, p3) = self.0;
//                 #vls
///         Params((p0, rhs, p2, p3))
///     }
/// }
//       #gin                                              #pdef
/// impl<T1, A0, A2, A3> std::ops::Add<D<1, T1>> for Params<(A0, D<1, T1>, A2, A3)> {
//                           #pout
///     type Output = ParamConflict<T1>;
///     fn add(self, _: D<1, T1>) -> Self::Output {
///         ParamConflict::new()
///     }
/// }
/// ```
fn impl_add_to_params(idx: u8, size: u8) -> TokenStream {
    let ti = format_ident!("T{idx}");
    let di = quote! { D<#idx, #ti> };
    let ui = quote! { U<#idx, #ti> };
    let mut gin = quote! {};
    let mut pundef = quote! {};
    let mut pdef = quote! {};
    let mut pout = quote! {};
    let mut dcs = quote! {};
    let mut vls = quote! {};
    for i in 0..size {
        if i == idx {
            pundef = quote! { #pundef #ui, };
            pdef = quote! { #pdef #di, };
            pout = quote! { #pout #di, };
            dcs = quote! { #dcs _, };
            vls = quote! { #vls rhs, };
        } else {
            let ai = format_ident!("A{i}");
            let pi = format_ident!("p{i}");
            gin = quote! { #gin #ai, };
            pundef = quote! { #pundef #ai, };
            pdef = quote! { #pdef #ai, };

            pout = quote! { #pout #ai, };
            dcs = quote! { #dcs #pi, };
            vls = quote! { #vls #pi, };
        }
    }
    quote! {
        impl<#ti, #gin> std::ops::Add<#di> for Params<(#pundef)> {
            type Output = Params<(#pout)>;
            fn add(self, rhs: #di) -> Self::Output {
                let (#dcs) = self.0;
                Params((#vls))
            }
        }

        impl<#ti, #gin> std::ops::Add<#di> for Params<(#pdef)> {
            type Output = ParamConflict<#ti>;
            fn add(self, _: #di) -> Self::Output {
                ParamConflict::new()
            }
        }
    }
}

/// ```ignore
/// impl<T0, T1> Extractable for (T0, T1) {
///     type Input = (D<0, T0>, D<1, T1>);
///     type Output = (T0, T1);
///     fn extract(input: Self::Input) -> Self::Output {
///         let (p0, p1) = input;
///         (p0.0, p1.0)
///     }
/// }
/// impl<T0, T1, T2, T3, E: Extractable<Input = (T0, T1)>> ExtractParams<2, E> for Params<(T0, T1, T2, T3)>
/// where
///     T2: Shift<0>,
///     T3: Shift<1>,
/// {
///     type Value = E::Output;
///     type Rest = Params<(T2::Target, T3::Target)>;
///     fn extract_params(self) -> (Self::Value, Self::Rest) {
///         let (p0, p1, p2, p3) = self.0;
///         (
///             E::extract((p0, p1)),
///             Params((p2.shift(), p3.shift()))
///         )
///     }
/// }
/// ```
fn impl_extractable(size: u8) -> TokenStream {
    let mut ein = quote! {};
    let mut edef = quote! {};
    let mut eout = quote! {};
    let mut dcstr = quote! {};

    for i in 0..size {
        let ti = format_ident!("T{i}");
        let pi = format_ident!("p{i}");
        ein = quote! { #ein #ti, };
        edef = quote! { #edef D<#i, #ti>, };
        dcstr = quote! { #dcstr #pi, };
        eout = quote! { #eout #pi.0, };
    }
    quote! {
        impl<#ein> Extractable for (#ein) {
            type Input = (#edef);
            type Output = (#ein);
            fn extract(input: Self::Input) -> Self::Output {
                let (#dcstr) = input;
                (#eout)
            }
        }
    }
}
fn impl_extract(defined: u8, size: u8) -> TokenStream {
    let mut ein = quote! {};
    let mut pin = quote! {};
    let mut pfor = quote! {};
    let mut pcstr = quote! {};
    let mut trest = quote! {};
    let mut pdcstr = quote! {};
    let mut pout = quote! {};
    let mut pparams = quote! {};

    for i in 0..size {
        let ti = format_ident!("T{i}");
        let pi = format_ident!("p{i}");
        if i < defined {
            ein = quote! { #ein #ti, };
            pout = quote! { #pout #pi, }
        } else {
            let j = i - defined;
            pcstr = quote! { #pcstr #ti: Shift<#j>, };
            trest = quote! { #trest #ti::Target, };
            pparams = quote! { #pparams #pi.shift(), };
        }
        pin = quote! { #pin #ti, };
        pfor = quote! { #pfor #ti, };
        pdcstr = quote! { #pdcstr #pi, };
    }
    quote! {
        impl<#pin E: Extractable<Input = (#ein)>> ExtractParams<#defined, E> for Params<(#pin)>
        where #pcstr
        {
            type Value = E::Output;
            type Rest = Params<(#trest)>;
            fn extract_params(self) -> (Self::Value, Self::Rest) {
                let (#pdcstr) = self.0;
                (
                    E::extract((#pout)),
                    Params((#pparams))
                )
            }
        }
    }
}

// impl<T0: ExtractValue, T1: ExtractValue, T2: ExtractValue> Params<(T0, T1, T2)> {
//     pub fn defined(self) -> Params<(D<0, T0::Value>, D<1, T1::Value>, D<2, T2::Value>)> {
//         let (p0,p1,p2) = self.0;
//         Params((
//             D::<0, _>(p0.extract_value()),
//             D::<1, _>(p1.extract_value()),
//             D::<2, _>(p2.extract_value()),
//         ))
//     }
// }
fn impl_defined(size: u8) -> TokenStream {
    let mut gin = quote! {};
    let mut gout = quote! {};
    let mut pout = quote! {};
    let mut dcstr = quote! {};
    let mut vals = quote! {};
    for i in 0..size {
        let ti = format_ident!("T{i}");
        let pi = format_ident!("p{i}");
        gin = quote! { #gin #ti: ExtractValue, };
        gout = quote! { #gout #ti, };
        pout = quote! { #pout D<#i, #ti::Value>, };
        dcstr = quote! { #dcstr #pi, };
        vals = quote! { #vals D::<#i, _>(#pi.extract_value()), }
    }
    quote! {
        impl<#gin> Params<(#gout)> {
            pub fn defined(self) -> Params<(#pout)> {
                let (#dcstr) = self.0;
                Params((#vals))
            }
        }
    }
}

/// ```ignore
/// impl<L0, R0, R1> Mixed<(D<0, R0>, D<1, R1>)> for (D<0, L0>,) {
///     type Output = (D<0, L0>, D<1, R0>, D<2, R1>);
///     fn split(joined: Self::Output) -> (Self, (D<0, R0>, D<1, R1>)) {
///         let (l0, r0, r1) = joined;
///         let r0 = D::<0, _>(r0.0);
///         let r1 = D::<1, _>(r1.0);
///         ((l0,), (r0, r1))
///         
///     }
/// }
/// ```
fn impl_mixed(left: u8, right: u8) -> TokenStream {
    let mut ls = quote! {}; // L0,
    let mut rs = quote! {}; // R0, R1,
    let mut dls = quote! {}; // D<0, L0>,
    let mut drs = quote! {}; // D<0, R0>, D<1, R1>,
    let mut lvs = quote! {}; // l0,
    let mut rvs = quote! {}; // r0, r1,
    let mut shift = quote! {}; // let r0 = D::<0, _>(r0.0);
                               // let r1 = D::<1, _>(r1.0);
    let mut output = quote! {}; // D<0, L0>, D<1, R0>, D<2, R1>
    for i in 0..left.max(right) {
        let li = format_ident!("L{i}");
        let ri = format_ident!("R{i}");
        let lv = format_ident!("l{i}");
        let rv = format_ident!("r{i}");
        if i < left {
            ls = quote! { #ls #li, };
            dls = quote! { #dls D<#i, #li>, };
            lvs = quote! { #lvs #lv, };
        }
        if i < right {
            rs = quote! { #rs #ri, };
            drs = quote! { #drs D<#i, #ri>, };
            rvs = quote! { #rvs #rv, };
            shift = quote! { #shift let #rv = D::<#i, _>(#rv.0); }
        }
    }
    for i in 0..left + right {
        let ti = if i < left {
            format_ident!("L{i}")
        } else {
            format_ident!("R{}", i - left)
        };
        output = quote! { #output D<#i, #ti>, };
    }
    quote! {
        impl<#ls #rs> Mixed<(#drs)> for (#dls) {
            type Output = (#output);
            fn split(joined: Self::Output) -> (Self, (#drs)) {
                let (#lvs #rvs) = joined;
                #shift
                ((#lvs), (#rvs))
            }
        }
    }
}
// output for impl_contains(4)
// impl<T0, T1, T2, T3> Contains<()> for (T0, (T1, (T2, (T3, ())))) { }
// impl<T0, T1, T2, T3> Contains<(T3, ())> for (T0, (T1, (T2, (T3, ())))) { }
// impl<T0, T1, T2, T3> Contains<(T2, (T3, ()))> for (T0, (T1, (T2, (T3, ())))) { }
// impl<T0, T1, T2, T3> Contains<(T1, (T2, (T3, ())))> for (T0, (T1, (T2, (T3, ())))) { }
fn impl_contains(size: u8) -> TokenStream {
    let mut out = quote! { };
    let mut tfor = quote! { () };
    let mut tin = quote! { };
    for i in 0..size {
        let ti = format_ident!("T{}", size - i - 1);
        tfor = quote! { (#ti, #tfor) };
        tin = quote! { #ti, #tin };
    }
    for impl_size in 0..size {
        let mut cnt = quote! { () };
        for i in 0..impl_size {
            let ti = format_ident!("T{}", size - i - 1);
            cnt = quote! { (#ti, #cnt) }
        }
        out = quote! { #out
            impl<I, #tin> Contains<I, #cnt> for #tfor { }
        }
    }
    out = quote! { #out
        impl<#tin> Contains<Inclusive, #tfor> for #tfor { }
    };
    out

}