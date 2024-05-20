use crate::{builder::system::SysBuilder, ir::node::*, ir::*};

use super::block::Block;

pub trait Visitor<T> {
  fn visit_module(&mut self, module: ModuleRef<'_>) -> Option<T> {
    for input in module.port_iter() {
      self.visit_input(input);
    }
    self.visit_block(module.get_body());
    None
  }

  fn visit_input(&mut self, _: FIFORef<'_>) -> Option<T> {
    None
  }

  fn visit_expr(&mut self, expr: ExprRef<'_>) -> Option<T> {
    for elem in expr.operand_iter() {
      if let Some(x) = self.visit_operand(elem) {
        return x.into();
      }
    }
    None
  }

  fn visit_array(&mut self, _: ArrayRef<'_>) -> Option<T> {
    None
  }

  fn visit_int_imm(&mut self, _: IntImmRef<'_>) -> Option<T> {
    None
  }

  fn visit_block(&mut self, block: BlockRef<'_>) -> Option<T> {
    match block.get_kind() {
      BlockKind::Condition(cond) | BlockKind::WaitUntil(cond) => {
        if let Some(x) = self.dispatch(block.sys, cond, vec![]) {
          return x.into();
        }
      }
      _ => {}
    }
    for elem in block.iter() {
      if let Some(x) = self.dispatch(block.sys, elem, vec![]) {
        return Some(x);
      }
    }
    None
  }

  fn enter(&mut self, sys: &SysBuilder) -> Option<T> {
    for elem in sys.module_iter() {
      let res = self.visit_module(elem);
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn visit_string_imm(&mut self, _: StrImmRef<'_>) -> Option<T> {
    None
  }

  fn visit_operand(&mut self, _: OperandRef<'_>) -> Option<T> {
    None
  }

  fn dispatch(&mut self, sys: &SysBuilder, node: &BaseNode, non_recur: Vec<NodeKind>) -> Option<T> {
    if non_recur.contains(&node.get_kind()) {
      return None;
    }
    match node.get_kind() {
      NodeKind::Expr => self.visit_expr(node.as_ref::<Expr>(sys).unwrap()),
      NodeKind::Block => self.visit_block(node.as_ref::<Block>(sys).unwrap()),
      NodeKind::Module => self.visit_module(node.as_ref::<Module>(sys).unwrap()),
      NodeKind::FIFO => self.visit_input(node.as_ref::<FIFO>(sys).unwrap()),
      NodeKind::Array => self.visit_array(node.as_ref::<Array>(sys).unwrap()),
      NodeKind::IntImm => self.visit_int_imm(node.as_ref::<IntImm>(sys).unwrap()),
      NodeKind::StrImm => self.visit_string_imm(node.as_ref::<StrImm>(sys).unwrap()),
      NodeKind::Operand => self.visit_operand(node.as_ref::<Operand>(sys).unwrap()),
      NodeKind::Unknown => {
        panic!("Unknown node type")
      }
    }
  }
}
