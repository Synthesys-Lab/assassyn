use std::fmt::Display;

use crate::{
  context::{cur_ctx_mut, IsElement},
  data::{Array, Typed},
  expr::{Expr, Opcode},
  port::Input,
  DataType,
  Module,
  Reference
};

// The top function.
pub struct SysBuilder {
  pub(crate) key: usize,
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

  pub fn new(name: &str) -> Reference {
    let mods = vec![Module::new("driver", vec![]).as_super()];
    let instance = Self {
      key: 0,
      name: name.into(),
      arrays: vec![],
      mods,
      cur_mod: None,
    };
    cur_ctx_mut().insert(instance)
  }

  pub fn get_driver(&self) -> &Box<Module> {
    self.mods[0].as_ref::<Module>().unwrap()
  }

  /// Create a new module, and set it as the current module to be built.
  pub fn create_module<'a, 'b>(&'a mut self, name: &str, inputs: Vec<PortInfo>) -> &'b Box<Module> {
    let inputs = inputs.into_iter().map(|x| Input::new(&x.ty, x.name.as_str())).collect();
    let module = Module::new(name, inputs);
    let reference = module.as_super();
    self.mods.push(reference.clone());
    self.cur_mod = Some(reference.clone());
    reference.as_mut::<Module>().unwrap().parent = Some(self.as_super());
    reference.as_ref::<Module>().unwrap()
  }

  pub fn create_expr<'a, 'b>(
    &'a self,
    dtype: &DataType,
    opcode: Opcode,
    operands: Vec<Reference>,
    pred: Option<Reference>) -> &'b Box<Expr> {
    if let Some(cur_mod) = &self.cur_mod {
      let instance = Expr::new(dtype.clone(), opcode, operands, cur_mod.clone(), pred);
      let res = cur_ctx_mut().insert(instance);
      cur_mod.as_mut::<Module>().unwrap().push(res.clone());
      res.as_mut::<Expr>().unwrap().parent = cur_mod.clone();
      res.as_ref::<Expr>().unwrap()
    } else {
      panic!("No module to insert into!");
    }
  }

  pub fn create_trigger(&self, src: &Box<Module>, dst: &Box<Module>, mut data: Vec<Reference>) {
    data.insert(0, src.as_super());
    data.insert(1, dst.as_super());
    self.create_expr(&DataType::void(), Opcode::Trigger, data, None);
  }

  pub fn create_spin_trigger(&self, src: &Box<Module>, dst: &Box<Module>, mut data: Vec<Reference>) {
    data.insert(0, src.as_super());
    data.insert(1, dst.as_super());
    self.create_expr(&DataType::void(), Opcode::SpinTrigger, data, None);
  }

  /// Create an addition operation in the current module.
  pub fn create_add<'a, 'b, 'c>(&self,
    a: &Box<impl Typed + IsElement<'a>>,
    b: &Box<impl Typed + IsElement<'b>>,
    pred: Option<Reference>) -> &'b Box<Expr> {
    self.create_expr(a.dtype(), Opcode::Add, vec![a.as_super(), b.as_super()], pred)
  }

  /// Create a register array associated to this system.
  /// An array can be a register, or memory.
  pub fn create_array<'a>(&mut self, ty: DataType, name: &str, size: usize) -> &'a Box<Array> {
    let instance = Array::new(ty, name.into(), size);
    self.arrays.push(instance.as_super());
    instance
  }

  /// Create a read operation on an array.
  pub fn create_array_read(
    &mut self,
    array: &Box<Array>,
    index: &Box<Expr>,
    cond: Option<Reference>) -> &Box<Expr> {
    self.create_expr(&array.dtype(), Opcode::Load, vec![array.as_super(), index.as_super()], cond)
  }

  /// Create a write operation on an array.
  pub fn create_array_write(
    &mut self,
    array: &Box<Array>,
    index: &Box<Expr>,
    value: &Box<Expr>,
    cond: Option<Reference>) -> &Box<Expr> {
    self.create_expr(&DataType::void(), Opcode::Store, vec![array.as_super(), index.as_super(), value.as_super()], cond)
  }

}

impl Display for SysBuilder {

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "system {} {{\n", self.name)?;
    for elem in self.mods.iter() {
      write!(f, "\n{}\n", elem.as_ref::<Module>().unwrap())?;
    }
    write!(f, "}}")
  }

}

