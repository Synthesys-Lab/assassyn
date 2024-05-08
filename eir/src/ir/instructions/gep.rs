use crate::ir::{
  node::{ArrayRef, BaseNode, ExprRef, NodeKind},
  Array, Opcode,
};

use super::AsExpr;

pub struct GetElementPtr<'a> {
  expr: ExprRef<'a>,
}

impl<'a> GetElementPtr<'a> {
  pub fn get_array(&self) -> ArrayRef<'a> {
    self
      .expr
      .get_operand(0)
      .unwrap()
      .get_value()
      .as_ref::<Array>(self.expr.sys)
      .unwrap()
  }

  pub fn get_index(&self) -> BaseNode {
    self.expr.get_operand(1).unwrap().get_value().clone()
  }

  pub fn is_const(&self) -> bool {
    match self.expr.get_operand(1).unwrap().get_value().get_kind() {
      NodeKind::IntImm => true,
      _ => false,
    }
  }
}

impl<'a> AsExpr<'a> for GetElementPtr<'a> {
  fn downcast(expr: ExprRef<'a>) -> Result<Self, String> {
    if expr.get_opcode() == Opcode::GetElementPtr {
      Ok(GetElementPtr { expr })
    } else {
      Err(format!(
        "Expecting Opcode::GetElementPtr, but got {:?}",
        expr.get_opcode().to_string()
      ))
    }
  }
}
