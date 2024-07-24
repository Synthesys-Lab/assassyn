use std::fmt::Display;

use super::{Concat, Slice};
use crate::ir::Typed;

impl Slice<'_> {
  pub fn l(&self) -> usize {
    self.l_intimm().get_value() as usize
  }

  pub fn r(&self) -> usize {
    self.r_intimm().get_value() as usize
  }
}

impl Display for Slice<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let a = self.x().to_string(self.expr.sys);
    let l = self.l();
    let r = self.r();
    write!(
      f,
      "{} = {}[{}:{}] // {}",
      self.expr.get_name(),
      a,
      l,
      r,
      self.expr.dtype()
    )
  }
}

impl Display for Concat<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let a = self.msb().to_string(self.expr.sys);
    let (b, b_bits) = {
      let b = self.lsb();
      (
        b.to_string(self.expr.sys),
        b.get_dtype(self.expr.sys).unwrap().get_bits(),
      )
    };
    write!(
      f,
      "{} = concat({}, {}) // {} << {} | {}",
      self.expr.get_name(),
      a,
      b,
      a,
      b_bits,
      b,
    )
  }
}
