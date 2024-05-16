use crate::ir::{
  node::{BaseNode, FIFORef},
  FIFO,
};

use super::FIFOPush;

impl FIFOPush<'_> {
  pub fn get_fifo(&self) -> FIFORef<'_> {
    self
      .expr
      .get_operand(0)
      .unwrap()
      .get_value()
      .as_ref::<FIFO>(self.expr.sys)
      .unwrap()
  }

  pub fn get_value(&self) -> BaseNode {
    self.expr.get_operand(1).unwrap().get_value().clone()
  }
}
