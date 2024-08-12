use std::collections::HashSet;

use crate::builder::symbol_table::SymbolTable;

use super::{user::ExternalInterface, Attribute, BaseNode};


/// The data structure for a module.
pub struct ModuleBase {
  /// The index key of this module in the slab buffer.
  pub(crate) key: usize,
  /// The name of this module, can be overridden by `set_name`.
  name: String,
  /// The body of the module.
  pub(crate) body: BaseNode,
  /// The set of external interfaces used by the module. (out bound)
  pub(crate) external_interface: ExternalInterface,
  /// The attributes of this module.
  pub(crate) attr: HashSet<Attribute>,
  /// The symbol table that maintains the unique identifiers.
  pub(crate) symbol_table: SymbolTable,
}
