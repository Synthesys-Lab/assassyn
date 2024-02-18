use crate::{context::{cur_ctx_mut, IsElement}, data::Typed, Arithmetic, Module, Reference};

// The top function.
pub struct SysBuilder {
  pub(crate) key: usize,
  name: String,
  // TODO(@were): Add data.
  mods: Vec<Reference>,
  cur_mod: Option<Reference>,
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
  pub fn create_module(&mut self, name: &str, inputs: Vec<Reference>) -> &Box<Module> {
    let module = Module::new(name, inputs);
    self.mods.push(module.as_super());
    self.cur_mod = Some(module.as_super());
    (module.as_super()).as_ref::<Module>().unwrap()
  }

  pub fn create_add<'a, 'b, 'c, T: Typed + IsElement<'a>, U: Typed + IsElement<'b>>(
    &self, a: &Box<impl Arithmetic<'a, 'b, 'c, T, U>>,
    b: &Box<T>,
    pred: Option<&Box<U>>) {
    if let Some(cur_mod) = &self.cur_mod {
      let module = cur_mod.as_mut::<Module>().unwrap();
      let parent = module.as_super();
      let add = a.add(b, pred, parent);
      module.push(add.as_super());
    }
    ()
  }

}

