use crate::data::{ArrayRead, DataType, Typed};

use super::module::Module;

use super::context::{cur_ctx_mut, IsElement, Parented, Reference};
use super::port::Input;

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
  operands: Vec<Reference>,
  pred: Option<Reference>, // The predication for this expression
}

impl Expr {

  pub fn dtype(&self) -> &DataType {
    &self.dtype
  }

}

impl Parented for Expr {

  fn parent(&self) -> Option<Reference> {
    self.parent.clone()
  }

}

impl Typed for Expr {

  fn dtype(&self) -> &DataType {
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

pub trait Arithmetic<'a, 'b, T: Typed + Parented + IsElement<'a>> {
  fn add(&self, other: &Box<T>, pred: Option<&Box<T>>) -> &'b Box<Expr>;
  fn mul(&self, other: &Box<T>, pred: Option<&Box<T>>) -> &'b Box<Expr>;
}

macro_rules! binary_op {
  ($func: ident, $opcode: expr) => {
    fn $func(&self, other: &Box<T>, pred: Option<&Box<T>>) -> &'b Box<Expr> {
      // FIXME(@were): We should not strictly check this here. O.w. we cannot do a + 1
      //               (where 1 has no parent)
      // if self.parent() != other.parent() {
      //   panic!("{:?} & {:?} are not in the same module!",
      //          self.as_super(), other.as_ref().as_super());
      // }
      let res = Expr {
        key: 0,
        parent: self.parent().clone(),
        dtype: self.dtype().clone(),
        opcode: $opcode,
        operands: vec![self.as_super(), other.as_ref().as_super()],
        pred: pred.map(|x| x.as_super()),
      };
      let res = cur_ctx_mut().insert(res);
      if let Some(parent) = &self.parent() {
        cur_ctx_mut().get_mut::<Module>(parent).unwrap().push(res.clone());
      } else {
        eprintln!("[WARN] No parent for {:?}", res);
      }
      res.as_ref::<Expr>().unwrap()
    }
  };
}

impl <'a, 'b, T: Typed + Parented + IsElement<'a>> Arithmetic<'a, 'b, T> for Input {
  binary_op!(add, Opcode::Add);
  binary_op!(mul, Opcode::Mul);
}

impl <'a, 'b, T: Typed + Parented + IsElement<'a>> Arithmetic<'a, 'b, T> for Expr {
  binary_op!(add, Opcode::Add);
  binary_op!(mul, Opcode::Mul);
}

impl <'a, 'b, T: Typed + Parented + IsElement<'a>> Arithmetic<'a, 'b, T> for ArrayRead {
  binary_op!(add, Opcode::Add);
  binary_op!(mul, Opcode::Mul);
}

