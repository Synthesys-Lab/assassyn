use super::DType;
use super::Expr;

pub(crate) struct Argument {
  pub(crate) id: syn::Ident,
  pub(crate) ty: DType,
}

pub(crate) struct ArrayAccess {
  pub(crate) id: syn::Ident,
  pub(crate) idx: Expr,
}

pub(crate) struct FuncCall {
  pub(crate) func: syn::Ident,
  pub(crate) args: Vec<(syn::Ident, Expr)>,
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
  SpinCall((ArrayAccess, FuncCall)),
  When((syn::Ident, Box<Body>)),
}
