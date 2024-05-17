use crate::ir::{expr::subcode, DataType, Opcode, Typed};

use super::{Binary, Cast, Compare, Unary};

impl Binary<'_> {
  pub fn get_opcode(&self) -> subcode::Binary {
    match self.expr.get_opcode() {
      Opcode::Binary { binop } => binop,
      _ => panic!(
        "Expecting Opcode::Binary, but got {:?}",
        self.expr.get_opcode()
      ),
    }
  }
}

impl Unary<'_> {
  pub fn get_opcode(&self) -> subcode::Unary {
    match self.expr.get_opcode() {
      Opcode::Unary { uop } => uop,
      _ => panic!(
        "Expecting Opcode::Unary, but got {:?}",
        self.expr.get_opcode()
      ),
    }
  }
}

impl Compare<'_> {
  pub fn get_opcode(&self) -> subcode::Compare {
    match self.expr.get_opcode() {
      Opcode::Compare { cmp } => cmp,
      _ => panic!(
        "Expecting Opcode::Compare, but got {:?}",
        self.expr.get_opcode()
      ),
    }
  }
}

impl Cast<'_> {
  pub fn get_opcode(&self) -> subcode::Cast {
    match self.expr.get_opcode() {
      Opcode::Cast { cast } => cast,
      _ => panic!(
        "Expecting Opcode::Case, but got {:?}",
        self.expr.get_opcode()
      ),
    }
  }

  pub fn dest_type(&self) -> DataType {
    self.expr.dtype()
  }

  pub fn src_type(&self) -> DataType {
    self.x().get_dtype(self.get().sys).unwrap()
  }
}
