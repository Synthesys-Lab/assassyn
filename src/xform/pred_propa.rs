use std::collections::HashMap;

use crate::{
  builder::system::SysBuilder,
  expr::Expr,
  ir::block::Block,
  node::{ExprRef, IsElement, NodeKind},
  BaseNode,
};

fn analyze_depth(sys: &SysBuilder) -> HashMap<BaseNode, usize> {
  let mut depth_map = HashMap::new();
  for module in sys.module_iter() {
    fn dfs<'a>(
      sys: &SysBuilder,
      iter: impl Iterator<Item = &'a BaseNode>,
      depth: usize,
      depth_map: &mut HashMap<BaseNode, usize>,
    ) {
      for expr in iter {
        depth_map.insert(expr.clone(), depth);
        if let NodeKind::Block = expr.get_kind() {
          let block = expr.as_ref::<Block>(sys).unwrap();
          let body_iter = block.iter();
          dfs(sys, body_iter, depth + 1, depth_map);
        }
      }
    }
    dfs(sys, module.get_body().iter(), 0, &mut depth_map);
  }
  depth_map
}

fn deepest_operand<'a>(
  expr: &ExprRef<'a>,
  sys: &SysBuilder,
  depth_map: &HashMap<BaseNode, usize>,
) -> Option<(usize, BaseNode)> {
  if let Some((depth, parent)) = expr
    .operand_iter()
    .filter(|x| match x.get_kind() {
      NodeKind::Expr => true,
      _ => false,
    })
    .fold(None, |acc: Option<(usize, BaseNode)>, x| {
      let new_depth = *depth_map.get(&x).unwrap();
      if let Some((depth, parent)) = acc {
        if new_depth > depth {
          Some((new_depth, x.get_parent(sys).unwrap()))
        } else {
          Some((depth, parent))
        }
      } else {
        Some((new_depth, x.get_parent(sys).unwrap()))
      }
    })
  {
    if depth > *depth_map.get(&expr.upcast()).unwrap() {
      return Some((depth, parent));
    }
  }
  None
}

fn analyze_expr_block<'a>(
  sys: &SysBuilder,
  iter: impl Iterator<Item = &'a BaseNode>,
  depth: &HashMap<BaseNode, usize>,
) -> Option<(BaseNode, BaseNode)> {
  for elem in iter {
    match elem.get_kind() {
      NodeKind::Expr => {
        let expr = elem.as_ref::<Expr>(sys).unwrap();
        if let Some((_, parent)) = deepest_operand(&expr, sys, depth) {
          return Some((expr.upcast(), parent));
        }
      }
      NodeKind::Block => {
        let block = elem.as_ref::<Block>(sys).unwrap();
        if let Some(cond) = block.get_pred() {
          let expr = cond.as_ref::<Expr>(sys).unwrap();
          if let Some((_, parent)) = deepest_operand(&expr, sys, depth) {
            return Some((elem.clone(), parent));
          }
        }
        let body = block.iter();
        if let Some(res) = analyze_expr_block(sys, body, depth) {
          return Some(res);
        }
      }
      _ => {
        panic!("unexpected reference type");
      }
    }
  }
  None
}

/// Analyze the propagate predication.
/// # Returns
/// * The expression to be moved into the new predication block. If None is returned, no propagation
/// is found.
fn analyze_propagatable(
  sys: &mut SysBuilder,
  depth: &HashMap<BaseNode, usize>,
) -> Option<(BaseNode, BaseNode)> {
  for module in sys.module_iter() {
    if let Some(res) = analyze_expr_block(sys, module.get_body().iter(), depth) {
      return res.into();
    }
  }
  None
}

/// Propagate predications.
/// # Example
///
/// Before the transformation:
/// ```
///   if cond {
///     _1 = a + b;
///   }
///   _2 = _1 + c;
/// ```
///
/// After the transformation:
/// ```
///   if cond {
///     _1 = a + b;
///     _2 = _1 + c;
///   }
/// ```
pub fn propagate_predications(sys: &mut SysBuilder) {
  while let Some((src, dst)) = {
    let depth = analyze_depth(sys);
    analyze_propagatable(sys, &depth)
  } {
    sys
      .get_mut::<Expr>(&src)
      .unwrap()
      .move_to_new_parent(dst, None);
  }
}
