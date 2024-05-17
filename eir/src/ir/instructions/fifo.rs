use crate::ir::{
  expr::subcode,
  node::{BaseNode, FIFORef},
  Opcode, FIFO,
};

use super::{FIFOField, FIFOPop, FIFOPush};

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

impl FIFOPop<'_> {
  pub fn get_fifo(&self) -> FIFORef<'_> {
    self
      .expr
      .get_operand(0)
      .unwrap()
      .get_value()
      .as_ref::<FIFO>(self.expr.sys)
      .unwrap()
  }
}

impl FIFOField<'_> {
  pub fn get_fifo(&self) -> FIFORef<'_> {
    self
      .expr
      .get_operand(0)
      .unwrap()
      .get_value()
      .as_ref::<FIFO>(self.expr.sys)
      .unwrap()
  }

  pub fn get_field(&self) -> subcode::FIFO {
    match self.expr.get_opcode() {
      Opcode::FIFOField { field } => field,
      _ => unreachable!(),
    }
  }
}
