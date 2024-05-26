use std::collections::HashMap;

use crate::{
  builder::SysBuilder,
  ir::{
    node::{BaseNode, BlockRef, ExprRef, IsElement, ModuleRef},
    visitor::Visitor,
    Block, BlockKind, Expr, Opcode,
  },
};

struct DepthAnalysis {
  depth: HashMap<BaseNode, usize>,
  cur: usize,
}

impl DepthAnalysis {
  fn get_depth(&self, node: &BaseNode) -> usize {
    *self.depth.get(node).unwrap()
  }
}

impl Visitor<()> for DepthAnalysis {
  fn visit_module(&mut self, module: ModuleRef<'_>) -> Option<()> {
    self.depth.insert(module.upcast(), self.cur);
    self.visit_block(module.get_body());
    None
  }
  fn visit_block(&mut self, block: BlockRef<'_>) -> Option<()> {
    self.depth.insert(block.upcast(), self.cur);
    self.cur += 1;
    if let BlockKind::WaitUntil(cond) = block.get_kind() {
      self.dispatch(block.sys, cond, vec![]);
    }
    for elem in block.iter() {
      self.dispatch(block.sys, elem, vec![]);
    }
    self.cur -= 1;
    None
  }
}

struct FindCommonSubexpression {
  common: HashMap<(Opcode, Vec<BaseNode>), Vec<BaseNode>>,
}

impl Visitor<()> for FindCommonSubexpression {
  fn visit_expr(&mut self, expr: ExprRef<'_>) -> Option<()> {
    if !expr.get_opcode().has_side_effect() {
      let key = (
        expr.get_opcode().clone(),
        expr
          .operand_iter()
          .map(|x| x.get_value().clone())
          .collect::<Vec<_>>(),
      );
      if !self.common.contains_key(&key) {
        self.common.insert(key.clone(), vec![]);
      }
      self.common.get_mut(&key).unwrap().push(expr.upcast());
    }
    None
  }
}

enum CommonExpr {
  Master {
    master: BaseNode,
    duplica: Vec<BaseNode>,
  },
}

fn find_common_subexpression(sys: &SysBuilder, da: &DepthAnalysis) -> Vec<CommonExpr> {
  let mut res = Vec::new();
  for m in sys.module_iter() {
    let mut finder = FindCommonSubexpression {
      common: HashMap::new(),
    };
    finder.visit_module(m);
    for (_, exprs) in finder.common {
      if exprs.len() != 1 {
        let mut parents = exprs
          .iter()
          .map(|x| x.get_parent(sys).unwrap())
          .collect::<Vec<_>>();
        // Hoist all parents to the same depth
        while let Some(x) = {
          let ref_depth = da.get_depth(&parents[0]);
          if let Some(diff) = parents
            .iter_mut()
            .filter(|x| {
              let depth = da.get_depth(&x);
              depth != ref_depth
            })
            .next()
          {
            if da.get_depth(diff) < ref_depth {
              Some(&mut parents[0])
            } else {
              Some(diff)
            }
          } else {
            None
          }
        } {
          *x = x.get_parent(sys).unwrap();
        }
        // Hoist all the parents to the same node
        while parents.iter().any(|x| x.ne(&parents[0])) {
          parents
            .iter_mut()
            .for_each(|x| *x = x.get_parent(sys).unwrap());
        }
        // TODO(@were): Support non-block parents
        if let Ok(block) = parents[0].as_ref::<Block>(sys) {
          let mut master_idx = None;
          for expr in exprs.iter() {
            if expr.get_parent(sys).unwrap() == block.upcast() {
              let idx = block.iter().position(|x| x.eq(expr)).unwrap();
              if master_idx.map_or(true, |x| idx < x) {
                master_idx = Some(idx);
              }
            }
          }
          if let Some(master_idx) = master_idx {
            let master = block.get().get(master_idx).unwrap().clone();
            let mut duplica = exprs.clone();
            duplica.retain(|x| x.ne(&master));
            res.push(CommonExpr::Master { master, duplica });
          }
        }
      }
    }
  }
  res
}

pub fn common_code_elimination(sys: &mut SysBuilder) {
  let mut depth = DepthAnalysis {
    depth: HashMap::new(),
    cur: 0,
  };
  depth.enter(sys);
  let ce = find_common_subexpression(sys, &depth);
  for elem in ce {
    match elem {
      CommonExpr::Master { master, duplica } => {
        for dup in duplica {
          sys.replace_all_uses_with(dup.clone(), master);
          let mut dup_mut = dup.as_mut::<Expr>(sys).unwrap();
          dup_mut.erase_from_parent();
        }
      }
    }
  }
}
