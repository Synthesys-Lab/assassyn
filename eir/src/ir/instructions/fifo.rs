use crate::ir::{expr::subcode, Opcode};

use super::FIFOField;

impl FIFOField<'_> {
  pub fn get_field(&self) -> subcode::FIFO {
    match self.expr.get_opcode() {
      Opcode::FIFOField { field } => field,
      _ => unreachable!(),
    }
  }
}
