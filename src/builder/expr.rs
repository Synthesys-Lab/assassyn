use crate::data::{DataType, Typed};

use super::context::{cur_ctx_mut, IsElement, Reference};
use super::port::Input;

pub(crate) enum Opcode {
  Load,
  Store,
  Add,
  Mul,
}

impl ToString for Opcode {

  fn to_string(&self) -> String {
    match self {
      Opcode::Add => "+".into(),
      Opcode::Mul => "*".into(),
      _ => panic!("Not implemented!"),
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

  pub(crate) fn new(dtype: DataType, opcode: Opcode, operands: Vec<Reference>,
                    parent: Reference, pred: Option<Reference>) -> Self {
    Self {
      key: 0,
      parent,
      dtype,
      opcode,
      operands,
      pred: None,
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
      _ => panic!("Not implemented!"),
    }
  }

}

pub trait Arithmetic<'a, 'b, 'c, T: Typed + IsElement<'a>, U: Typed + IsElement<'b>> {
  fn add(&self, other: &Box<T>, pred: Option<&Box<U>>, parent: Reference) -> &'c Box<Expr>;
  fn mul(&self, other: &Box<T>, pred: Option<&Box<U>>, parent: Reference) -> &'c Box<Expr>;
}

macro_rules! binary_op {
  ($func: ident, $opcode: expr) => {
    fn $func(&self, other: &Box<T>, pred: Option<&Box<U>>, parent: Reference)
      -> &'c Box<Expr> {
      // FIXME(@were): We should not strictly check this here. O.w. we cannot do a + 1
      //               (where 1 has no parent)
      // if self.parent() != other.parent() {
      //   panic!("{:?} & {:?} are not in the same module!",
      //          self.as_super(), other.as_ref().as_super());
      // }
      let res = Expr::new(
        self.dtype().clone(),
        $opcode,
        vec![self.as_super(), other.as_ref().as_super()],
        parent,
        pred.map(|x| x.as_super()),
      );
      let res = cur_ctx_mut().insert(res);
      res.as_ref::<Expr>().unwrap()
    }
  };
}

impl <'a, 'b, 'c, T: Typed + IsElement<'a>, U: Typed + IsElement<'b>>
Arithmetic<'a, 'b, 'c, T, U> for Input {
  binary_op!(add, Opcode::Add);
  binary_op!(mul, Opcode::Mul);
}

impl <'a, 'b, 'c, T: Typed + IsElement<'a>, U: Typed + IsElement<'b>>
Arithmetic<'a, 'b, 'c, T, U> for Expr {
  binary_op!(add, Opcode::Add);
  binary_op!(mul, Opcode::Mul);
}

