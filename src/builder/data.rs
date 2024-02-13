use super::context::{ cur_ctx_mut, Parented, Reference };

#[derive(Clone, PartialEq, Eq)]
enum DataKind {
  Int,
  UInt,
  Float,
}

#[derive(Clone, PartialEq, Eq)]
pub struct DataType {
  kind: DataKind,
  bits: usize,
}

pub trait Typed {
  fn dtype(&self) -> &DataType;
}

impl DataType {

  fn new(kind: DataKind, bits: usize) -> Self {
    Self {
      kind,
      bits,
    }
  }

  pub fn int(bits: usize) -> Self {
    Self::new(DataKind::Int, bits)
  }

  pub fn uint(bits: usize) -> Self {
    Self::new(DataKind::UInt, bits)
  }

  pub fn fp(bits: usize) -> Self {
    Self::new(DataKind::Float, bits)
  }

  pub fn bits(&self) -> usize {
    self.bits
  }

}

impl ToString for DataType {
  
  fn to_string(&self) -> String {
    match self.kind {
      DataKind::Int => format!("i{}", self.bits),
      DataKind::UInt => format!("u{}", self.bits),
      DataKind::Float => format!("f{}", self.bits),
    }
  }

}

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

