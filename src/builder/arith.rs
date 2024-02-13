use crate::data::{DataType, Input, Typed};

use super::module::Module;

use super::context::{cur_ctx_mut, IsElement, Parented, Reference};

enum Opcode {
  Add,
  Mul,
}

impl ToString for Opcode {

  fn to_string(&self) -> String {
    match self {
      Opcode::Add => "+".into(),
      Opcode::Mul => "*".into(),
    }
  }

}

pub struct Expr {
  pub(crate) key: usize,
  pub(crate) parent: Option<Reference>,
  dtype: DataType,
  opcode: Opcode,
  operands: Vec<Reference>
}

impl Expr {

  pub fn dtype(&self) -> &DataType {
    &self.dtype
  }

}

impl ToString for Expr {

  fn to_string(&self) -> String {
    match self.opcode {
      Opcode::Add | Opcode::Mul => {
        format!("let _{} = {} {} {};",
                self.key,
                self.operands[0].to_string(),
                self.opcode.to_string(),
                self.operands[1].to_string())
      }
    }
  }

}

pub trait Arithmetic<T> {
  fn add(&self, other: &Box<T>) -> Reference;
  fn mul(&self, other: &Box<T>) -> Reference;
}

macro_rules! binary_op {
  ($func: ident, $opcode: expr) => {
    fn $func(&self, other: &Box<T>) -> Reference {
      let res = Expr {
        key: 0,
        parent: self.parent.clone(),
        dtype: self.dtype().clone(),
        opcode: $opcode,
        operands: vec![self.as_ref(), other.as_ref().as_ref()]
      };
      let res = cur_ctx_mut().insert(res);
      if let Some(parent) = &self.parent {
        cur_ctx_mut().get_mut::<Module>(parent).unwrap().push(res.clone());
      } else {
        eprintln!("[WARN] No parent for {:?}", res);
      }
      res
    }
  };
}

impl <'a, T: Typed + Parented + IsElement<'a>> Arithmetic<T> for Input {
  binary_op!(add, Opcode::Add);
  binary_op!(mul, Opcode::Mul);
}

impl <'a, T: Typed + Parented + IsElement<'a>> Arithmetic<T> for Expr {
  binary_op!(add, Opcode::Add);
  binary_op!(mul, Opcode::Mul);
}

