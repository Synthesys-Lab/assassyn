use crate::{context::cur_ctx_mut, Reference};

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

pub struct Array {
  pub(crate) key: usize,
  scalar_ty: DataType,
  size: usize,
}

impl Typed for Array {
  fn dtype(&self) -> &DataType {
    &self.scalar_ty
  }
}

impl Array {

  pub fn new(scalar_ty: DataType, size: usize) -> &Box<Array> {
    let res = Self {
      key: 0,
      scalar_ty,
      size,
    };
    cur_ctx_mut().insert(res).as_ref::<Self>().unwrap()
  }

  pub fn size(&self) -> usize {
    self.size
  }

  pub fn read(&self) {
  }

}

