use crate::{
  builder::{mutator::Mutable, system::SysBuilder},
  expr::Expr,
  reference::{IsElement, Parented},
  register_mutator, Reference,
};

pub struct Block {
  pub(crate) key: usize,
  pred: Option<Reference>,
  body: Vec<Reference>,
  parent: Reference,
}

impl Block {
  pub(crate) fn new(pred: Option<Reference>, parent: Reference) -> Block {
    Block {
      key: 0,
      pred,
      body: Vec::new(),
      parent,
    }
  }

  pub fn get_num_exprs(&self) -> usize {
    self.body.len()
  }

  pub fn get(&self, idx: usize) -> Option<&Reference> {
    self.body.get(idx)
  }

  pub fn iter<'a>(&'a self, sys: &'a SysBuilder) -> impl Iterator<Item = &'a Box<Expr>> {
    self.body.iter().map(|x| x.as_ref::<Expr>(sys).unwrap())
  }
}

impl Parented for Block {
  fn get_parent(&self) -> Reference {
    self.parent.clone()
  }

  fn set_parent(&mut self, parent: Reference) {
    self.parent = parent;
  }
}

register_mutator!(BlockMut, Block);

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
    expr: Reference,
  ) -> (Reference, Option<usize>) {
    let idx = at.unwrap_or_else(|| self.get().get_num_exprs());
    self.get_mut().body.insert(idx, expr);
    (self.get().get(idx).unwrap().clone(), at.map(|x| x + 1))
  }
}

impl SysBuilder {
  /// Create a block.
  pub fn create_block(&mut self, cond: Option<Reference>) -> Reference {
    let parent = self.get_current_module().unwrap().upcast();
    let instance = Block::new(cond, parent.clone());
    let block = self.insert_element(instance);
    let ip = &self.get_insert_point();
    self.get_mut::<Block>(&ip.1).unwrap().insert_at(ip.2, block);
    block
  }
}
