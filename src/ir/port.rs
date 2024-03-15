use crate::frontend::{BaseNode, DataType, NodeKind, Typed, Parented};

pub struct FIFO {
  pub(crate) key: usize,
  pub(super) parent: BaseNode,
  name: String,
  dtype: DataType,
}

impl FIFO {
  pub fn new(dtype: &DataType, name: &str) -> Self {
    Self {
      key: 0,
      parent: BaseNode::new(NodeKind::Unknown, 0), // Make a placeholder when instantiating.
      name: name.to_string(),
      dtype: dtype.clone(),
    }
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
