use super::node::ExprRef;

pub mod gep;

pub trait AsExpr<'a>: Sized {
  fn downcast(expr: ExprRef<'a>) -> Result<Self, String>;
}
