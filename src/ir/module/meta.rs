use std::collections::HashMap;

use super::node::BaseNode;

#[derive(Default)]
pub struct TemplateMaster {
  params: Vec<BaseNode>,
}

pub struct TemplateInstance {
  master: BaseNode,
  params: HashMap<BaseNode, BaseNode>,
}

impl TemplateMaster {
  pub fn new(params: Vec<BaseNode>) -> Self {
    TemplateMaster { params }
  }

  pub fn get_params(&self) -> &Vec<BaseNode> {
    &self.params
  }
}

impl TemplateInstance {
  pub fn new(master: BaseNode, params: HashMap<BaseNode, BaseNode>) -> Self {
    TemplateInstance { master, params }
  }

  pub fn get_master(&self) -> &BaseNode {
    &self.master
  }

  pub fn get_params(&self) -> &HashMap<BaseNode, BaseNode> {
    &self.params
  }
}

impl Default for TemplateInstance {
  fn default() -> Self {
    TemplateInstance {
      master: BaseNode::unknown(),
      params: HashMap::new(),
    }
  }
}
