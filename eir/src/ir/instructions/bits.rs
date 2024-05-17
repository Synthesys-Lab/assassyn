use super::Slice;

impl Slice<'_> {
  pub fn l(&self) -> usize {
    self.l_intimm().get_value() as usize
  }

  pub fn r(&self) -> usize {
    self.r_intimm().get_value() as usize
  }
}
