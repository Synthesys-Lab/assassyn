use super::expr::Opcode;
use super::node::ExprRef;
use crate::{ir, ir::node::BaseNode};

pub mod arith;
pub mod bits;
pub mod call;
pub mod data;
pub mod fifo;
pub mod log;

pub trait AsExpr<'a>: Sized {
  fn downcast(expr: ExprRef<'a>) -> Result<Self, String>;
}

macro_rules! register_opcode {
  (@emit_sema $operator:ident => { ($method:ident, $idx:expr, BaseNode) $( ( $($rest_sema:tt)* ) )* }) => {
    impl $operator<'_> {
      pub fn $method(&self) -> BaseNode {
        self.expr.get_operand_value($idx).unwrap()
      }
    }
    register_opcode!(@emit_sema $operator => { $( ( $($rest_sema)* ) )* } );
  };

  (@emit_sema $operator:ident => { ($method:ident, $idx:expr, expr::$op:ident) $( ( $($rest_sema:tt)* ) )* }) => {
    impl $operator<'_> {
      pub fn $method(&self) -> $op<'_> {
        self.expr.get_operand_value($idx).unwrap().as_expr::<$op>(self.get().sys).unwrap()
      }
    }
    register_opcode!(@emit_sema $operator => { $( ( $($rest_sema)* ) )* } );
  };

  (@emit_sema $operator:ident => { ($method:ident, $idx:expr, node::$ty:ident) $( ( $($rest_sema:tt)* ) )* }) => {
    paste::paste! {
      impl $operator<'_> {
        pub fn $method(&self) -> ir::node::[< $ty Ref>]<'_> {
          self.expr.get_operand_value($idx).unwrap().as_ref::<ir::$ty>(self.get().sys).unwrap()
        }
      }
    }
    register_opcode!(@emit_sema $operator => { $( ( $($rest_sema)* ) )* } );
  };

  (@emit_sema $operator:ident => { }) => { };

  ($operator:ident $( { $subcode:ident } )? => { $( ( $($sema_info:tt)* ) )* }, $( $rest:tt )* ) => {
    impl<'a> AsExpr<'a> for $operator<'a> {
      fn downcast(expr: ExprRef<'a>) -> Result<Self, String> {
        if let Opcode::$operator $( { $subcode } )? = expr.get_opcode() {
          $( let _ = $subcode; )?
          Ok($operator { expr })
        } else {
          Err(format!(
            "Expecting Opcode::{}, but got {:?}",
            stringify!($elem),
            expr.get_opcode()
          ))
        }
      }
    }

    pub struct $operator<'a> {
      expr: ExprRef<'a>,
    }

    impl $operator<'_> {
      pub fn get(&self) -> &ExprRef<'_> {
        &self.expr
      }
    }

    register_opcode!(@emit_sema $operator => { $( ( $($sema_info)* ) )* });

    register_opcode!( $( $rest )* );
  };

  () => {};
}

register_opcode!(
  GetElementPtr => { (array, 0, node::Array) (index, 1, BaseNode) },
  Load => { (pointer, 0, expr::GetElementPtr) },
  Store => { (pointer, 0, expr::GetElementPtr) (value, 1, BaseNode) },
  Bind => {  },
  AsyncCall => { (bind, 0, expr::Bind) },
  FIFOPush => { (fifo, 0, node::FIFO) (value, 1, BaseNode) },
  FIFOPop => { (fifo, 0, node::FIFO) },
  FIFOField { field } => { (fifo, 0, node::FIFO) },
  Binary { binop } => { (a, 0, BaseNode) (b, 1, BaseNode) },
  Unary { uop } => { (x, 0, BaseNode) },
  Select => { (cond, 0, BaseNode) (true_value, 1, BaseNode) (false_value, 2, BaseNode) },
  Compare { cmp } => { (a, 0, BaseNode) (b, 1, BaseNode) },
  Slice => { (x, 0, BaseNode) (l_intimm, 1, node::IntImm) (r_intimm, 2, node::IntImm) },
  Concat => { (msb, 0, BaseNode) (lsb, 1, BaseNode) },
  Cast { cast } => { (x, 0, BaseNode) }, // NOTE: This "," cannot be omitted!
  Log => { },
);
