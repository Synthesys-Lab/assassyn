pub mod attrs;
pub mod base;
pub mod downstream;
pub mod memory;
pub mod meta;

use std::collections::{HashMap, HashSet};

use crate::builder::system::PortInfo;
use crate::builder::SysBuilder;
use crate::ir::node::*;
use crate::ir::*;

pub use attrs::Attribute;
use base::ModuleBase;

/// The data structure for a module.
pub struct Module {
  /// The index key of this module in the slab buffer.
  pub(crate) key: usize,
  /// The base data of this module.
  pub(crate) base: ModuleBase,
  /// The redundant data of this module. The set of users that use this module. (in bound)
  pub(crate) user_set: HashSet<BaseNode>,
  /// The input ports of this module.
  ports: HashMap<String, BaseNode>,
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
      base: ModuleBase::new(name.to_string()),
      ports,
      user_set: HashSet::new(),
    }
  }

  pub fn get_attrs(&self) -> &HashSet<Attribute> {
    &self.base.attr
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
    self.base.name.as_str()
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
    self.base.body.as_ref::<Block>(self.sys).unwrap()
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
    self.base.external_interface.iter()
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
  pub fn add_attr(&mut self, attr: Attribute) {
    self.get_mut().base.attr.insert(attr);
  }

  pub fn set_attrs(&mut self, attr: HashSet<Attribute>) {
    self.get_mut().base.attr = attr;
  }

  /// Set the name of a module. Override the name given by the module builder.
  pub fn set_name(&mut self, name: String) {
    self.get_mut().base.name = name.to_string();
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
    module.as_mut::<Module>(self).unwrap().get_mut().base.name = new_name;
    let body = Block::new(module);
    let body = self.insert_element(body);
    self.get_mut::<Module>(&module).unwrap().get_mut().base.body = body;
    module
  }

  pub(crate) fn update_module_symbol_table(
    &mut self,
    module: BaseNode,
    old_name: Option<String>,
    new_name: String,
    node: BaseNode,
  ) -> String {
    if let Some(name) = old_name {
    let mut module_mut = module.as_mut::<Module>(self).unwrap();
    assert!(module_mut
      .get_mut()
      .base
      .symbol_table
      .remove(&name)
      .is_some());
    }
    let mut module_mut = module.as_mut::<Module>(self).unwrap();
    module_mut.get_mut().base.symbol_table.insert(&new_name, node)
  }
}
