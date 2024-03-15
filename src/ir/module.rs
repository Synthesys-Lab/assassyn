use std::collections::{HashMap, HashSet};

use crate::frontend::*;

/// The data structure for a module.
pub struct Module {
  pub(crate) key: usize,
  name: String,
  inputs: Vec<BaseNode>,
  body: BaseNode,
  /// The set of external interfaces used by the module.
  pub(crate) external_interfaces: HashMap<BaseNode, HashSet<Opcode>>,
}

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
  /// let a = FIFO::new("a", 32);
  /// Module::new("a_plus_b", vec![a.clone()]);
  /// ```
  pub fn new(name: &str, inputs: Vec<BaseNode>) -> Module {
    Module {
      key: 0,
      name: name.to_string(),
      inputs,
      body: BaseNode::new(NodeKind::Unknown, 0),
      external_interfaces: HashMap::new(),
    }
  }
}

impl<'sys> ModuleRef<'sys> {
  /// Get the number of inputs to the module.
  pub fn get_num_inputs(&self) -> usize {
    self.inputs.len()
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
  pub fn get_name<'res, 'elem: 'res>(&'elem self) -> &'res str {
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

  pub(crate) fn ext_interf_iter<'borrow, 'res>(
    &'borrow self,
  ) -> impl Iterator<Item = (&BaseNode, &HashSet<Opcode>)>
  where
    'sys: 'borrow,
    'sys: 'res,
  {
    self.external_interfaces.iter()
  }

  pub fn port_iter<'borrow, 'res>(&'borrow self) -> impl Iterator<Item = FIFORef<'res>> + 'res
  where
    'sys: 'borrow,
    'sys: 'res,
    'borrow: 'res,
  {
    self
      .inputs
      .iter()
      .map(|x| x.as_ref::<FIFO>(self.sys).unwrap())
  }
}

impl<'a> ModuleMut<'a> {
  /// Maintain the redundant information, array used in the module.
  pub(crate) fn insert_external_interface(&mut self, array: BaseNode, opcode: Opcode) {
    if !self.get().external_interfaces.contains_key(&array) {
      self
        .get_mut()
        .external_interfaces
        .insert(array.clone(), HashSet::new());
    }
    let operations = self.get_mut().external_interfaces.get_mut(&array).unwrap();
    operations.insert(opcode);
  }
}

impl Typed for ModuleRef<'_> {
  fn dtype(&self) -> DataType {
    let types = self
      .inputs
      .iter()
      .map(|x| x.as_ref::<FIFO>(self.sys).unwrap().scalar_ty())
      .collect::<Vec<_>>();
    DataType::module(types)
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
    let n_inputs = inputs.len();
    let ports = inputs
      .into_iter()
      .map(|x| self.insert_element(FIFO::new(&x.ty, x.name.as_str())))
      .collect::<Vec<_>>();
    let module_name = self.identifier(name);
    let module = Module::new(&module_name, ports);
    let module = self.insert_element(module);
    // Set the parents of the inputs after instantiating the parent module.
    for i in 0..n_inputs {
      let input = module
        .as_ref::<Module>(self)
        .unwrap()
        .get_input(i)
        .unwrap()
        .clone();
      self
        .get_mut::<FIFO>(&input)
        .unwrap()
        .get_mut()
        .set_parent(module.clone());
    }
    self.sym_tab.insert(module_name, module.clone());
    let body = Block::new(None, module.clone());
    let body = self.insert_element(body);
    self.get_mut::<Module>(&module).unwrap().get_mut().body = body.clone();
    module
  }
}
