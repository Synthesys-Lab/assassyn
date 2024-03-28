use crate::frontend::{DataType, ExprMut, ExprRef, IsElement, Parented, Typed};

use super::{block::Block, node::BaseNode};

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
  EQ,
  // Unary operations
  Neg,
  Flip,
  // Eventual operations
  FIFOPush,
  FIFOPop,
  FIFOPeek,
  Trigger,
  // Level-2 syntax sugar, will be re-written in synthesizable operations
  SpinTrigger,
  CallbackTrigger,
  // Non-synthesizable operations
  Log,
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
  pub fn is_unary(&self) -> bool {
    match self {
      Opcode::Neg | Opcode::Flip => true,
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
      Opcode::EQ => "==".into(),
      Opcode::Neg => "-".into(),
      Opcode::Flip => "!".into(),
      Opcode::Load => "load".into(),
      Opcode::Store => "store".into(),
      Opcode::Trigger => "trigger".into(),
      Opcode::SpinTrigger => "wait_until".into(),
      Opcode::CallbackTrigger => "callback".into(),
      Opcode::FIFOPush => "push".into(),
      Opcode::FIFOPop => "pop".into(),
      Opcode::FIFOPeek => "peek".into(),
      Opcode::Log => "log".into(),
    }
  }
}

pub struct Expr {
  pub(super) key: usize,
  parent: BaseNode,
  dtype: DataType,
  opcode: Opcode,
  operands: Vec<BaseNode>,
}

impl Expr {
  pub(crate) fn new(
    dtype: DataType,
    opcode: Opcode,
    operands: Vec<BaseNode>,
    parent: BaseNode,
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

  pub fn get_operand(&self, i: usize) -> Option<&BaseNode> {
    self.operands.get(i)
  }

  pub fn get_num_operands(&self) -> usize {
    self.operands.len()
  }

  pub fn operand_iter(&self) -> impl Iterator<Item = &BaseNode> {
    self.operands.iter()
  }
}

impl Typed for Expr {
  fn dtype(&self) -> DataType {
    self.dtype.clone()
  }
}

impl Parented for Expr {
  fn get_parent(&self) -> BaseNode {
    self.parent.clone()
  }

  fn set_parent(&mut self, parent: BaseNode) {
    self.parent = parent;
  }
}

impl ExprRef<'_> {
  // Get the next expression in the block
  pub fn next(&self) -> Option<BaseNode> {
    let parent = self.get().get_parent();
    let block = self.sys.get::<Block>(&parent).unwrap();
    let pos = block.iter().position(|x| *x == self.upcast());
    block.get().get(pos.unwrap()).map(|x| x.clone())
  }
}

impl ExprMut<'_> {
  pub fn move_to_new_parent(&mut self, new_parent: BaseNode, at: Option<usize>) {
    let old_parent = self.get().get_parent();
    let expr = self.get().upcast();
    let mut block_mut = self.sys.get_mut::<Block>(&old_parent).unwrap();
    block_mut.erase(&expr);
    let mut new_parent_mut = self.sys.get_mut::<Block>(&new_parent).unwrap();
    new_parent_mut.insert_at(at, expr);
    self.get_mut().set_parent(new_parent)
  }

  /// Erase the expression from its parent block
  pub fn erase_from_parent(&mut self) {
    let parent = self.get().get_parent();
    let expr = self.get().upcast();
    let mut block_mut = self.sys.get_mut::<Block>(&parent).unwrap();
    block_mut.erase(&expr);
  }
}
