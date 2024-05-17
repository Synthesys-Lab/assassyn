use crate::ir::node::NodeKind;

use super::GetElementPtr;

impl<'a> GetElementPtr<'a> {
  pub fn is_const(&self) -> bool {
    match self.expr.get_operand(1).unwrap().get_value().get_kind() {
      NodeKind::IntImm => true,
      _ => false,
    }
  }
}
