use std::{collections::HashMap, fmt::Display};

use crate::{
  builder::ir_printer,
  data::Array,
  expr::{Expr, Opcode},
  port::Input,
  reference::{Element, IsElement, Visitor},
  DataType, IntImm, Module, Reference,
};

// The top function.
pub struct SysBuilder {
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

macro_rules! create_binary_op_impl {
  ($func_name:ident, $opcode: expr) => {
    pub fn $func_name(
      &mut self,
      ty: DataType,
      a: Reference,
      b: Reference,
      pred: Option<Reference>,
    ) -> Reference {
      self.create_expr(ty, $opcode, vec![a, b], pred)
    }
  };
}

impl SysBuilder {
  pub fn new(name: &str) -> Self {
    let mut res = Self {
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
    // Set the parents of the inputs after instantiating the parent module.
    for i in 0..module.get_num_inputs() {
      let input = module.get_input(i).unwrap();
      self.get_mut::<Input>(input).unwrap().parent = module.upcast();
    }
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
    cond: Option<Reference>,
  ) -> Reference {
    let cur_mod = if let Some(cur_mod) = &self.cur_mod {
      cur_mod.clone()
    } else {
      panic!("No module to insert into!");
    };
    let instance = Expr::new(dtype.clone(), opcode, operands, cur_mod.clone());
    let key = {
      let origin = self.insert(instance);
      if let Some(cond) = cond {
        let predicated = Expr::new(
          DataType::void(),
          Opcode::Predicate,
          vec![cond, origin.clone()],
          cur_mod.clone(),
        );
        let res = self.insert(predicated);
        self.get_mut::<Expr>(&origin).unwrap().parent = (res.clone(), 1);
        res
      } else {
        origin
      }
    };
    self.get_mut::<Module>(&cur_mod).unwrap().push(key)
  }

  pub fn create_trigger(
    &mut self,
    dst: Reference,
    mut data: Vec<Reference>,
    cond: Option<Reference>,
  ) {
    data.insert(0, dst);
    self.create_expr(DataType::void(), Opcode::Trigger, data, cond);
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

  create_binary_op_impl!(create_add, Opcode::Add);
  create_binary_op_impl!(create_sub, Opcode::Sub);
  create_binary_op_impl!(create_mul, Opcode::Mul);
  create_binary_op_impl!(create_igt, Opcode::IGT);
  create_binary_op_impl!(create_ige, Opcode::IGE);
  create_binary_op_impl!(create_ilt, Opcode::ILT);
  create_binary_op_impl!(create_ile, Opcode::ILE);

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
    self
      .get_mut::<Module>(&cur_mod)
      .unwrap()
      .insert_array_used(array, Opcode::Load);
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
    self
      .get_mut::<Module>(&cur_mod)
      .unwrap()
      .insert_array_used(array, Opcode::Store);
    res
  }
}

impl Display for SysBuilder {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut printer = ir_printer::IRPrinter::new(self);

    write!(f, "system {} {{\n", self.name)?;
    for elem in self.arrays.iter() {
      let array = elem.as_ref::<Array>(self).unwrap();
      write!(f, "\n  {};\n", printer.visit_array(array))?;
    }
    printer.inc_indent();
    for elem in self.mods.iter() {
      let module = elem.as_ref::<Module>(self).unwrap();
      write!(f, "\n{}\n", printer.visit_module(module))?;
    }
    printer.dec_indent();
    write!(f, "}}")
  }
}
