use std::fmt::Display;

use crate::ir::node::BaseNode;
use crate::ir::{expr::subcode, Opcode};

use super::BlockIntrinsic;
use super::PureIntrinsic;

impl BlockIntrinsic<'_> {
  pub fn get_subcode(&self) -> subcode::BlockIntrinsic {
    match self.expr.get_opcode() {
      Opcode::BlockIntrinsic { intrinsic } => intrinsic,
      _ => panic!("Expecting Opcode::BlockIntrinsic, but got {:?}", self.expr.get_opcode()),
    }
  }

  pub fn value(&self) -> Option<BaseNode> {
    return self.get().get_operand_value(0);
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
        subcode::BlockIntrinsic::Finish => "halt",
      },
      self
        .value()
        .map_or("".into(), |x| x.to_string(self.get().sys)),
      match self.get_subcode() {
        subcode::BlockIntrinsic::Value
        | subcode::BlockIntrinsic::WaitUntil
        | subcode::BlockIntrinsic::Finish => "",
        subcode::BlockIntrinsic::Condition | subcode::BlockIntrinsic::Cycled => "{",
      },
    )
  }
}

impl Display for PureIntrinsic<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self.expr.get_opcode() {
        Opcode::PureIntrinsic { intrinsic } => match intrinsic {
          subcode::PureIntrinsic::FIFOValid
          | subcode::PureIntrinsic::FIFOReady
          | subcode::PureIntrinsic::FIFOPeek
          | subcode::PureIntrinsic::ValueValid
          | subcode::PureIntrinsic::ModuleTriggered => {
            format!(
              "{}.{}()",
              self
                .get()
                .get_operand_value(0)
                .unwrap()
                .to_string(self.get().sys),
              intrinsic
            )
          }
        },
        _ => panic!("Expecting Opcode::PureIntrinsic, but got {:?}", self.expr.get_opcode()),
      }
    )
  }
}
