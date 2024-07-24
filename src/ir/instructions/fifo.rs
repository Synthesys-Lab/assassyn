use std::fmt::Display;

use crate::ir::{
  expr::subcode,
  node::{FIFORef, IsElement, Parented},
  visitor::Visitor,
  Opcode,
};

use super::{FIFOField, FIFOPop, FIFOPush};

impl FIFOField<'_> {
  pub fn get_field(&self) -> subcode::FIFO {
    match self.expr.get_opcode() {
      Opcode::FIFOField { field } => field,
      _ => unreachable!(),
    }
  }
}

struct FIFODumper;

impl Visitor<String> for FIFODumper {
  fn visit_input(&mut self, fifo: FIFORef<'_>) -> Option<String> {
    format!(
      "{}.{}",
      fifo.get_parent().to_string(fifo.sys),
      fifo.get_name()
    )
    .into()
  }
}

impl Display for FIFOField<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}.{}",
      FIFODumper.visit_input(self.fifo()).unwrap(),
      self.get_field()
    )
  }
}

impl Display for FIFOPop<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{} = {}.pop()",
      self.expr.get_name(),
      FIFODumper.visit_input(self.fifo()).unwrap()
    )
  }
}

impl Display for FIFOPush<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}.push({}) // handle: _{}",
      FIFODumper.visit_input(self.fifo()).unwrap(),
      self.value().to_string(self.expr.sys),
      self.expr.get_key()
    )
  }
}
