use expr::subcode;

use crate::builder::SysBuilder;
use crate::ir::node::*;
use crate::ir::*;

pub struct Block {
  pub(crate) key: usize,
  body: Vec<BaseNode>,
  parent: BaseNode,
}

impl Block {
  pub(in super::super) fn new(parent: BaseNode) -> Block {
    Block {
      key: 0,
      body: Vec::new(),
      parent,
    }
  }

  pub fn get_num_exprs(&self) -> usize {
    self.body.len()
  }

  pub fn get(&self, idx: usize) -> Option<&BaseNode> {
    self.body.get(idx)
  }

  pub fn idx_of(&self, node: &BaseNode) -> Option<usize> {
    self.body.iter().position(|x| x.eq(node))
  }
}

impl Parented for Block {
  fn get_parent(&self) -> BaseNode {
    self.parent
  }

  fn set_parent(&mut self, parent: BaseNode) {
    self.parent = parent;
  }
}

impl BlockRef<'_> {
  pub fn get_module(&self) -> BaseNode {
    let mut runner = self.upcast();
    while runner.get_kind() != NodeKind::Module {
      let parent: BaseNode = match runner.get_kind() {
        NodeKind::Block => runner.as_ref::<Block>(self.sys).unwrap().get_parent(),
        _ => panic!("Invalid parent type"),
      };
      runner = parent;
    }
    runner
  }

  fn get_block_intrinsic(
    &self,
    idx: usize,
    subcode: subcode::BlockIntrinsic,
  ) -> Option<instructions::BlockIntrinsic<'_>> {
    if let Some(Ok(bi)) = self
      .body
      .get(idx)
      .map(|x| x.as_expr::<instructions::BlockIntrinsic>(self.sys))
    {
      if bi.get_subcode() == subcode {
        return Some(bi);
      }
    }
    None
  }

  pub fn get_value(&self) -> Option<BaseNode> {
    self
      .get_block_intrinsic(self.get_num_exprs() - 1, subcode::BlockIntrinsic::Value)
      .map(|x| x.value())
  }

  pub fn get_cycle(&self) -> Option<usize> {
    self
      .get_block_intrinsic(0, subcode::BlockIntrinsic::Cycled)
      .map(|x| x.value().as_ref::<IntImm>(self.sys).unwrap().get_value() as usize)
  }

  pub fn get_condition(&self) -> Option<BaseNode> {
    self
      .get_block_intrinsic(0, subcode::BlockIntrinsic::Condition)
      .map(|x| x.value())
  }

  pub fn get_wait_until(&self) -> Option<BaseNode> {
    self.body_iter().find(|x| {
      x.as_expr::<instructions::BlockIntrinsic>(self.sys)
        .map_or(false, |x| x.get_subcode() == subcode::BlockIntrinsic::WaitUntil)
    })
  }

  /// Get the next node in the IR tree.
  pub fn next(&self) -> Option<BaseNode> {
    let parent = self.get().get_parent();
    if let Ok(block) = self.sys.get::<Block>(&parent) {
      let idx = self.idx().unwrap();
      block.body.get(idx + 1).copied()
    } else {
      None
    }
  }

  /// Get the index of the current node in the parent block.
  pub fn idx(&self) -> Option<usize> {
    let parent = self.get().get_parent();
    if let Ok(block) = self.sys.get::<Block>(&parent) {
      block.body.iter().position(|x| *x == self.upcast())
    } else {
      None
    }
  }
}

impl BlockRef<'_> {
  pub fn body_iter(&self) -> impl Iterator<Item = BaseNode> + '_ {
    self.body.iter().cloned()
  }
}

impl BlockMut<'_> {
  /// Insert an expression at the given position of the module.
  /// If `at` is `None`, the expression is inserted at the end of the module.
  ///
  /// # Arguments
  /// * `at` - The position to insert the expression.
  /// * `expr` - The expression to insert.
  /// # Returns
  /// * The reference to the inserted expression.
  /// * The new position to insert the next expression.
  pub fn insert_at(&mut self, at: Option<usize>, expr: BaseNode) -> (BaseNode, Option<usize>) {
    let idx = at.unwrap_or_else(|| self.elem.as_ref::<Block>(self.sys).unwrap().get_num_exprs());
    self.get_mut().body.insert(idx, expr);
    (expr, at.map(|x| x + 1))
  }

  /// Insert an expression at the current insert point of the SysBuilder, and maintain the
  /// insert-at pointer forward.
  ///
  /// # Arguments
  /// * `expr` - The expression to insert.
  pub fn insert_at_ip(&mut self, expr: BaseNode) -> BaseNode {
    let at = self.sys.inesert_point.at;
    let (expr, new_at) = self.insert_at(at, expr);
    self.sys.inesert_point.at = new_at;
    expr
  }

  /// Erase the given instruction from the block.
  pub fn erase(&mut self, expr: &BaseNode) {
    let idx = self
      .elem
      .as_ref::<Block>(self.sys)
      .unwrap()
      .body_iter()
      .position(|x| expr.eq(&x))
      .expect("Element not found");
    self.get_mut().body.remove(idx);
  }
}

impl SysBuilder {
  fn create_block_intrinsic(
    &mut self,
    dtype: DataType,
    subcode: subcode::BlockIntrinsic,
    value: BaseNode,
  ) -> BaseNode {
    self.create_expr(dtype, subcode.into(), vec![value], true)
  }

  pub fn create_wait_until(&mut self, cond: BaseNode) -> BaseNode {
    self.create_block_intrinsic(DataType::void(), subcode::BlockIntrinsic::WaitUntil, cond)
  }

  pub fn create_cycled_block(&mut self, cycle: u32) -> BaseNode {
    let block = self.create_block();
    let ip = self.get_current_ip();
    self.set_current_block(block);
    let dtype = DataType::int_ty(32);
    let cycle = self.get_const_int(dtype, cycle as u64);
    let void_ty = DataType::void();
    self.create_block_intrinsic(void_ty, subcode::BlockIntrinsic::Cycled, cycle);
    self.set_insert_point(ip);
    block
  }

  pub fn create_conditional_block(&mut self, cond: BaseNode) -> BaseNode {
    let block = self.create_block();
    let ip = self.get_current_ip();
    self.set_current_block(block);
    let dtype = cond.get_dtype(self).unwrap();
    self.create_block_intrinsic(dtype, subcode::BlockIntrinsic::Condition, cond);
    self.set_insert_point(ip);
    block
  }

  /// Create a block in the IR.
  pub fn create_block(&mut self) -> BaseNode {
    let parent = self.get_current_block().unwrap().upcast();
    let instance = Block::new(parent);
    let block = self.insert_element(instance);
    let (insert_block, at) = {
      let ip_ref = &self.get_insert_point();
      (ip_ref.block, ip_ref.at)
    };
    let (block, new_at) = self
      .get_mut::<Block>(&insert_block)
      .unwrap()
      .insert_at(at, block);
    self.inesert_point.at = new_at;
    block
  }
}
