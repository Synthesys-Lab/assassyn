use std::collections::{HashMap, HashSet};

use crate::{
  builder::{mutator::Mutable, system::{InsertPoint, PortInfo, SysBuilder}},
  data::Array,
  expr::Opcode,
  reference::{IsElement, Parented, Reference},
  register_mutator,
};

use super::{block::Block, port::Input};

/// The data structure for a module.
pub struct Module {
  pub(crate) key: usize,
  name: String,
  inputs: Vec<Reference>,
  body: Reference,
  /// The set of arrays used in the module.
  array_used: HashMap<Reference, HashSet<Opcode>>,
}

pub struct Driver {}

impl Module {
  /// Returns a reference to the created new module.
  ///
  /// # Arguments
  ///
  /// * `name` - The name of the module.
  /// * `inputs` - The inputs to the module.
  ///
  /// # Example
  ///
  /// ```
  /// let a = Input::new("a", 32);
  /// Module::new("a_plus_b", vec![a.clone()]);
  /// ```
  pub fn new(name: &str, inputs: Vec<Reference>) -> Module {
    Module {
      key: 0,
      name: name.to_string(),
      inputs,
      body: Reference::Unknown,
      array_used: HashMap::new(),
    }
  }

  /// Get the number of inputs to the module.
  pub fn get_num_inputs(&self) -> usize {
    self.inputs.len()
  }

  /// Get the given input reference.
  ///
  /// # Arguments
  ///
  /// * `i` - The index of the input.
  pub fn get_input(&self, i: usize) -> Option<&Reference> {
    self.inputs.get(i)
  }

  /// Get the name of the module.
  pub fn get_name(&self) -> &str {
    self.name.as_str()
  }

  // TODO(@were): Overengineer a *Ref class for these methods accepts a sys reference.

  /// Get the number of expressions in the module.
  pub fn get_num_exprs(&self, sys: &SysBuilder) -> usize {
    self.get_body(sys).unwrap().get_num_exprs()
  }

  /// Get the number of expressions in the module.
  pub fn get_body<'a>(&'a self, sys: &'a SysBuilder) -> Result<&'a Box<Block>, String> {
    self.body.as_ref::<Block>(sys)
  }

  pub(crate) fn array_iter<'a>(
    &'a self,
    sys: &'a SysBuilder,
  ) -> impl Iterator<Item = (&'a Box<Array>, &'a HashSet<Opcode>)> {
    self
      .array_used
      .iter()
      .map(|(k, v)| (k.as_ref::<Array>(sys).unwrap(), v))
  }

  pub fn port_iter<'a>(&'a self, sys: &'a SysBuilder) -> impl Iterator<Item = &'a Box<Input>> {
    self.inputs.iter().map(|x| x.as_ref::<Input>(sys).unwrap())
  }

  pub fn iter<'a>(&'a self, sys: &'a SysBuilder) -> impl Iterator<Item = &Reference> {
    self.get_body(sys).unwrap().iter()
  }

}

register_mutator!(ModuleMut, Module);

impl <'a>ModuleMut<'a> {

  /// Maintain the redundant information, array used in the module.
  pub fn insert_array_used(&mut self, array: Reference, opcode: Opcode) {
    if !self.get().array_used.contains_key(&array) {
      self.get_mut().array_used.insert(array.clone(), HashSet::new());
    }
    let operations = self.get_mut().array_used.get_mut(&array).unwrap();
    operations.insert(opcode);
  }

}

impl SysBuilder {

  /// Create a new module, and set it as the current module to be built.
  ///
  /// # Arguments
  ///
  /// * `name` - The name of the module.
  /// * `inputs` - The inputs' information to the module. Refer to `PortInfo` for more details.
  pub fn create_module(&mut self, name: &str, inputs: Vec<PortInfo>) -> Reference {
    let ports = inputs
      .into_iter()
      .map(|x| self.insert_element(Input::new(&x.ty, x.name.as_str())))
      .collect::<Vec<_>>();
    let module_name = self.identifier(name);
    let module = Module::new(&module_name, ports);
    // Set the parents of the inputs after instantiating the parent module.
    for i in 0..module.get_num_inputs() {
      let input = module.get_input(i).unwrap();
      Input::downcast_mut(&mut self.slab, input)
        .unwrap()
        .set_parent(module.upcast());
    }
    let module = self.insert_element(module);
    self.sym_tab.insert(module_name, module.clone());
    let body = Block::new(None, module.clone());
    let body = self.insert_element(body);
    self.get_mut::<Module>(&module).unwrap().get_mut().body = body.clone();
    self.inesert_point = InsertPoint(module.clone(), body, None);
    module
  }

}

