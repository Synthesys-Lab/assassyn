use std::collections::HashMap;

use crate::{
  builder::system::SysBuilder,
  expr::Expr,
  ir::block::Block,
  reference::IsElement,
  Reference,
};

fn analyze_depth(sys: &SysBuilder) -> HashMap<Reference, usize> {
  let mut depth_map = HashMap::new();
  for module in sys.module_iter() {
    fn dfs<'a>(
      sys: &SysBuilder,
      iter: impl Iterator<Item = &'a Reference>,
      depth: usize,
      depth_map: &mut HashMap<Reference, usize>,
    ) {
      for expr in iter {
        depth_map.insert(expr.clone(), depth);
        if let Reference::Block(_) = expr {
          let block_body = expr.as_ref::<Block>(sys).unwrap().iter();
          dfs(sys, block_body, depth + 1, depth_map);
        }
      }
    }
    dfs(sys, module.get_body(sys).unwrap().iter(), 0, &mut depth_map);
  }
  depth_map
}

fn deepest_operand(
  expr: &Expr,
  sys: &SysBuilder,
  depth_map: &HashMap<Reference, usize>,
) -> Option<(usize, Reference)> {
  if let Some((depth, parent)) = expr
    .operand_iter()
    .filter(|x| match x {
      Reference::Expr(_) => true,
      _ => false,
    })
    .fold(None, |acc: Option<(usize, Reference)>, x| {
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
  iter: impl Iterator<Item = &'a Reference>,
  depth: &HashMap<Reference, usize>,
) -> Option<(Reference, Reference)> {
  for elem in iter {
    match elem {
      Reference::Expr(_) => {
        let expr = elem.as_ref::<Expr>(sys).unwrap();
        if let Some((_, parent)) = deepest_operand(expr, sys, depth) {
          return Some((expr.upcast(), parent));
        }
      }
      Reference::Block(_) => {
        let block = elem.as_ref::<Block>(sys).unwrap();
        if let Some(cond) = block.get_pred() {
          let expr = cond.as_ref::<Expr>(sys).unwrap();
          if let Some((_, parent)) = deepest_operand(expr, sys, depth) {
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
  depth: &HashMap<Reference, usize>,
) -> Option<(Reference, Reference)> {
  for module in sys.module_iter() {
    if let Some(res) = analyze_expr_block(sys, module.iter(sys), depth) {
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
    sys.get_mut::<Expr>(&src).unwrap().move_to_new_parent(dst, None);
  }
}
