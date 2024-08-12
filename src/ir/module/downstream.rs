use std::collections::HashMap;

use crate::builder::SysBuilder;

use super::{
  base::ModuleBase,
  node::{BaseNode, BlockRef, DownstreamRef, OptionalRef, Parented},
  Block, Optional,
};

/// The data structure for a downstream module.
pub struct Downstream {
  /// The key of this module in the slab buffer.
  pub(crate) key: usize,
  /// The base of the module.
  base: ModuleBase,
  /// The set of the ports of this module.
  ports: HashMap<String, BaseNode>,
}

impl Downstream {
  pub fn new(name: String, ports: HashMap<String, BaseNode>) -> Self {
    Downstream {
      key: 0,
      base: ModuleBase::new(name),
      ports,
    }
  }
}

impl<'sys> DownstreamRef<'sys> {
  pub fn get_body(&self) -> BlockRef<'_> {
    self.base.body.as_ref::<Block>(self.sys).unwrap()
  }

  pub fn get_name(&self) -> &str {
    &self.base.name
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

impl SysBuilder {
  /// Create a downstream module.
  pub fn create_downstream(&mut self, name: String, ports: HashMap<String, BaseNode>) -> BaseNode {
    let mut downstream = Downstream::new(name.clone(), ports);
    let body = self.create_block();
    downstream.base.body = body;
    let res = self.insert_element(downstream);
    let name = self.symbol_table.insert(&name, res);
    res.as_mut::<Downstream>(self).unwrap().get_mut().base.name = name;
    body
      .as_mut::<Block>(self)
      .unwrap()
      .get_mut()
      .set_parent(res);
    res
  }
}
