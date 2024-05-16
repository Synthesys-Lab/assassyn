use std::collections::{HashMap, HashSet};

use crate::{
  builder::{PortInfo, SysBuilder},
  ir::{
    instructions::{Bind, FIFOPush},
    ir_printer::IRPrinter,
    node::{BaseNode, ExprRef, IsElement},
    visitor::Visitor,
    DataType, Module,
  },
};

struct GatherBinds {
  // Key: The module calleee.
  // Value: The binds to this module.
  binds: HashMap<BaseNode, HashSet<BaseNode>>,
}

impl Visitor<()> for GatherBinds {
  fn visit_expr(&mut self, expr: &ExprRef<'_>) -> Option<()> {
    let value = expr.upcast();
    let expr = expr.clone();
    if let Ok(bind) = expr.as_sub::<Bind>() {
      eprintln!("bind: {}", bind.to_string());
      let callee = bind.get_callee();
      if !self.binds.contains_key(&callee) {
        self.binds.insert(callee, HashSet::new());
      }
      self.binds.get_mut(&callee).unwrap().insert(value);
    }
    None
  }
}

fn find_module_with_multi_callers(sys: &SysBuilder) -> HashMap<BaseNode, HashSet<BaseNode>> {
  let mut gather_binds = GatherBinds {
    binds: HashMap::new(),
  };
  for m in sys.module_iter() {
    eprintln!("@module: {}", m.get_name());
    gather_binds.visit_module(&m);
  }
  gather_binds.binds.retain(|_, v| v.len() > 1);
  return gather_binds.binds;
}

pub fn inject_arbiter(sys: &mut SysBuilder) {
  let module_with_multi_caller = find_module_with_multi_callers(sys);
  for (callee, callers) in module_with_multi_caller {
    let module_name = callee.as_ref::<Module>(sys).unwrap().get_name().to_string();
    let mut ports = Vec::new();
    for (i, caller) in callers.iter().enumerate() {
      let bind = caller.as_expr::<Bind>(sys).unwrap();
      bind
        .arg_iter()
        .filter(|x| !x.is_unknown())
        .enumerate()
        .for_each(|(j, arg)| {
          let fifo_push = arg.as_expr::<FIFOPush>(sys).unwrap();
          ports.push(PortInfo::new(
            &format!("{}.caller{}.arg{}", module_name, i, j),
            fifo_push.get_value().get_dtype(sys).unwrap(),
          ));
        });
    }
    let nvec = ports.len();
    let (last_grant, grant_scalar_ty) = {
      let dtype = DataType::int_ty(nvec);
      let one = sys.get_const_int(dtype.clone(), 1);
      (
        sys.create_array(dtype.clone(), "last_grant", 1, Some(vec![one])),
        dtype,
      )
    };
    let arbiter = sys.create_module("arbiter", ports);
    eprintln!(
      "{}",
      IRPrinter::new(false)
        .dispatch(sys, &arbiter, vec![])
        .unwrap()
    );
    sys.set_current_module(arbiter);
    sys.set_current_block_wait_until();
  }
}
