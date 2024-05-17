use crate::ir::node::BaseNode;

use super::{GetElementPtr, Load, Store};

impl Load<'_> {
  pub fn get_pointer(&self) -> GetElementPtr {
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
  pub fn get_pointer(&self) -> GetElementPtr {
    self
      .expr
      .get_operand(0)
      .unwrap()
      .get_value()
      .clone()
      .as_expr(self.expr.sys)
      .unwrap()
  }

  pub fn get_value(&self) -> BaseNode {
    self.expr.get_operand(1).unwrap().get_value().clone()
  }
}
