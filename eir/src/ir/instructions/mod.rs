use super::expr::Opcode;
use super::node::ExprRef;
use super::{ir_printer::IRPrinter, visitor::Visitor};

pub mod arith;
pub mod call;
pub mod data;
pub mod fifo;
pub mod gep;
pub mod slice;

pub trait AsExpr<'a>: Sized {
  fn downcast(expr: ExprRef<'a>) -> Result<Self, String>;
}

macro_rules! register_opcode {

  ($($operator:ident $( { $subcode:ident } )? ),* $(,)?) => {
    $(
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

      impl ToString for $operator<'_> {
        fn to_string(&self) -> String {
          IRPrinter::new(false).visit_expr(self.expr.clone()).unwrap()
        }
      }
    )*
  };

  () => {};
}

register_opcode!(
  GetElementPtr,
  Load,
  Store,
  Bind,
  AsyncCall,
  FIFOPush,
  FIFOPop,
  FIFOField { field },
  Binary { binop },
  Unary { uop },
  Compare { cmp },
  Slice,
);
