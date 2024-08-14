use std::fmt::Display;
use std::ops::RangeInclusive;

use crate::ir::node::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemoryParams {
  pub array: BaseNode,
  pub width: usize,
  pub depth: usize,
  pub lat: RangeInclusive<usize>,
  pub init_file: Option<String>,
}

impl Default for MemoryParams {
  fn default() -> Self {
    Self {
      array: BaseNode::unknown(),
      width: 0,
      depth: 0,
      lat: 0..=0,
      init_file: None,
    }
  }
}

impl MemoryParams {
  pub fn new(
    array: BaseNode,
    width: usize,
    depth: usize,
    lat: RangeInclusive<usize>,
    init_file: Option<String>,
  ) -> Self {
    Self {
      array,
      width,
      depth,
      lat,
      init_file,
    }
  }
}

impl Display for MemoryParams {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "width: {} depth: {} lat: [{:?}], file: {}",
      self.width,
      self.depth,
      self.lat,
      self.init_file.clone().map_or("None".to_string(), |x| x)
    )
  }
}
