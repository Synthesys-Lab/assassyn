use std::collections::{HashMap, HashSet};

use crate::{
  builder::SysBuilder,
  ir::{
    node::{BaseNode, ExprRef, IsElement, ModuleRef},
    visitor::Visitor,
    Opcode, Operand,
  },
};

use super::utils::select_1h;

/// Gather is a data structure that gathers multiple conditional values into a single value.
/// Typically used by FIFO and Array writes.
pub(super) struct Gather {
  pub(super) bits: usize,
  // The condition of the gather
  pub(super) condition: Vec<String>,
  // The value of the gather
  pub(super) value: Vec<String>,
}

impl Gather {
  /// Create a new Gather with the given condition and value.
  pub(super) fn new(cond: String, value: String, bits: usize) -> Gather {
    Gather {
      bits,
      condition: vec![cond],
      value: vec![value],
    }
  }

  pub(super) fn and(&self, cond: &str, join: &str) -> String {
    if self.is_conditional() {
      let gather_cond = self
        .condition
        .iter()
        .map(|x| format!("({})", x))
        .collect::<Vec<_>>()
        .join(join);
      format!("({}) && ({})", cond, gather_cond)
    } else {
      cond.into()
    }
  }

  pub(super) fn select_1h(&self) -> String {
    if self.is_conditional() {
      select_1h(
        self
          .value
          .iter()
          .zip(self.condition.iter())
          .map(|(value, cond)| (cond.clone(), value.clone()))
          .collect::<Vec<_>>()
          .into_iter(),
        self.bits,
      )
    } else {
      self.value.first().unwrap().clone()
    }
  }

  /// If this gather is conditional.
  pub(super) fn is_conditional(&self) -> bool {
    assert!(!self.condition.is_empty());
    !self.condition.first().unwrap().is_empty()
  }

  /// Push a new conditional value into the gather.
  pub(super) fn push(&mut self, cond: String, value: String, bits: usize) {
    assert!(self.is_conditional());
    assert_eq!(self.bits, bits);
    self.condition.push(cond);
    self.value.push(value);
  }
}

pub(super) struct ExternalUsage {
  module_use_external_expr: HashMap<BaseNode, HashSet<BaseNode>>,
  expr_externally_used: HashMap<BaseNode, HashSet<BaseNode>>,
}

impl ExternalUsage {
  pub(super) fn is_externally_used(&self, expr: &ExprRef<'_>) -> bool {
    if let Some(used) = self
      .expr_externally_used
      .get(&expr.get_block().get_module())
    {
      used.contains(&expr.upcast())
    } else {
      false
    }
  }

  pub(super) fn out_bounds(&self, module: &ModuleRef<'_>) -> Option<&HashSet<BaseNode>> {
    self.expr_externally_used.get(&module.upcast())
  }

  pub(super) fn in_bounds(&self, module: &ModuleRef<'_>) -> Option<&HashSet<BaseNode>> {
    self.module_use_external_expr.get(&module.upcast())
  }
}

impl Visitor<()> for ExternalUsage {
  fn visit_expr(&mut self, expr: ExprRef<'_>) -> Option<()> {
    let m = expr.get_block().get_module();
    let externals = expr
      .users()
      .iter()
      .filter_map(|x| {
        let ext = x
          .as_ref::<Operand>(expr.sys)
          .unwrap()
          .get_expr()
          .get_block()
          .get_module();
        if ext != m {
          Some(ext)
        } else {
          None
        }
      })
      .collect::<HashSet<_>>();

    if !expr.get_opcode().is_valued() || matches!(expr.get_opcode(), Opcode::Bind) {
      return None;
    }

    if !externals.is_empty() {
      self
        .expr_externally_used
        .entry(expr.get_block().get_module())
        .or_default();
      self
        .expr_externally_used
        .get_mut(&expr.get_block().get_module())
        .unwrap()
        .insert(expr.upcast());

      for elem in externals {
        self.module_use_external_expr.entry(elem).or_default();
        self
          .module_use_external_expr
          .get_mut(&elem)
          .unwrap()
          .insert(expr.upcast());
      }
    }
    None
  }
}

/// Gather all expressions used by external modules and sort them by the modules to which the usage
/// belong.
pub(super) fn gather_exprs_externally_used(sys: &SysBuilder) -> ExternalUsage {
  let mut res = ExternalUsage {
    module_use_external_expr: HashMap::new(),
    expr_externally_used: HashMap::new(),
  };
  res.enter(sys);
  res
}
