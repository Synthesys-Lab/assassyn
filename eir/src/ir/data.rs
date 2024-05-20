use crate::ir::node::*;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DataType {
  Void,
  Int(usize),
  UInt(usize),
  Bits(usize),
  Fp32,
  Str,
  Module(Vec<Box<DataType>>),
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

  pub fn module(inputs: Vec<DataType>) -> Self {
    DataType::Module(inputs.into_iter().map(|x| Box::new(x)).collect())
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
    match self {
      DataType::Bits(_) | DataType::Int(_) | DataType::UInt(_) | DataType::Fp32 => true,
      _ => false,
    }
  }

  pub fn get_bits(&self) -> usize {
    match self {
      DataType::Void => 0,
      DataType::Int(bits) => *bits,
      DataType::UInt(bits) => *bits,
      DataType::Fp32 => 32,
      DataType::Str => 0,
      DataType::Module(_) => 0,
      DataType::Bits(bits) => *bits,
      DataType::ArrayType(ty, size) => ty.get_bits() * size,
    }
  }

  pub fn is_signed(&self) -> bool {
    match self {
      DataType::Int(_) | DataType::Fp32 => true,
      _ => false,
    }
  }

  pub fn is_fp(&self) -> bool {
    match self {
      DataType::Fp32 => true,
      _ => false,
    }
  }

  pub fn is_int(&self) -> bool {
    match self {
      DataType::Int(_) | DataType::UInt(_) => true,
      _ => false,
    }
  }

  pub fn is_raw(&self) -> bool {
    match self {
      DataType::Bits(_) => true,
      _ => false,
    }
  }

  pub fn is_void(&self) -> bool {
    match self {
      DataType::Void => true,
      _ => false,
    }
  }

  pub fn is_module(&self) -> bool {
    match self {
      DataType::Module(_) => true,
      _ => false,
    }
  }
}

impl ToString for DataType {
  fn to_string(&self) -> String {
    match &self {
      &DataType::Int(_) => format!("i{}", self.get_bits()),
      &DataType::UInt(_) => format!("u{}", self.get_bits()),
      &DataType::Bits(bits) => format!("b{}", bits),
      &DataType::Fp32 => format!("f{}", self.get_bits()),
      &DataType::Str => "Str".to_string(),
      &DataType::Void => String::from("()"),
      &DataType::ArrayType(ty, size) => format!("array[{} x {}]", ty.to_string(), size),
      &DataType::Module(args) => format!(
        "module[{}]",
        args
          .iter()
          .map(|x| x.to_string())
          .collect::<Vec<String>>()
          .join(", ")
      ),
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

pub struct Array {
  pub(crate) key: usize,
  name: String,
  scalar_ty: DataType,
  size: usize,
  init: Option<Vec<BaseNode>>,
}

impl Typed for Array {
  fn dtype(&self) -> DataType {
    DataType::array(self.scalar_ty.clone(), self.size)
  }
}

impl Array {
  pub fn new(scalar_ty: DataType, name: String, size: usize, init: Option<Vec<BaseNode>>) -> Array {
    Self {
      key: 0,
      scalar_ty,
      name,
      size,
      init,
    }
  }

  pub fn get_size(&self) -> usize {
    self.size
  }

  pub fn get_name(&self) -> &str {
    self.name.as_str()
  }

  pub fn scalar_ty(&self) -> DataType {
    self.scalar_ty.clone()
  }

  pub fn get_initializer(&self) -> Option<&Vec<BaseNode>> {
    self.init.as_ref()
  }
}

impl ArrayMut<'_> {
  pub fn set_name(&mut self, name: String) {
    let new_name = self.sys.symbol_table.identifier(name.as_str());
    self.get_mut().name = new_name;
  }
}
