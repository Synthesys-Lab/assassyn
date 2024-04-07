use std::collections::HashMap;

use crate::ir::node::BaseNode;

pub(super) struct FingerPrintCache {
  master_cache: HashMap<usize, BaseNode>,
}

impl FingerPrintCache {
  pub(super) fn new() -> Self {
    FingerPrintCache {
      master_cache: HashMap::new(),
    }
  }

  pub(super) fn get_master(&self, fp: usize) -> Option<&BaseNode> {
    self.master_cache.get(&fp)
  }

  pub(super) fn set_master(&mut self, fp: usize, node: BaseNode) {
    self.master_cache.insert(fp, node);
  }
}
