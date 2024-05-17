use std::collections::HashSet;

use crate::{
  builder::SysBuilder,
  ir::{node::*, Operand, FIFO, *},
  xform::callback::instructions::FIFOPush,
};

pub(super) fn gather_single_callback_fifos(sys: &SysBuilder) -> Vec<(BaseNode, BaseNode)> {
  let mut res = Vec::new();
  for m in sys.module_iter() {
    for port in m.port_iter() {
      if port.scalar_ty().is_module() {
        let unique_modules = port
          .users()
          .iter()
          .map(|user| user.as_ref::<Operand>(sys).unwrap())
          .filter_map(|operand| {
            let expr = operand.get_user().as_ref::<Expr>(sys).unwrap();
            match expr.get_opcode() {
              Opcode::FIFOPush => {
                let push = expr.as_sub::<FIFOPush>().unwrap();
                let module = push.value();
                assert!(module.get_kind() == NodeKind::Module);
                module.into()
              }
              Opcode::FIFOPop => None,
              x => panic!("Unexpected opcode {:?}", x),
            }
          })
          .collect::<HashSet<_>>();
        if unique_modules.len() == 1 {
          res.push((port.upcast(), unique_modules.into_iter().next().unwrap()));
        }
      }
    }
  }
  res
}

pub(super) fn rewrite_single_callbacks(
  sys: &mut SysBuilder,
  to_rewrite: Vec<(BaseNode, BaseNode)>,
) {
  for (port, module) in to_rewrite {
    let (pushes, pops): (Vec<_>, Vec<_>) = {
      let port_fifo = port.as_ref::<FIFO>(sys).unwrap();
      port_fifo
        .users()
        .iter()
        .map(|x| x.clone())
        .partition(|operand| {
          let operand = operand.as_ref::<Operand>(sys).unwrap();
          let expr = operand.get_user().as_ref::<Expr>(sys).unwrap();
          match expr.get_opcode() {
            Opcode::FIFOPush => true,
            Opcode::FIFOPop => false,
            x => panic!("Unexpected opcode {:?}", x),
          }
        })
    };

    // For a pair of caller and callee (and the callee with a callback), the compiler first checks
    // if the value fed to the callee's callback is the only possible value. If so, the compiler
    // will replace all the usage of this callback argument with this given module.
    //
    // 0: module callee(..., callback: module) {
    //   1: x = callback.pop()
    //   2: call x(...)
    // }
    //
    // module caller(...) {
    //   3: x = callee.callback.push(callback_module)
    //   4: call callee(..., x, ...)
    // }
    //
    // 1. Replace the call in `2`, with the `callback_module`.
    // 2. Remove the callback channel pop in `1`.
    // 3. Remove the argument `x`, push handle, in `4`.
    // 4. Remove the callback channel push in `3`.
    // 5. Remove the argument in the argument list of `callee` in `0`.

    // Replace all the pops with the designated module
    for operand in pops {
      let operand = operand.as_ref::<Operand>(sys).unwrap();
      let pop_expr = operand.get_user().as_ref::<Expr>(sys).unwrap();
      let pop_expr = pop_expr.upcast();
      // Doing 1.
      sys.replace_all_uses_with(pop_expr, module.clone());
      // Doing 2.
      pop_expr.as_mut::<Expr>(sys).unwrap().erase_from_parent();
    }
    // Remove all the pushes
    for operand in pushes {
      let (call_operand, _) = {
        let operand = operand.as_ref::<Operand>(sys).unwrap();
        let push_expr = operand.get_user().as_ref::<Expr>(sys).unwrap();
        // Check this push only has one user, which is a call that uses this push.
        let mut iter = push_expr.users().iter();
        let res = iter.next().unwrap().clone();
        assert!(iter.next().is_none());
        let call_operand = res.as_ref::<Operand>(sys).unwrap();
        (call_operand.upcast(), call_operand.get_user().clone())
      };
      // Doing 3.
      call_operand.as_mut::<Operand>(sys).unwrap().erase_self();
      // Doing 4.
      let (mut push_expr, _) = {
        let operand = operand.as_ref::<Operand>(sys).unwrap();
        let expr = operand.get_user().clone().as_ref::<Expr>(sys).unwrap();
        let block = expr.get_parent();
        let block = block.as_ref::<Block>(sys).unwrap();
        let caller = block.get_module().upcast();
        (
          operand.get_user().clone().as_mut::<Expr>(sys).unwrap(),
          caller,
        )
      };
      push_expr.erase_from_parent();
    }
    // Doing 5.
    let (mut parent_module, idx) = {
      let port_fifo = port.as_ref::<FIFO>(sys).unwrap();
      let idx = port_fifo.idx();
      (port_fifo.get_parent().as_mut::<Module>(sys).unwrap(), idx)
    };
    parent_module.remove_port(idx);
  }
}

/// This module aims at rewriting the FIFOs.
pub(super) fn rewrite_fifos(sys: &mut SysBuilder) {
  let to_rewrite = gather_single_callback_fifos(sys);
  rewrite_single_callbacks(sys, to_rewrite);
}
