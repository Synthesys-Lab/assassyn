use super::utils::select_1h;

/// Gather is a data structure that gathers multiple conditional values into a single value.
/// Typically used by FIFO and Array writes.
pub(super) struct Gather {
  pub(super) bits: usize,
  // The condition of the gather
  pub(super) condition: Vec<String>,
  // The value of the gather
  pub(super) value: Vec<String>,
}

impl Gather {
  /// Create a new Gather with the given condition and value.
  pub(super) fn new(cond: String, value: String, bits: usize) -> Gather {
    Gather {
      bits,
      condition: vec![cond],
      value: vec![value],
    }
  }

  pub(super) fn and(&self, cond: &str, join: &str) -> String {
    if self.is_conditional() {
      let gather_cond = self
        .condition
        .iter()
        .map(|x| format!("({})", x))
        .collect::<Vec<_>>()
        .join(join);
      format!("({}) && ({})", cond, gather_cond)
    } else {
      cond.into()
    }
  }

  pub(super) fn select_1h(&self) -> String {
    if self.is_conditional() {
      select_1h(
        self
          .value
          .iter()
          .zip(self.condition.iter())
          .map(|(value, cond)| (cond.clone(), value.clone()))
          .collect::<Vec<_>>()
          .into_iter(),
        self.bits,
      )
    } else {
      self.value.first().unwrap().clone()
    }
  }

  /// If this gather is conditional.
  pub(super) fn is_conditional(&self) -> bool {
    assert!(!self.condition.is_empty());
    !self.condition.first().unwrap().is_empty()
  }

  /// Push a new conditional value into the gather.
  pub(super) fn push(&mut self, cond: String, value: String, bits: usize) {
    assert!(self.is_conditional());
    assert_eq!(self.bits, bits);
    self.condition.push(cond);
    self.value.push(value);
  }
}
