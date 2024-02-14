use crate::{context::{cur_ctx, cur_ctx_mut, IsElement, Parented}, Reference};

#[derive(Clone, PartialEq, Eq, Hash)]
enum DataKind {
  Int,
  UInt,
  Float,
}

#[derive(Clone, PartialEq, Eq, Hash)]
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

pub struct IntImm {
  pub(crate) key: usize,
  dtype: DataType,
  value: u64,
}

impl Typed for IntImm {
  fn dtype(&self) -> &DataType {
    &self.dtype
  }
}

impl Parented for IntImm {

  fn parent(&self) -> Option<Reference> {
    None
  }

}

impl IntImm {

  pub(super) fn instantiate(dtype: DataType, value: u64) -> Self {
    Self { key: 0, dtype, value, }
  }

  pub fn new<'a>(dtype: &DataType, value: u64) -> &'a Box<IntImm> {
    let res = cur_ctx_mut().int_imm(dtype, value);
    res.as_ref::<IntImm>().unwrap()
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

pub struct ArrayRead {
  pub(crate) key: usize,
  parent: Option<Reference>,
  dtype: DataType,
  array: Reference,
  idx: Reference,
}

pub struct ArrayWrite {
  pub(crate) key: usize,
  parent: Option<Reference>,
  array: Reference,
  idx: Reference,
  value: Reference,
}

impl Parented for ArrayRead {

  fn parent(&self) -> Option<Reference> {
    self.parent.clone()
  }

}

impl Typed for ArrayRead {

  fn dtype(&self) -> &DataType {
    &self.dtype
  }

}

impl Array {

  pub fn new<'a>(scalar_ty: DataType, size: usize) -> &'a Box<Array> {
    let res = Self {
      key: 0,
      scalar_ty,
      size,
    };
    let key = cur_ctx_mut().insert(res);
    cur_ctx().get(&key).unwrap()
  }

  pub fn size(&self) -> usize {
    self.size
  }

  pub fn read<'a, 'b>(&self, idx: &Box<impl IsElement<'b> + Parented + Typed>,
                      reader: Reference) -> &'a Box<ArrayRead> {
    let instance = ArrayRead {
      key: 0,
      parent: Some(reader),
      dtype: self.scalar_ty.clone(),
      array: self.as_super(),
      idx: idx.as_super(),
    };
    let res = cur_ctx_mut().insert(instance);
    res.as_ref::<ArrayRead>().unwrap()
  }

  pub fn write<'a, 'b>(&self,
                       idx: &Box<impl IsElement<'a> + Parented + Typed>,
                       value: &Box<impl IsElement<'b> + Parented + Typed>,
                       cond: Option<Reference>) -> &'a Box<ArrayWrite> {

    let instance = ArrayWrite {
      key: 0,
      parent: None,
      array: self.as_super(),
      idx: idx.as_super(),
      value: value.as_super(),
    };
    let res = cur_ctx_mut().insert(instance);
    res.as_ref::<ArrayWrite>().unwrap()
  }

}

