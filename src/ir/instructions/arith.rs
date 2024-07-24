use std::fmt::{Display, Formatter};

use crate::ir::{expr::subcode, node::BaseNode, DataType, Opcode, Typed};

use super::{Binary, Cast, Compare, Select, Select1Hot, Unary};

impl Binary<'_> {
  pub fn get_opcode(&self) -> subcode::Binary {
    match self.expr.get_opcode() {
      Opcode::Binary { binop } => binop,
      _ => panic!(
        "Expecting Opcode::Binary, but got {:?}",
        self.expr.get_opcode()
      ),
    }
  }
}

impl Display for Binary<'_> {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(
      f,
      "{} = {} {} {}",
      self.expr.get_name(),
      self.a().to_string(self.get().sys),
      self.get_opcode(),
      self.b().to_string(self.get().sys)
    )
  }
}

impl Unary<'_> {
  pub fn get_opcode(&self) -> subcode::Unary {
    match self.expr.get_opcode() {
      Opcode::Unary { uop } => uop,
      _ => panic!(
        "Expecting Opcode::Unary, but got {:?}",
        self.expr.get_opcode()
      ),
    }
  }
}

impl Display for Unary<'_> {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(
      f,
      "{} = {}{}",
      self.expr.get_name(),
      self.get_opcode(),
      self.x().to_string(self.get().sys)
    )
  }
}

impl Compare<'_> {
  pub fn get_opcode(&self) -> subcode::Compare {
    match self.expr.get_opcode() {
      Opcode::Compare { cmp } => cmp,
      _ => panic!(
        "Expecting Opcode::Compare, but got {:?}",
        self.expr.get_opcode()
      ),
    }
  }
}

impl Display for Compare<'_> {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(
      f,
      "{} = {} {} {}",
      self.expr.get_name(),
      self.a().to_string(self.get().sys),
      self.get_opcode(),
      self.b().to_string(self.get().sys)
    )
  }
}

impl Cast<'_> {
  pub fn get_opcode(&self) -> subcode::Cast {
    match self.expr.get_opcode() {
      Opcode::Cast { cast } => cast,
      _ => panic!(
        "Expecting Opcode::Case, but got {:?}",
        self.expr.get_opcode()
      ),
    }
  }

  pub fn dest_type(&self) -> DataType {
    self.expr.dtype()
  }

  pub fn src_type(&self) -> DataType {
    self.x().get_dtype(self.get().sys).unwrap()
  }
}

impl Display for Cast<'_> {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(
      f,
      "{} = {} {}({})",
      self.expr.get_name(),
      self.get_opcode(),
      self.dest_type(),
      self.x().to_string(self.get().sys)
    )
  }
}

impl Display for Select<'_> {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(
      f,
      "{} = select {} ? {} : {}",
      self.expr.get_name(),
      self.cond().to_string(self.get().sys),
      self.true_value().to_string(self.get().sys),
      self.false_value().to_string(self.get().sys)
    )
  }
}

impl Select1Hot<'_> {
  pub fn value_iter(&self) -> impl Iterator<Item = BaseNode> + '_ {
    self.expr.operand_iter().skip(1).map(|x| *x.get_value())
  }
}

impl Display for Select1Hot<'_> {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    let values = self
      .value_iter()
      .map(|x| x.to_string(self.expr.sys))
      .collect::<Vec<_>>()
      .join(", ");
    write!(
      f,
      "{} = select1hot {} ({})",
      self.expr.get_name(),
      self.cond().to_string(self.get().sys),
      values
    )
  }
}
