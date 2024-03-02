use std::collections::{HashMap, HashSet};

use crate::{
  builder::system::{InsertPoint, PortInfo, SysBuilder},
  data::Array,
  expr::Opcode,
  node::{ArrayRef, BaseNode, BlockRef, InputRef, IsElement, ModuleMut, ModuleRef, Parented},
};

use super::{block::Block, port::Input};

/// The data structure for a module.
pub struct Module {
  pub(crate) key: usize,
  name: String,
  inputs: Vec<BaseNode>,
  body: BaseNode,
  /// The set of arrays used in the module.
  pub(crate) array_used: HashMap<BaseNode, HashSet<Opcode>>,
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
  pub fn new(name: &str, inputs: Vec<BaseNode>) -> Module {
    Module {
      key: 0,
      name: name.to_string(),
      inputs,
      body: BaseNode::Unknown,
      array_used: HashMap::new(),
    }
  }
}

impl<'sys> ModuleRef<'sys> {
  /// Get the number of inputs to the module.
  pub fn get_num_inputs(&self) -> usize {
    self.get().inputs.len()
  }

  /// Get the given input reference.
  ///
  /// # Arguments
  ///
  /// * `i` - The index of the input.
  pub fn get_input(&self, i: usize) -> Option<&BaseNode> {
    self.inputs.get(i)
  }

  /// Get the name of the module.
  pub fn get_name(&self) -> &str {
    self.name.as_str()
  }

  /// Get the number of expressions in the module.
  pub fn get_num_exprs(&self) -> usize {
    self.get_body().get_num_exprs()
  }

  /// Get the number of expressions in the module.
  pub fn get_body<'elem>(&self) -> BlockRef<'elem>
  where
    'sys: 'elem,
  {
    self.body.as_ref::<Block>(self.sys).unwrap()
  }

  pub(crate) fn array_iter<'borrow, 'res>(
    &'borrow self,
  ) -> impl Iterator<Item = (ArrayRef<'res>, &HashSet<Opcode>)>
  where
    'sys: 'borrow,
    'sys: 'res,
  {
    self
      .array_used
      .iter()
      .map(|(k, v)| (k.as_ref::<Array>(self.sys).unwrap(), v))
  }

  pub fn port_iter<'borrow, 'res>(&'borrow self) -> impl Iterator<Item = InputRef<'res>> + 'res
  where
    'sys: 'borrow,
    'sys: 'res,
    'borrow: 'res,
  {
    self
      .inputs
      .iter()
      .map(|x| x.as_ref::<Input>(self.sys).unwrap())
  }
}

impl<'a> ModuleMut<'a> {
  /// Maintain the redundant information, array used in the module.
  pub fn insert_array_used(&mut self, array: BaseNode, opcode: Opcode) {
    if !self.get().array_used.contains_key(&array) {
      self
        .get_mut()
        .array_used
        .insert(array.clone(), HashSet::new());
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
  pub fn create_module(&mut self, name: &str, inputs: Vec<PortInfo>) -> BaseNode {
    let ports = inputs
      .into_iter()
      .map(|x| self.insert_element(Input::new(&x.ty, x.name.as_str())))
      .collect::<Vec<_>>();
    let module_name = self.identifier(name);
    let module = Module::new(&module_name, ports);
    // Set the parents of the inputs after instantiating the parent module.
    for i in 0..module.inputs.len() {
      let input = &module.inputs[i];
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
