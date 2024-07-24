use std::collections::HashSet;

use crate::ir::{node::*, *};

pub struct FIFO {
  /// A unique key of this instance in the slab buffer.
  pub(crate) key: usize,
  /// The parent module of this FIFO.
  pub(super) parent: BaseNode,
  /// The name of this FIFO.
  name: String,
  /// The data type of this FIFO.
  dtype: DataType,
  /// The index of this FIFO in the parent module.
  idx: usize,
  /// The redundant data structure to store the users of this FIFO.
  pub(crate) user_set: HashSet<BaseNode>,
}

impl FIFO {
  pub fn new(dtype: &DataType, name: &str) -> Self {
    Self {
      key: 0,
      // When instantiating a port input FIFO, the parent module is not constructed yet.
      // To avoid running into a chicken-egg paradox, we set the parent to a dummy node for now.
      // Later SysBuilder will call set_parent() to set the correct parent.
      parent: BaseNode::new(NodeKind::Unknown, 0),
      name: name.to_string(),
      dtype: dtype.clone(),
      // Similar to the parent field.
      idx: usize::MAX,
      user_set: HashSet::new(),
    }
  }

  /// A redundant data structure to store the index of the port in the parent module.
  pub fn idx(&self) -> usize {
    self.idx
  }

  /// A redundant data structure to store the index of the port in the parent module.
  pub(crate) fn set_idx(&mut self, idx: usize) {
    self.idx = idx;
  }

  pub fn get_name(&self) -> &String {
    &self.name
  }

  pub fn scalar_ty(&self) -> DataType {
    self.dtype.clone()
  }
}

impl FIFOMut<'_> {
  pub fn set_name(&mut self, name: String) {
    self.get_mut().name = name;
  }
}

impl Typed for FIFO {
  fn dtype(&self) -> DataType {
    DataType::void()
  }
}

impl Parented for FIFO {
  fn get_parent(&self) -> BaseNode {
    self.parent.clone()
  }
  fn set_parent(&mut self, parent: BaseNode) {
    self.parent = parent;
  }
}
