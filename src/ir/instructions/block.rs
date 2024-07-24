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

impl ToString for BlockIntrinsic<'_> {
  fn to_string(&self) -> String {
    format!(
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
