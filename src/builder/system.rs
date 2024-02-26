use std::{collections::HashMap, fmt::Display};

use crate::{
  context::{Element, IsElement},
  data::Array,
  expr::{Expr, Opcode},
  port::Input,
  DataType, IntImm, Module, Reference,
};

// The top function.
pub struct SysBuilder {
  pub(crate) key: usize,
  pub(crate) slab: slab::Slab<Element>,
  const_cache: HashMap<(DataType, u64), Reference>,
  name: String,
  // TODO(@were): Data.
  arrays: Vec<Reference>,
  // TODO(@were): Add data.
  mods: Vec<Reference>,
  cur_mod: Option<Reference>,
}

pub struct PortInfo {
  pub name: String,
  pub ty: DataType,
}

impl PortInfo {
  pub fn new(name: &str, ty: DataType) -> Self {
    Self {
      name: name.into(),
      ty,
    }
  }
}

impl SysBuilder {
  pub fn new(name: &str) -> Self {
    let mut res = Self {
      key: 0,
      name: name.into(),
      arrays: vec![],
      slab: slab::Slab::new(),
      mods: Vec::new(),
      const_cache: HashMap::new(),
      cur_mod: None,
    };
    let driver = Module::new("driver", vec![]);
    let key = res.slab.insert(Element::Module(driver.into()));
    res.mods.push(Reference::Module(key));
    res
  }

  pub fn get<'a, T: IsElement<'a>>(&'a self, key: &Reference) -> Result<&'a Box<T>, String> {
    T::downcast(&self.slab, key)
  }

  pub fn module_iter<'a>(&'a self) -> impl Iterator<Item = &'a Box<Module>> {
    self.mods.iter().map(|x| x.as_ref::<Module>(self).unwrap())
  }

  pub fn array_iter<'a>(&'a self) -> impl Iterator<Item = &'a Box<Array>> {
    self.arrays.iter().map(|x| x.as_ref::<Array>(self).unwrap())
  }

  pub fn get_mut<'a, T: IsElement<'a>>(
    &'a mut self,
    key: &Reference,
  ) -> Result<&'a mut Box<T>, String> {
    T::downcast_mut(&mut self.slab, key)
  }

  pub fn get_driver(&self) -> &Module {
    self.get::<Module>(self.mods.first().unwrap()).unwrap()
  }

  pub fn set_current_module(&mut self, module: Reference) {
    self.cur_mod = Some(module);
  }

  pub fn insert<'a, T: IsElement<'a> + Into<Element> + 'a>(&'a mut self, elem: T) -> Reference {
    let key = self.slab.insert(elem.into());
    let res = T::into_reference(key);
    self.get_mut::<T>(&res).unwrap().set_key(key);
    res
  }

  pub fn get_const_int(&mut self, dtype: DataType, value: u64) -> Reference {
    let key = (dtype, value);
    if let Some(cached) = self.const_cache.get(&key) {
      return cached.clone();
    }
    let cloned_key = key.clone();
    let instance = IntImm::new(key.0, key.1);
    let key = self.insert(instance);
    self.const_cache.insert(cloned_key, key.clone());
    key
  }

  /// Create a new module, and set it as the current module to be built.
  pub fn create_module(&mut self, name: &str, inputs: Vec<PortInfo>) -> Reference {
    let ports = inputs
      .into_iter()
      .map(|x| self.insert(Input::new(&x.ty, x.name.as_str())))
      .collect::<Vec<_>>();
    let module = Module::new(name, ports);
    let key = self.insert(module);
    self.mods.push(key.clone());
    self.cur_mod = Some(key.clone());
    key
  }

  pub fn create_expr(
    &mut self,
    dtype: DataType,
    opcode: Opcode,
    operands: Vec<Reference>,
    pred: Option<Reference>,
  ) -> Reference {
    let cur_mod = if let Some(cur_mod) = &self.cur_mod {
      cur_mod.clone()
    } else {
      panic!("No module to insert into!");
    };
    let instance = Expr::new(dtype.clone(), opcode, operands, cur_mod.clone(), pred);
    let key = self.insert(instance);
    self.get_mut::<Module>(&cur_mod).unwrap().push(key)
  }

  pub fn create_trigger(&mut self, dst: Reference, mut data: Vec<Reference>) {
    data.insert(0, dst);
    self.create_expr(DataType::void(), Opcode::Trigger, data, None);
  }

  pub fn create_spin_trigger(
    &mut self,
    src: &Box<Module>,
    dst: &Box<Module>,
    mut data: Vec<Reference>,
  ) {
    data.insert(0, src.upcast());
    data.insert(1, dst.upcast());
    self.create_expr(DataType::void(), Opcode::SpinTrigger, data, None);
  }

  /// Create an addition operation in the current module.
  pub fn create_add<'a, 'b, 'c>(
    &mut self,
    ty: DataType,
    a: Reference,
    b: Reference,
    pred: Option<Reference>,
  ) -> Reference {
    self.create_expr(ty, Opcode::Add, vec![a, b], pred)
  }

  /// Create a register array associated to this system.
  /// An array can be a register, or memory.
  pub fn create_array(&mut self, ty: DataType, name: &str, size: usize) -> Reference {
    let instance = Array::new(ty, name.into(), size);
    let key = self.insert(instance);
    self.arrays.push(key.clone());
    key
  }

  /// Create a read operation on an array.
  pub fn create_array_read<'elem>(
    &mut self,
    dtype: DataType,
    array: Reference,
    index: Reference,
    cond: Option<Reference>,
  ) -> Reference {
    let operands = vec![array.clone(), index];
    let res = self.create_expr(dtype, Opcode::Load, operands, cond);
    let cur_mod = self.cur_mod.as_ref().unwrap().clone();
    self.get_mut::<Module>(&cur_mod).unwrap().insert_array_used(array, Opcode::Load);
    res
  }

  /// Create a write operation on an array.
  pub fn create_array_write(
    &mut self,
    array: Reference,
    index: Reference,
    value: Reference,
    cond: Option<Reference>,
  ) -> Reference {
    let operands = vec![array.clone(), index, value];
    let res = self.create_expr(DataType::void(), Opcode::Store, operands, cond);
    let cur_mod = self.cur_mod.as_ref().unwrap().clone();
    self.get_mut::<Module>(&cur_mod).unwrap().insert_array_used(array, Opcode::Store);
    res
  }
}

impl Display for SysBuilder {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "system {} {{\n", self.name)?;
    for elem in self.arrays.iter() {
      let array = elem.as_ref::<Array>(self).unwrap();
      write!(f, "\n  {};\n", array)?;
    }
    for elem in self.mods.iter() {
      let module = elem.as_ref::<Module>(self).unwrap();
      write!(f, "\n{}\n", module.to_string(self, 2))?;
    }
    write!(f, "}}")
  }
}
