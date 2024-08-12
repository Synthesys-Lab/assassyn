pub mod attrs;
pub mod memory;
pub mod meta;

use std::collections::{HashMap, HashSet};

use crate::builder::symbol_table::SymbolTable;
use crate::builder::system::PortInfo;
use crate::builder::SysBuilder;
use crate::ir::node::*;
use crate::ir::*;

pub use attrs::Attribute;

/// The data structure for a module.
pub struct Module {
  /// The index key of this module in the slab buffer.
  pub(crate) key: usize,
  /// The name of this module.
  name: String,
  /// The input ports of this module.
  ports: HashMap<String, BaseNode>,
  /// The body of the module.
  pub(crate) body: BaseNode,
  /// The set of external interfaces used by the module.
  pub(crate) external_interfaces: HashMap<BaseNode, HashSet<BaseNode>>,
  /// The metadata of this module. The pointer to the module builder.
  builder_func_ptr: Option<usize>,
  /// The metadata of this module. The nodes that are parameterized by the module builder.
  parameterizable: Option<Vec<BaseNode>>,
  /// The redundant data of this module. The set of users that use this module.
  pub(crate) user_set: HashSet<BaseNode>,
  /// The attributes of this module.
  pub(crate) attr: HashSet<Attribute>,
  /// The symbol table that maintains the unique identifiers.
  pub(crate) symbol_table: SymbolTable,
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
  pub fn new(name: &str, ports: HashMap<String, BaseNode>) -> Module {
    Module {
      key: 0,
      name: name.to_string(),
      ports,
      body: BaseNode::new(NodeKind::Unknown, 0),
      external_interfaces: HashMap::new(),
      builder_func_ptr: None,
      parameterizable: None,
      user_set: HashSet::new(),
      attr: HashSet::new(),
      symbol_table: SymbolTable::new(),
    }
  }

  pub fn get_attrs(&self) -> &HashSet<Attribute> {
    &self.attr
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
    self.ports.len()
  }

  /// Get the input by name.
  ///
  /// # Arguments
  ///
  /// * `name` - The name of the input.
  pub fn get_port(&self, name: &str) -> Option<FIFORef<'_>> {
    self
      .ports
      .get(name)
      .map(|x| x.clone().as_ref::<FIFO>(self.sys).unwrap())
  }

  /// Get the name of the module.
  pub fn get_name<'res, 'elem: 'res>(&'elem self) -> &'res str {
    self.name.as_str()
  }

  /// Get the number of expressions in body of the module.
  pub fn get_num_exprs(&self) -> usize {
    self.get_body().get_num_exprs()
  }

  /// Get the body of this module.
  pub fn get_body<'elem>(&self) -> BlockRef<'elem>
  where
    'sys: 'elem,
  {
    self.body.as_ref::<Block>(self.sys).unwrap()
  }

  /// Iterate over the external interfaces. External interfaces under the context of this project
  /// typically refers to the arrays (both read and write) and FIFOs (typically push)
  /// that are used by the module.
  pub(crate) fn ext_interf_iter<'borrow, 'res>(
    &'borrow self,
  ) -> impl Iterator<Item = (&BaseNode, &HashSet<BaseNode>)>
  where
    'sys: 'borrow,
    'sys: 'res,
  {
    self.external_interfaces.iter()
  }

  /// Iterate over the ports of the module.
  pub fn port_iter<'borrow, 'res>(&'borrow self) -> impl Iterator<Item = FIFORef<'res>> + 'res
  where
    'sys: 'borrow,
    'sys: 'res,
    'borrow: 'res,
  {
    self
      .ports
      .values()
      .map(|x| x.as_ref::<FIFO>(self.sys).unwrap())
  }
}

impl<'a> ModuleMut<'a> {
  /// Maintain the redundant information, array used in the module.
  ///
  /// # Arguments
  /// * `ext_node` - The external interface node.
  /// * `operand` - The operand node that uses this external interface.
  pub(crate) fn insert_external_interface(&mut self, ext_node: BaseNode, operand: BaseNode) {
    assert!(
      matches!(ext_node.get_kind(), NodeKind::Array | NodeKind::FIFO),
      "Expecting Array or FIFO but got {:?}",
      ext_node
    );
    assert!(operand.get_kind() == NodeKind::Operand);
    if !self.get().external_interfaces.contains_key(&ext_node) {
      self
        .get_mut()
        .external_interfaces
        .insert(ext_node, HashSet::new());
    }
    let users = self
      .get_mut()
      .external_interfaces
      .get_mut(&ext_node)
      .unwrap();
    users.insert(operand);
  }

  pub fn add_attr(&mut self, attr: Attribute) {
    self.get_mut().attr.insert(attr);
  }

  pub fn set_attrs(&mut self, attr: HashSet<Attribute>) {
    self.get_mut().attr = attr;
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
      .ports
      .values()
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
  pub fn create_module(&mut self, name: &str, ports: Vec<PortInfo>) -> BaseNode {
    let port_table = ports
      .into_iter()
      .map(|x| {
        (
          x.name.clone(),
          self.insert_element(FIFO::new(&x.ty, x.name.as_str())),
        )
      })
      .collect::<HashMap<_, _>>();
    let ports = port_table.values().cloned().collect::<Vec<_>>();
    let module = Module::new(name, port_table);
    let module = self.insert_element(module);
    // This part is kinda dirty, since we run into a chicken-egg problem: the port parent cannot
    // be set before the module is constructed. However, module's constructor accepts the ports
    // as inputs. The parent of the ports after the module is constructed.
    for input in ports {
      let mut fifo_mut = self.get_mut::<FIFO>(&input).unwrap();
      fifo_mut.get_mut().set_parent(module);
    }
    let new_name = self.symbol_table.insert(name, module);
    module.as_mut::<Module>(self).unwrap().get_mut().name = new_name;
    let body = Block::new(module);
    let body = self.insert_element(body);
    self.get_mut::<Module>(&module).unwrap().get_mut().body = body;
    module
  }
}
