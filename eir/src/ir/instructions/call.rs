use crate::ir::{
  node::{BaseNode, IsElement, ModuleRef},
  Module,
};

use super::{AsyncCall, Bind};

impl Bind<'_> {
  /// Get the arguments of this bind expression.
  pub fn get_arg(&self, i: usize) -> Option<BaseNode> {
    if i < self.expr.get_num_operands() - 1 {
      self.expr.get_operand(i).map(|x| x.get_value().clone())
    } else {
      None
    }
  }
  /// Get the callee of this bind expression.
  pub fn callee(&self) -> ModuleRef<'_> {
    self
      .expr
      .get_operand_value(self.get().get_num_operands() - 1)
      .unwrap()
      .as_ref::<Module>(self.get().sys)
      .unwrap()
  }
  /// Get the number of arguments of the callee.
  pub fn get_num_args(&self) -> usize {
    self.expr.get_num_operands() - 1
  }
  /// Get an iterator over all arguments.
  pub fn arg_iter(&self) -> impl Iterator<Item = BaseNode> + '_ {
    (0..self.get_num_args()).map(|i| self.get_arg(i).unwrap())
  }
  /// Check if all arguments are fully bound.
  pub fn fully_bound(&self) -> bool {
    let n = self.expr.get_num_operands();
    self
      .expr
      .operand_iter()
      .take(n)
      .all(|x| !x.get_value().is_unknown())
  }
}

impl ToString for Bind<'_> {
  fn to_string(&self) -> String {
    let callee = self.callee();
    let arg_list = self
      .arg_iter()
      .enumerate()
      .map(|(i, v)| {
        let arg = callee.get_port(i).unwrap().get_name().to_string();
        let feed = if v.is_unknown() {
          "None".to_string()
        } else {
          v.to_string(self.expr.sys)
        };
        format!("{} {}", arg, feed)
      })
      .collect::<Vec<String>>()
      .join(", ");
    format!("bind {} {{ {} }}", callee.get_name(), arg_list).into()
  }
}

impl ToString for AsyncCall<'_> {
  fn to_string(&self) -> String {
    let bind = self.bind().get().upcast().to_string(self.expr.sys);
    format!("async_call {}", bind)
  }
}
