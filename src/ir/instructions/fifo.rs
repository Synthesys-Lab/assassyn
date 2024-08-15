use std::fmt::Display;

use crate::ir::{
  node::{FIFORef, IsElement, Parented},
  visitor::Visitor,
};

use super::{FIFOPop, FIFOPush};

struct FIFODumper;

impl Visitor<String> for FIFODumper {
  fn visit_fifo(&mut self, fifo: FIFORef<'_>) -> Option<String> {
    format!(
      "{}.{}",
      fifo.get_parent().to_string(fifo.sys),
      fifo.get_name()
    )
    .into()
  }
}

impl Display for FIFOPop<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{} = {}.pop()",
      self.expr.get_name(),
      FIFODumper.visit_fifo(self.fifo()).unwrap()
    )
  }
}

impl Display for FIFOPush<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}.push({}) // handle: _{}",
      FIFODumper.visit_fifo(self.fifo()).unwrap(),
      self.value().to_string(self.expr.sys),
      self.expr.get_key()
    )
  }
}
