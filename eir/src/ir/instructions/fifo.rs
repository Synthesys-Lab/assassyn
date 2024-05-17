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
    if fifo.is_placeholder() {
      format!("{}.{}", fifo.get_parent().to_string(fifo.sys), fifo.idx())
    } else {
      format!(
        "{}.{}",
        fifo.get_parent().to_string(fifo.sys),
        fifo.get_name()
      )
    }
    .into()
  }
}

impl ToString for FIFOField<'_> {
  fn to_string(&self) -> String {
    format!(
      "{}.{}()",
      FIFODumper.visit_input(self.fifo()).unwrap(),
      self.get_field().to_string()
    )
  }
}

impl ToString for FIFOPop<'_> {
  fn to_string(&self) -> String {
    format!(
      "{} = {}.pop()",
      self.expr.get_name(),
      FIFODumper.visit_input(self.fifo()).unwrap()
    )
  }
}

impl ToString for FIFOPush<'_> {
  fn to_string(&self) -> String {
    format!(
      "{}.push({}) // handle: _{}",
      FIFODumper.visit_input(self.fifo()).unwrap(),
      self.value().to_string(self.expr.sys),
      self.expr.get_key()
    )
  }
}
