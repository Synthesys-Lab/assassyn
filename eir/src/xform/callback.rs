use std::collections::{HashMap, HashSet};

use crate::{
  builder::SysBuilder,
  ir::*,
  ir::{node::*, visitor::Visitor, Opcode},
};

struct GatherIndirectTriggers(Vec<BaseNode>);

impl Visitor<()> for GatherIndirectTriggers {
  fn visit_block(&mut self, block: &BlockRef<'_>) -> Option<()> {
    for elem in block.iter() {
      self.dispatch(block.sys, &elem, vec![]);
    }
    ().into()
  }

  fn visit_expr(&mut self, expr: &ExprRef<'_>) -> Option<()> {
    match expr.get_opcode() {
      Opcode::Trigger => {
        if expr.get_operand(0).unwrap().get_kind() == NodeKind::Expr {
          self.0.push(expr.upcast());
        }
      }
      _ => {}
    }
    ().into()
  }
}

pub fn gather_indirect_triggers(sys: &SysBuilder) -> Vec<BaseNode> {
  let mut gatherer = GatherIndirectTriggers(vec![]);
  for m in sys.module_iter() {
    gatherer.visit_module(&m);
  }
  gatherer.0
}

// pub fn gather_modules_with_callbacks(sys: &SysBuilder) -> Vec<BaseNode> {
//   for m in sys.module_iter() {
//   }
// }

struct GatherIndirectHandles(HashMap<BaseNode, HashSet<BaseNode>>);

impl Visitor<()> for GatherIndirectHandles {
  fn visit_expr(&mut self, expr: &ExprRef<'_>) -> Option<()> {
    match expr.get_opcode() {
      Opcode::FIFOPush => {
        let fifo = expr
          .get_operand(0)
          .unwrap()
          .as_ref::<FIFO>(expr.sys)
          .unwrap();
        match fifo.scalar_ty() {
          DataType::Module(_) => {
            let key = fifo.upcast();
            if !self.0.contains_key(&key) {
              self.0.insert(key, HashSet::new());
            }
            self
              .0
              .get_mut(&key)
              .unwrap()
              .insert(expr.get_operand(1).unwrap().clone());
          }
          _ => {}
        }
      }
      _ => {}
    }
    ().into()
  }
}

pub fn gather_indirect_handles(sys: &SysBuilder) -> HashMap<BaseNode, HashSet<BaseNode>> {
  let mut gather = GatherIndirectHandles(HashMap::new());
  for m in sys.module_iter() {
    gather.visit_module(&m);
  }
  let mut res = gather.0;
  let mut iterative = true;
  while iterative {
    iterative = false;
    let (mut src, mut to_remove, mut update) = (None, None, None);
    'outer: for (k, v) in res.iter() {
      for elem in v.iter() {
        if elem.get_kind() == NodeKind::Expr {
          let expr = elem.as_ref::<Expr>(sys).unwrap();
          assert!(expr.get_opcode() == Opcode::FIFOPop);
          let fifo = expr
            .get_operand(0)
            .unwrap()
            .as_ref::<FIFO>(sys)
            .unwrap()
            .upcast();
          src = Some(k.clone());
          to_remove = Some(elem.clone());
          update = Some(fifo);
          break 'outer;
        }
      }
    }
    if let (Some(src), Some(to_remove), Some(update)) = (src, to_remove, update) {
      assert!(res.contains_key(&update));
      let update = res.get(&update).unwrap().clone();
      res.get_mut(&src).unwrap().remove(&to_remove);
      let dst = res.get_mut(&src).unwrap();
      for elem in update.iter() {
        dst.insert(elem.clone());
      }
      iterative = true;
    }
  }
  res
}

/// This module aims at rewriting the FIFOs.
pub(super) fn rewrite_fifos(sys: &mut SysBuilder) {
  // All these triggers will be rewritten in handle + argument
  gather_indirect_triggers(sys);
  // Gather all the modules passed to these indirect modules
  gather_indirect_handles(sys);
}
