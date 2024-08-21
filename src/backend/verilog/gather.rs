/// Condition enum is used to determine the condition of a gather.
pub(super) enum Condition {
  // The value is unconditionally from a single source
  Unconditional,
  // The value is conditionally from multiple source, and the string is the "||" reduced condition.
  Conditional(String),
}

impl Condition {
  pub(super) fn and(&self, cond: String) -> String {
    match self {
      Condition::Unconditional => cond.clone(),
      Condition::Conditional(c) => format!("({}) && ({})", c, cond),
    }
  }
}

/// Gather is a data structure that gathers multiple conditional values into a single value.
/// Typically used by FIFO and Array writes.
pub(super) struct Gather {
  // The condition of the gather
  pub(super) condition: Condition,
  // The value of the gather
  pub(super) value: String,
}

impl Gather {
  /// Create a new Gather with the given condition and value.
  pub(super) fn new(cond: String, value: String, bits: usize) -> Gather {
    Gather {
      condition: if cond.is_empty() {
        Condition::Unconditional
      } else {
        Condition::Conditional(cond.clone())
      },
      value: if cond.is_empty() {
        value
      } else {
        format!("({{ {} {{ {} }} }} & {})", bits, cond, value)
      },
    }
  }

  /// Push a new conditional value into the gather.
  pub(super) fn push(&mut self, cond: String, value: String, bits: usize) {
    match self.condition {
      Condition::Unconditional => panic!("Mixed conditional and unconditional gather!"),
      Condition::Conditional(ref mut c) => {
        *c = format!("{} || ({})", c, cond);
        self.value = format!("{} | ({{ {} {{ {} }} }} & {})", self.value, bits, cond, value);
      }
    }
  }
}
