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
use user::ExternalInterface;

/// The data structure for a module.
pub struct Module {
  /// The index key of this module in the slab buffer.
  pub(crate) key: usize,
  /// The name of this module, can be overridden by `set_name`.
  pub(super) name: String,
  /// The body of the module.
  pub(crate) body: BaseNode,
  /// The set of external interfaces used by the module. (out bound)
  pub(crate) external_interface: ExternalInterface,
  /// The attributes of this module.
  pub(crate) attr: HashSet<Attribute>,
  /// The symbol table that maintains the unique identifiers.
  pub(crate) symbol_table: SymbolTable,
  /// The sub-class data structures of this module.
  ports: HashMap<String, BaseNode>,
  /// The set of users of this module.
  pub(crate) user_set: HashSet<BaseNode>,
}

impl Default for Module {
  fn default() -> Self {
    Module {
      key: 0,
      name: String::new(),
      body: BaseNode::unknown(),
      external_interface: ExternalInterface::new(),
      attr: HashSet::new(),
      symbol_table: SymbolTable::new(),
      ports: HashMap::new(),
      user_set: HashSet::new(),
    }
  }
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
      user_set: HashSet::new(),
      ..Default::default()
    }
  }

  pub fn get_attrs(&self) -> &HashSet<Attribute> {
    &self.attr
  }
}

impl<'sys> ModuleRef<'sys> {
  /// Get the number of inputs to the module.
  pub fn get_num_inputs(&self) -> usize {
    self.ports.len()
  }

  pub fn is_downstream(&self) -> bool {
    self.has_attr(Attribute::Downstream)
  }

  /// Get the input by name.
  ///
  /// # Arguments
  ///
  /// * `name` - The name of the input.
  pub fn get_fifo(&self, name: &str) -> Option<FIFORef<'_>> {
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
    self.external_interface.iter()
  }

  /// Iterate over the ports of the module.
  pub fn fifo_iter<'borrow, 'res>(&'borrow self) -> impl Iterator<Item = FIFORef<'res>> + 'res
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

  /// Iterate over the callers that triggers this module.
  pub fn callers<'borrow, 'res>(&'borrow self) -> impl Iterator<Item = ModuleRef<'res>> + 'res
  where
    'sys: 'borrow,
    'sys: 'res,
    'borrow: 'res,
  {
    let mut seen = HashSet::new();
    self
      .user_set
      .iter()
      .map(|x| {
        x.as_ref::<Operand>(self.sys)
          .unwrap()
          .get_expr()
          .get_block()
          .get_module()
          .as_ref::<Module>(self.sys)
          .unwrap()
      })
      .filter(move |x| seen.insert(x.key))
  }

  /// Iterate over the callees that are triggered by this module.
  pub fn callees<'borrow, 'res>(&'borrow self) -> impl Iterator<Item = ModuleRef<'res>> + 'res
  where
    'sys: 'borrow,
    'sys: 'res,
    'borrow: 'res,
  {
    self
      .ext_interf_iter()
      .filter_map(|(k, _)| k.as_ref::<Module>(self.sys).ok())
  }
}

impl<'a> ModuleMut<'a> {
  pub fn add_attr(&mut self, attr: Attribute) {
    self.get_mut().attr.insert(attr);
  }

  pub fn set_attrs(&mut self, attr: HashSet<Attribute>) {
    self.get_mut().attr = attr;
  }

  /// Set the name of a module. Override the name given by the module builder.
  pub fn set_name(&mut self, name: String) {
    self.get_mut().name = name.to_string();
  }
}

impl Typed for ModuleRef<'_> {
  fn dtype(&self) -> DataType {
    let types = self
      .ports
      .values()
      .map(|x| x.as_ref::<FIFO>(self.sys).unwrap().scalar_ty())
      .collect::<Vec<_>>();
    DataType::module(
      if self.is_downstream() {
        "downstream"
      } else {
        "module"
      }
      .into(),
      types,
    )
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
    // This is a BIG PITFALL here. We CANNOT call `sys.create_block` here, because the created block
    // will be inserted at the current insert point of the system builder, which is NOT this module.
    // We should: 1. manually new a block instance; 2. insert this newed block into the slab buffer;
    // 3. set the body of the module to this newed block.
    //
    // This same issue applies to the downstream module. See below.
    //
    // TODO(@were): I am not sure if this is a good design. Maybe I should unify the way of creating
    // blocks. When extending downstream module, I totally forgot about this issue.
    let body = Block::new(module);
    let body = self.insert_element(body);
    self.get_mut::<Module>(&module).unwrap().get_mut().body = body;
    module
  }

  /// Create a downstream module.
  pub fn create_downstream(&mut self, name: &str) -> BaseNode {
    let downstream = Module::new(name, HashMap::new());
    let res = self.insert_element(downstream);
    // This is a BIG PITFALL here. See the comment in `create_module`.
    let body = Block::new(res);
    let body = self.insert_element(body);
    let name = self.symbol_table.insert(name, res);
    let mut downstream_mut = res.as_mut::<Module>(self).unwrap();
    downstream_mut.get_mut().name = name;
    downstream_mut.get_mut().body = body;
    downstream_mut.get_mut().attr.insert(Attribute::Downstream);
    res
  }
}
