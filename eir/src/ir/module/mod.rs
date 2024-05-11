pub mod attrs;
pub mod memory;
pub mod meta;

use std::collections::{HashMap, HashSet};

use crate::builder::symbol_table::SymbolTable;
use crate::builder::system::PortInfo;
use crate::builder::SysBuilder;
use crate::ir::node::*;
use crate::ir::*;

use self::instructions::GetElementPtr;
use self::user::Operand;
pub use attrs::Attribute;

/// The data structure for a module.
pub struct Module {
  /// The index key of this module in the slab buffer.
  pub(crate) key: usize,
  /// The name of this module, can be overridden by `set_name`.
  name: String,
  /// The input ports of this module.
  ports: Vec<BaseNode>,
  /// The body of the module.
  body: BaseNode,
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
  pub fn new(name: &str, inputs: Vec<BaseNode>) -> Module {
    Module {
      key: 0,
      name: name.to_string(),
      ports: inputs,
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

  /// Get the given input reference.
  ///
  /// # Arguments
  ///
  /// * `i` - The index of the input.
  pub fn get_port(&self, i: usize) -> Option<FIFORef<'_>> {
    self
      .ports
      .get(i)
      .map(|x| x.as_ref::<FIFO>(&self.sys).unwrap())
  }

  /// Get the input by name.
  ///
  /// # Arguments
  ///
  /// * `name` - The name of the input.
  pub fn get_port_by_name(&self, name: &str) -> Option<FIFORef<'_>> {
    self
      .ports
      .iter()
      .find(|x| x.as_ref::<FIFO>(self.sys).unwrap().get_name().eq(name))
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
      .iter()
      .map(|x| x.as_ref::<FIFO>(self.sys).unwrap())
  }

  /// Gather all the related external interfaces with the given operand. This is typically used to
  /// maintain the redundant information when modifying this IR.
  /// If the given operand is an operand, gather just this specific operand.
  /// If the given operand is a value reference, gather all the operands that `get_value == this
  /// operand`.
  ///
  /// # Arguments
  ///
  /// * `operand` - The operand to gather the related external interfaces.
  pub(crate) fn gather_related_externals(&self, operand: BaseNode) -> Vec<(BaseNode, BaseNode)> {
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
            .filter(|x| {
              (*x).eq(&operand) || {
                let user = (*x).as_ref::<Operand>(self.sys).unwrap();
                user.get_value().eq(&operand)
              }
            })
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
  ///
  /// # Arguments
  /// * `ext_node` - The external interface node.
  /// * `operand` - The operand node that uses this external interface.
  pub(crate) fn insert_external_interface(&mut self, ext_node: BaseNode, operand: BaseNode) {
    assert!(
      ext_node.get_kind() == NodeKind::Array || ext_node.get_kind() == NodeKind::FIFO,
      "Expecting Array or FIFO but got {:?}",
      ext_node
    );
    assert!(operand.get_kind() == NodeKind::Operand);
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
    users.insert(operand);
  }

  pub fn add_attr(&mut self, attr: Attribute) {
    self.get_mut().attr.insert(attr);
  }

  pub fn set_attrs(&mut self, attr: HashSet<Attribute>) {
    self.get_mut().attr = attr;
  }

  /// Remove a specific external interface's usage. If this usage set is empty after the removal,
  /// remove the external interface from the module, too.
  ///
  /// # Arguments
  ///
  /// * `ext_node` - The external interface node.
  /// * `operand` - The operand node that uses this external interface.
  pub(crate) fn remove_external_interface(&mut self, ext_node: BaseNode, operand: BaseNode) {
    if let Some(operations) = self.get_mut().external_interfaces.get_mut(&ext_node) {
      assert!(operations.contains(&operand));
      operations.remove(&operand);
      if operations.is_empty() {
        self.get_mut().external_interfaces.remove(&ext_node);
      }
    }
  }

  /// Remove all the related external interfaces with the given condition.
  pub(crate) fn remove_related_externals(&mut self, operand: BaseNode) {
    let to_remove = self.get().gather_related_externals(operand);
    to_remove.into_iter().for_each(|(ext, operand)| {
      self.remove_external_interface(ext, operand);
    });
  }

  /// Add related external interfaces to the module.
  pub(crate) fn add_related_externals(&mut self, operand: BaseNode) {
    // Reconnect the external interfaces if applicable.
    // TODO(@were): Maybe later unify a common interface for this.
    let operand_ref = operand.as_ref::<Operand>(self.sys).unwrap();
    let value = operand_ref.get_value();
    match value.get_kind() {
      NodeKind::Expr => {
        if let Ok(gep) = value.as_expr::<GetElementPtr>(self.sys) {
          let array = gep.get_array();
          self.insert_external_interface(array.upcast(), operand);
        }
      }
      NodeKind::FIFO => {
        self.insert_external_interface(value.clone(), operand);
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

  /// Remove a given port of the module.
  /// TODO: Stricter check for the port usage.
  pub fn remove_port(&mut self, idx: usize) {
    self.get_mut().ports.remove(idx);
  }
}

impl Typed for ModuleRef<'_> {
  fn dtype(&self) -> DataType {
    let types = self
      .ports
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
  pub fn create_module(&mut self, name: &str, ports: Vec<PortInfo>) -> BaseNode {
    let n_inputs = ports.len();
    let ports = ports
      .into_iter()
      .map(|x| self.insert_element(FIFO::new(&x.ty, x.name.as_str())))
      .collect::<Vec<_>>();
    let module_name = self.symbol_table.identifier(name);
    let module = Module::new(&module_name, ports);
    let module = self.insert_element(module);
    // This part is kinda dirty, since we run into a chicken-egg problem: the port parent cannot
    // be set before the module is constructed. However, module's constructor accepts the ports
    // as inputs. The parent of the ports after the module is constructed.
    for i in 0..n_inputs {
      let (input, name) = {
        let module = module.as_ref::<Module>(self).unwrap();
        let port = module.get_port(i).unwrap();
        (port.upcast(), port.get_name().clone())
      };
      // Use the symbol table of the module to register the names.
      let new_name = {
        let mut module_mut = module.as_mut::<Module>(self).unwrap();
        module_mut.get_mut().symbol_table.identifier(&name)
      };
      assert_eq!(
        name, new_name,
        "The names of the ports should be unique! Otherwise, `Module::get_port_by_name` will not work!"
      );
      let mut fifo_mut = self.get_mut::<FIFO>(&input).unwrap();
      fifo_mut.get_mut().set_parent(module.clone());
      fifo_mut.get_mut().set_idx(i);
    }
    self.global_symbols.insert(module_name, module.clone());
    let body = Block::new(BlockKind::None, module.clone());
    let body = self.insert_element(body);
    self.get_mut::<Module>(&module).unwrap().get_mut().body = body.clone();
    module
  }
}
