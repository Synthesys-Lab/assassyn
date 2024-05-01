use crate::builder::SysBuilder;

use super::{
  node::{BaseNode, ExprRef},
  Expr, Opcode,
};

// Util procedures for bind expressions.

pub(crate) fn as_bind_expr(sys: &SysBuilder, node: BaseNode) -> Option<ExprRef<'_>> {
  node
    .as_ref::<Expr>(sys)
    .map_or(None, |x| match x.get_opcode() {
      Opcode::Bind(_) => Some(x),
      _ => None,
    })
}

pub(crate) fn is_full(sys: &SysBuilder, node: BaseNode) -> bool {
  let bind_expr = as_bind_expr(sys, node).unwrap();
  bind_expr.operands.iter().skip(1).all(|x| !x.is_unknown())
}
