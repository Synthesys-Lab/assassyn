use crate::ir::{
  node::{ArrayRef, BaseNode, NodeKind},
  Array,
};

use super::{GetElementPtr, Load, Store};

impl Load<'_> {
  pub fn pointer(&self) -> GetElementPtr {
    self
      .expr
      .get_operand(0)
      .unwrap()
      .get_value()
      .clone()
      .as_expr(self.expr.sys)
      .unwrap()
  }
}

impl Store<'_> {
  pub fn pointer(&self) -> GetElementPtr {
    self
      .expr
      .get_operand_value(0)
      .unwrap()
      .as_expr(self.expr.sys)
      .unwrap()
  }

  pub fn value(&self) -> BaseNode {
    self.expr.get_operand(1).unwrap().get_value().clone()
  }
}

impl<'a> GetElementPtr<'a> {
  pub fn array(&self) -> ArrayRef<'a> {
    self
      .expr
      .get_operand_value(0)
      .unwrap()
      .as_ref::<Array>(self.expr.sys)
      .unwrap()
  }

  pub fn index(&self) -> BaseNode {
    self.expr.get_operand(1).unwrap().get_value().clone()
  }

  pub fn is_const(&self) -> bool {
    match self.expr.get_operand(1).unwrap().get_value().get_kind() {
      NodeKind::IntImm => true,
      _ => false,
    }
  }
}
