use syn::parse::Parse;
use syn::punctuated::Punctuated;

use super::expr;
use super::expr::LValue;
use super::DType;

pub trait WeakSpanned {
  fn span(&self) -> proc_macro2::Span;
}

/// A port declaration is something like `a: int<32>`.
pub(crate) struct PortDecl {
  pub(crate) id: syn::Ident,
  pub(crate) ty: DType,
}

/// An array access is something like `a[<expr>]`
pub(crate) struct ArrayAccess {
  pub(crate) id: syn::Ident,
  pub(crate) idx: Box<expr::Expr>,
}

impl Parse for ArrayAccess {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let id = input.parse::<syn::Ident>()?;
    let idx;
    syn::bracketed!(idx in input);
    let idx = Box::new(idx.parse::<expr::Expr>()?);
    Ok(Self { id, idx })
  }
}

/// A function call is something like `foo((<expr>,)*)`.
pub(crate) struct FuncCall {
  pub(crate) func: syn::Ident,
  pub(crate) args: FuncArgs,
}

/// Function arguments can either be {a: v0, b: v1}, or (v0, v1).
pub(crate) enum FuncArgs {
  Bound(Vec<(syn::Ident, expr::Expr)>),
  Plain(Vec<expr::Expr>),
}

/// Key value pair is a component of FuncArgs::Bound.
pub(crate) struct KVPair {
  pub(crate) key: syn::Ident,
  pub(crate) value: expr::Expr,
}

/// A body is a sequence of instructions.
pub(crate) struct Body {
  pub(crate) stmts: Vec<Statement>,
  pub(crate) valued: bool,
}

pub(crate) enum BodyPred {
  WaitUntil(Box<Body>),
  Condition(expr::Expr),
  Cycle(syn::LitInt),
}

pub(crate) enum CallKind {
  Inline(Punctuated<syn::Ident, syn::Token![,]>),
  Async,
}

// TODO(@were): Add a span to this data structure.
pub(crate) enum Statement {
  Assign((LValue, expr::Expr)),
  Call((CallKind, FuncCall)),
  BodyScope((BodyPred, Box<Body>)),
  Log(Vec<expr::Expr>),
  ExprTerm(expr::ExprTerm),
}
