use std::collections::{HashMap, HashSet};

use crate::builder::system::PortInfo;
use crate::builder::SysBuilder;
use crate::ir::node::*;
use crate::ir::*;

use self::expr::OperandOf;

/// The data structure for a module.
pub struct Module {
  /// The index key of this module in the slab buffer.
  pub(crate) key: usize,
  /// The name of this module, can be overridden by `set_name`.
  name: String,
  /// The input ports of this module.
  inputs: Vec<BaseNode>,
  /// The body of the module.
  body: BaseNode,
  /// The set of external interfaces used by the module.
  pub(crate) external_interfaces: HashMap<BaseNode, HashSet<OperandOf>>,
  /// The metadata of this module. The pointer to the module builder.
  builder_func_ptr: Option<usize>,
  /// The metadata of this module. The nodes that are parameterized by the module builder.
  parameterizable: Option<Vec<BaseNode>>,
  /// The redundant data of this module. The set of users that use this module.
  pub(crate) user_set: HashSet<OperandOf>,
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
      builder_func_ptr: None,
      parameterizable: None,
      user_set: HashSet::new(),
    }
  }

  /// Get the finger print of the module.
  pub fn get_builder_func_ptr(&self) -> Option<usize> {
    self.builder_func_ptr
  }

  /// Get the nodes that are parameterized by the module builder.
  pub fn get_parameterizable(&self) -> Option<&Vec<BaseNode>> {
    self.parameterizable.as_ref()
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
  pub fn get_input(&self, i: usize) -> Option<BaseNode> {
    self.inputs.get(i).map(|x| x.clone())
  }

  /// Get the input by name.
  ///
  /// # Arguments
  ///
  /// * `name` - The name of the input.
  pub fn get_input_by_name(&self, name: &str) -> Option<FIFORef<'_>> {
    self
      .inputs
      .iter()
      .find(|x| x.as_ref::<FIFO>(self.sys).unwrap().get_name().eq(name))
      .map(|x| x.clone().as_ref::<FIFO>(self.sys).unwrap())
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
  ) -> impl Iterator<Item = (&BaseNode, &HashSet<OperandOf>)>
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

  pub(crate) fn gather_related_externals(
    &self,
    user: BaseNode,
    idx: Option<usize>,
  ) -> Vec<(BaseNode, OperandOf)> {
    // Remove all the external interfaces related to this instruction.
    let tmp = self
      .get()
      .external_interfaces
      .iter()
      .map(|(ext, users)| {
        (
          ext.clone(),
          users
            .iter()
            .filter(|x| x.user == user && idx.map_or(true, |i| x.idx == i))
            .cloned()
            .collect::<Vec<_>>(),
        )
      })
      .filter(|(_, users)| !users.is_empty())
      .collect::<Vec<_>>();
    tmp
      .iter()
      .map(|(ext, users)| users.iter().map(|x| (ext.clone(), x.clone())))
      .flatten()
      .collect()
  }
}

impl<'a> ModuleMut<'a> {
  /// Maintain the redundant information, array used in the module.
  pub(crate) fn insert_external_interface(&mut self, ext_node: BaseNode, user: OperandOf) {
    if !self.get().external_interfaces.contains_key(&ext_node) {
      self
        .get_mut()
        .external_interfaces
        .insert(ext_node.clone(), HashSet::new());
    }
    let users = self
      .get_mut()
      .external_interfaces
      .get_mut(&ext_node)
      .unwrap();
    users.insert(user);
  }

  /// Remove a specific external interface.
  pub(crate) fn remove_external_interface(&mut self, ext_node: BaseNode, user: OperandOf) {
    if let Some(operations) = self.get_mut().external_interfaces.get_mut(&ext_node) {
      operations.remove(&user);
      if operations.is_empty() {
        self.get_mut().external_interfaces.remove(&ext_node);
      }
    }
  }

  /// Remove all the related external interfaces with the given condition.
  pub(crate) fn remove_related_externals(&mut self, user: BaseNode, idx: Option<usize>) {
    let to_remove = self.get().gather_related_externals(user, idx);
    to_remove.into_iter().for_each(|(ext, user)| {
      self.remove_external_interface(ext, user);
    });
  }

  /// Add related external interfaces to the module.
  pub(crate) fn add_related_externals(&mut self, operand: BaseNode, operand_of: OperandOf) {
    // Reconnect the external interfaces if applicable.
    // TODO(@were): Maybe later unify a common interface for this.
    match operand.get_kind() {
      NodeKind::ArrayPtr => {
        let aptr = operand.as_ref::<ArrayPtr>(self.sys).unwrap();
        let array = aptr.get_array().clone();
        self.insert_external_interface(array, operand_of);
      }
      NodeKind::FIFO => {
        self.insert_external_interface(operand, operand_of);
      }
      _ => {}
    }
  }

  /// Set the name of a module. Override the name given by the module builder.
  pub fn set_name(&mut self, name: String) {
    self.get_mut().name = name.to_string();
  }

  /// Set the metadata, the function pointer to the module builder. As part of the fingerprint of
  /// comparing the equality of the modules.
  pub fn set_builder_func_ptr(&mut self, key: usize) {
    self.get_mut().builder_func_ptr = key.into();
  }

  /// Set the metadata, these base nodes are parameterized --- plugged in by the module builder.
  pub fn set_parameterizable(&mut self, param: Vec<BaseNode>) {
    self.get_mut().parameterizable = Some(param);
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
      let mut fifo_mut = self.get_mut::<FIFO>(&input).unwrap();
      fifo_mut.get_mut().set_parent(module.clone());
      fifo_mut.get_mut().set_idx(i);
    }
    self.sym_tab.insert(module_name, module.clone());
    let body = Block::new(None, module.clone());
    let body = self.insert_element(body);
    self.get_mut::<Module>(&module).unwrap().get_mut().body = body.clone();
    module
  }
}
