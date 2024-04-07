use std::collections::{HashMap, HashSet};

use crate::{
  builder::SysBuilder,
  ir::{data::Typed, node::*, visitor::Visitor, *},
};

struct ModuleEqual {
  lhs_param: Vec<BaseNode>,
  rhs_param: Vec<BaseNode>,
  rhs: BaseNode,
  eq_cache: HashSet<(BaseNode, BaseNode)>,
}

impl ModuleEqual {
  fn shallow_equal(&mut self, lhs: &BaseNode, rhs: &BaseNode) -> bool {
    if self.eq_cache.contains(&(lhs.clone(), rhs.clone())) {
      true
    } else {
      if lhs.get_kind() != rhs.get_kind() {
        return false;
      }
      if lhs == rhs {
        return true;
      }
      let lhs_pos = self.lhs_param.iter().position(|x| x == lhs);
      let rhs_pos = self.rhs_param.iter().position(|x| x == rhs);
      if let (Some(lhs_pos), Some(rhs_pos)) = (lhs_pos, rhs_pos) {
        if lhs_pos == rhs_pos {
          self.eq_cache.insert((lhs.clone(), rhs.clone()));
          return true;
        }
      }
      false
    }
  }

  fn deep_equal(&mut self, sys: &SysBuilder, lhs: &BaseNode, rhs: &BaseNode) -> bool {
    let restore = self.rhs;
    self.rhs = rhs.clone();
    let result = self.dispatch(sys, lhs, vec![NodeKind::Module]);
    self.rhs = restore;
    result.unwrap()
  }
}

impl Visitor<bool> for ModuleEqual {
  fn visit_module(&mut self, lhs: &ModuleRef<'_>) -> Option<bool> {
    let rhs = self.rhs.as_ref::<Module>(lhs.sys).unwrap();
    let lhs_builder = lhs.get_builder_func_ptr();
    let rhs_builder = rhs.get_builder_func_ptr();
    if let (Some(lhs_builder), Some(rhs_builder)) = (lhs_builder, rhs_builder) {
      if lhs_builder != rhs_builder {
        return Some(false);
      }
      let lhs_param = lhs.get_parameterizable();
      let rhs_param = rhs.get_parameterizable();
      if let (Some(lhs_param), Some(rhs_param)) = (lhs_param, rhs_param) {
        if lhs_param.len() != rhs_param.len() {
          return Some(false);
        } else {
          self.lhs_param = lhs_param.clone();
          self.rhs_param = rhs_param.clone();
        }
      }
      let lhs_body = lhs.get_body().upcast();
      let rhs_body = rhs.get_body().upcast();
      return self.deep_equal(lhs.sys, &lhs_body, &rhs_body).into();
    } else {
      return Some(false);
    }
  }

  fn visit_block(&mut self, lhs: &BlockRef<'_>) -> Option<bool> {
    let rhs = self.rhs.as_ref::<Block>(lhs.sys).unwrap();
    if lhs.get_num_exprs() != rhs.get_num_exprs() {
      return Some(false);
    }
    for i in 0..lhs.get_num_exprs() {
      let lhs_expr = lhs.get().get(i).unwrap();
      let rhs_expr = rhs.get().get(i).unwrap().clone();
      self.rhs = rhs_expr;
      if let Some(false) = self.visit_expr(&lhs_expr.as_ref::<Expr>(lhs.sys).unwrap()) {
        return Some(false);
      }
    }
    return Some(true);
  }

  fn visit_expr(&mut self, lhs: &ExprRef<'_>) -> Option<bool> {
    let rhs = self.rhs.as_ref::<Expr>(lhs.sys).unwrap();
    if lhs.get_opcode() != rhs.get_opcode() {
      return Some(false);
    }
    if lhs.get_num_operands() != rhs.get_num_operands() {
      return Some(false);
    }
    for i in 0..lhs.get_num_operands() {
      let lhs_op = lhs.get_operand(i).unwrap();
      let rhs_op = rhs.get_operand(i).unwrap();
      match (lhs_op.get_kind(), rhs_op.get_kind()) {
        (NodeKind::Module, NodeKind::Module) => {
          if !self.shallow_equal(lhs_op, rhs_op) {
            return Some(false);
          }
        }
        (NodeKind::Block, NodeKind::Block)
        | (NodeKind::FIFO, NodeKind::FIFO)
        | (NodeKind::ArrayPtr, NodeKind::ArrayPtr)
        | (NodeKind::IntImm, NodeKind::IntImm)
        | (NodeKind::StrImm, NodeKind::StrImm) => {
          if !self.deep_equal(lhs.sys, lhs_op, rhs_op) {
            return Some(false);
          }
        }
        _ => return Some(false),
      }
    }
    self.eq_cache.insert((lhs.upcast(), rhs.upcast()));
    return Some(true);
  }

  fn visit_input(&mut self, lhs: &FIFORef<'_>) -> Option<bool> {
    let rhs = self.rhs.as_ref::<FIFO>(lhs.sys).unwrap();
    return (lhs.idx() == rhs.idx()).into();
  }

  fn visit_int_imm(&mut self, int_imm: &IntImmRef<'_>) -> Option<bool> {
    let rhs = self.rhs.as_ref::<IntImm>(int_imm.sys).unwrap();
    return (int_imm.get_value() == rhs.get_value() && int_imm.dtype() == rhs.dtype()).into();
  }

  fn visit_string_imm(&mut self, str_imm: &StrImmRef<'_>) -> Option<bool> {
    let rhs = self.rhs.as_ref::<StrImm>(str_imm.sys).unwrap();
    return (str_imm.get_value() == rhs.get_value()).into();
  }

  fn visit_handle(&mut self, array_ptr: &ArrayPtrRef<'_>) -> Option<bool> {
    let rhs = self.rhs.as_ref::<ArrayPtr>(array_ptr.sys).unwrap();
    if !self.deep_equal(array_ptr.sys, array_ptr.get_array(), rhs.get_array()) {
      return Some(false);
    }
    if !self.deep_equal(array_ptr.sys, array_ptr.get_idx(), rhs.get_idx()) {
      return Some(false);
    }
    return Some(true);
  }

  fn visit_array(&mut self, array: &ArrayRef<'_>) -> Option<bool> {
    let rhs = self.rhs.as_ref::<Array>(array.sys).unwrap();
    if array.scalar_ty() != rhs.scalar_ty() {
      return Some(false);
    }
    if array.get_size() != rhs.get_size() {
      return Some(false);
    }
    return Some(true);
  }
}

pub(super) fn module_equal(lhs: &ModuleRef<'_>, rhs: &ModuleRef<'_>) -> bool {
  let mut visitor = ModuleEqual {
    rhs: rhs.upcast(),
    lhs_param: vec![],
    rhs_param: vec![],
    eq_cache: HashSet::new(),
  };
  visitor.visit_module(&lhs).unwrap()
}

pub(super) struct CommonModuleCache {
  master_cache: HashMap<BaseNode, BaseNode>,
}

impl CommonModuleCache {
  pub(super) fn new() -> Self {
    CommonModuleCache {
      master_cache: HashMap::new(),
    }
  }

  pub(super) fn get_master(&self, node: &BaseNode) -> Option<&BaseNode> {
    self.master_cache.get(node)
  }
}
