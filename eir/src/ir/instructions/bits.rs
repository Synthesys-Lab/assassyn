use crate::ir::{node::BaseNode, IntImm};

use super::{Slice, Concat};

impl Slice<'_> {
  pub fn x(&self) -> BaseNode {
    self.expr.get_operand(0).unwrap().get_value().clone()
  }

  pub fn l(&self) -> usize {
    self
      .expr
      .get_operand(1)
      .unwrap()
      .get_value()
      .as_ref::<IntImm>(self.expr.sys)
      .unwrap()
      .get_value() as usize
  }

  pub fn r(&self) -> usize {
    self
      .expr
      .get_operand(2)
      .unwrap()
      .get_value()
      .as_ref::<IntImm>(self.expr.sys)
      .unwrap()
      .get_value() as usize
  }
}

impl Concat<'_> {

  pub fn msb(&self) -> BaseNode {
    self.expr.get_operand(0).unwrap().get_value().clone()
  }

  pub fn lsb(&self) -> BaseNode {
    self.expr.get_operand(1).unwrap().get_value().clone()
  }

}
