use std::collections::HashMap;

pub(crate) struct SymbolTable {
  unique_ids: HashMap<String, u32>,
}

impl SymbolTable {
  pub(crate) fn identifier(&mut self, id: &str) -> String {
    // If the identifier is already in the symbol table, we append a number to it.
    if let Some(x) = self.unique_ids.get_mut(id) {
      // Append a number after.
      let res = format!("{}_{}", id, x);
      *x += 1;
      // To avoid user to use the appended identifier, we also insert it into the symbol table.
      self.unique_ids.insert(res.clone(), 0);
      return res;
    }
    // If not, we just use itself.
    self.unique_ids.insert(id.into(), 0);
    id.into()
  }

  pub(crate) fn new() -> Self {
    SymbolTable {
      unique_ids: HashMap::new(),
    }
  }
}
