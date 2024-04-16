/// This module checks if two modules are homomorphic.
use std::collections::{HashMap, HashSet};

use crate::{
  builder::SysBuilder,
  ir::{data::Typed, node::*, visitor::Visitor, *},
};

pub(super) struct ModuleEqual {
  lhs_param: Vec<BaseNode>,
  rhs_param: Vec<BaseNode>,
  rhs: BaseNode,
  eq_cache: HashSet<(BaseNode, BaseNode)>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum NodeCmp {
  Eq,
  Ne(String),
}

impl ModuleEqual {
  fn shallow_equal(&mut self, lhs: &BaseNode, rhs: &BaseNode) -> NodeCmp {
    if self.eq_cache.contains(&(lhs.clone(), rhs.clone())) {
      NodeCmp::Eq
    } else {
      if lhs.get_kind() != rhs.get_kind() {
        return NodeCmp::Ne(format!(
          "Kind not equal: {:?} {:?}",
          lhs.get_kind(),
          rhs.get_kind()
        ));
      }
      if lhs == rhs {
        return NodeCmp::Eq;
      }
      let lhs_pos = self.lhs_param.iter().position(|x| x == lhs);
      let rhs_pos = self.rhs_param.iter().position(|x| x == rhs);
      if let (Some(lhs_pos), Some(rhs_pos)) = (lhs_pos, rhs_pos) {
        if lhs_pos == rhs_pos {
          self.eq_cache.insert((lhs.clone(), rhs.clone()));
          return NodeCmp::Eq;
        }
      }
      NodeCmp::Ne(format!("Shallow not equal: {:?} {:?}", lhs, rhs))
    }
  }

  fn deep_equal(&mut self, sys: &SysBuilder, lhs: &BaseNode, rhs: &BaseNode) -> bool {
    let restore = self.rhs;
    self.rhs = rhs.clone();
    let result = self.dispatch(sys, lhs, vec![NodeKind::Module]).unwrap();
    self.rhs = restore;
    result
  }

  fn expand_param(
    &self,
    sys: &SysBuilder,
    param: &Vec<BaseNode>,
    module: BaseNode,
  ) -> Vec<BaseNode> {
    let mut res = Vec::new();
    param.iter().for_each(|x| {
      if let Ok(bind) = x.as_ref::<Bind>(sys) {
        res.push(bind.get_callee());
        res.extend(bind.to_args());
      } else {
        res.push(x.clone());
      }
    });
    res.push(module);
    res
  }
}

impl Visitor<bool> for ModuleEqual {
  fn visit_module(&mut self, lhs: &ModuleRef<'_>) -> Option<bool> {
    let rhs = self.rhs.as_ref::<Module>(lhs.sys).unwrap();
    let lhs_builder = lhs.get_builder_func_ptr();
    let rhs_builder = rhs.get_builder_func_ptr();
    if let (Some(lhs_builder), Some(rhs_builder)) = (lhs_builder, rhs_builder) {
      // If two modules are not built by the same function, just skip it!
      if lhs_builder != rhs_builder {
        return Some(false);
      }
      // Get all the parameterization of these two modules.
      // Say we have two modules m1 and m2
      // m1 is built by foo(a, b), and m2 is built by foo(c, d).
      // Then for m1, [a, b] are parameterized, and for m2, [c, d] are parameterized.
      // Then we say m1 and m2 are homomorphic by checking each component in this module.
      //
      // For component a each operand should either by identical or placed in the same position
      // of the parameter list.
      let lhs_param = lhs.get_parameterizable();
      let rhs_param = rhs.get_parameterizable();
      if let (Some(lhs_param), Some(rhs_param)) = (lhs_param, rhs_param) {
        if lhs_param.len() != rhs_param.len() {
          return Some(false);
        } else {
          self.lhs_param = self.expand_param(lhs.sys, lhs_param, lhs.upcast());
          self.rhs_param = self.expand_param(rhs.sys, rhs_param, rhs.upcast());
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
      let rhs_expr = rhs.get().get(i).unwrap();
      if !self.deep_equal(lhs.sys, &lhs_expr, &rhs_expr) {
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
      let lhs_op = lhs.get_operand(i).unwrap().get_value().clone();
      let rhs_op = rhs.get_operand(i).unwrap().get_value().clone();
      match (lhs_op.get_kind(), rhs_op.get_kind()) {
        (NodeKind::Module, NodeKind::Module) | (NodeKind::Expr, NodeKind::Expr) => {
          let res = self.shallow_equal(&lhs_op, &rhs_op);
          if res != NodeCmp::Eq {
            return Some(false);
          }
        }
        (NodeKind::Block, NodeKind::Block)
        | (NodeKind::FIFO, NodeKind::FIFO)
        | (NodeKind::ArrayPtr, NodeKind::ArrayPtr)
        | (NodeKind::IntImm, NodeKind::IntImm)
        | (NodeKind::StrImm, NodeKind::StrImm) => {
          if !self.deep_equal(lhs.sys, &lhs_op, &rhs_op) {
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
    self.eq_cache.insert((lhs.upcast(), rhs.upcast()));
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
    self.eq_cache.insert((array_ptr.upcast(), rhs.upcast()));
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
    self.eq_cache.insert((array.upcast(), rhs.upcast()));
    return Some(true);
  }
}

fn module_equal(lhs: &ModuleRef<'_>, rhs: &ModuleRef<'_>) -> Option<HashSet<(BaseNode, BaseNode)>> {
  let mut visitor = ModuleEqual {
    rhs: rhs.upcast(),
    lhs_param: vec![],
    rhs_param: vec![],
    eq_cache: HashSet::new(),
  };
  if visitor.visit_module(&lhs).unwrap() {
    Some(visitor.eq_cache)
  } else {
    None
  }
}

pub(in crate::sim) struct CommonModuleCache {
  dsu: Vec<usize>, // Use a DSU to store the master of each module
  union_size: Vec<usize>,
  node_to_idx: HashMap<BaseNode, usize>,
  modules: Vec<BaseNode>,
  placeholder: Vec<Option<Vec<BaseNode>>>,
}

impl CommonModuleCache {
  pub(in crate::sim) fn new(sys: &SysBuilder) -> Self {
    let node_to_idx = sys
      .module_iter()
      .enumerate()
      .map(|(idx, node)| (node.upcast(), idx))
      .collect::<HashMap<BaseNode, usize>>();
    let modules = sys
      .module_iter()
      .map(|x| x.upcast())
      .collect::<Vec<BaseNode>>();
    let cnt = node_to_idx.len();
    let dsu = (0..cnt).collect::<Vec<usize>>();
    let union_size = vec![1; cnt];
    let mut res = CommonModuleCache {
      node_to_idx,
      modules,
      dsu,
      union_size,
      placeholder: vec![None; cnt],
    };

    for i in 0..res.modules.len() {
      for j in 0..i {
        let master_j = res.get_master(&res.modules[j].clone());
        let master_j = *res.node_to_idx.get(&master_j).unwrap();
        let lhs = &res.modules[i].as_ref::<Module>(sys).unwrap();
        let rhs = &res.modules[master_j].as_ref::<Module>(sys).unwrap();
        // eprintln!("[Common Module] Compare {} and {}", lhs.get_name(), rhs.get_name());
        if let Some(eq) = module_equal(lhs, rhs) {
          // eprintln!(
          //   "[Common Module] Module {} and {} are equal",
          //   lhs.get_name(),
          //   rhs.get_name()
          // );
          // print!("{{ ");
          // for (k, v) in eq.iter() {
          //   print!("_{}: _{}, ", k.get_key(), v.get_key());
          // }
          // println!("}}");
          // println!("{}", crate::ir::ir_printer::IRPrinter::new().visit_module(lhs).unwrap());
          let mut eq = eq.into_iter().collect::<Vec<_>>();
          eq.sort_by_key(|x| x.1.get_key());
          let (secondary_ph, maseter_ph): (Vec<_>, Vec<_>) = eq.into_iter().unzip();

          res.placeholder[i] = Some(secondary_ph);
          if res.placeholder[master_j].is_none() {
            res.placeholder[master_j] = Some(maseter_ph);
          } else {
            assert_eq!(res.placeholder[master_j].as_ref().unwrap(), &maseter_ph);
          }

          res.dsu[i] = master_j;
          res.union_size[master_j] += res.union_size[i];
          break;
        }
      }
    }

    res
  }

  pub(in crate::sim) fn get_master(&mut self, node: &BaseNode) -> BaseNode {
    let idx = self.node_to_idx.get(node).unwrap();
    let mut runner = self.dsu[*idx];
    let mut to_merge = vec![];
    while runner != self.dsu[runner] {
      to_merge.push(runner);
      runner = self.dsu[runner];
    }
    for idx in to_merge.into_iter() {
      self.dsu[idx] = runner;
    }
    self.modules[runner].clone()
  }

  pub(in crate::sim) fn get_size(&mut self, node: &BaseNode) -> usize {
    let idx = self.get_master(node);
    self.union_size[*self.node_to_idx.get(&idx).unwrap()]
  }
}
