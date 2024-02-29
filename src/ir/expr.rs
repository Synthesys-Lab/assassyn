use crate::{
  builder::{mutator::Mutable, system::SysBuilder},
  data::{DataType, Typed},
  reference::{IsElement, Parented},
  register_mutator,
};

use super::{block::Block, reference::Reference};

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
    }
  }
}

pub struct Expr {
  pub(super) key: usize,
  parent: Reference,
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
      parent,
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

  pub fn get_num_operands(&self) -> usize {
    self.operands.len()
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
  fn get_parent(&self) -> Reference {
    self.parent.clone()
  }

  fn set_parent(&mut self, parent: Reference) {
    self.parent = parent;
  }
}

register_mutator!(ExprMut, Expr);

impl ExprMut<'_> {

  pub fn move_to_new_parent(&mut self, new_parent: Reference, at: Option<usize>) {
    let old_parent = self.get().get_parent();
    let expr = self.get().upcast();
    let mut block_mut = self.sys.get_mut::<Block>(&old_parent).unwrap();
    block_mut.erase(&expr);
    let mut new_parent_mut = self.sys.get_mut::<Block>(&new_parent).unwrap();
    new_parent_mut.insert_at(at, expr);
    self.get_mut().set_parent(new_parent)
  }

}
