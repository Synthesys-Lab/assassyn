use crate::frontend::{BaseNode, NodeKind};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DataType {
  Void,
  Int(usize),
  UInt(usize),
  Fp32,
  Module(Vec<Box<DataType>>),
}

pub trait Typed {
  fn dtype(&self) -> DataType;
}

impl DataType {
  pub fn void() -> Self {
    DataType::Void
  }

  pub fn module(inputs: Vec<DataType>) -> Self {
    DataType::Module(inputs.into_iter().map(|x| Box::new(x)).collect())
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
      DataType::Module(_) => 0,
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
      DataType::Int(_) => true,
      _ => false,
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
    match &self {
      &DataType::Int(_) => format!("i{}", self.bits()),
      &DataType::UInt(_) => format!("u{}", self.bits()),
      &DataType::Fp32 => format!("f{}", self.bits()),
      &DataType::Void => String::from("()"),
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

/// Handle is like a "pointer" to an array element.
/// It is similar to LLVM's `GetElementPtrInst`, but 1-d.
pub struct ArrayPtr {
  pub(crate) key: usize,
  /// The array to be accessed.
  array: BaseNode,
  /// The index of the array.
  idx: BaseNode,
}

impl ArrayPtr {
  pub fn new(array: BaseNode, idx: BaseNode) -> Self {
    Self { key: 0, array, idx }
  }

  pub fn get_array(&self) -> &BaseNode {
    &self.array
  }

  pub fn get_idx(&self) -> &BaseNode {
    &self.idx
  }

  pub fn is_const(&self) -> bool {
    self.get_idx().get_kind() == NodeKind::IntImm
  }
}

pub struct Array {
  pub(crate) key: usize,
  name: String,
  scalar_ty: DataType,
  size: usize,
}

impl Typed for Array {
  fn dtype(&self) -> DataType {
    DataType::void()
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

  pub fn scalar_ty(&self) -> DataType {
    self.scalar_ty.clone()
  }
}
