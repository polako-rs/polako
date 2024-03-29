use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use constructivist::{
    proc::{build, Construct, ContextLike, Params, Ref, Value},
    throw,
};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseBuffer},
    parse2,
    token::{self, Paren},
    Lit, LitStr, Token,
};

use crate::eml::{EmlContext, Mark, MarkKind};

/// Samples:
/// ```ignore
/// resource(time, Time);
/// entity(label, Label);
/// label.on.update() -> {
///     label.text = time.elapsed_seconds.fmt("{:0.2}");
/// }
/// ```
///
/// ```ignore
/// Resolves into ast:
/// Statement::AssignToComponent(
///     Path(label.text),
///     Expr::Format(
///         Expr::ReadResource(time.elapsed_seconds),
///         "{:0.2}"
///     )
/// )
/// ```

#[derive(Clone)]
pub struct AccessPoint {
    mark: Mark,
    prop: Ident,
    write: bool,
}

impl Eq for AccessPoint {}
impl PartialEq for AccessPoint {
    fn eq(&self, other: &Self) -> bool {
        match (&self.mark.kind, &other.mark.kind) {
            (MarkKind::Entity, MarkKind::Entity) => {
                self.mark.ty == other.mark.ty && self.prop == other.prop
            }
            (MarkKind::Resource, MarkKind::Resource) => self.mark.ty == other.mark.ty,
            _ => false,
        }
    }
}

pub struct HandBuilder {
    ctx: Ref<EmlContext>,
    access: Vec<AccessPoint>,
    reads: HashMap<Path, Option<usize>>,
    args: HashSet<Ident>,
}

impl ContextLike for HandBuilder {
    fn path(&self, name: &'static str) -> TokenStream {
        self.ctx.path(name)
    }
}

impl std::ops::Deref for HandBuilder {
    type Target = EmlContext;
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl HandBuilder {
    pub fn new(eml_context: Ref<EmlContext>, locals: Vec<Ident>) -> Self {
        HandBuilder {
            ctx: eml_context,
            access: vec![],
            reads: HashMap::new(),
            args: locals.into_iter().collect(),
        }
    }
    pub fn signature(&self) -> syn::Result<TokenStream> {
        let mut items = quote! {};
        for point in self.access.iter() {
            match point.mark.kind {
                MarkKind::Entity => {
                    if point.write {
                        items = quote! { #items ::bevy::prelude::Query<&mut _>, };
                    } else {
                        items = quote! { #items ::bevy::prelude::Query<& _>, };
                    }
                }
                MarkKind::Resource => {
                    if point.write {
                        items = quote! { #items ::bevy::prelude::ResMut<_>, };
                    } else {
                        items = quote! { #items ::bevy::prelude::Res<_>, };
                    }
                }
            }
        }
        let event = if self.args.is_empty() {
            quote! { _event }
        } else if self.args.len() == 1 {
            let mut it = self.args.iter();
            let e = it.next().unwrap();
            quote! { #e }
        } else {
            let mut it = self.args.iter().skip(1);
            let e = it.next().unwrap();
            throw!(e, "Unexpected extra hand argument.")
        };
        let params = if items.is_empty() {
            quote! { () }
        } else {
            quote! { ::bevy::ecs::system::ParamSet<(#items)> }
        };
        Ok(quote! {
            #event: &_,
            _params: &mut ::bevy::ecs::system::StaticSystemParam<(
                ::bevy::prelude::Commands,
                #params,
            )>
        })
    }
    pub fn header(&self) -> syn::Result<TokenStream> {
        let mut header = quote! {};
        for (path, idx) in self.reads.iter() {
            let ident = path.mark();
            let var = path.var();
            let mut get = quote! { #ident.getters() };
            for (idx, part) in path.0.iter().skip(1).enumerate() {
                if idx == 0 {
                    get = quote! { #get.#part(&_host) };
                } else {
                    get = quote! { #get.#part() };
                }
            }
            if let Some(event) = self.args.get(&ident) {
                header = quote! { #header
                    let #var = {
                        let _host = #event;
                        #get.into_value().get()
                    };
                };
            } else if let Some(idx) = idx {
                let point = &self.access[*idx];
                let param_idx = format_ident!("p{idx}");
                if !point.write {
                    match point.mark.kind {
                        MarkKind::Entity => {
                            header = quote! {
                                #header
                                let #var = {
                                    let _inset = _params.#param_idx();
                                    let _host = _inset.get(#ident.entity).unwrap();
                                    #get.into_value().get()
                                };
                            }
                        }
                        MarkKind::Resource => {
                            header = quote! {
                                #header
                                let #var = {
                                    let _host = _params.#param_idx();
                                    #get.into_value().get()
                                };
                            }
                        }
                    }
                } else {
                    match point.mark.kind {
                        MarkKind::Entity => {
                            header = quote! {
                                #header
                                let #var = {
                                    let _inset = _params.#param_idx();
                                    let _mut = _inset.get(#ident.entity).unwrap();
                                    let _host = &_mut;
                                    #get.into_value().get()
                                };
                            }
                        }
                        MarkKind::Resource => {
                            header = quote! {
                                #header
                                let #var = {
                                    let _inset = _params.#param_idx();
                                    let _host = &_inset;
                                    #get.into_value().get()
                                };
                            }
                        }
                    }
                }
            }
        }
        Ok(quote! {
            let (_commands, _params) = ::std::ops::DerefMut::deref_mut(_params);
            #header
        })
    }
    pub fn footer(&self) -> syn::Result<TokenStream> {
        Ok(quote! {})
    }
    pub fn read(&mut self, path: &Path) -> syn::Result<TokenStream> {
        let ident = path.var();
        let idx = self.add_input(&path)?;
        self.reads.insert(path.clone(), idx);
        Ok(quote! { #ident })
    }
    pub fn write(&mut self, path: &Path, value: TokenStream) -> syn::Result<TokenStream> {
        let ident = path.var();
        let (mark, idx) = self.add_output(&path)?;
        let mark_ident = &mark.ident;
        let param = format_ident!("p{idx}");
        let host = match mark.kind {
            MarkKind::Entity => quote! {
                _params.#param().get_mut(#mark_ident.entity).unwrap()
            },
            MarkKind::Resource => quote! {
                _params.#param()
            },
        };
        let notify_change = match mark.kind {
            MarkKind::Entity => {
                let descriptor = &path.0[1];
                quote! {
                    #mark_ident.descriptor().#descriptor().notify_changed(
                        _commands,
                        #mark_ident.entity
                    )
                }
            }
            MarkKind::Resource => quote! {},
        };
        let last = path.0.len() - 2;
        let mut set = quote! { #mark_ident.setters() };
        for (idx, part) in path.0.iter().skip(1).enumerate() {
            let setter = format_ident!("set_{}", part);
            if idx == 0 && idx == last {
                set = quote! { #set.#setter(#host.as_mut(), _val) };
            } else if idx == last {
                set = quote! { #set.#setter(_val)};
            } else if idx == 0 {
                set = quote! { #set.#part(#host.as_mut()) };
            } else {
                set = quote! { #set.#part() };
            }
        }
        self.reads.insert(path.clone(), Some(idx));
        Ok(quote! {
            {
                let _val = (#value).into();
                if #ident != _val {
                    #set;
                    #notify_change;
                }
            }
        })
    }

    pub fn add_input(&mut self, path: &Path) -> syn::Result<Option<usize>> {
        let ident = path.mark();
        Ok(if self.args.contains(&ident) {
            // do nothing, this is an argument
            None
        } else if let Some(mark) = self.ctx.variables.get(&ident).cloned() {
            let point = AccessPoint {
                mark: mark.clone(),
                prop: path.prop(),
                write: false,
            };
            if let Some(idx) = self.access.iter().position(|p| p == &point) {
                Some(idx)
            } else {
                let idx = self.access.len();
                self.access.push(point);
                Some(idx)
            }
        } else {
            throw!(ident, "Undefined mark");
        })
    }
    pub fn add_output(&mut self, path: &Path) -> syn::Result<(Mark, usize)> {
        let ident = path.mark();
        Ok(if self.args.contains(&ident) {
            throw!(ident, "Can't write to hand local mark");
        } else if let Some(mark) = self.ctx.variables.get(&ident).cloned() {
            let point = AccessPoint {
                mark: mark.clone(),
                prop: path.prop(),
                write: true,
            };
            if let Some(idx) = self.access.iter().position(|p| p == &point) {
                self.access[idx].write = true;
                (mark, idx)
            } else {
                let idx = self.access.len();
                self.access.push(point);
                (mark, idx)
            }
        } else {
            throw!(ident, "Undefined mark");
        })
    }
}

#[derive(Clone)]
pub struct Hand {
    locals: Vec<Ident>,
    statements: Vec<Statement>,
}

impl Parse for Hand {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args;
        // throw!(input, "parsing hand");
        parenthesized!(args in input);
        let locals = args
            .parse_terminated(Ident::parse, Token![,])?
            .iter()
            .cloned()
            .collect();
        input.parse::<Token![=>]>()?;
        Ok(Hand {
            locals,
            statements: if input.peek(token::Brace) {
                let stmts;
                braced!(stmts in input);
                Statement::parse_multiple(&stmts)?
            } else {
                vec![input.parse()?]
            },
        })
    }
}

impl Hand {
    pub fn build(&self, ctx: Ref<EmlContext>) -> syn::Result<TokenStream> {
        build(HandBuilder::new(ctx, self.locals.clone()), move |ctx| {
            let mut body = quote! {};
            for stmt in self.statements.iter() {
                let stmt = stmt.build(ctx)?;
                body = quote! { #body #stmt; }
            }
            let signature = ctx.signature()?;
            let header = ctx.header()?;
            Ok(quote! {
                move |#signature| {
                    #header
                    #body
                }
            })
        })
    }
}

#[derive(Clone, Eq)]
pub struct Path(Vec<Ident>);

impl std::ops::Deref for Path {
    type Target = Vec<Ident>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Path {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Path {
    pub fn mark(&self) -> Ident {
        self.0[0].clone()
    }

    pub fn prop(&self) -> Ident {
        self.0[1].clone()
    }

    pub fn var(&self) -> Ident {
        let mut ident = format_ident!("_v_{}", self.0[0]);
        for part in self.0.iter().skip(1) {
            ident = format_ident!("{}_{}", ident, part);
        }
        ident
    }
}

impl<S: AsRef<str>> From<Vec<S>> for Path {
    fn from(value: Vec<S>) -> Self {
        Path(
            value
                .into_iter()
                .map(|s| format_ident!("{}", s.as_ref()))
                .collect(),
        )
    }
}

impl ToString for Path {
    fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(".")
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        let a = self.0.iter().map(|i| i.to_string()).collect::<Vec<_>>();
        let b = other.0.iter().map(|i| i.to_string()).collect::<Vec<_>>();
        a == b
    }
}

impl std::hash::Hash for Path {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for part in self.0.iter() {
            state.write(part.to_string().as_bytes())
        }
    }
}

impl Parse for Path {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut parts = vec![];
        loop {
            // ite is tail method
            if input.peek(Token![.]) && input.peek2(syn::Ident) && input.peek3(token::Paren) {
                if parts.is_empty() {
                    throw!(input, "Expected Path");
                } else {
                    return Ok(Path(parts));
                }
            }
            if input.peek(Token![.]) {
                input.parse::<Token![.]>()?;
            }
            parts.push(input.parse()?);
            if !input.peek(Token![.]) {
                break;
            }
        }
        Ok(Path(parts))
    }
}

#[derive(Clone)]
pub struct Format(LitStr);

impl<S: AsRef<str>> From<S> for Format {
    fn from(value: S) -> Self {
        let value = format!("\"{}\"", value.as_ref());
        Format(parse2(value.parse().unwrap()).unwrap())
    }
}

impl Parse for Format {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![.]>()?;
        let ident = input.parse::<Ident>()?;
        if &ident.to_string() != "fmt" {
            throw![ident, "Expected .fmt(...)"]
        }
        let content;
        parenthesized!(content in input);
        Ok(Format(content.parse()?))
    }
}

#[derive(Clone)]
pub struct Args(Vec<Box<Expr>>);

impl Args {
    pub fn build(&self, ctx: Ref<HandBuilder>) -> syn::Result<TokenStream> {
        let mut args = quote! {};
        for e in self.0.iter() {
            let arg = e.build(ctx.clone())?;
            args = quote! { #args #arg, }
        }
        Ok(args)
    }
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = vec![];
        while input.fork().parse::<Expr>().is_ok() {
            let arg = input.parse()?;
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
            args.push(arg)
        }
        Ok(Args(args))
    }
}

#[derive(Clone)]
pub enum LogStatement {
    Debug(LitStr, Args),
    Info(LitStr, Args),
    Warn(LitStr, Args),
    Error(LitStr, Args),
}

impl Parse for LogStatement {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        let args;
        parenthesized!(args in input);
        let format = args.parse()?;
        if args.peek(Token![,]) {
            args.parse::<Token![,]>()?;
        }
        let args = args.parse()?;
        Ok(match ident.to_string().as_str() {
            "debug" => LogStatement::Debug(format, args),
            "info" => LogStatement::Info(format, args),
            "warn" => LogStatement::Warn(format, args),
            "error" => LogStatement::Error(format, args),
            _ => {
                throw!(ident, "Expected LogStatement");
            }
        })
    }
}

impl LogStatement {
    pub fn peek(input: &syn::parse::ParseStream) -> bool {
        if input.peek(syn::Ident) && input.peek2(Paren) {
            match input.fork().parse::<Ident>().unwrap().to_string().as_str() {
                "debug" => true,
                "info" => true,
                "warn" => true,
                "error" => true,
                _ => false,
            }
        } else {
            false
        }
    }
    pub fn build(&self, ctx: Ref<HandBuilder>) -> syn::Result<TokenStream> {
        let (log, message, args) = match self {
            LogStatement::Debug(message, args) => (quote! { debug }, message, args.build(ctx)?),
            LogStatement::Info(message, args) => (quote! { info }, message, args.build(ctx)?),
            LogStatement::Warn(message, args) => (quote! { warn }, message, args.build(ctx)?),
            LogStatement::Error(message, args) => (quote! { error }, message, args.build(ctx)?),
        };
        Ok(quote! {
            #log!(#message, #args);
        })
    }
}

#[derive(Clone)]
pub enum Statement {
    Assign(Path, Expr),
    Log(LogStatement),
    IfElse(Expr, Vec<Statement>, Option<Box<Statement>>),
    Emit(Path, Expr),
}

impl Parse for Statement {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // log
        // info/warn/error/debug(...)
        Ok(if LogStatement::peek(&input) {
            let log = Statement::Log(input.parse()?);
            input.parse::<Token![;]>()?;
            log

        // if
        // if x > 0 { ... }
        } else if input.peek(Token![if]) {
            input.parse::<Token![if]>()?;
            let expr = input.parse()?;

            let content;
            braced!(content in input);
            let stmts = Statement::parse_multiple(&content)?;
            let then = if input.peek(Token![else]) {
                input.parse::<Token![else]>()?;
                let stmt = input.parse()?;
                Some(Box::new(stmt))
            } else {
                None
            };
            Statement::IfElse(expr, stmts, then)
        } else {
            let path = input.parse::<Path>()?;
            // emit
            // entity.signal.emit(.name: "hello", .value: "23")
            if input.peek(Token![.]) && input.peek2(syn::Ident) && input.peek3(token::Paren) {
                //} && &path.last().unwrap().to_string() == "emit" {
                input.parse::<Token![.]>()?;
                let method = input.parse::<Ident>()?;
                if &method.to_string() == "emit" {
                    let params = Params::parenthesized(input)?;
                    if input.peek(Token![;]) {
                        input.parse::<Token![;]>()?;
                    }
                    Statement::Emit(
                        path,
                        Expr::Construct(Construct {
                            ty: None,
                            flattern: false,
                            params,
                        }),
                    )
                } else {
                    throw!(method, "Only .emit(...) method supported");
                }
            // assign
            // entity.prop = value
            } else {
                input.parse::<Token![=]>()?;
                let expr = input.parse()?;
                input.parse::<Token![;]>()?;
                Statement::Assign(path, expr)
            }
        })
    }
}

impl Statement {
    pub fn parse_multiple<'a>(input: &ParseBuffer<'a>) -> syn::Result<Vec<Statement>> {
        let mut stmts = vec![];
        while !input.is_empty() {
            stmts.push(input.parse()?);
        }
        if !input.is_empty() {
            throw!(input, "Unexpected input.")
        }
        Ok(stmts)
    }

    pub fn build(&self, mut ctx: Ref<HandBuilder>) -> syn::Result<TokenStream> {
        match self {
            Statement::Assign(path, expr) => {
                let expr = expr.build(ctx)?;
                ctx.write(path, expr)
            }
            Statement::Log(log) => log.build(ctx),
            Statement::IfElse(condition, stmts, then) => {
                let expr = condition.build(ctx.clone())?;
                let mut body = quote! {};
                for stmt in stmts.iter() {
                    let stmt = stmt.build(ctx.clone())?;
                    body = quote! { #stmt };
                }
                let then = if let Some(then) = then {
                    let stmt = then.build(ctx.clone())?;
                    quote! { else #stmt }
                } else {
                    quote! {}
                };
                Ok(quote! {
                    if { #expr } {
                        #body
                    } #then
                })
            }
            Statement::Emit(path, expr) => {
                let Expr::Construct(args) = expr else {
                    throw!(path.last().unwrap(), "Non-construct arguments");
                };
                let mark = &path[0];
                let signal = &path[1];
                let args = args.build(ctx)?;
                Ok(quote! {{
                    let _descriptor = #mark.descriptor().#signal();
                    let _args = _descriptor.args().construct(|fields, params| { #args });
                    _commands.add(move |world: &mut ::bevy::prelude::World| {
                        _descriptor.emit(world, #mark.entity, _args);
                    });
                }})
            }
        }
    }
}

#[derive(Clone)]
pub enum Expr {
    Const(Lit),
    Format(Box<Expr>, Format),
    Read(Path),
    Group(Box<Expr>),
    Construct(Construct<Expr>),

    // Expr
    Or(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Ne(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Gte(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Lte(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Not(Box<Expr>),
}

impl Value for Expr {
    type Context = HandBuilder;
    fn build(item: &Self, ctx: Ref<Self::Context>) -> syn::Result<TokenStream> {
        Expr::build(&item, ctx)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Op {
    Or,
    And,
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    Mul,
    Div,
    Add,
    Sub,
    Max,
}

impl Op {
    pub fn priority(&self) -> u8 {
        match self {
            Self::Or => 0,
            Self::And => 1,
            Self::Eq | Self::Ne | Self::Gt | Self::Gte | Self::Lt | Self::Lte => 2,
            Self::Mul | Self::Div => 3,
            Self::Add | Self::Sub => 4,
            Self::Max => 5,
        }
    }

    pub fn priorities() -> std::ops::Range<u8> {
        0..Op::Max.priority()
    }

    pub fn into_binary_expr(self, left: Expr, right: Expr) -> Expr {
        match self {
            Op::Or => Expr::Or(left.into(), right.into()),
            Op::And => Expr::And(left.into(), right.into()),
            Op::Eq => Expr::Eq(left.into(), right.into()),
            Op::Ne => Expr::Ne(left.into(), right.into()),
            Op::Gt => Expr::Gte(left.into(), right.into()),
            Op::Gte => Expr::Gte(left.into(), right.into()),
            Op::Lt => Expr::Lt(left.into(), right.into()),
            Op::Lte => Expr::Lte(left.into(), right.into()),
            Op::Mul => Expr::Mul(left.into(), right.into()),
            Op::Div => Expr::Div(left.into(), right.into()),
            Op::Add => Expr::Add(left.into(), right.into()),
            Op::Sub => Expr::Sub(left.into(), right.into()),
            Op::Max => panic!("Non-binary expression: {self:?}"),
        }
    }
}

impl Expr {
    pub fn flattern(self) -> Vec<(Op, Expr)> {
        let mut root = Some(self);
        let mut flat = vec![];
        let mut op = Op::Max;
        while let Some(node) = root {
            (root, op) = match node {
                Expr::Or(left, right) => {
                    flat.push((op, *left));
                    (Some(*right), Op::Or)
                }
                Expr::And(left, right) => {
                    flat.push((op, *left));
                    (Some(*right), Op::And)
                }
                Expr::Eq(left, right) => {
                    flat.push((op, *left));
                    (Some(*right), Op::Eq)
                }
                Expr::Ne(left, right) => {
                    flat.push((op, *left));
                    (Some(*right), Op::Ne)
                }
                Expr::Mul(left, right) => {
                    flat.push((op, *left));
                    (Some(*right), Op::Mul)
                }
                Expr::Div(left, right) => {
                    flat.push((op, *left));
                    (Some(*right), Op::Div)
                }
                Expr::Add(left, right) => {
                    flat.push((op, *left));
                    (Some(*right), Op::Add)
                }
                Expr::Sub(left, right) => {
                    flat.push((op, *left));
                    (Some(*right), Op::Sub)
                }
                expr => {
                    flat.push((op, expr));
                    (None, Op::Max)
                }
            }
        }
        flat
    }

    pub fn reduce(self) -> Self {
        let mut flat = self.flattern();
        for priority in Op::priorities() {
            let mut idx = 0;
            while flat.len() > idx + 1 {
                if flat[idx + 1].0.priority() > priority {
                    idx += 1;
                    continue;
                }
                let (prev, left) = flat.remove(idx);
                let (op, right) = flat.remove(idx);
                flat.insert(idx, (prev, op.into_binary_expr(left, right)));
            }
        }
        flat.pop().unwrap().1
    }

    pub fn build(&self, mut ctx: Ref<HandBuilder>) -> syn::Result<TokenStream> {
        Ok(match self {
            Expr::Const(lit) => quote! { #lit },
            Expr::Format(expr, Format(lit)) => {
                let expr = expr.build(ctx)?;
                quote! { format!(#lit, #expr) }
            }
            Expr::Group(group) => {
                let group = group.build(ctx)?;
                quote! { ( #group ) }
            }
            Expr::Read(path) => ctx.read(path)?,
            Expr::Construct(cst) => cst.build(ctx)?,
            Expr::Or(left, right) => {
                let left = left.build(ctx.clone())?;
                let right = right.build(ctx.clone())?;
                quote! { #left || #right }
            }
            Expr::And(left, right) => {
                let left = left.build(ctx.clone())?;
                let right = right.build(ctx.clone())?;
                quote! { #left && #right }
            }
            Expr::Eq(left, right) => {
                let left = left.build(ctx.clone())?;
                let right = right.build(ctx.clone())?;
                quote! { #left == #right }
            }
            Expr::Ne(left, right) => {
                let left = left.build(ctx.clone())?;
                let right = right.build(ctx.clone())?;
                quote! { #left != #right }
            }
            Expr::Gte(left, right) => {
                let left = left.build(ctx.clone())?;
                let right = right.build(ctx.clone())?;
                quote! { #left >= #right }
            }
            Expr::Gt(left, right) => {
                let left = left.build(ctx.clone())?;
                let right = right.build(ctx.clone())?;
                quote! { #left > #right }
            }
            Expr::Lte(left, right) => {
                let left = left.build(ctx.clone())?;
                let right = right.build(ctx.clone())?;
                quote! { #left <= #right }
            }
            Expr::Lt(left, right) => {
                let left = left.build(ctx.clone())?;
                let right = right.build(ctx.clone())?;
                quote! { #left < #right }
            }
            Expr::Mul(left, right) => {
                let left = left.build(ctx.clone())?;
                let right = right.build(ctx.clone())?;
                quote! { #left * #right }
            }
            Expr::Div(left, right) => {
                let left = left.build(ctx.clone())?;
                let right = right.build(ctx.clone())?;
                quote! { #left / #right }
            }
            Expr::Add(left, right) => {
                let left = left.build(ctx.clone())?;
                let right = right.build(ctx.clone())?;
                quote! { #left + #right }
            }
            Expr::Sub(left, right) => {
                let left = left.build(ctx.clone())?;
                let right = right.build(ctx.clone())?;
                quote! { #left - #right }
            }
            Expr::Neg(expr) => {
                let expr = expr.build(ctx.clone())?;
                quote! { -#expr }
            }
            Expr::Not(expr) => {
                let expr = expr.build(ctx.clone())?;
                quote! { !#expr }
            }
        })
    }
}

impl From<f32> for Expr {
    fn from(value: f32) -> Self {
        let value = format!("{value}");
        Expr::Const(parse2(value.parse().unwrap()).unwrap())
    }
}
impl From<f32> for Box<Expr> {
    fn from(value: f32) -> Self {
        let value = format!("{value}");
        Box::new(Expr::Const(parse2(value.parse().unwrap()).unwrap()))
    }
}
impl From<i32> for Expr {
    fn from(value: i32) -> Self {
        let value = format!("{value}");
        Expr::Const(parse2(value.parse().unwrap()).unwrap())
    }
}
impl From<i32> for Box<Expr> {
    fn from(value: i32) -> Self {
        let value = format!("{value}");
        Box::new(Expr::Const(parse2(value.parse().unwrap()).unwrap()))
    }
}
impl From<&'static str> for Expr {
    fn from(value: &'static str) -> Self {
        let value = format!("\"{value}\"");
        Expr::Const(parse2(quote! { #value }).unwrap())
    }
}
impl From<&'static str> for Box<Expr> {
    fn from(value: &'static str) -> Self {
        let value = format!("\"{value}\"");
        Box::new(Expr::Const(parse2(quote! { #value }).unwrap()))
    }
}

impl Parse for Expr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut result = None;
        loop {
            if input.is_empty()
                || input.peek(Token![;])
                || input.peek(Token![,])
                || input.peek(token::Brace)
            {
                break;
            }
            let Some(expr) = result else {
                if input.peek(Lit) {
                    result = Some(Expr::Const(input.parse()?));
                } else if input.peek(Token![!]) {
                    input.parse::<Token![!]>()?;
                    result = Some(Expr::Not(input.parse()?));
                } else if input.peek(Token![-]) {
                    input.parse::<Token![-]>()?;
                    result = Some(Expr::Neg(input.parse()?));
                } else if input.peek(Paren) {
                    let group;
                    parenthesized!(group in input);
                    result = Some(Expr::Group(Box::new(group.parse()?)));
                } else if input.fork().parse::<Path>().is_ok() {
                    result = Some(Expr::Read(input.parse()?));
                } else {
                    throw!(input, "Not an hand expression");
                }
                continue;
            };
            if input.peek(Paren) {
                let group;
                parenthesized!(group in input);
                result = Some(Expr::Group(Box::new(group.parse()?)));
                continue;
            }
            if input.fork().parse::<Format>().is_ok() {
                result = Some(Expr::Format(Box::new(expr), input.parse()?));
                continue;
            }
            if input.peek(Token![*]) {
                input.parse::<Token![*]>()?;
                result = Some(Expr::Mul(Box::new(expr), Box::new(input.parse()?)));
                continue;
            }
            if input.peek(Token![/]) {
                input.parse::<Token![/]>()?;
                result = Some(Expr::Div(Box::new(expr), Box::new(input.parse()?)));
                continue;
            }
            if input.peek(Token![+]) {
                input.parse::<Token![+]>()?;
                result = Some(Expr::Add(Box::new(expr), Box::new(input.parse()?)));
                continue;
            }
            if input.peek(Token![-]) {
                input.parse::<Token![-]>()?;
                result = Some(Expr::Sub(Box::new(expr), Box::new(input.parse()?)));
                continue;
            }
            if input.peek(Token![==]) {
                input.parse::<Token![==]>()?;
                result = Some(Expr::Eq(Box::new(expr), Box::new(input.parse()?)));
                continue;
            }
            if input.peek(Token![!=]) {
                input.parse::<Token![!=]>()?;
                result = Some(Expr::Ne(Box::new(expr), Box::new(input.parse()?)));
                continue;
            }
            if input.peek(Token![>=]) {
                input.parse::<Token![>=]>()?;
                result = Some(Expr::Gte(Box::new(expr), Box::new(input.parse()?)));
                continue;
            }
            if input.peek(Token![>]) {
                input.parse::<Token![>]>()?;
                result = Some(Expr::Gt(Box::new(expr), Box::new(input.parse()?)));
                continue;
            }
            if input.peek(Token![<=]) {
                input.parse::<Token![<=]>()?;
                result = Some(Expr::Lte(Box::new(expr), Box::new(input.parse()?)));
                continue;
            }
            if input.peek(Token![<]) {
                input.parse::<Token![<]>()?;
                result = Some(Expr::Lt(Box::new(expr), Box::new(input.parse()?)));
                continue;
            }
            if input.peek(Token![||]) {
                input.parse::<Token![||]>()?;
                result = Some(Expr::Or(Box::new(expr), Box::new(input.parse()?)));
                continue;
            }
            if input.peek(Token![&&]) {
                input.parse::<Token![&&]>()?;
                result = Some(Expr::And(Box::new(expr), Box::new(input.parse()?)));
                continue;
            }
            throw!(input, "Unexpected expression");
        }
        if let Some(expr) = result {
            Ok(expr)
        } else {
            throw!(input, "Expected expression.");
        }
    }
}

impl Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatted = match self {
            Expr::Const(v) => format!("{}", v.clone().into_token_stream().to_string()),
            Expr::Read(path) => format!("read({})", path.to_string()),
            Expr::Format(expr, fmt) => format!("fmt({:?}, \"{}\")", expr, fmt.0.value()),
            Expr::Group(group) => format!("group({:?})", group),
            Expr::Construct(_) => format!("construct(...)"),
            Expr::Or(left, right) => format!("or({:?}, {:?})", left, right),
            Expr::And(left, right) => format!("and({:?}, {:?})", left, right),
            Expr::Eq(left, right) => format!("eq({:?}, {:?})", left, right),
            Expr::Ne(left, right) => format!("ne({:?}, {:?})", left, right),
            Expr::Gt(left, right) => format!("gt({:?}, {:?})", left, right),
            Expr::Gte(left, right) => format!("gte({:?}, {:?})", left, right),
            Expr::Lt(left, right) => format!("lt({:?}, {:?})", left, right),
            Expr::Lte(left, right) => format!("lte({:?}, {:?})", left, right),
            Expr::Mul(left, right) => format!("mul({:?}, {:?})", left, right),
            Expr::Div(left, right) => format!("div({:?}, {:?})", left, right),
            Expr::Add(left, right) => format!("add({:?}, {:?})", left, right),
            Expr::Sub(left, right) => format!("sub({:?}, {:?})", left, right),
            Expr::Neg(expr) => format!("neg({:?})", expr),
            Expr::Not(expr) => format!("not({:?})", expr),
        };
        f.write_str(&formatted)?;
        Ok(())
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Expr::Const(ca), Expr::Const(cb)) => {
                let ca = ca.clone().into_token_stream().to_string();
                let cb = cb.clone().into_token_stream().to_string();
                ca == cb
            }
            (Expr::Group(group_a), Expr::Group(group_b)) => group_a == group_b,
            (Expr::Format(expr_a, fmt_a), Expr::Format(expr_b, fmt_b)) => {
                expr_a == expr_b && fmt_a.0.value() == fmt_b.0.value()
            }
            (Expr::Read(path_a), Expr::Read(path_b)) => path_a == path_b,
            (Expr::Add(left_a, right_a), Expr::Add(left_b, right_b)) => {
                left_a == left_b && right_a == right_b
            }
            (Expr::Mul(left_a, right_a), Expr::Mul(left_b, right_b)) => {
                left_a == left_b && right_a == right_b
            }
            (Expr::Sub(left_a, right_a), Expr::Sub(left_b, right_b)) => {
                left_a == left_b && right_a == right_b
            }
            (Expr::Div(left_a, right_a), Expr::Div(left_b, right_b)) => {
                left_a == left_b && right_a == right_b
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use syn::parse2;

    use super::*;
    fn expr(from: &'static str) -> Expr {
        let expr = parse2::<Expr>(from.parse().unwrap()).unwrap();
        expr.reduce()
    }

    fn add<A: Into<Box<Expr>>, B: Into<Box<Expr>>>(a: A, b: B) -> Expr {
        Expr::Add(a.into(), b.into())
    }
    fn sub<A: Into<Box<Expr>>, B: Into<Box<Expr>>>(a: A, b: B) -> Expr {
        Expr::Sub(a.into(), b.into())
    }
    fn mul<A: Into<Box<Expr>>, B: Into<Box<Expr>>>(a: A, b: B) -> Expr {
        Expr::Mul(a.into(), b.into())
    }
    fn div<A: Into<Box<Expr>>, B: Into<Box<Expr>>>(a: A, b: B) -> Expr {
        Expr::Div(a.into(), b.into())
    }
    fn read<S: AsRef<str>>(value: S) -> Expr {
        Expr::Read(Path(
            value
                .as_ref()
                .split(".")
                .map(|s| format_ident!("{s}"))
                .collect(),
        ))
    }
    fn group<G: Into<Box<Expr>>>(value: G) -> Expr {
        Expr::Group(value.into())
    }
    #[test]
    fn test_expr_basics() {
        assert_eq!(expr("a.b"), read("a.b"));
        assert_eq!(
            expr("a.b.fmt(\"{}\")"),
            Expr::Format(read("a.b").into(), "{}".into()),
        );
        assert_eq!(expr("42"), 42.into());
        assert_eq!(expr("1 + 2"), add(1, 2));
        assert_eq!(expr("1 - 2"), sub(1, 2));
        assert_eq!(expr("1 * 2"), mul(1, 2));
        assert_eq!(expr("1 / 2"), div(1, 2));
    }
    #[test]
    fn test_expr_basic_op_priority() {
        let e = expr("1 + 2 * 3");
        assert_eq!(e, add(1, mul(2, 3)));
        let e = expr("1 * 2 + 3");
        assert_eq!(e, add(mul(1, 2), 3));
        let e = expr("1 * 2 + 3 * 4");
        assert_eq!(e, add(mul(1, 2), mul(3, 4),));
        let e = expr("1 + 2 * 3 + 4");
        assert_eq!(e, add(add(1, mul(2, 3)), 4));
        let e = expr("1 - 2 + 3");
        assert_eq!(e, add(sub(1, 2), 3));
        let e = expr("1 - 2 * 3 + 4");
        assert_eq!(e, add(sub(1, mul(2, 3)), 4));
        let e = expr("(1 - 2) * 3 + 4");
        assert_eq!(e, add(mul(group(sub(1, 2)), 3), 4));
        let e = expr("1 - 2 * (3 + 4)");
        assert_eq!(e, sub(1, mul(2, group(add(3, 4)))));
        let e = expr("1 - 2 * 3 - 4");
        assert_eq!(e, sub(sub(1, mul(2, 3)), 4));
        let e = expr("1 * 2 - 3 * 4");
        assert_eq!(e, sub(mul(1, 2), mul(3, 4)));
        let e = expr("1 * 2 - 3 * 4 + 5 * 6");
        assert_eq!(e, add(sub(mul(1, 2), mul(3, 4)), mul(5, 6)));
        // [1, 2, 3].iter().fo
        let e = expr("1 / 2 * 3");
        assert_eq!(e, mul(div(1, 2), 3));
        let e = expr("1 / 2 / 3");
        assert_eq!(e, div(div(1, 2), 3));
        let e = expr("1 * 2 * 3 + 4 * 5 * 6");
        assert_eq!(e, add(mul(mul(1, 2), 3), mul(mul(4, 5), 6),));
        let e = expr("1 / 2 * 3 + 4 / 5 * 6");
        assert_eq!(e, add(mul(div(1, 2), 3), mul(div(4, 5), 6),));
        let e = expr("1 / 2 / 3 + 4 / 5 / 6");
        assert_eq!(e, add(div(div(1, 2), 3), div(div(4, 5), 6),));
        let e = expr("1 - 2 / 3 / 4 - 5 / 6");
        assert_eq!(e, sub(sub(1, div(div(2, 3), 4)), div(5, 6)));
    }
    #[test]
    fn test_expr_prop_op_priority() {
        let e = expr("a.b + c.d * e.f");
        assert_eq!(e, add(read("a.b"), mul(read("c.d"), read("e.f"))));
        let e = expr("a.b * c.d + e.f");
        assert_eq!(e, add(mul(read("a.b"), read("c.d")), read("e.f")));
        let e = expr("a.b * 1 + b.c * 2");
        assert_eq!(e, add(mul(read("a.b"), 1), mul(read("b.c"), 2)));
        let e = expr("1 / 2 - a.b * 3 + 4");
        assert_eq!(e, add(sub(div(1, 2), mul(read("a.b"), 3)), 4));
    }
}
