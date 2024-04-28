use crate::builder::{InsertPoint, SysBuilder};
use crate::ir::node::*;
use crate::ir::*;

pub enum BlockKind {
  Condition(BaseNode),
  Cycle(usize),
  WaitUntil(BaseNode),
  Valued(BaseNode),
  None,
}

pub struct Block {
  pub(crate) key: usize,
  kind: BlockKind,
  body: Vec<BaseNode>,
  parent: BaseNode,
}

impl Block {
  pub(crate) fn new(pred: BlockKind, parent: BaseNode) -> Block {
    Block {
      key: 0,
      kind: pred,
      body: Vec::new(),
      parent,
    }
  }

  pub fn get_kind(&self) -> &BlockKind {
    &self.kind
  }

  pub fn get_num_exprs(&self) -> usize {
    self.body.len()
  }

  pub fn get(&self, idx: usize) -> Option<&BaseNode> {
    self.body.get(idx)
  }

  pub fn get_value(&self) -> Option<&BaseNode> {
    match &self.kind {
      BlockKind::Valued(x) => Some(x),
      _ => None,
    }
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

  /// Get the next node in the IR tree.
  pub fn next(&self) -> Option<BaseNode> {
    let parent = self.get().get_parent();
    if let Ok(block) = self.sys.get::<Block>(&parent) {
      let idx = block.body.iter().position(|x| *x == self.upcast());
      block.body.get(idx.unwrap() + 1).map(|x| x.clone())
    } else {
      None
    }
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
    let idx = at.unwrap_or_else(|| self.elem.as_ref::<Block>(self.sys).unwrap().get_num_exprs());
    self.get_mut().body.insert(idx, expr.clone());
    (expr, at.map(|x| x + 1))
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

  pub(crate) fn erase(&mut self, expr: &BaseNode) {
    let idx = self
      .elem
      .as_ref::<Block>(self.sys)
      .unwrap()
      .iter()
      .position(|x| *x == *expr)
      .expect("Element not found");
    self.get_mut().body.remove(idx);
  }
}

impl SysBuilder {
  /// Create a block.
  pub fn create_block(&mut self, pred: BlockKind) -> BaseNode {
    let pred = match pred {
      BlockKind::WaitUntil(arr_ptr) => {
        let arr_read = self.create_array_read(arr_ptr);
        BlockKind::WaitUntil(arr_read)
      }
      x => x,
    };
    let parent = self.get_current_block().unwrap().upcast();
    let instance = Block::new(pred, parent);
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
