use crate::builder::SysBuilder;

use super::{
  node::{BaseNode, ExprRef},
  Expr, Opcode, Operand,
};

// Util procedures for bind expressions.

pub(crate) fn as_bind_expr(sys: &SysBuilder, node: BaseNode) -> Option<ExprRef<'_>> {
  node
    .as_ref::<Expr>(sys)
    .map_or(None, |x| match x.get_opcode() {
      Opcode::Bind => Some(x),
      _ => {
        eprintln!("{:?}", x.get_opcode());
        None
      }
    })
}

pub(crate) fn is_fully_bound(sys: &SysBuilder, node: BaseNode) -> bool {
  let bind_expr = as_bind_expr(sys, node).unwrap();
  let mut operands = bind_expr.operand_iter();
  operands.all(|x| !x.get_value().is_unknown())
}

pub(crate) fn get_bind_callee(sys: &SysBuilder, node: BaseNode) -> BaseNode {
  let bind_expr = as_bind_expr(sys, node).unwrap();
  bind_expr
    .operands
    .last()
    .unwrap()
    .as_ref::<Operand>(sys)
    .unwrap()
    .get_value()
    .clone()
}
