use crate::ir::{expr::OperandOf, node::*, visitor::Visitor};

use crate::ir::*;

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
    for (i, operand) in expr.operand_iter().enumerate() {
      match operand.get_kind() {
        NodeKind::FIFO => {
          let fifo = operand.as_ref::<FIFO>(expr.sys).unwrap();
          fifo.users().contains(&OperandOf::new(node, i));
        }
        NodeKind::Expr => {
          let expr = operand.as_ref::<Expr>(expr.sys).unwrap();
          expr.users().contains(&OperandOf::new(node, i));
        }
        NodeKind::Module => {
          let module = operand.as_ref::<Module>(expr.sys).unwrap();
          module.users().contains(&OperandOf::new(node, i));
        }
        _ => {}
      }
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
