use crate::ir::node::IsElement;
use crate::ir::*;

use self::node::{ExprMut, ExprRef, NodeKind, Parented};

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
  // Other synthesizable operations
  Slice,
  // Level-2 syntax sugar, will be re-written in synthesizable operations
  SpinTrigger,
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
      | Opcode::EQ
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
      Opcode::FIFOPush => "push".into(),
      Opcode::FIFOPop => "pop".into(),
      Opcode::FIFOPeek => "peek".into(),
      Opcode::Log => "log".into(),
      Opcode::Slice => "slice".into(),
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

/// This struct indicates this a certain node is an operand of the user expr's idx-th operand.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct OperandOf {
  pub(crate) user: BaseNode,
  pub(crate) idx: usize,
}

impl OperandOf {
  pub(crate) fn new(user: BaseNode, idx: usize) -> Self {
    OperandOf { user, idx }
  }
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
    let block = self.sys.get::<Block>(&parent).unwrap();

    // Remove all the external interfaces related to this instruction.
    let module = block.get_module().upcast();
    let mut module_mut = self.sys.get_mut::<Module>(&module).unwrap();
    module_mut.remove_related_externals(expr, None);

    let mut block_mut = self.sys.get_mut::<Block>(&parent).unwrap();
    block_mut.erase(&expr);
  }

  pub fn set_operand(&mut self, i: usize, operand: BaseNode) {
    let block = self.sys.get::<Block>(&self.get().get_parent()).unwrap();
    let module = block.get_module();

    // Remove all the external interfaces related to this instruction.
    let module = module.upcast();
    let expr = self.get().upcast();
    let mut module_mut = self.sys.get_mut::<Module>(&module).unwrap();
    module_mut.remove_related_externals(expr, Some(i));

    // Reconnect the external interfaces if applicable.
    // TODO(@were): Maybe later unify a common interface for this.
    match operand.get_kind() {
      NodeKind::ArrayPtr => {
        let aptr = operand.as_ref::<ArrayPtr>(self.sys).unwrap();
        let array = aptr.get_array().clone();
        let mut module_mut = self.sys.get_mut::<Module>(&module).unwrap();
        module_mut.insert_external_interface(array, OperandOf::new(expr, i));
      }
      NodeKind::FIFO => {
        let mut module_mut = self.sys.get_mut::<Module>(&module).unwrap();
        module_mut.insert_external_interface(operand, OperandOf::new(expr, i));
      }
      _ => {}
    }

    self.get_mut().operands[i] = operand;
  }
}
