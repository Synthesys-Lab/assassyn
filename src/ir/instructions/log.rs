use super::Log;

impl ToString for Log<'_> {
  fn to_string(&self) -> String {
    let mut res = "log(".to_string();
    for op in self.expr.operand_iter() {
      res.push_str(&op.get_value().to_string(self.expr.sys));
      res.push_str(", ");
    }
    res.push(')');
    res
  }
}
