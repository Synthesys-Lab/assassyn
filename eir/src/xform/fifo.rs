use crate::{
  builder::SysBuilder,
  ir::{node::*, visitor::Visitor, Opcode},
};

struct GatherFIFOPush(Vec<BaseNode>);

impl Visitor<()> for GatherFIFOPush {
  fn visit_block(&mut self, block: &BlockRef<'_>) -> Option<()> {
    for elem in block.iter() {
      self.dispatch(block.sys, &elem, vec![]);
    }
    ().into()
  }

  fn visit_expr(&mut self, expr: &ExprRef<'_>) -> Option<()> {
    match expr.get_opcode() {
      Opcode::FIFOPush => self.0.push(expr.upcast()),
      _ => {}
    }
    ().into()
  }
}

/// This module aims at rewriting the FIFOs.
pub(super) fn rewrite_fifos(_: &mut SysBuilder) {}
