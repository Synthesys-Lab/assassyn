use crate::backend::simulator::camelize;

use super::memory::MemoryParams;
use super::Module;

macro_rules! define_attrs {
  ( $($attrs: ident $( ( $vty:ty ) )? ),* $(,)* ) => {

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum Attribute {
      $($attrs $( ( $vty ) )? ),*
    }

    impl Attribute {
      pub fn from_string(s: &str) -> Option<Attribute> {
        let s = camelize(s);
        match s.as_str() {
          $(stringify!($attrs) => Some(Attribute::$attrs $( ( <$vty>::default() ) )? ),)*
          _ => None,
        }
      }
    }

  };
}

impl Module {
  pub fn has_attr(&self, attr: Attribute) -> bool {
    self.attr.contains(&attr)
  }
}

define_attrs!(
  ExplicitPop,      // In this module, FIFO pops are explicitly defined.
  OptNone,          // Avoid optimization on this module.
  EagerCallee,      // All the binds in this module will be called after arguments are fully bound.
  AllowPartialCall, // Allow some arguments are not given to call this module.
  NoArbiter,        // The compiler will skip to generate an arbiter for this module,
  // even if it has multiple callers.
  Systolic, // The module is a systolic array.
  Memory(MemoryParams),
);
