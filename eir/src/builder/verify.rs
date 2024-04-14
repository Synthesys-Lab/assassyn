use crate::ir::{
  expr::OperandOf,
  node::{BaseNode, ExprRef, IsElement},
  visitor::Visitor,
  Expr,
};

/// NOTE: This module is to verify the soundness of an system IR. Not a RTL verification generator.
use super::SysBuilder;

impl OperandOf {
  fn verify(&self, sys: &SysBuilder, node: &BaseNode) {
    let expr = self
      .user
      .as_ref::<Expr>(sys)
      .expect("User should be an expression!");
    let operand = expr
      .get_operand(self.idx)
      .expect(&format!("No such operand {}", self.idx));
    assert_eq!(operand, node);
  }
}

struct Verifier;

impl Visitor<()> for Verifier {
  fn visit_expr(&mut self, expr: &ExprRef<'_>) -> Option<()> {
    let node = expr.upcast();
    for user in expr.users().iter() {
      user.verify(expr.sys, &node);
    }
    None
  }
}

pub fn verify(sys: &SysBuilder) {
  for m in sys.module_iter() {
    let node = m.upcast();
    for user in m.users().iter() {
      user.verify(sys, &node);
    }
    let body = m.get_body();
    Verifier.visit_block(&body);
  }
}
