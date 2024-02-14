use crate::{context::{cur_ctx_mut, IsElement}, Module, Reference};

// The top function.
pub struct System {
  pub(crate) key: usize,
  // TODO(@were): Add data.
  mods: Vec<Reference>,
}


impl System {

  pub fn new() -> Reference {
    let mods = vec![Module::new("driver", vec![]).as_super()];
    let instance = Self {
      key: 0,
      mods,
    };
    cur_ctx_mut().insert(instance)
  }

  pub fn get_driver(&self) -> &Box<Module> {
    self.mods[0].as_ref::<Module>().unwrap()
  }

  pub fn add_module(&mut self, module: Reference) {
    self.mods.push(module);
    self.mods.last().unwrap().as_mut::<Module>().unwrap().parent = Some(self.as_super());
  }

}

