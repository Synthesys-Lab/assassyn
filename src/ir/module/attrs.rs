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
  // In this module, FIFO pops are explicitly defined. TODO: remove this, since it will be handled
  // by our Python frontend behaviorally.
  ExplicitPop,
  // Avoid optimization on this module.
  OptNone,
  // All the binds in this module will be called after arguments are fully bound. TODO: remove this,
  // since it will be supported by our new Python frontend behaviorally.
  EagerCallee,
  // Allow some arguments are not given to call this module. TODO: More strict enforcement for
  // function calls.
  AllowPartialCall,
  // The compiler will skip to generate an arbiter for this module,
  // even if it has multiple callers.
  NoArbiter,
  // This module's timing is systolic.
  Systolic,
  // This module is a downstream module, which is combinationally connected to the upstream module.
  Downstream,
);
