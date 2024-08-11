use std::collections::{HashMap, HashSet};

use super::node::BaseNode;

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
}

impl Downstream {
}
