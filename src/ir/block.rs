use crate::{
  builder::system::{InsertPoint, SysBuilder},
  node::{BlockMut, BlockRef, IsElement, ModuleRef, NodeKind, Parented},
  BaseNode, Module,
};

pub struct Block {
  pub(crate) key: usize,
  pred: Option<BaseNode>,
  body: Vec<BaseNode>,
  parent: BaseNode,
}

impl Block {
  pub(crate) fn new(pred: Option<BaseNode>, parent: BaseNode) -> Block {
    Block {
      key: 0,
      pred,
      body: Vec::new(),
      parent,
    }
  }

  pub fn get_pred(&self) -> Option<BaseNode> {
    self.pred.clone()
  }

  pub fn get_num_exprs(&self) -> usize {
    self.body.len()
  }

  pub fn get(&self, idx: usize) -> Option<&BaseNode> {
    self.body.get(idx)
  }

  pub fn iter<'a>(&'a self) -> impl Iterator<Item = &BaseNode> + 'a {
    self.body.iter()
  }

}

impl Parented for Block {
  fn get_parent(&self) -> BaseNode {
    self.parent.clone()
  }

  fn set_parent(&mut self, parent: BaseNode) {
    self.parent = parent;
  }
}

impl BlockRef<'_> {

  pub fn get_module(&self) -> ModuleRef<'_> {
    let mut runner = self.upcast().clone();
    while runner.get_kind() != NodeKind::Module {
      let parent: BaseNode = match runner.get_kind() {
        NodeKind::Block => runner.as_ref::<Block>(self.sys).unwrap().get_parent(),
        _ => panic!("Invalid parent type"),
      };
      runner = parent;
    }
    runner.as_ref::<Module>(self.sys).unwrap()
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
  pub(crate) fn insert_at(
    &mut self,
    at: Option<usize>,
    expr: BaseNode,
  ) -> (BaseNode, Option<usize>) {
    let idx = at.unwrap_or_else(|| self.get().get_num_exprs());
    self.get_mut().body.insert(idx, expr);
    (self.get().get(idx).unwrap().clone(), at.map(|x| x + 1))
  }

  /// Insert an expression at the current insert point of the SysBuilder, and maintain the
  /// insert-at pointer forward.
  ///
  /// # Arguments
  /// * `expr` - The expression to insert.
  pub(crate) fn insert_at_ip(&mut self, expr: BaseNode) -> BaseNode {
    let InsertPoint(_, _, at) = self.sys.inesert_point;
    let (expr, new_at) = self.insert_at(at.clone(), expr.clone());
    self.sys.inesert_point.2 = new_at;
    expr
  }

  pub(crate) fn push(&mut self, expr: BaseNode) -> BaseNode {
    self.get_mut().body.push(expr.clone());
    expr
  }

  pub(crate) fn erase(&mut self, expr: &BaseNode) {
    let idx = self
      .get()
      .body
      .iter()
      .position(|x| *x == *expr)
      .expect("Element not found");
    self.get_mut().body.remove(idx);
  }
}

impl SysBuilder {
  /// Create a block.
  pub fn create_block(&mut self, cond: Option<BaseNode>) -> BaseNode {
    let parent = self.get_current_block().unwrap().upcast();
    let instance = Block::new(cond, parent);
    let block = self.insert_element(instance);
    let InsertPoint(_, insert_block, at) = &self.get_insert_point();
    let (block, new_at) = self
      .get_mut::<Block>(insert_block)
      .unwrap()
      .insert_at(at.clone(), block.clone());
    self.inesert_point.2 = new_at;
    block
  }
}
