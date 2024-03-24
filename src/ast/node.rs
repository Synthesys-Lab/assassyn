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
