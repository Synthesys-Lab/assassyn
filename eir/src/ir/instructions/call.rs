use crate::ir::node::BaseNode;

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
  pub fn get_callee(&self) -> BaseNode {
    self
      .expr
      .get_operand(self.expr.get_num_operands() - 1)
      .unwrap()
      .get_value()
      .clone()
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

impl AsyncCall<'_> {
  pub fn get_bind(&self) -> Bind<'_> {
    return self
      .expr
      .get_operand(0)
      .unwrap()
      .get_value()
      .as_expr(self.expr.sys)
      .unwrap();
  }
}
