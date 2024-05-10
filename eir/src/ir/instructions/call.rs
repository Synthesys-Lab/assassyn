use crate::ir::node::BaseNode;

use super::Bind;

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
