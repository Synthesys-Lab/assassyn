use std::collections::HashSet;

use crate::builder::SysBuilder;

use super::{node::BaseNode, DataType, Typed};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ArrayAttr {
  FullyPartitioned,
}

impl ArrayAttr {
  pub fn to_string(&self, _: &SysBuilder) -> String {
    match self {
      ArrayAttr::FullyPartitioned => "FullyPartitioned".into(),
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

  pub fn get_flattened_size(&self) -> usize {
    self.get_size() * self.scalar_ty().get_bits()
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
