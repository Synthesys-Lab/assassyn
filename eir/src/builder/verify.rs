use crate::ir::{node::*, visitor::Visitor};

use crate::ir::*;

use self::user::Operand;

/// NOTE: This module is to verify the soundness of an system IR. Not a RTL verification generator.
use super::SysBuilder;

impl Operand {
  fn verify(&self, sys: &SysBuilder) {
    assert!(sys.contains(&self.upcast()));
    let expr = self
      .get_user()
      .as_ref::<Expr>(sys)
      .expect("User should be an expression!");
    let operand = expr
      .operand_iter()
      .position(|op| self.upcast().eq(&op.upcast()));
    assert!(operand.is_some());
  }
}

struct Verifier;

impl Visitor<()> for Verifier {
  fn visit_expr(&mut self, expr: &ExprRef<'_>) -> Option<()> {
    for user in expr.users().iter() {
      let operand = user.as_ref::<Operand>(expr.sys).unwrap();
      assert!(operand.get_value().eq(&expr.upcast()));
      operand.verify(expr.sys);
    }
    for operand in expr.operand_iter() {
      let operand = operand.get_value();
      match operand.get_kind() {
        NodeKind::FIFO => {
          let fifo = operand.as_ref::<FIFO>(expr.sys).unwrap();
          fifo.users().contains(operand);
        }
        NodeKind::Expr => {
          let expr = operand.as_ref::<Expr>(expr.sys).unwrap();
          expr.users().contains(operand);
        }
        NodeKind::Module => {
          let module = operand.as_ref::<Module>(expr.sys).unwrap();
          module.users().contains(operand);
        }
        _ => {}
      }
    }
    if expr.get_opcode() == Opcode::Cast {
      let src_ty = expr
        .get_operand(0)
        .unwrap()
        .get_value()
        .get_dtype(expr.sys)
        .unwrap();
      let dest_ty = expr.dtype();
      assert!(
        // uint to int, width must be expanded
        (
          dest_ty.is_int() && dest_ty.is_signed() &&
          src_ty.is_int() && !src_ty.is_signed() &&
          dest_ty.get_bits() > src_ty.get_bits()
        ) ||
        // other senario, disallow trimming
        (dest_ty.get_bits() >= src_ty.get_bits())
      );
    }
    if expr.get_opcode() == Opcode::Sext {
      let src_ty = expr
        .get_operand(0)
        .unwrap()
        .get_value()
        .get_dtype(expr.sys)
        .unwrap();
      let dest_ty = expr.dtype();
      assert!(
        // dest needs to be int
        dest_ty.is_int() && dest_ty.is_signed() &&
        // disallow trimming
        dest_ty.get_bits() >= src_ty.get_bits()
      );
    }
    None
  }
}

pub fn verify(sys: &SysBuilder) {
  for m in sys.module_iter() {
    for user in m.users().iter() {
      user.as_ref::<Operand>(sys).unwrap().verify(sys);
    }
    let body = m.get_body();
    Verifier.visit_block(&body);
  }
}
