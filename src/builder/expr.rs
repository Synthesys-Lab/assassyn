use crate::data::{DataType, Typed};

use super::{context::Reference, system::SysBuilder};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Opcode {
  Load,
  Store,
  Add,
  Mul,
  Trigger,
  SpinTrigger,
}

impl ToString for Opcode {
  fn to_string(&self) -> String {
    match self {
      Opcode::Add => "+".into(),
      Opcode::Mul => "*".into(),
      Opcode::Load => "load".into(),
      Opcode::Store => "store".into(),
      Opcode::Trigger => "trigger".into(),
      Opcode::SpinTrigger => "wait_until".into(),
    }
  }
}

pub struct Expr {
  pub(crate) key: usize,
  pub(crate) parent: Reference,
  dtype: DataType,
  opcode: Opcode,
  operands: Vec<Reference>,
  pred: Option<Reference>, // The predication for this expression
}

impl Expr {
  pub(crate) fn new(
    dtype: DataType,
    opcode: Opcode,
    operands: Vec<Reference>,
    parent: Reference,
    pred: Option<Reference>,
  ) -> Self {
    Self {
      key: 0,
      parent,
      dtype,
      opcode,
      operands,
      pred,
    }
  }

  pub fn dtype(&self) -> &DataType {
    &self.dtype
  }
}

impl Typed for Expr {
  fn dtype(&self) -> &DataType {
    &self.dtype
  }
}

impl Expr {
  pub fn to_string(&self, sys: &SysBuilder) -> String {
    let mnem = self.opcode.to_string();
    match self.opcode {
      Opcode::Add | Opcode::Mul => {
        format!(
          "let _{} = {} {} {};",
          self.key,
          self.operands[0].to_string(sys),
          mnem,
          self.operands[1].to_string(sys)
        )
      }
      Opcode::Load => {
        format!(
          "let _{} = {}[{}];",
          self.key,
          self.operands[0].to_string(sys),
          self.operands[1].to_string(sys)
        )
      }
      Opcode::Store => {
        format!(
          "{}[{}] = {};",
          self.operands[0].to_string(sys),
          self.operands[1].to_string(sys),
          self.operands[2].to_string(sys)
        )
      }
      Opcode::Trigger => {
        let mut res = format!("self.{}(\"{}\", [", mnem, self.operands[0].to_string(sys));
        for op in self.operands.iter().skip(1) {
          res.push_str(op.to_string(sys).as_str());
          res.push_str(", ");
        }
        res.push_str("]);");
        res
      }
      Opcode::SpinTrigger => {
        format!("{} {};", mnem, self.operands[0].to_string(sys))
      }
    }
  }
}
