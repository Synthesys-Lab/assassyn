use crate::ir::{expr::subcode, node::BaseNode, Opcode};

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

  pub fn a(&self) -> BaseNode {
    self.expr.get_operand(0).unwrap().get_value().clone()
  }

  pub fn b(&self) -> BaseNode {
    self.expr.get_operand(1).unwrap().get_value().clone()
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

  pub fn x(&self) -> BaseNode {
    self.expr.get_operand(0).unwrap().get_value().clone()
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

  pub fn a(&self) -> BaseNode {
    self.expr.get_operand(0).unwrap().get_value().clone()
  }

  pub fn b(&self) -> BaseNode {
    self.expr.get_operand(1).unwrap().get_value().clone()
  }
}
