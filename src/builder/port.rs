use crate::{context::{cur_ctx_mut, Parented}, data::Typed, DataType, Reference};


pub struct Input {
  pub(crate) key: usize,
  pub(super) parent: Option<Reference>,
  name: String,
  dtype: DataType,
}

impl Input {

  pub fn new(dtype: &DataType, name: &str) -> Reference {
    let res = Self {
      key: 0,
      parent: None,
      name: name.to_string(),
      dtype: dtype.clone()
    };
    cur_ctx_mut().insert(res)
  }

  pub fn name(&self) -> &String {
    &self.name
  }

}

impl Typed for Input {

  fn dtype(&self) -> &DataType {
    &self.dtype
  }

}

impl Parented for Input {

  fn parent(&self) -> Option<Reference> {
    self.parent.clone()
  }

}

pub struct Output {
  pub(crate) key: usize,
  pub(crate) data: Reference,
}

impl Output {

  pub fn new(data: Reference) -> Reference {
    let res = Self { key: 0, data };
    cur_ctx_mut().insert(res)
  }

}

