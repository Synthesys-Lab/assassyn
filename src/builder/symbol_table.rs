use std::collections::HashMap;

use crate::ir::node::BaseNode;

pub(crate) struct SymbolTable {
  unique_ids: HashMap<String, u32>,
  symbols: HashMap<String, BaseNode>,
}

impl SymbolTable {
  /// The helper function to find a unique identifier for the given identifier.
  fn identifier(&mut self, id: &str) -> String {
    // If the identifier is already in the symbol table, we append a number to it.
    if let Some(x) = self.unique_ids.get_mut(id) {
      // Append a number after.
      let res = format!("{}_{}", id, x);
      *x += 1;
      // To avoid user to use the appended identifier, we also insert it into the symbol table.
      self.unique_ids.insert(res.clone(), 0);
      res
    } else {
      // If not, we just use itself.
      self.unique_ids.insert(id.into(), 0);
      id.into()
    }
  }

  /// Insert the given node into the symbol table.
  pub(crate) fn insert(&mut self, id: &str, node: BaseNode) -> String {
    let id = self.identifier(id);
    self.symbols.insert(id.clone(), node);
    id
  }

  /// Get the node from the symbol table.
  pub(crate) fn get(&self, id: &str) -> Option<&BaseNode> {
    self.symbols.get(id)
  }

  /// Erase the given node from the symbol table.
  pub(crate) fn remove(&mut self, id: &str) -> Option<BaseNode> {
    self.symbols.remove(id)
  }

  pub(crate) fn iter(&self) -> impl Iterator<Item = (&String, &BaseNode)> {
    self.symbols.iter()
  }

  pub(crate) fn new() -> Self {
    SymbolTable {
      unique_ids: HashMap::new(),
      symbols: HashMap::new(),
    }
  }
}
