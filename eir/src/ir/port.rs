use crate::ir::{node::*, *};

pub struct FIFO {
  pub(crate) key: usize,
  pub(super) parent: BaseNode,
  name: String,
  dtype: DataType,
  idx: usize,
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
    }
  }

  /// When instantiating a port whose module is not deterministically known, we use a placeholder
  /// FIFO to represent the port. The parent is the module's expression, not the module reference,
  /// which will typically be a FIFOPop expression from the parent module.
  pub fn placeholder(dtype: DataType, parent: BaseNode, idx: usize) -> Self {
    Self {
      key: 0,
      parent,
      name: idx.to_string(),
      dtype,
      idx,
    }
  }

  pub fn is_placeholder(&self) -> bool {
    match self.parent.get_kind() {
      NodeKind::Module => false,
      _ => true,
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
