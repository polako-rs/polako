use std::fmt::Debug;

use constructivist::throw;
use proc_macro2::Ident;
use quote::{ToTokens, format_ident, quote};
use syn::{Lit, parse::Parse, Token, token, LitStr, parenthesized, parse2};

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
pub struct Path(Vec<Ident>);

impl<S: AsRef<str>> From<Vec<S>> for Path {
    fn from(value: Vec<S>) -> Self {
        Path(value.into_iter().map(|s| format_ident!("{}", s.as_ref())).collect())
    }
}

impl ToString for Path {
    fn to_string(&self) -> String {
        self.0.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(".")
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        let a = self.0.iter().map(|i| i.to_string()).collect::<Vec<_>>();
        let b = other.0.iter().map(|i| i.to_string()).collect::<Vec<_>>();
        a == b
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
                    return Ok(Path(parts))
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

pub enum Statement {
    Assign(Path, Expr),
}

pub enum Expr {
    Const(Lit),
    Format(Box<Expr>, Format),
    Read(Path),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

#[derive(Clone, Copy, Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Max,
}

impl Op {
    pub fn priority(&self) -> u8 {
        match self {
            Self::Mul |
            Self::Div => 0,
            Self::Add |
            Self::Sub => 1,
            Self::Max => 2
        }
    }

    pub fn priorities() -> std::ops::Range<u8> {
        0..Op::Max.priority()
    }

    pub fn into_expr(self, left: Expr, right: Expr) -> Expr {
        match self {
            Op::Add => Expr::Add(left.into(), right.into()),
            Op::Sub => Expr::Sub(left.into(), right.into()),
            Op::Mul => Expr::Mul(left.into(), right.into()),
            Op::Div => Expr::Div(left.into(), right.into()),
            Op::Max => panic!("Non-binary expression: {self:?}")
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
                Expr::Add(left, right) => {
                    flat.push((op, *left));
                    (Some(*right), Op::Add)
                },
                Expr::Sub(left, right) => {
                    flat.push((op, *left));
                    (Some(*right), Op::Sub)
                },
                Expr::Mul(left, right) => {
                    flat.push((op, *left));
                    (Some(*right), Op::Mul)
                },
                Expr::Div(left, right) => {
                    flat.push((op, *left));
                    (Some(*right), Op::Div)
                },
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
                flat.insert(idx, (prev, op.into_expr(left, right)));
            }
        }
        flat.pop().unwrap().1
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
        let mut step = None;
        let span = input.fork();
        loop {
            if input.is_empty() || input.peek(Token![;]) {
                break;
            }
            let Some(expr) = step else {
                if input.peek(Lit) {
                    step = Some(Expr::Const(input.parse()?));
                } else if input.fork().parse::<Path>().is_ok() {
                    step = Some(Expr::Read(input.parse()?));
                } else {
                    throw!(input, "Not an hand expression");
                }
                continue;
            };
            if input.fork().parse::<Format>().is_ok() {
                step = Some(Expr::Format(Box::new(expr), input.parse()?));
                continue;
            }
            if input.peek(Token![*]) {
                input.parse::<Token![*]>()?;
                step = Some(Expr::Mul(Box::new(expr), Box::new(input.parse()?)));
                continue;
            }
            if input.peek(Token![/]) {
                input.parse::<Token![/]>()?;
                step = Some(Expr::Div(Box::new(expr), Box::new(input.parse()?)));
                continue;
            }
            if input.peek(Token![+]) {
                input.parse::<Token![+]>()?;
                step = Some(Expr::Add(Box::new(expr), Box::new(input.parse()?)));
                continue;
            }
            if input.peek(Token![-]) {
                input.parse::<Token![-]>()?;
                step = Some(Expr::Sub(Box::new(expr), Box::new(input.parse()?)));
                continue;
            }
            throw!(input, "Unexpected expression");
        }
        if let Some(expr) = step {
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
            Expr::Add(left, right) => format!("add({:?}, {:?})", left, right),
            Expr::Sub(left, right) => format!("sub({:?}, {:?})", left, right),
            Expr::Mul(left, right) => format!("mul({:?}, {:?})", left, right),
            Expr::Div(left, right) => format!("div({:?}, {:?})", left, right),
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
            },
            (Expr::Format(expr_a, fmt_a), Expr::Format(expr_b, fmt_b)) => {
                expr_a == expr_b && fmt_a.0.value() == fmt_b.0.value()
            },
            (Expr::Read(path_a), Expr::Read(path_b)) => {
                path_a == path_b
            },
            (Expr::Add(left_a, right_a), Expr::Add(left_b, right_b)) => {
                left_a == left_b && right_a == right_b
            },
            (Expr::Mul(left_a, right_a), Expr::Mul(left_b, right_b)) => {
                left_a == left_b && right_a == right_b
            },
            (Expr::Sub(left_a, right_a), Expr::Sub(left_b, right_b)) => {
                left_a == left_b && right_a == right_b
            },
            (Expr::Div(left_a, right_a), Expr::Div(left_b, right_b)) => {
                left_a == left_b && right_a == right_b
            },
            _ => false
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
        Expr::Read(Path(value.as_ref().split(".").map(|s| format_ident!("{s}")).collect()))
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
        assert_eq!(e, add(
            mul(1, 2),
            mul(3, 4),
        ));
        let e = expr("1 + 2 * 3 + 4");
        assert_eq!(e, add(
            add(1, mul(2, 3)), 4
        ));
        let e = expr("1 - 2 + 3");
        assert_eq!(e, add(sub(1, 2), 3));
        let e = expr("1 - 2 * 3 + 4");
        assert_eq!(e, add(
            sub(1, mul(2, 3)), 4
        ));
        let e = expr("1 - 2 * 3 - 4");
        assert_eq!(e, sub(
            sub(1, mul(2, 3)), 4
        ));
        let e = expr("1 * 2 - 3 * 4");
        assert_eq!(e, sub(mul(1, 2), mul(3,4)));
        let e = expr("1 * 2 - 3 * 4 + 5 * 6");
        assert_eq!(e, add(
            sub(mul(1, 2), mul(3,4)),
            mul(5, 6)
        ));
        // [1, 2, 3].iter().fo
        let e = expr("1 / 2 * 3");
        assert_eq!(e, mul(div(1, 2), 3));
        let e = expr("1 / 2 / 3");
        assert_eq!(e, div(div(1, 2), 3));
        let e = expr("1 * 2 * 3 + 4 * 5 * 6");
        assert_eq!(e, add(
            mul(mul(1, 2), 3),
            mul(mul(4, 5), 6),
        ));
        let e = expr("1 / 2 * 3 + 4 / 5 * 6");
        assert_eq!(e, add(
            mul(div(1, 2), 3),
            mul(div(4, 5), 6),
        ));
        let e = expr("1 / 2 / 3 + 4 / 5 / 6");
        assert_eq!(e, add(
            div(div(1, 2), 3),
            div(div(4, 5), 6),
        ));
        let e = expr("1 - 2 / 3 / 4 - 5 / 6");
        assert_eq!(e, sub(
            sub(1, div(div(2, 3), 4)),
            div(5, 6)
        ));
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
        assert_eq!(e, add(
            sub(div(1, 2), mul(read("a.b"), 3)), 4
        ));
    }
}