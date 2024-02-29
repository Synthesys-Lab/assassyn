use crate::{reference::Parented, data::{DataType, Typed}};

use super::{reference::Reference, system::SysBuilder};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Opcode {
  // Side-effect operations
  Load,
  Store,
  // Binary operations
  Add,
  Sub,
  Mul,
  // Comparison operations
  IGT,
  ILT,
  IGE,
  ILE,
  // Eventual operations
  Trigger,
  SpinTrigger,
}

impl Opcode {

  pub fn is_binary(&self) -> bool {
    match self {
      Opcode::Add | Opcode::Mul | Opcode::Sub | Opcode::IGT | Opcode::ILT | Opcode::IGE | Opcode::ILE => true,
      _ => false,
    }
  }

}


impl ToString for Opcode {

  fn to_string(&self) -> String {
    match self {
      Opcode::Add => "+".into(),
      Opcode::Mul => "*".into(),
      Opcode::Sub => "-".into(),
      Opcode::IGT => ">".into(),
      Opcode::ILT => "<".into(),
      Opcode::IGE => ">=".into(),
      Opcode::ILE => "<=".into(),
      Opcode::Load => "load".into(),
      Opcode::Store => "store".into(),
      Opcode::Trigger => "trigger".into(),
      Opcode::SpinTrigger => "wait_until".into(),
    }
  }
}

pub struct Expr {
  pub(super) key: usize,
  parent: Reference,
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

  pub fn get_opcode(&self) -> Opcode {
    self.opcode.clone()
  }

  pub fn get_operand(&self, i: usize) -> Option<&Reference> {
    self.operands.get(i)
  }

  pub fn operand_iter(&self) -> impl Iterator<Item = &Reference> {
    self.operands.iter()
  }

  pub fn get_pred(&self) -> &Option<Reference> {
    &self.pred
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
    let res = if self.opcode.is_binary() {
      return format!(
        "_{} = {} {} {}",
        self.key,
        self.operands[0].to_string(sys),
        mnem,
        self.operands[1].to_string(sys)
      )
    } else {
      match self.opcode {
        Opcode::Load => {
          format!(
            "_{} = {}[{}];",
            self.key,
            self.operands[0].to_string(sys),
            self.operands[1].to_string(sys)
          )
        }
        Opcode::Store => {
          format!(
            "{}[{}] = {}",
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
          res.push_str("])");
          res
        }
        Opcode::SpinTrigger => {
          format!("{} {}", mnem, self.operands[0].to_string(sys))
        }
        _ => {
          panic!("Unimplemented opcode: {:?}", self.opcode);
        }
      }
    };
    if let Some(pred) = &self.pred {
      format!("{} when {}", res, pred.to_string(sys))
    } else {
      res
    }
  }
}

impl Parented for Expr {

  fn parent(&self) -> Reference {
    self.parent.clone()
  }

}

