use std::fmt::Display;

use crate::ir::{expr::subcode, Opcode};

use super::BlockIntrinsic;

impl BlockIntrinsic<'_> {
  pub fn get_subcode(&self) -> subcode::BlockIntrinsic {
    match self.expr.get_opcode() {
      Opcode::BlockIntrinsic { intrinsic } => intrinsic,
      _ => panic!(
        "Expecting Opcode::BlockIntrinsic, but got {:?}",
        self.expr.get_opcode()
      ),
    }
  }
}

impl Display for BlockIntrinsic<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{} {} {}",
      match self.get_subcode() {
        subcode::BlockIntrinsic::Value => "value",
        subcode::BlockIntrinsic::Condition => "if",
        subcode::BlockIntrinsic::WaitUntil => "wait_until",
        subcode::BlockIntrinsic::Cycled => "cycled",
      },
      self.value().to_string(self.get().sys),
      match self.get_subcode() {
        subcode::BlockIntrinsic::Value | subcode::BlockIntrinsic::WaitUntil => "",
        subcode::BlockIntrinsic::Condition | subcode::BlockIntrinsic::Cycled => "{",
      },
    )
  }
}
