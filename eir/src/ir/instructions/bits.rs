use super::{Concat, Slice};

impl Slice<'_> {
  pub fn l(&self) -> usize {
    self.l_intimm().get_value() as usize
  }

  pub fn r(&self) -> usize {
    self.r_intimm().get_value() as usize
  }
}

impl ToString for Slice<'_> {
  fn to_string(&self) -> String {
    let a = self.x().to_string(self.expr.sys);
    let l = self.l();
    let r = self.r();
    format!("{} = {}[{}:{}] // {}", self.expr.get_name(), a, l, r, a,)
  }
}

impl ToString for Concat<'_> {
  fn to_string(&self) -> String {
    let a = self.msb().to_string(self.expr.sys);
    let (b, b_bits) = {
      let b = self.lsb();
      (
        b.to_string(self.expr.sys),
        b.get_dtype(self.expr.sys).unwrap().get_bits(),
      )
    };
    format!(
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
