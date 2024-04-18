use std::collections::HashMap;

use super::node::BaseNode;

pub struct Template {
  module: BaseNode,
  params: HashMap<BaseNode, BaseNode>,
}

impl Template {
  pub fn new(module: BaseNode, params: HashMap<BaseNode, BaseNode>) -> Self {
    Self { module, params }
  }

  pub fn get_module(&self) -> &BaseNode {
    &self.module
  }

  pub fn get_params(&self) -> &HashMap<BaseNode, BaseNode> {
    &self.params
  }
}
