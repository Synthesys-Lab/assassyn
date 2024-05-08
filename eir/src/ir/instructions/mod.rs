use super::expr::Opcode;
use super::node::ExprRef;

pub mod gep;
pub mod load;

pub trait AsExpr<'a>: Sized {
  fn downcast(expr: ExprRef<'a>) -> Result<Self, String>;
}

macro_rules! register_opcode {

  (emit_impl $elem:ident) => {

    impl<'a> AsExpr<'a> for $elem<'a> {
      fn downcast(expr: ExprRef<'a>) -> Result<Self, String> {
        if expr.get_opcode() == Opcode::$elem {
          Ok($elem { expr })
        } else {
          Err(format!(
            "Expecting Opcode::{}, but got {:?}",
            stringify!($elem),
            expr.get_opcode()
          ))
        }
      }
    }

    pub struct $elem<'a> {
      expr: ExprRef<'a>,
    }
  };

  (emit_impl $elem:ident, $($rest:ident),* ) => {
    register_opcode!(emit_impl $elem);
    register_opcode!(emit_impl $($rest),* );
  };


  ($($all:ident),* $(,)?) => {
    register_opcode!(emit_impl $($all),* );
  };

  () => {};
}

register_opcode!(GetElementPtr, Load);
