use std::fmt::Display;

use crate::{
  context::{cur_ctx_mut, IsElement, Parented},
  expr::{Expr, Opcode},
  Reference
};

#[derive(Clone, PartialEq, Eq, Hash)]
enum DataKind {
  Void,
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

  pub fn void() -> Self {
    Self {
      kind: DataKind::Void,
      bits: 0,
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
      DataKind::Void => String::from("()"),
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
  name: String,
  scalar_ty: DataType,
  size: usize,
}

impl Display for Array {

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Array({}, {})", self.name, self.size)
  }

}

impl Typed for Array {
  fn dtype(&self) -> &DataType {
    &self.scalar_ty
  }
}

impl Array {

  pub fn new(scalar_ty: DataType, name: String, size: usize) -> Array {
    Self {
      key: 0,
      scalar_ty,
      name,
      size,
    }
  }

  pub fn size(&self) -> usize {
    self.size
  }

  pub fn read<'a, 'b>(&self, idx: &Box<impl IsElement<'b> + Typed>,
                      cond: Option<Reference>,
                      reader: Reference) -> &'a Box<Expr> {
    let instance = Expr::new(
      self.scalar_ty.clone(),
      Opcode::Load,
      vec![self.as_super(), idx.as_super()],
      reader,
      cond,
    );
    let res = cur_ctx_mut().insert(instance);
    res.as_ref::<Expr>().unwrap()
  }

  pub fn write<'a, 'b>(&self,
                       idx: &Box<impl IsElement<'a> + Typed>,
                       value: &Box<impl IsElement<'b> + Typed>,
                       cond: Option<Reference>,
                       writer: Reference) -> &'a Box<Expr> {

    let instance = Expr::new(
      DataType::void(),
      Opcode::Store,
      vec![value.as_super(), self.as_super(), idx.as_super()],
      writer,
      None,
    );
    let res = cur_ctx_mut().insert(instance);
    res.as_ref::<Expr>().unwrap()
  }

}

