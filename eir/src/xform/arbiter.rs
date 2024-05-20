use std::collections::{HashMap, HashSet};

use crate::{
  builder::{PortInfo, SysBuilder},
  ir::{
    instructions::{Bind, FIFOPush},
    ir_printer::IRPrinter,
    module,
    node::{BaseNode, ExprRef, IsElement},
    visitor::Visitor,
    Block, BlockKind, DataType, Expr, Module,
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
      eprintln!("bind: {}", bind.to_string());
      let callee = bind.callee();
      if !self.binds.contains_key(&callee) {
        self.binds.insert(callee, HashSet::new());
      }
      self.binds.get_mut(&callee).unwrap().insert(value);
    }
    None
  }
}

fn bits_to_int(sys: &mut SysBuilder, x: &BaseNode) -> BaseNode {
  let dtype = x.get_dtype(sys).unwrap();
  let bits = dtype.get_bits();
  sys.create_bitcast(x.clone(), DataType::int_ty(bits))
}

fn find_module_with_multi_callers(sys: &SysBuilder) -> HashMap<BaseNode, HashSet<BaseNode>> {
  let mut gather_binds = GatherBinds {
    binds: HashMap::new(),
  };
  for m in sys.module_iter() {
    eprintln!("@module: {}", m.get_name());
    gather_binds.visit_module(m);
  }
  gather_binds.binds.retain(|_, v| v.len() > 1);
  return gather_binds.binds;
}

pub fn inject_arbiter(sys: &mut SysBuilder) {
  let module_with_multi_caller = find_module_with_multi_callers(sys);
  for (callee, callers) in module_with_multi_caller.iter() {
    if {
      let module = callee.as_ref::<Module>(sys).unwrap();
      module.get_attrs().contains(&module::Attribute::NoArbiter)
        || module.get_attrs().contains(&module::Attribute::OptNone)
    } {
      continue;
    }
    let module_name = callee.as_ref::<Module>(sys).unwrap().get_name().to_string();
    {
      let mut callee_mut = callee.as_mut::<Module>(sys).unwrap();
      callee_mut.add_attr(module::Attribute::OptNone);
    }
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
            fifo_push.value().get_dtype(sys).unwrap(),
          ));
        });
    }
    let arbiter = sys.create_module("arbiter", ports);
    let mut arbiter_mut = arbiter.as_mut::<Module>(sys).unwrap();
    arbiter_mut.add_attr(module::Attribute::NoArbiter);
    sys.set_current_module(arbiter);
    sys.set_current_block_wait_until();
    if let BlockKind::WaitUntil(cond) = sys.get_current_block().unwrap().get_kind() {
      let cond = cond.clone();
      let restore_block = sys.get_current_block().unwrap().upcast();
      sys.set_current_block(cond.clone());
      let mut idx = 0;
      let mut sub_valids = Vec::new();
      for caller in callers.iter() {
        let bind = caller.as_expr::<Bind>(sys).unwrap();
        let n_args = bind.arg_iter().filter(|x| !x.is_unknown()).count();
        let valids = (0..n_args)
          .map(|_| {
            let arbiter = arbiter.as_ref::<Module>(sys).unwrap();
            let port = arbiter.get_port(idx).unwrap().upcast();
            idx += 1;
            sys.create_fifo_valid(port)
          })
          .collect::<Vec<_>>();
        let mut valid_runner = valids[0].clone();
        for valid in valids.iter().skip(1) {
          valid_runner = sys.create_bitwise_and(valid_runner, valid.clone());
        }
        sub_valids.push(valid_runner);
      }
      let mut valid = sub_valids[0].clone();
      for sub_valid in sub_valids.iter().skip(1) {
        valid = sys.create_bitwise_or(valid, sub_valid.clone());
      }
      let mut cond_mut = cond.as_mut::<Block>(sys).unwrap();
      cond_mut.set_value(valid);
      sys.set_current_block(restore_block);
      let mut valid_hot = sub_valids[callers.len() - 1].clone();
      for sub_valid in sub_valids.iter().rev().skip(1) {
        valid_hot = sys.create_concat(valid_hot, sub_valid.clone());
      }

      let (last_grant_reg, grant_scalar_ty, grant_hot_ty) = {
        let dtype = DataType::int_ty(callers.len());
        let bits = usize::BITS - callers.len().next_power_of_two().leading_zeros();
        let one = sys.get_const_int(dtype.clone(), 1);
        (
          sys.create_array(dtype.clone(), "last_grant", 1, Some(vec![one])),
          DataType::int_ty(bits as usize),
          dtype,
        )
      };

      let zero = sys.get_const_int(DataType::int_ty(1), 0);
      let ptr = sys.create_array_ptr(last_grant_reg, zero);
      let last_grant_1h = sys.create_array_read(ptr);

      // low_mask = ((last_grant_1h - 1) << 1) + 1
      let one = sys.get_const_int(grant_hot_ty.clone(), 1);
      let lo = sys.create_sub(last_grant_1h, one);
      let lo = sys.create_shl(lo, one);
      let lo = sys.create_add(lo, one);
      // high_mask = ~low_mask
      let hi = sys.create_flip(lo);
      // low_valid = valid_hot & low_mask
      let lo_valid = sys.create_bitwise_and(lo.clone(), valid_hot.clone());
      let signed_lo_valid = bits_to_int(sys, &lo_valid);
      let lo_valid_neg = sys.create_neg(signed_lo_valid);
      let lo_grant = sys.create_bitwise_and(lo_valid, lo_valid_neg);
      // high_valid = valid_hot & high_mask
      let hi_valid = sys.create_bitwise_and(hi.clone(), valid_hot.clone());
      let signed_hi_valid = bits_to_int(sys, &hi_valid);
      let hi_valid_neg = sys.create_neg(signed_hi_valid);
      let hi_grant = sys.create_bitwise_and(hi_valid, hi_valid_neg);
      let zero = sys.get_const_int(hi_grant.get_dtype(sys).unwrap(), 0);
      let hi_nez = sys.create_neq(hi_grant.clone(), zero);
      // grant = high_valid != 0 ? high_valid : low_valid
      let grant = sys.create_select(hi_nez, hi_grant, lo_grant);

      let mut idx = 0;
      for (i, caller) in callers.iter().enumerate() {
        let i_1h = sys.get_const_int(grant_hot_ty.clone(), 1 << i);
        let i = sys.get_const_int(grant_scalar_ty.clone(), i as u64);
        let grant_to = sys.create_slice(grant, i, i);
        let block = sys.create_block(BlockKind::Condition(grant_to));
        sys.set_current_block(block);
        sys.create_array_write(ptr, i_1h);
        let bind = caller.as_expr::<Bind>(sys).unwrap();
        let n_args = bind.arg_iter().filter(|x| !x.is_unknown()).count();
        let mut new_bind = sys.get_init_bind(callee.clone());
        for i in 0..n_args {
          let key = {
            let module = callee.as_ref::<Module>(sys).unwrap();
            module.get_port(i).unwrap().get_name().clone()
          };
          let port = {
            let arbiter = arbiter.as_ref::<Module>(sys).unwrap();
            arbiter.get_port(idx).unwrap().upcast()
          };

          // Push to new arbiter
          let bind = caller.as_expr::<Bind>(sys).unwrap();
          let callee_idx = bind.get_num_args();
          let push = bind.get_arg(i).unwrap();
          let mut push_mut = push.as_mut::<Expr>(sys).unwrap();
          push_mut.set_operand(0, port);

          // Set to new callee
          let mut caller = caller.as_mut::<Expr>(sys).unwrap();
          caller.set_operand(callee_idx, arbiter);

          // Arbiter calls origin
          idx += 1;
          let pop = sys.create_fifo_pop(port);
          new_bind = sys.add_bind(new_bind, key, pop, Some(false));
        }
        sys.create_async_call(new_bind);
        sys.set_current_block(restore_block);
      }
    }
    eprintln!(
      "{}",
      IRPrinter::new(false)
        .dispatch(sys, &arbiter, vec![])
        .unwrap()
    );
  }
  eprintln!("{}", sys);
}
