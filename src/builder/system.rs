use std::fmt::Display;

use crate::{
  context::{cur_ctx_mut, IsElement},
  data::Array,
  expr::{Expr, Opcode},
  DataType,
  Module,
  Reference
};

// The top function.
pub struct SysBuilder {
  pub(crate) key: usize,
  name: String,
  // TODO(@were): Data.
  arrays: Vec<Array>,
  // TODO(@were): Add data.
  mods: Vec<Module>,
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

  pub fn new(name: &str) -> Reference {
    let mods = vec![Module::new("driver", vec![])];
    let instance = Self {
      key: 0,
      name: name.into(),
      arrays: vec![],
      mods,
      cur_mod: None,
    };
    cur_ctx_mut().insert(instance)
  }

  pub fn get_driver(&self) -> &Module {
    self.mods.first().unwrap()
  }

  pub fn set_current_module(&mut self, module: Reference) {
    self.cur_mod = Some(module);
  }

  /// Create a new module, and set it as the current module to be built.
  pub fn create_module(&mut self, name: &str, inputs: Vec<PortInfo>) -> &Module {
    let module = Module::new(name, inputs);
    self.mods.push(module);
    self.cur_mod = Some(self.mods.last().unwrap().as_super());
    self.mods.last().unwrap()
  }

  pub fn create_expr(
    &self,
    dtype: DataType,
    opcode: Opcode,
    operands: Vec<Reference>,
    pred: Option<Reference>) -> Reference {
    if let Some(cur_mod) = &self.cur_mod {
      let instance = Expr::new(dtype.clone(), opcode, operands, cur_mod.clone(), pred);
      cur_mod.as_mut::<Module>().unwrap().push(instance)
    } else {
      panic!("No module to insert into!");
    }
  }

  pub fn create_trigger(&self, src: &Box<Module>, dst: &Box<Module>, mut data: Vec<Reference>) {
    data.insert(0, src.as_super());
    data.insert(1, dst.as_super());
    self.create_expr(DataType::void(), Opcode::Trigger, data, None);
  }

  pub fn create_spin_trigger(
    &self, src: &Box<Module>, dst: &Box<Module>, mut data: Vec<Reference>) {
    data.insert(0, src.as_super());
    data.insert(1, dst.as_super());
    self.create_expr(DataType::void(), Opcode::SpinTrigger, data, None);
  }

  /// Create an addition operation in the current module.
  pub fn create_add<'a, 'b, 'c>(&mut self,
    ty: DataType,
    a: Reference,
    b: Reference,
    pred: Option<Reference>) -> Reference {
    self.create_expr(ty, Opcode::Add, vec![a, b], pred)
  }

  /// Create a register array associated to this system.
  /// An array can be a register, or memory.
  pub fn create_array(&mut self, ty: DataType, name: &str, size: usize) -> Reference {
    let instance = Array::new(ty, name.into(), size);
    self.arrays.push(instance);
    Reference::Array(self.arrays.len() - 1)
  }

  /// Create a read operation on an array.
  pub fn create_array_read<'elem>(
    &mut self,
    dtype: DataType,
    array: Reference,
    index: Reference,
    cond: Option<Reference>) -> Reference {
    let operands = vec![array, index];
    self.create_expr(dtype, Opcode::Load, operands, cond)
  }

  /// Create a write operation on an array.
  pub fn create_array_write(
    &mut self,
    array: Reference,
    index: Reference,
    value: Reference,
    cond: Option<Reference>) -> Reference {
    let operands = vec![array, index, value];
    self.create_expr(DataType::void(), Opcode::Store, operands, cond)
  }

}

impl Display for SysBuilder {

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "system {} {{\n", self.name)?;
    for elem in self.arrays.iter() {
      write!(f, "\n{}\n", elem)?;
    }
    for elem in self.mods.iter() {
      write!(f, "\n{}\n", elem)?;
    }
    write!(f, "}}")
  }

}

