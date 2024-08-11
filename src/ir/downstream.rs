use std::collections::{HashMap, HashSet};

use super::{
  node::{BaseNode, BlockRef, DownstreamRef, OptionalRef},
  Block, Optional,
};

/// The data structure for a downstream module.
pub struct Downstream {
  /// The index key of this downstream node in the slab buffer.
  pub(crate) key: usize,
  /// The name of this down stream module, can be overriden by `set_name`.
  name: String,
  /// The set of the external interfaces used by this module.
  pub(crate) external_interfaces: HashMap<BaseNode, HashSet<BaseNode>>,
  /// The body of this downstream module.
  pub(crate) body: BaseNode,
  /// The set of the ports of this module.
  pub(crate) ports: HashMap<String, BaseNode>,
}

impl Downstream {
  pub fn new(name: String, body: BaseNode, ports: HashMap<String, BaseNode>) -> Self {
    Downstream {
      key: 0,
      name,
      external_interfaces: HashMap::new(),
      body,
      ports,
    }
  }
}

impl<'sys> DownstreamRef<'sys> {
  pub fn get_body(&self) -> BlockRef<'_> {
    self.body.as_ref::<Block>(self.sys).unwrap()
  }

  pub fn get_name(&self) -> &str {
    &self.name
  }

  /// Iterate over the ports of the module.
  pub fn port_iter<'borrow, 'res>(&'borrow self) -> impl Iterator<Item = OptionalRef<'res>> + 'res
  where
    'sys: 'borrow,
    'sys: 'res,
    'borrow: 'res,
  {
    self
      .ports
      .iter()
      .map(|(_, x)| x.as_ref::<Optional>(self.sys).unwrap())
  }
}
