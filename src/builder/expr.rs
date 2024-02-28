use crate::{
  data::{DataType, Typed},
  reference::Parented,
};

use super::reference::Reference;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Opcode {
  // Side-effect operations
  Load,
  Store,
  // Binary operations
  Add,
  Sub,
  Mul,
  BitwiseAnd,
  BitwiseOr,
  BitwiseXor,
  // Comparison operations
  IGT,
  ILT,
  IGE,
  ILE,
  // Eventual operations
  Trigger,
  SpinTrigger,
  // Predicated operations
  Predicate,
}

impl Opcode {
  pub fn is_binary(&self) -> bool {
    match self {
      Opcode::Add
      | Opcode::Mul
      | Opcode::Sub
      | Opcode::IGT
      | Opcode::ILT
      | Opcode::IGE
      | Opcode::ILE
      | Opcode::BitwiseAnd
      | Opcode::BitwiseOr
      | Opcode::BitwiseXor => true,
      _ => false,
    }
  }
}

impl ToString for Opcode {
  fn to_string(&self) -> String {
    match self {
      Opcode::Add => "+".into(),
      Opcode::Sub => "-".into(),
      Opcode::Mul => "*".into(),
      Opcode::BitwiseAnd => "&".into(),
      Opcode::BitwiseOr => "|".into(),
      Opcode::BitwiseXor => "^".into(),
      Opcode::IGT => ">".into(),
      Opcode::ILT => "<".into(),
      Opcode::IGE => ">=".into(),
      Opcode::ILE => "<=".into(),
      Opcode::Load => "load".into(),
      Opcode::Store => "store".into(),
      Opcode::Trigger => "trigger".into(),
      Opcode::SpinTrigger => "wait_until".into(),
      Opcode::Predicate => "predicate".into(),
    }
  }
}

pub struct Expr {
  pub(super) key: usize,
  pub(super) parent: (Reference, usize),
  dtype: DataType,
  opcode: Opcode,
  operands: Vec<Reference>,
}

impl Expr {
  pub(crate) fn new(
    dtype: DataType,
    opcode: Opcode,
    operands: Vec<Reference>,
    parent: Reference,
  ) -> Self {
    Self {
      key: 0,
      parent: (parent, 0),
      dtype,
      opcode,
      operands,
    }
  }

  pub fn get_opcode(&self) -> Opcode {
    self.opcode.clone()
  }

  pub fn get_operand(&self, i: usize) -> Option<&Reference> {
    self.operands.get(i)
  }

  pub fn operand_iter(&self) -> impl Iterator<Item = &Reference> {
    self.operands.iter()
  }
}

impl Typed for Expr {
  fn dtype(&self) -> &DataType {
    &self.dtype
  }
}

impl Parented for Expr {
  fn parent(&self) -> Reference {
    self.parent.0.clone()
  }
}
