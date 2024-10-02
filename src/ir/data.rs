use std::fmt::Display;

use super::array::Array;
use crate::{builder::SysBuilder, ir::node::*};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DataType {
  Void,
  Int(usize),
  UInt(usize),
  Bits(usize),
  Fp32,
  Str,
  Module(String, Vec<Box<DataType>>),
  ArrayType(Box<DataType>, usize),
}

pub trait Typed {
  fn dtype(&self) -> DataType;
}

impl DataType {
  pub fn array(scalar_ty: DataType, size: usize) -> Self {
    assert!(scalar_ty.is_scalar());
    DataType::ArrayType(Box::new(scalar_ty), size)
  }

  pub fn void() -> Self {
    DataType::Void
  }

  pub fn module(kind: String, inputs: Vec<DataType>) -> Self {
    DataType::Module(kind, inputs.into_iter().map(Box::new).collect())
  }

  pub fn int_ty(bits: usize) -> Self {
    DataType::Int(bits)
  }

  pub fn bits_ty(bits: usize) -> Self {
    DataType::Bits(bits)
  }

  pub fn uint_ty(bits: usize) -> Self {
    DataType::UInt(bits)
  }

  pub fn fp32_ty() -> Self {
    DataType::Fp32
  }

  pub fn raw_ty(bits: usize) -> Self {
    DataType::Bits(bits)
  }

  pub fn is_scalar(&self) -> bool {
    matches!(self, DataType::Bits(_) | DataType::Int(_) | DataType::UInt(_) | DataType::Fp32)
  }

  pub fn get_bits(&self) -> usize {
    match self {
      DataType::Void => 0,
      DataType::Int(bits) => *bits,
      DataType::UInt(bits) => *bits,
      DataType::Fp32 => 32,
      DataType::Str => 0,
      DataType::Module(_, _) => 0,
      DataType::Bits(bits) => *bits,
      DataType::ArrayType(ty, size) => ty.get_bits() * size,
    }
  }

  pub fn is_signed(&self) -> bool {
    matches!(self, DataType::Int(_) | DataType::Fp32)
  }

  pub fn is_fp(&self) -> bool {
    matches!(self, DataType::Fp32)
  }

  pub fn is_int(&self) -> bool {
    matches!(self, DataType::Int(_) | DataType::UInt(_))
  }

  pub fn is_raw(&self) -> bool {
    matches!(self, DataType::Bits(_))
  }

  pub fn is_void(&self) -> bool {
    matches!(self, DataType::Void)
  }

  pub fn is_module(&self) -> bool {
    matches!(self, DataType::Module(_, _))
  }
}

impl Display for DataType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DataType::Int(_) => write!(f, "i{}", self.get_bits()),
      DataType::UInt(_) => write!(f, "u{}", self.get_bits()),
      DataType::Bits(bits) => write!(f, "b{}", bits),
      DataType::Fp32 => write!(f, "f{}", self.get_bits()),
      DataType::Str => write!(f, "Str"),
      DataType::Void => write!(f, "void"),
      DataType::ArrayType(ty, size) => write!(f, "array[{} x {}]", ty, size),
      DataType::Module(prefix, args) => {
        write!(
          f,
          "{}[{}]",
          prefix,
          args
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ")
        )
      }
    }
  }
}

pub struct IntImm {
  pub(crate) key: usize,
  dtype: DataType,
  value: u64,
}

pub struct StrImm {
  pub(crate) key: usize,
  dtype: DataType,
  value: String,
}

impl StrImm {
  pub fn new(value: String) -> Self {
    Self {
      key: 0,
      dtype: DataType::Str,
      value,
    }
  }

  pub fn get_value(&self) -> &str {
    self.value.as_str()
  }
}

impl Typed for StrImm {
  fn dtype(&self) -> DataType {
    self.dtype.clone()
  }
}

impl Typed for IntImm {
  fn dtype(&self) -> DataType {
    self.dtype.clone()
  }
}

impl IntImm {
  pub(crate) fn new(dtype: DataType, value: u64) -> Self {
    Self {
      key: 0,
      dtype,
      value,
    }
  }

  pub fn get_value(&self) -> u64 {
    self.value
  }
}

impl SysBuilder {
  /// Remove the given array from the system.
  pub fn remove_array(&mut self, array: BaseNode) {
    let key = array.as_ref::<Array>(self).unwrap().get_name().to_string();
    self.symbol_table.remove(&key);
    self.dispose(array);
  }
}
