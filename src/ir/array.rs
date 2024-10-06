use std::collections::HashSet;

use crate::builder::SysBuilder;

use super::{node::BaseNode, DataType, Typed};

use std::fmt::Display;
use std::ops::RangeInclusive;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemoryParams {
  pub width: usize,
  pub depth: usize,
  pub lat: RangeInclusive<usize>,
  pub init_file: Option<String>,
}

impl Default for MemoryParams {
  fn default() -> Self {
    Self {
      width: 0,
      depth: 0,
      lat: 1..=1,
      init_file: None,
    }
  }
}

impl MemoryParams {
  pub fn new(
    width: usize,
    depth: usize,
    lat: RangeInclusive<usize>,
    init_file: Option<String>,
  ) -> Self {
    Self {
      width,
      depth,
      lat,
      init_file,
    }
  }

  pub fn is_sram(&self) -> bool {
    return self.lat.start().eq(&1) && self.lat.end().eq(&1);
  }
}

impl Display for MemoryParams {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "width: {}, depth: {}, lat: [{:?}], file: {}",
      self.width,
      self.depth,
      self.lat,
      self.init_file.clone().map_or("None".to_string(), |x| x)
    )
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ArrayAttr {
  FullyPartitioned,
  MemoryParams(MemoryParams),
}

impl Display for ArrayAttr {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ArrayAttr::FullyPartitioned => write!(f, "FullyPartitioned"),
      ArrayAttr::MemoryParams(params) => write!(f, "MemoryParams({})", params),
    }
  }
}

pub struct Array {
  pub(crate) key: usize,
  name: String,
  scalar_ty: DataType,
  size: usize,
  init: Option<Vec<BaseNode>>,
  attrs: Vec<ArrayAttr>,
  pub(crate) user_set: HashSet<BaseNode>,
}

impl Typed for Array {
  fn dtype(&self) -> DataType {
    DataType::array(self.scalar_ty.clone(), self.size)
  }
}

impl Array {
  pub fn new(
    scalar_ty: DataType,
    name: String,
    size: usize,
    init: Option<Vec<BaseNode>>,
    attrs: Vec<ArrayAttr>,
  ) -> Array {
    Self {
      key: 0,
      scalar_ty,
      name,
      size,
      init,
      attrs,
      user_set: HashSet::new(),
    }
  }

  pub fn get_attrs(&self) -> &Vec<ArrayAttr> {
    &self.attrs
  }

  pub fn get_size(&self) -> usize {
    self.size
  }

  pub fn get_idx_type(&self) -> DataType {
    let bits = self.size.ilog2().max(1);
    let bits = if 1 << bits < self.size {
      bits + 1
    } else {
      bits
    };
    DataType::int_ty(bits as usize)
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

  pub fn user(&self) -> &HashSet<BaseNode> {
    &self.user_set
  }
}

impl SysBuilder {
  /// Create a register array associated to this system.
  /// An array can be a register, or memory.
  ///
  /// # Arguments
  /// * `ty` - The data type of data in the array.
  /// * `name` - The name of the array.
  /// * `size` - The size of the array.
  /// * `init` - A vector of initial values of this array.
  // TODO(@were): Add array types, memory, register, or signal wire.
  pub fn create_array(
    &mut self,
    ty: DataType,
    name: &str,
    size: usize,
    init: Option<Vec<BaseNode>>,
    attrs: Vec<ArrayAttr>,
  ) -> BaseNode {
    if let Some(init) = &init {
      assert_eq!(init.len(), size);
      init.iter().for_each(|x| {
        assert_eq!(x.get_dtype(self).unwrap(), ty);
      });
    }
    let instance = Array::new(ty.clone(), name.to_string(), size, init, attrs);
    let array_node = self.insert_element(instance);
    let new_name = self.symbol_table.insert(name, array_node);
    array_node.as_mut::<Array>(self).unwrap().get_mut().name = new_name;
    array_node
  }
}
