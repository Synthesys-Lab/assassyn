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

struct CommonExpr {
  ip: (BaseNode, usize),
  duplica: Vec<BaseNode>,
}

fn idx_of(sys: &SysBuilder, x: &BaseNode) -> Option<usize> {
  if let Ok(expr) = x.as_ref::<Expr>(sys) {
    Some(expr.idx())
  } else if let Ok(block) = x.as_ref::<Block>(sys) {
    block.idx()
  } else {
    None
  }
}

fn climb_up(sys: &SysBuilder, x: &BaseNode) -> (BaseNode, Option<usize>) {
  let parent = x.get_parent(sys).unwrap();
  (parent, idx_of(sys, x))
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
        let mut parents = exprs.iter().map(|x| climb_up(sys, x)).collect::<Vec<_>>();
        // Hoist all parents to the same depth
        while let Some(x) = {
          let ref_depth = da.get_depth(&parents[0].0);
          if let Some(diff) = parents
            .iter_mut()
            .filter(|x| {
              let depth = da.get_depth(&x.0);
              depth != ref_depth
            })
            .next()
          {
            if da.get_depth(&diff.0) < ref_depth {
              Some(&mut parents[0])
            } else {
              Some(diff)
            }
          } else {
            None
          }
        } {
          *x = climb_up(sys, &x.0);
        }
        // Hoist all the parents to the same node
        while parents.iter().any(|x| x.0.ne(&parents[0].0)) {
          parents.iter_mut().for_each(|x| *x = climb_up(sys, &x.0));
        }
        // TODO(@were): Support non-block parents
        if let Some((block, idx)) = {
          if let Ok(block) = parents[0].0.as_ref::<Block>(sys) {
            let idx = parents
              .iter()
              .min_by(|x, y| x.1.unwrap().cmp(&y.1.unwrap()))
              .unwrap()
              .1;
            Some((block, idx.unwrap()))
          } else {
            None
          }
        } {
          res.push(CommonExpr {
            ip: (block.upcast(), idx),
            duplica: exprs,
          });
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
    // match elem {
    //   CommonExpr::Master { master, duplica } => {
    //     for dup in duplica {
    //       sys.replace_all_uses_with(dup.clone(), master);
    //       let mut dup_mut = dup.as_mut::<Expr>(sys).unwrap();
    //       dup_mut.erase_from_parent();
    //     }
    //   }
    // }
  }
}
