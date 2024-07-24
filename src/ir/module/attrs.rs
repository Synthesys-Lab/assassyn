use super::memory::MemoryParams;
use super::Module;

macro_rules! define_attrs {
  ( $($attrs: ident $( ( $vty:ty ) )? ),* $(,)* ) => {

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum Attribute {
      $($attrs $( ( $vty ) )? ),*
    }

  };
}

impl Module {
  pub fn has_attr(&self, attr: Attribute) -> bool {
    self.attr.contains(&attr)
  }
}

define_attrs!(
  ExplicitPop,      // In this module, FIFO pops are explicitly defined. TODO: remove this.
  OptNone,          // Avoid optimization on this module.
  EagerCallee, // All the binds in this module will be called after arguments are fully bound. TODO: remove this.
  AllowPartialCall, // Allow some arguments are not given to call this module.
  NoArbiter,   // The compiler will skip to generate an arbiter for this module,
  // even if it has multiple callers.
  Systolic, // This module's timing is systolic.
  Memory(MemoryParams),
);
