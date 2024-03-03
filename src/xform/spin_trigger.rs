use crate::{
  builder::system::SysBuilder,
  expr::{Expr, Opcode},
  ir::{ir_printer::IRPrinter, visitor::Visitor},
  node::{BlockRef, ExprRef, IsElement, ModuleRef, },
  BaseNode,
};

struct SpinTriggerFinder {}

impl SpinTriggerFinder {
  pub fn new() -> Self {
    Self {}
  }
}

impl Visitor<BaseNode> for SpinTriggerFinder {
  fn visit_expr(&mut self, expr: &ExprRef<'_>) -> Option<BaseNode> {
    if let Opcode::SpinTrigger = expr.get_opcode() {
      return Some(expr.upcast());
    }
    None
  }
  fn visit_block(&mut self, block: &BlockRef<'_>) -> Option<BaseNode> {
    for elem in block.iter() {
      if let Some(x) = self.dispatch(block.sys, elem, vec![]) {
        return x.into();
      }
    }
    None
  }
  fn visit_module(&mut self, module: &ModuleRef<'_>) -> Option<BaseNode> {
    self.visit_block(&module.get_body())
  }
}

pub fn rewrite_spin_triggers(sys: &mut SysBuilder) {
  let mut finder = SpinTriggerFinder::new();

  if let Some(spin_trigger) = finder.enter(sys) {
    println!(
      "{}",
      IRPrinter::new(sys).visit_expr(&spin_trigger.as_ref::<Expr>(sys).unwrap()).unwrap()
    );
  } else {
    println!("No spin triggers found");
  }
}
