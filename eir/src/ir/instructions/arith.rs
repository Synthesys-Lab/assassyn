use crate::ir::{expr::subcode, DataType, Opcode, Typed};

use super::{Binary, Cast, Compare, Select, Unary};

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

impl ToString for Binary<'_> {
  fn to_string(&self) -> String {
    format!(
      "{} = {} {} {}",
      self.expr.get_name(),
      self.a().to_string(self.get().sys),
      self.get_opcode().to_string(),
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

impl ToString for Unary<'_> {
  fn to_string(&self) -> String {
    format!(
      "{} = {}{}",
      self.expr.get_name(),
      self.get_opcode().to_string(),
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

impl ToString for Compare<'_> {
  fn to_string(&self) -> String {
    format!(
      "{} = {} {} {}",
      self.expr.get_name(),
      self.a().to_string(self.get().sys),
      self.get_opcode().to_string(),
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

impl ToString for Cast<'_> {
  fn to_string(&self) -> String {
    format!(
      "{} = {} {}({})",
      self.expr.get_name(),
      self.get_opcode().to_string(),
      self.dest_type().to_string(),
      self.x().to_string(self.get().sys)
    )
  }
}

impl ToString for Select<'_> {
  fn to_string(&self) -> String {
    format!(
      "{} = select {} ? {} : {}",
      self.expr.get_name(),
      self.cond().to_string(self.get().sys),
      self.true_value().to_string(self.get().sys),
      self.false_value().to_string(self.get().sys)
    )
  }
}
