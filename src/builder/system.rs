use crate::{context::{cur_ctx_mut, IsElement}, data::Typed, expr::{Expr, Opcode}, port::Input, Arithmetic, DataType, Module, Reference};

// The top function.
pub struct SysBuilder {
  pub(crate) key: usize,
  name: String,
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
    self.mods.push(module.as_super());
    self.cur_mod = Some(module.as_super());
    (module.as_super()).as_ref::<Module>().unwrap()
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
      res.as_ref::<Expr>().unwrap()
    } else {
      panic!("No module to insert into!");
    }
  }

  pub fn create_add<'a, 'b, 'c>(&self,
    a: &Box<impl Typed + IsElement<'a>>,
    b: &Box<impl Typed + IsElement<'b>>,
    pred: Option<Reference>) -> &'b Box<Expr> {
    self.create_expr(a.dtype(), Opcode::Add, vec![a.as_super(), b.as_super()], pred)
  }

}

