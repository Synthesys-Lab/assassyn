use crate::ir::{expr::subcode, Opcode};

use super::{Binary, Compare, Unary};

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

