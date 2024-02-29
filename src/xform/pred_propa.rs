use crate::{
  builder::system::SysBuilder,
  expr::{Expr, Opcode},
  reference::{IsElement, Parented},
  Reference,
};

fn deepest_operand(expr: &Expr, sys: &SysBuilder) -> Option<(usize, Reference)> {
  if let Some((depth, parent)) = expr
    .operand_iter()
    .take({
      if expr.get_opcode() == Opcode::Predicate {
        1
      } else {
        expr.get_num_operands()
      }
    })
    .filter(|x| match x {
      Reference::Expr(_) => true,
      _ => false,
    })
    .map(|x| x.as_ref::<Expr>(sys).unwrap())
    .fold(None, |acc: Option<(usize, Reference)>, x| {
      if let Some((depth, parent)) = acc {
        if x.get_depth() > depth {
          Some((x.get_depth(), x.get_parent()))
        } else {
          Some((depth, parent))
        }
      } else {
        Some((x.get_depth(), x.get_parent()))
      }
    })
  {
    if depth > expr.get_depth() {
      return Some((depth, parent));
    }
  }
  None
}

fn analyze_expr_block<'a>(
  sys: &SysBuilder,
  iter: impl Iterator<Item = &'a Box<Expr>>,
) -> Option<(Reference, Reference)> {
  for expr in iter {
    if let Some((_, parent)) = deepest_operand(expr, sys) {
      return Some((expr.upcast(), parent));
    }
    if expr.get_opcode() == Opcode::Predicate {
      let body = expr.operand_iter().skip(1).filter_map(|x| match x {
        Reference::Expr(_) => Some(x.as_ref::<Expr>(sys).unwrap()),
        _ => None,
      });
      if let Some((_, parent)) = analyze_expr_block(sys, body) {
        return Some((expr.upcast(), parent));
      }
    }
  }
  None
}

/// Analyze the propagate predication.
/// # Returns
/// * The expression to be moved into the new predication block. If None is returned, no propagation
/// is found.
fn analyze_propagatable(sys: &mut SysBuilder) -> Option<(Reference, Reference)> {
  for module in sys.module_iter() {
    if let Some(res) = analyze_expr_block(sys, module.expr_iter(sys)) {
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
  if let Some((src, dst)) = analyze_propagatable(sys) {
    let dst_expr = dst.as_ref::<Expr>(sys).unwrap();
    assert!(dst_expr.get_opcode() == Opcode::Predicate);
    let orig_parent = src.as_ref::<Expr>(sys).unwrap().get_parent();
    sys.erase_from_parent(&src);
    match orig_parent {
      Reference::Expr(_) => {}
      Reference::Module(_) => {}
      _ => {
        panic!("unexpected parent type");
      }
    }
  } else {
    println!("no propagatable predications found");
  }
}
