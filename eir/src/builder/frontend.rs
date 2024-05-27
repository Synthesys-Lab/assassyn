use crate::ir::node::BaseNode;

impl BaseNode {
  pub fn add(&self, _: BaseNode) -> BaseNode {
    BaseNode::unknown()
  }
}
