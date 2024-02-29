use crate::{reference::Parented, data::Typed, DataType, Reference};

pub struct Input {
  pub(crate) key: usize,
  pub(super) parent: Reference,
  name: String,
  dtype: DataType,
}

impl Input {
  pub fn new(dtype: &DataType, name: &str) -> Self {
    Self {
      key: 0,
      parent: Reference::Unknown, // Make a placeholder when instantiating.
      name: name.to_string(),
      dtype: dtype.clone(),
    }
  }

  pub fn get_name(&self) -> &String {
    &self.name
  }
}

impl Typed for Input {
  fn dtype(&self) -> &DataType {
    &self.dtype
  }
}

impl Parented for Input {
  fn get_parent(&self) -> Reference {
    self.parent.clone()
  }
  fn set_parent(&mut self, parent: Reference) {
    self.parent = parent;
  }
}
