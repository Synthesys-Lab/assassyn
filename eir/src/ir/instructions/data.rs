use crate::ir::node::{IsElement, NodeKind};

use super::{GetElementPtr, Load, Store};

impl<'a> GetElementPtr<'a> {
  pub fn is_const(&self) -> bool {
    match self.expr.get_operand(1).unwrap().get_value().get_kind() {
      NodeKind::IntImm => true,
      _ => false,
    }
  }
}

impl ToString for GetElementPtr<'_> {
  fn to_string(&self) -> String {
    format!(
      "{} = &{}[{}]",
      self.expr.get_name(),
      self.array().get_name(),
      self.index().to_string(self.expr.sys)
    )
  }
}

impl ToString for Load<'_> {
  fn to_string(&self) -> String {
    let ptr = self.pointer().get().upcast().to_string(self.expr.sys);
    format!("{} = *{}", self.expr.get_name(), ptr)
  }
}

impl ToString for Store<'_> {
  fn to_string(&self) -> String {
    let ptr = self.pointer().get().upcast().to_string(self.expr.sys);
    let value = self.value().to_string(self.expr.sys);
    format!("*{} = {}", ptr, value)
  }
}
