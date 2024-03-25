use super::DType;
use super::Expr;

pub(crate) struct PortDecl {
  pub(crate) id: syn::Ident,
  pub(crate) ty: DType,
}

pub(crate) struct ArrayAccess {
  pub(crate) id: syn::Ident,
  pub(crate) idx: Expr,
}

pub(crate) struct FuncCall {
  pub(crate) func: syn::Ident,
  pub(crate) args: FuncArgs,
}

pub(crate) enum FuncArgs {
  Bound(Vec<(syn::Ident, Expr)>),
  Plain(Vec<Expr>),
}

pub(crate) struct KVPair {
  pub(crate) key: syn::Ident,
  pub(crate) value: Expr,
}

pub(crate) struct Body {
  pub(crate) stmts: Vec<Instruction>,
}

pub(crate) enum Instruction {
  Assign((syn::Ident, syn::Expr)),
  ArrayAlloc((syn::Ident, DType, syn::LitInt)),
  ArrayAssign((ArrayAccess, syn::Expr)),
  ArrayRead((syn::Ident, ArrayAccess)),
  AsyncCall(FuncCall),
  Bind((syn::Ident, FuncCall)),
  SpinCall((ArrayAccess, FuncCall)),
  When((syn::Ident, Box<Body>)),
}
