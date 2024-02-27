use std::collections::{HashMap, HashSet};

use crate::{
  reference::Reference,
  data::Array,
  expr::{Expr, Opcode},
};

use super::{port::Input, system::SysBuilder};

/// The data structure for a module.
pub struct Module {
  pub(crate) key: usize,
  name: String,
  inputs: Vec<Reference>,
  body: Vec<Reference>,
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
      body: Vec::new(),
      array_used: HashMap::new(),
    }
  }

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

  pub fn get_name(&self) -> &str {
    self.name.as_str()
  }

  pub(crate) fn push(&mut self, expr: Reference) -> Reference {
    self.body.push(expr);
    self.body.last().unwrap().clone()
  }

  pub(super) fn insert_array_used(&mut self, array: Reference, opcode: Opcode) {
    if !self.array_used.contains_key(&array) {
      self.array_used.insert(array.clone(), HashSet::new());
    }
    let operations = self.array_used.get_mut(&array).unwrap();
    operations.insert(opcode);
  }

  pub(crate) fn array_iter<'a>(
    &'a self,
    sys: &'a SysBuilder,
  ) -> impl Iterator<Item = (&'a Box<Array>, &'a HashSet<Opcode>)> {
    self.array_used.iter().map(|(k, v)| (k.as_ref::<Array>(sys).unwrap(), v))
  }

  pub fn port_iter<'a>(&'a self, sys: &'a SysBuilder) -> impl Iterator<Item = &'a Box<Input>> {
    self.inputs.iter().map(|x| x.as_ref::<Input>(sys).unwrap())
  }

  pub fn expr_iter<'a>(&'a self, sys: &'a SysBuilder) -> impl Iterator<Item = &'a Box<Expr>> {
    self.body.iter().map(|x| x.as_ref::<Expr>(sys).unwrap())
  }

}
