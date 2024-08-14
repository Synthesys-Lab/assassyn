use std::fmt::Display;

use crate::ir::{
  node::{BaseNode, IsElement, ModuleRef},
  Module,
};

use super::{AsyncCall, Bind, FIFOPush};

impl<'sys> Bind<'sys> {
  /// Get the arguments of this bind expression.
  pub fn get_arg(&self, key: &str) -> Option<BaseNode> {
    let n = self.get_num_args();
    self
      .expr
      .operand_iter()
      .take(n)
      .find(|x| {
        x.get_value()
          .as_expr::<FIFOPush>(self.expr.sys)
          .unwrap()
          .fifo()
          .get_name()
          .eq(key)
      })
      .map(|x| *x.get_value())
  }

  /// Get the callee of this bind expression.
  pub fn callee<'res, 'borrow>(&'borrow self) -> ModuleRef<'res>
  where
    'sys: 'res,
    'sys: 'borrow,
    'borrow: 'res,
  {
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
    let n = self.expr.get_num_operands() - 1;
    self.expr.operand_iter().take(n).map(|x| *x.get_value())
  }
  /// Check if all arguments are fully bound.
  pub fn fully_bound(&self) -> bool {
    self.callee().get_num_inputs() == self.get_num_args()
  }
}

impl Display for Bind<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let callee = self.callee();
    let arg_list = self
      .arg_iter()
      .map(|arg| {
        let fifo_push = arg.as_expr::<FIFOPush>(self.expr.sys).unwrap();
        let value = fifo_push.value().to_string(self.expr.sys);
        format!("{}: {}", fifo_push.fifo().get_name(), value)
      })
      .collect::<Vec<String>>()
      .join(", ");
    write!(f, "bind {} {{ {} }}", callee.get_name(), arg_list)
  }
}

impl Display for AsyncCall<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let bind = self.bind().get().upcast().to_string(self.expr.sys);
    write!(f, "async_call {}", bind)
  }
}
