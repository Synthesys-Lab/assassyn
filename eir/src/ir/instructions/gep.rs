use crate::ir::{
  node::{ArrayRef, BaseNode, NodeKind},
  Array,
};

use super::GetElementPtr;

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
