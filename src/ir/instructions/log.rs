use std::fmt::{Display, Formatter};

use super::Log;

impl Display for Log<'_> {
  fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
    let mut res = "log(".to_string();
    for op in self.expr.operand_iter() {
      res.push_str(&op.get_value().to_string(self.expr.sys));
      res.push_str(", ");
    }
    res.push(')');
    write!(fmt, "{}", res)
  }
}
