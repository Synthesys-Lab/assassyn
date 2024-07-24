use std::fmt::Display;

use crate::ir::node::IsElement;

use super::{Load, Store};

impl Display for Load<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let array = self.array().get_name().to_string();
    let idx = self.idx().to_string(self.expr.sys);
    write!(f, "{} = {}[{}]", self.expr.get_name(), array, idx)
  }
}

impl Display for Store<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let array = self.array().get().upcast().to_string(self.expr.sys);
    let idx = self.idx().to_string(self.expr.sys);
    let value = self.value().to_string(self.expr.sys);
    write!(f, "{}[{}] = {}", array, idx, value)
  }
}
