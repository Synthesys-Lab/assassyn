use syn::parse::Parse;
use syn::punctuated::Punctuated;

use super::expr;
use super::DType;
use super::ExprTerm;

/// A port declaration is something like `a: int<32>`.
pub(crate) struct PortDecl {
  pub(crate) id: syn::Ident,
  pub(crate) ty: DType,
}

/// An array access is something like `a[<expr>]`
pub(crate) struct ArrayAccess {
  pub(crate) id: syn::Ident,
  pub(crate) idx: ExprTerm,
}

impl Parse for ArrayAccess {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let id = input.parse::<syn::Ident>()?;
    let idx;
    syn::bracketed!(idx in input);
    let idx = idx.parse::<ExprTerm>()?;
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
  Bound(Vec<(syn::Ident, ExprTerm)>),
  Plain(Vec<ExprTerm>),
}

/// Key value pair is a component of FuncArgs::Bound.
pub(crate) struct KVPair {
  pub(crate) key: syn::Ident,
  pub(crate) value: ExprTerm,
}

/// A body is a sequence of instructions.
pub(crate) struct Body {
  pub(crate) stmts: Vec<Statement>,
  pub(crate) valued: bool,
}

pub(crate) enum BodyPred {
  WaitUntil(Box<Body>),
  Condition(syn::Ident),
  Cycle(syn::LitInt),
}

pub(crate) enum CallKind {
  Inline(Punctuated<syn::Ident, syn::Token![,]>),
  Async,
  Spin(ArrayAccess),
}

pub(crate) enum Statement {
  Assign((syn::Ident, expr::Expr)),
  ArrayAlloc((syn::Ident, DType, syn::LitInt)),
  ArrayAssign((ArrayAccess, expr::Expr)),
  ArrayRead((syn::Ident, ArrayAccess)),
  Call((CallKind, FuncCall)),
  Bind((syn::Ident, FuncCall, bool)),
  BodyScope((BodyPred, Box<Body>)),
  Log(Vec<expr::Expr>),
  ExprTerm(expr::ExprTerm),
}
