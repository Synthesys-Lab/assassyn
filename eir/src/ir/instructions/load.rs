use crate::ir::node::BaseNode;

use super::Load;

impl Load<'_> {
  pub fn get_pointer(&self) -> BaseNode {
    self.expr.get_operand(0).unwrap().get_value().clone()
  }
}
