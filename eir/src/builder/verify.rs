use crate::ir::expr::subcode;
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

struct Verifier {
  in_wait_until_cond: bool,
}

impl Verifier {
  fn new() -> Self {
    Self {
      in_wait_until_cond: false,
    }
  }
}

impl Visitor<()> for Verifier {
  fn visit_block(&mut self, block: &BlockRef<'_>) -> Option<()> {
    if let BlockKind::WaitUntil(cond) = block.get_kind() {
      if let Ok(cond) = cond.as_ref::<Block>(block.sys) {
        assert!(
          cond.get_value().is_some(),
          "WaitUntil condition must be a valued block!"
        );
        self.in_wait_until_cond = true;
        for elem in cond.iter() {
          self.dispatch(block.sys, &elem, vec![]);
        }
        self.in_wait_until_cond = false;
      } else {
        panic!("WaitUntil condition must be a valued block!");
      }
    }
    for expr in block.iter() {
      self.dispatch(block.sys, &expr, vec![]);
    }
    ().into()
  }

  fn visit_expr(&mut self, expr: &ExprRef<'_>) -> Option<()> {
    if self.in_wait_until_cond {
      assert!(
        !expr.get_opcode().has_side_effect(),
        "WaitUntil operations should have no side effects, but {:?} found!",
        expr.get_opcode()
      );
    }
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
    match expr.get_opcode() {
      Opcode::Cast { cast } => {
        let src_ty = expr
          .get_operand(0)
          .unwrap()
          .get_value()
          .get_dtype(expr.sys)
          .unwrap();
        let dest_ty = expr.dtype();
        match cast {
          subcode::Cast::Cast => {
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
          subcode::Cast::SExt => {
            assert!(
              // dest needs to be int
              dest_ty.is_int() && dest_ty.is_signed() &&
              // disallow trimming
              dest_ty.get_bits() >= src_ty.get_bits()
            );
          }
          _ => {}
        }
      }
      _ => {}
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
    Verifier::new().visit_block(&body);
  }
}
