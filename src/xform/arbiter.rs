use std::collections::{HashMap, HashSet};

use crate::{
  builder::{system::ModuleKind, PortInfo, SysBuilder},
  ir::{
    instructions::{Bind, FIFOPush},
    module,
    node::{BaseNode, ExprRef, IsElement},
    visitor::Visitor,
    DataType, Expr, Module, FIFO,
  },
};

struct GatherBinds {
  // Key: The module calleee.
  // Value: The binds to this module.
  binds: HashMap<BaseNode, HashSet<BaseNode>>,
}

impl Visitor<()> for GatherBinds {
  fn visit_expr(&mut self, expr: ExprRef<'_>) -> Option<()> {
    let value = expr.upcast();
    let expr = expr.clone();
    if let Ok(bind) = expr.as_sub::<Bind>() {
      let callee = bind.callee().upcast();
      self.binds.entry(callee).or_default();
      self.binds.get_mut(&callee).unwrap().insert(value);
    }
    None
  }
}

fn bits_to_int(sys: &mut SysBuilder, x: &BaseNode) -> BaseNode {
  let dtype = x.get_dtype(sys).unwrap();
  let bits = dtype.get_bits();
  sys.create_bitcast(*x, DataType::int_ty(bits))
}

fn find_module_with_multi_callers(sys: &SysBuilder) -> HashMap<BaseNode, HashSet<BaseNode>> {
  let mut gather_binds = GatherBinds {
    binds: HashMap::new(),
  };
  // Only FIFO port upstream modules are considered to be arbitrated.
  for m in sys.module_iter(ModuleKind::Module) {
    gather_binds.visit_module(m);
  }
  gather_binds.binds.retain(|_, v| v.len() > 1);
  gather_binds.binds
}

pub fn inject_arbiter(sys: &mut SysBuilder) {
  // Find all the modules with more than one caller.
  let module_with_multi_caller = find_module_with_multi_callers(sys);
  // Inject an arbiter for each module like this.
  // Callee is the module with multiple callers, and callers are the multiple callers.
  for (callee, callers) in module_with_multi_caller.iter() {
    let res = {
      let module = callee.as_ref::<Module>(sys).unwrap();
      module.get_attrs().contains(&module::Attribute::NoArbiter)
        || module.get_attrs().contains(&module::Attribute::OptNone)
    };
    if res {
      continue;
    }
    let module_name = callee.as_ref::<Module>(sys).unwrap().get_name().to_string();
    {
      let mut callee_mut = callee.as_mut::<Module>(sys).unwrap();
      callee_mut.add_attr(module::Attribute::OptNone);
    }
    let mut ports = Vec::new();
    // First, flatten all the ports from the callers.
    // Something like [caller0.arg0, caller0.arg1, ...], [caller1.arg0, caller1.arg1, ...]
    // will be merged into one flatten list
    // [caller0.arg0, caller0.arg1, ..., caller1.arg0, caller1.arg1, ...]
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
            fifo_push.value().get_dtype(sys).unwrap(),
          ));
        });
    }
    // Use this flattened list to create a new arbiter module.
    let arbiter = sys.create_module("arbiter", ports);
    let mut arbiter_mut = arbiter.as_mut::<Module>(sys).unwrap();
    arbiter_mut.add_attr(module::Attribute::NoArbiter);
    sys.set_current_module(arbiter);
    let restore_block = sys.get_current_block().unwrap().upcast();
    // For each cluster of callers' input arguments, create a valid signal.
    let mut sub_valids = Vec::new();
    for (i, caller) in callers.iter().enumerate() {
      let bind = caller.as_expr::<Bind>(sys).unwrap();
      let n_args = bind.arg_iter().filter(|x| !x.is_unknown()).count();
      let valids = {
        let arbiter = arbiter.as_ref::<Module>(sys).unwrap();
        let ports = (0..n_args)
          .map(|x| {
            arbiter
              .get_fifo(&format!("{}.caller{}.arg{}", module_name, i, x))
              .unwrap()
              .upcast()
          })
          .collect::<Vec<_>>();
        ports
          .into_iter()
          .map(|x| sys.create_fifo_valid(x))
          .collect::<Vec<_>>()
      };
      let mut valid_runner = valids[0];
      for valid in valids.iter().skip(1) {
        valid_runner = sys.create_bitwise_and(valid_runner, *valid);
      }
      sub_valids.push(valid_runner);
    }
    let mut valid = sub_valids[0];
    for sub_valid in sub_valids.iter().skip(1) {
      valid = sys.create_bitwise_or(valid, *sub_valid);
    }
    sys.create_wait_until(valid);
    let mut valid_hot = sub_valids[callers.len() - 1];
    for sub_valid in sub_valids.iter().rev().skip(1) {
      valid_hot = sys.create_concat(valid_hot, *sub_valid);
    }

    let (last_grant_reg, grant_scalar_ty, grant_hot_ty) = {
      let dtype = DataType::int_ty(callers.len());
      let bits = usize::BITS - callers.len().next_power_of_two().leading_zeros();
      let one = sys.get_const_int(dtype.clone(), 1);
      (
        sys.create_array(dtype.clone(), "last_grant", 1, Some(vec![one]), vec![]),
        DataType::int_ty(bits as usize),
        dtype,
      )
    };

    let zero = sys.get_const_int(DataType::int_ty(1), 0);
    let last_grant_1h = sys.create_array_read(last_grant_reg, zero);

    // low_mask = ((last_grant_1h - 1) << 1) + 1
    let one = sys.get_const_int(grant_hot_ty.clone(), 1);
    let lo = sys.create_sub(last_grant_1h, one);
    let lo = sys.create_shl(lo, one);
    let lo = sys.create_add(lo, one);
    // high_mask = ~low_mask
    let hi = sys.create_flip(lo);
    // low_valid = valid_hot & low_mask
    let lo_valid = sys.create_bitwise_and(lo, valid_hot);
    let signed_lo_valid = bits_to_int(sys, &lo_valid);
    let lo_valid_neg = sys.create_neg(signed_lo_valid);
    let lo_grant = sys.create_bitwise_and(lo_valid, lo_valid_neg);
    // high_valid = valid_hot & high_mask
    let hi_valid = sys.create_bitwise_and(hi, valid_hot);
    let signed_hi_valid = bits_to_int(sys, &hi_valid);
    let hi_valid_neg = sys.create_neg(signed_hi_valid);
    let hi_grant = sys.create_bitwise_and(hi_valid, hi_valid_neg);
    let zero = sys.get_const_int(hi_grant.get_dtype(sys).unwrap(), 0);
    let hi_nez = sys.create_neq(hi_grant, zero);
    // grant = high_valid != 0 ? high_valid : low_valid
    let grant = sys.create_select(hi_nez, hi_grant, lo_grant);

    for (i, caller) in callers.iter().enumerate() {
      let i_1h = sys.get_const_int(grant_hot_ty.clone(), 1 << i);
      let ii = sys.get_const_int(grant_scalar_ty.clone(), i as u64);
      let grant_to = sys.create_slice(grant, ii, ii);
      let block = sys.create_conditional_block(grant_to);
      sys.set_current_block(block);
      sys.create_array_write(last_grant_reg, zero, i_1h);
      let new_bind = sys.get_init_bind(*callee);
      let module_ports = {
        let module = callee.as_ref::<Module>(sys).unwrap();
        module.fifo_iter().map(|x| x.upcast()).collect::<Vec<_>>()
      };
      for (j, port) in module_ports.iter().enumerate() {
        let key = port.as_ref::<FIFO>(sys).unwrap().get_name().clone();

        // Push to new arbiter
        let bind = caller.as_expr::<Bind>(sys).unwrap();
        let callee_idx = bind.get_num_args();
        let push = bind.get_arg(&key).unwrap_or_else(|| {
          panic!("{} not found in bind {}", key, bind);
        });
        {
          let arbiter = arbiter.as_ref::<Module>(sys).unwrap();
          let arbiter_port = {
            let port = arbiter.get_fifo(&format!("{}.caller{}.arg{}", module_name, i, j));
            port.unwrap().upcast()
          };
          // Caller calls the arbiter
          let mut push_mut = push.as_mut::<Expr>(sys).unwrap();
          push_mut.set_operand(0, arbiter_port);
          // Arbiter calls origin
          let pop = sys.create_fifo_pop(arbiter_port);
          sys.bind_arg(new_bind, key, pop);
        }

        // Set to new callee
        let mut caller = caller.as_mut::<Expr>(sys).unwrap();
        caller.set_operand(callee_idx, arbiter);
      }
      sys.create_async_call(new_bind);
      sys.set_current_block(restore_block);
    }
  }
}
