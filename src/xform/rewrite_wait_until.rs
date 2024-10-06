use crate::{
  builder::{
    system::{InsertPoint, ModuleKind},
    SysBuilder,
  },
  ir::{
    module,
    node::{BaseNode, IsElement, ModuleRef},
    visitor::Visitor,
    Module,
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
        // Skip the module if it has the systolic attribute.
        if module
          .get_attrs()
          .iter()
          .any(|x| matches!(x, module::Attribute::Systolic))
        {
          return None;
        }
        if module.get_body().get_wait_until().is_none() {
          if module.get_num_inputs() == 0 {
            eprintln!(
              "Warning: module {} has no inputs, but is neither the driver nor testbench.",
              module.get_name()
            );
          }
          // All the unconditional root block should be rewritten.
          self.to_rewrite.push(module.upcast());
        }
      }
    }
    None
  }

  fn enter(&mut self, sys: &SysBuilder) -> Option<()> {
    for module in sys.module_iter(ModuleKind::Module) {
      self.visit_module(module);
    }
    ().into()
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
          .fifo_iter()
          .map(|port| port.upcast())
          .collect::<Vec<_>>(),
        module.get_body().upcast(),
      )
    };
    if ports.is_empty() {
      continue;
    }
    sys.set_current_ip(InsertPoint {
      module,
      block: body,
      at: 0.into(),
    });

    let valids = ports
      .into_iter()
      .map(|port| sys.create_fifo_valid(port))
      .collect::<Vec<_>>();
    let valid = valids
      .into_iter()
      .fold(None, |acc, v| match acc {
        None => Some(v),
        Some(acc) => Some(sys.create_bitwise_and(acc, v)),
      })
      .unwrap();
    sys.create_wait_until(valid);
  }
}
