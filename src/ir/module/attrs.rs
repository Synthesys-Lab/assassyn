use std::ops::RangeInclusive;

use crate::builder::SysBuilder;

use super::{BaseNode, Module};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemoryParams {
  pub width: usize,
  pub depth: usize,
  pub lat: RangeInclusive<usize>,
  pub init_file: Option<String>,
  // Pins for memory module.
  pub array: BaseNode,
  pub re: BaseNode,
  pub we: BaseNode,
  pub addr: BaseNode,
  pub wdata: BaseNode,
}

impl MemoryParams {
  pub fn new(
    width: usize,
    depth: usize,
    lat: RangeInclusive<usize>,
    init_file: Option<String>,
    array: BaseNode,
    re: BaseNode,
    we: BaseNode,
    addr: BaseNode,
    wdata: BaseNode,
  ) -> Self {
    Self {
      width,
      depth,
      lat,
      init_file,
      array,
      re,
      we,
      addr,
      wdata,
    }
  }

  pub fn is_sram(&self) -> bool {
    return self.lat.start().eq(&1) && self.lat.end().eq(&1);
  }

  pub fn to_string(&self, sys: &SysBuilder) -> String {
    format!(
      "width: {}, depth: {}, lat: [{:?}], file: {}, array: {} (.we({}), .re({}), .addr({}), .wdata({}))",
      self.width,
      self.depth,
      self.lat,
      self.init_file.clone().map_or("None".to_string(), |x| x),
      self.array.to_string(sys),
      self.we.to_string(sys),
      self.re.to_string(sys),
      self.addr.to_string(sys),
      self.wdata.to_string(sys),
    )
  }
}

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
  // This module is a memory downstream module.
  MemoryParams(MemoryParams),
);
