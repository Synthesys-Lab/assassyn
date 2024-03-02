#[derive(Clone, PartialEq, Eq, Hash)]
pub enum DataType {
  Void,
  Int(usize),
  UInt(usize),
  Fp32,
}

pub trait Typed {
  fn dtype(&self) -> &DataType;
}

impl DataType {
  pub fn void() -> Self {
    DataType::Void
  }

  pub fn int(bits: usize) -> Self {
    DataType::Int(bits)
  }

  pub fn uint(bits: usize) -> Self {
    DataType::UInt(bits)
  }

  pub fn fp32() -> Self {
    DataType::Fp32
  }

  pub fn bits(&self) -> usize {
    match self {
      DataType::Void => 0,
      DataType::Int(bits) => *bits,
      DataType::UInt(bits) => *bits,
      DataType::Fp32 => 32,
    }
  }

  pub fn is_void(&self) -> bool {
    match self {
      DataType::Void => true,
      _ => false,
    }
  }
}

impl ToString for DataType {
  fn to_string(&self) -> String {
    match self {
      &DataType::Int(_) => format!("i{}", self.bits()),
      &DataType::UInt(_) => format!("u{}", self.bits()),
      &DataType::Fp32 => format!("f{}", self.bits()),
      &DataType::Void => String::from("()"),
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

  pub fn get_size(&self) -> usize {
    self.size
  }

  pub fn get_name(&self) -> &str {
    self.name.as_str()
  }
}
