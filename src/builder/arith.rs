use super::ctx::Reference;

enum Opcode {
  Add,
  Mul,
}

pub struct Expr {
  opcode: Opcode,
  operands: Vec<Reference>
}

