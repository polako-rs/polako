use constructivism_macro::implement_constructivism_core; /* @constructivist-no-expose */
use std::marker::PhantomData;

pub mod traits {
    pub use super::AsField;
    pub use super::ConstructItem;
    pub use super::ExtractField;
    pub use super::ExtractValue;
    pub use super::Flattern;
    pub use super::Singleton;
    pub use super::A;
    pub use super::Construct;
    pub use super::Mixed;
    pub use super::Mixin;
    pub use super::New;
}

pub trait ConstructItem {
    type Params: Extractable;
    fn construct_item(params: Self::Params) -> Self;
}

pub trait Construct: ConstructItem {
    type Extends: Construct;
    type Fields: Singleton;
    type Protocols: Singleton;
    type MixedParams: Extractable;
    type ExpandedParams: Extractable;
    type NestedComponents: Flattern;
    type Components;
    type Inheritance;

    fn construct<P, const I: u8>(params: P) -> Self::NestedComponents where P: ExtractParams<
        I, Self::MixedParams,
        Value = <Self::MixedParams as Extractable>::Output,
        Rest = <<<Self::Extends as Construct>::ExpandedParams as Extractable>::Input as AsParams>::Defined
    >;
}

pub trait Mixin: ConstructItem {
    type Fields<T: Singleton + 'static>: Singleton;
    type Protocols<T: Singleton + 'static>: Singleton;
}

// #[macro_export]
// macro_rules! construct {
//     (@field $fields:ident $params:ident $f:ident $e:expr) => {
//         let param: &$crate::Param<_, _> = &$fields.$f;
//         let field = param.field();
//         let value = $params.field(&field).define(param.value($e.into()));
//         let $params = $params + value;
//         $params.validate(&param)();
//     };
//     (@fields $fields:ident $params:ident) => {

//     };
//     (@fields $fields:ident $params:ident $f:ident: $e:expr) => {
//         $crate::construct!(@field $fields $params $f $e)
//     };
//     (@fields $fields:ident $params:ident $f:ident) => {
//         $crate::construct!(@field $fields $params $f $f);
//         $crate::construct!(@fields $fields $params $($rest)*)
//     };
//     (@fields $fields:ident $params:ident $f:ident: $e:expr,) => {
//         $crate::construct!(@field $fields $params $f $e);
//     };
//     (@fields $fields:ident $params:ident $f:ident,) => {
//         $crate::construct!(@field $fields $params $f $f);
//         $crate::construct!(@fields $fields $params $($rest)*)
//     };
//     (@fields $fields:ident $params:ident $f:ident: $e:expr, $($rest:tt)*) => {
//         $crate::construct!(@field $fields $params $f $e);
//         $crate::construct!(@fields $fields $params $($rest)*)
//     };
//     (@fields $fields:ident $params:ident $f:ident, $($rest:tt)*) => {
//         $crate::construct!(@field $fields $params $f $f);
//         $crate::construct!(@fields $fields $params $($rest)*)
//     };
//     ($t:ty { $($rest:tt)* } ) => {
//         {
//             use $crate::traits::*;
//             type Fields = <$t as $crate::Construct>::Fields;
//             let fields = <<$t as $crate::Construct>::Fields as $crate::Singleton>::instance();
//             let params = <<$t as $crate::Construct>::ExpandedParams as $crate::Extractable>::as_params();
//             $crate::construct!(@fields fields params $($rest)*);
//             let defined_params = params.defined();
//             <$t as $crate::Construct>::construct(defined_params).flattern()
//         }
//     };
// }

#[macro_export]
macro_rules! protocols {
    ($t:ty) => {
        <<$t as $crate::Construct>::Protocols as $crate::Singleton>::instance()
    };
}


impl ConstructItem for () {
    type Params = ();

    fn construct_item(_: Self::Params) -> Self {
        ()
    }
}

impl Construct for () {
    type Fields = ();
    type Protocols = ();
    type Extends = ();
    type NestedComponents = ();
    type MixedParams = ();
    type ExpandedParams = ();
    type Inheritance = ();
    type Components = <Self::NestedComponents as Flattern>::Output;
    fn construct<P, const I: u8>(_: P) -> Self::NestedComponents where P: ExtractParams<
        I, Self::MixedParams,
        Value = <Self::MixedParams as Extractable>::Output,
        Rest = <<<Self::Extends as Construct>::ExpandedParams as Extractable>::Input as AsParams>::Defined
    >{
        ()
    }
}

pub struct Params<T>(T);
impl<T> Params<T> {
    pub fn validate<P>(&self, _: P) -> fn() -> () {
        || {}
    }
}



pub trait Extends<T: Construct> { }
impl<E: Construct<Inheritance = EInheritance>, T: Construct<Inheritance = TInheritance>, TInheritance: Contains<Exclusive, EInheritance>, EInheritance> Extends<E> for T { }
pub trait Is<T: Construct> { }
impl<E: Construct<Inheritance = EInheritance>, T: Construct<Inheritance = TInheritance>, TInheritance: Contains<Inclusive, EInheritance>, EInheritance> Is<E> for T { }

pub struct Inclusive;
pub struct Exclusive;
pub trait Contains<I, T> { }

pub struct ParamConflict<N>(PhantomData<N>);
impl<N> ParamConflict<N> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
    pub fn validate<T>(&self, _: &Param<N, T>) -> ParamRedefined<N> {
        ParamRedefined(PhantomData)
    }
}

pub struct ParamRedefined<N>(PhantomData<N>);

pub struct Param<N, T>(pub PhantomData<(N, T)>);
impl<N, T> Param<N, T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
    pub fn field(&self) -> Field<N> {
        Field(PhantomData)
    }
}

pub trait New<T> {
    fn new(from: T) -> Self;
}
impl<N: New<T>, T> Param<N, T> {
    pub fn value(&self, value: T) -> N {
        N::new(value)
    }
}

pub trait Protocols<Protocol: ?Sized> {}

pub struct MutableMethod<This, In, Out>(pub fn(this: &mut This, input: In) -> Out);
impl<This, In, Out> MutableMethod<This, In, Out> {
    pub fn call(&self, this: &mut This, input: In) -> Out {
        (self.0)(this, input)
    }
}
pub struct ImutableMethod<This, In, Out>(pub fn(this: &This, input: In) -> Out);
impl<This, In, Out> ImutableMethod<This, In, Out> {
    pub fn call(&self, this: &This, input: In) -> Out {
        (self.0)(this, input)
    }
}

pub struct StaticMethod<This, In, Out>(pub PhantomData<This>, pub fn(input: In) -> Out);
impl<This, In, Out> StaticMethod<This, In, Out> {
    pub fn call(&self, input: In) -> Out {
        (self.1)(input)
    }
}

pub struct Field<T>(PhantomData<T>);
impl<T> Field<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

pub struct D<const I: u8, T>(pub T);
pub struct U<const I: u8, T>(pub PhantomData<T>);
pub struct F<const I: u8, T>(PhantomData<T>);

pub trait A<const I: u8, T> {}

pub trait Singleton {
    fn instance() -> &'static Self;
}

impl Singleton for () {
    fn instance() -> &'static Self {
        &()
    }
}

pub trait Extractable {
    type Input: AsParams;
    type Output;
    fn extract(input: Self::Input) -> Self::Output;

    fn as_params() -> <Self::Input as AsParams>::Undefined {
        <Self::Input as AsParams>::as_params()
    }
}

impl Extractable for () {
    type Input = ();
    type Output = ();
    fn extract(_: Self::Input) -> Self::Output {
        ()
    }
}

pub trait Mixed<Right>
where
    Self: Sized,
{
    type Output;
    fn split(mixed: Self::Output) -> (Self, Right);
}

impl Mixed<()> for () {
    type Output = ();
    fn split(_: Self::Output) -> (Self, ()) {
        ((), ())
    }
}
pub struct Mix<L, R>(PhantomData<(L, R)>);

impl<O: AsParams, L: Extractable, R: Extractable> Extractable for Mix<L, R>
where
    L::Input: Mixed<R::Input, Output = O>,
{
    type Input = O;
    type Output = (L::Output, R::Output);
    fn extract(input: Self::Input) -> Self::Output {
        let (left, right) = <L::Input as Mixed<R::Input>>::split(input);
        (L::extract(left), R::extract(right))
    }
}

pub trait ExtractParams<const S: u8, T> {
    type Value;
    type Rest;
    fn extract_params(self) -> (Self::Value, Self::Rest);
}

// impl ExtractParams<0, ()> for Params<()> {
//     type Value = ();
//     type Rest = Params<()>;
//     fn extract_params(self) -> (Self::Value, Self::Rest) {
//         ((), Params(()))
//     }
// }
impl<E: Extractable<Input = ()>> ExtractParams<0, E> for Params<()> {
    type Value = E::Output;
    type Rest = Params<()>;
    fn extract_params(self) -> (Self::Value, Self::Rest) {
        (E::extract(()), Params(()))
    }
}

pub trait ExtractField<F, T> {
    fn field(&self, f: &Field<T>) -> F;
}

pub trait AsField
where
    Self: Sized,
{
    fn as_field() -> Field<Self>;
}

pub trait Shift<const I: u8> {
    type Target;
    fn shift(self) -> Self::Target;
}

pub trait ExtractValue {
    type Value;
    fn extract_value(self) -> Self::Value;
}

pub trait Flattern {
    type Output;
    fn flattern(self) -> Self::Output;
}
impl Flattern for () {
    type Output = ();
    fn flattern(self) -> Self::Output {
        ()
    }
}

impl<const I: u8, T> F<I, T> {
    pub fn define(self, value: T) -> D<I, T> {
        D::<I, T>(value)
    }
}

impl<const I: u8, T> A<I, T> for D<I, T> {}
impl<const I: u8, T> A<I, T> for U<I, T> {}

impl<const I: u8, const J: u8, T> Shift<J> for D<I, T> {
    type Target = D<J, T>;
    fn shift(self) -> Self::Target {
        D::<J, T>(self.0)
    }
}
impl<const I: u8, const J: u8, T> Shift<J> for U<I, T> {
    type Target = U<J, T>;
    fn shift(self) -> Self::Target {
        U::<J, T>(PhantomData)
    }
}
impl<const I: u8, const J: u8, T> Shift<J> for F<I, T> {
    type Target = F<J, T>;
    fn shift(self) -> Self::Target {
        F::<J, T>(PhantomData)
    }
}

impl<const I: u8, T: Default> ExtractValue for U<I, T> {
    type Value = T;
    fn extract_value(self) -> T {
        T::default()
    }
}
impl<const I: u8, T> ExtractValue for D<I, T> {
    type Value = T;
    fn extract_value(self) -> T {
        self.0
    }
}

impl Params<()> {
    pub fn defined(self) -> Self {
        self
    }
}

pub trait AsParams {
    type Defined;
    type Undefined;
    fn as_params() -> Self::Undefined;
}

implement_constructivism_core!(16);