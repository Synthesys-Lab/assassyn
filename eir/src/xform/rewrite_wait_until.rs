use crate::{
  builder::SysBuilder,
  ir::{node::IsElement, Block, BlockKind, Module},
};

pub(super) fn rewrite_wait_until(sys: &mut SysBuilder) {
  let mut to_rewrite = Vec::new();
  for module in sys.module_iter() {
    match module.get_name() {
      "driver" | "testbench" => {} // Both driver and testbench are unconditionally executed, skip!
      _ => {
        let body = module.get_body();
        match body.get_kind() {
          BlockKind::None => {
            if module.get_num_inputs() == 0 {
              eprintln!(
                "Warning: module {} has no inputs, but is neither the driver nor testbench.",
                module.get_name()
              );
              continue;
            }
            // All the unconditional root block should be rewritten.
            to_rewrite.push(module.upcast());
          }
          BlockKind::WaitUntil(_) => {} // We respect existing wait_until blocks.
          _ => {
            unreachable!()
          }
        }
      }
    }
  }
  for module in to_rewrite.into_iter() {
    let (ports, body) = {
      let module = module.as_ref::<Module>(sys).unwrap();
      (
        module
          .port_iter()
          .map(|port| port.upcast())
          .collect::<Vec<_>>(),
        module.get_body().upcast(),
      )
    };
    sys.set_current_module(module);
    sys.set_current_block(body);
    sys.set_current_block_wait_until();
    if let BlockKind::WaitUntil(cond) = sys.get_current_block().unwrap().get_kind() {
      let cond = cond.clone();
      sys.set_current_block(cond.clone());
      let valids = ports
        .into_iter()
        .map(|port| sys.create_fifo_valid(port))
        .collect::<Vec<_>>();
      let valid = valids
        .into_iter()
        .fold(None, |acc, v| match acc {
          None => Some(v),
          Some(acc) => Some(sys.create_bitwise_and(None, acc, v)),
        })
        .unwrap();
      // FIXME: Use the real condition.
      cond.as_mut::<Block>(sys).unwrap().set_value(valid);
    }
  }
}
