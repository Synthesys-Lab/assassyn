use crate::ir::node::IsElement;

use super::{Load, Store};

impl ToString for Load<'_> {
  fn to_string(&self) -> String {
    let array = self.array().get_name().to_string();
    let idx = self.idx().to_string(self.expr.sys);
    format!("{} = {}[{}]", self.expr.get_name(), array, idx)
  }
}

impl ToString for Store<'_> {
  fn to_string(&self) -> String {
    let array = self.array().get().upcast().to_string(self.expr.sys);
    let idx = self.idx().to_string(self.expr.sys);
    let value = self.value().to_string(self.expr.sys);
    format!("{}[{}] = {}", array, idx, value)
  }
}
