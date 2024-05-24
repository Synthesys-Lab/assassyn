use crate::{
  builder::SysBuilder,
  created_here,
  ir::{
    node::{BaseNode, IsElement, ModuleRef},
    visitor::Visitor,
    Block, BlockKind, Module,
  },
};

struct GatherModulesToRewrite {
  to_rewrite: Vec<BaseNode>,
}

impl Visitor<()> for GatherModulesToRewrite {
  fn visit_module(&mut self, module: ModuleRef<'_>) -> Option<()> {
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
            }
            // All the unconditional root block should be rewritten.
            self.to_rewrite.push(module.upcast());
          }
          BlockKind::WaitUntil(_) => {} // We respect existing wait_until blocks.
          _ => {
            unreachable!()
          }
        }
      }
    }
    None
  }
}

pub(super) fn rewrite_wait_until(sys: &mut SysBuilder) {
  let mut analyzer = GatherModulesToRewrite {
    to_rewrite: Vec::new(),
  };
  analyzer.enter(sys);
  let to_rewrite = analyzer.to_rewrite;
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
          Some(acc) => Some(sys.create_bitwise_and(created_here!(), acc, v)),
        })
        .unwrap();
      // FIXME: Use the real condition.
      cond.as_mut::<Block>(sys).unwrap().set_value(valid);
    }
  }
}
