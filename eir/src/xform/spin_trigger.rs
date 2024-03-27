use crate::{
  builder::system::{PortInfo, SysBuilder},
  frontend::*,
  ir::visitor::Visitor,
};

struct SpinTriggerFinder {
  module_parent: BaseNode,
}

impl SpinTriggerFinder {
  pub fn new() -> Self {
    Self {
      module_parent: BaseNode::unknown(),
    }
  }
}

impl Visitor<(BaseNode, BaseNode)> for SpinTriggerFinder {
  fn visit_expr(&mut self, expr: &ExprRef<'_>) -> Option<(BaseNode, BaseNode)> {
    if let Opcode::SpinTrigger = expr.get_opcode() {
      return Some((self.module_parent.clone(), expr.upcast()));
    }
    None
  }
  fn visit_block(&mut self, block: &BlockRef<'_>) -> Option<(BaseNode, BaseNode)> {
    for elem in block.iter() {
      if let Some(x) = self.dispatch(block.sys, elem, vec![]) {
        return x.into();
      }
    }
    None
  }
  fn visit_module(&mut self, module: &ModuleRef<'_>) -> Option<(BaseNode, BaseNode)> {
    self.module_parent = module.upcast();
    let res = self.visit_block(&module.get_body());
    self.module_parent = BaseNode::unknown();
    res
  }
}

pub fn rewrite_spin_triggers(sys: &mut SysBuilder) {
  let mut finder = SpinTriggerFinder::new();

  if let Some((parent, spin_trigger)) = finder.enter(sys) {
    let parent_name = {
      let parent = parent.as_ref::<Module>(sys).unwrap();
      parent.get_name().to_string()
    };
    let mut mutator = sys.get_mut::<Expr>(&spin_trigger).unwrap();
    // Conditional lock.
    let lock_handle = mutator.get().get_operand(0).unwrap().clone();
    // Destination module
    let dest_module = mutator.get().get_operand(1).unwrap().clone();
    let module_signature = dest_module.get_dtype(mutator.sys).unwrap();
    let mut ports = match module_signature {
      DataType::Module(ports) => ports
        .into_iter()
        .enumerate()
        .map(|(i, x)| PortInfo::new(format!("arg.{}", i).as_str(), *x)),
      _ => panic!("Destination module is not a module"),
    }
    .collect::<Vec<_>>();
    let handle_tuple = {
      let lock_handle = lock_handle.as_ref::<ArrayPtr>(mutator.sys).unwrap();
      match lock_handle.get_idx().get_kind() {
        NodeKind::IntImm => None,
        _ => Some((
          lock_handle.get_array().clone(),
          lock_handle.get_idx().clone(),
        )),
      }
    };
    // If a[i]'s i is NOT a constant, we need to push it to the agent module.
    if let Some((_, idx)) = &handle_tuple {
      ports.insert(0, PortInfo::new("idx", idx.get_dtype(mutator.sys).unwrap()));
    }
    // Find the data to this module
    let agent = mutator
      .sys
      .create_module(format!("{}.async.agent", parent_name).as_str(), ports);
    // Create trigger to the agent module.
    mutator.sys.set_current_module(parent);
    mutator.sys.set_insert_before(mutator.get().upcast());
    let bundle = mutator
      .get()
      .operand_iter()
      .skip(1)
      .cloned()
      .collect::<Vec<_>>();
    mutator
      .sys
      .create_expr(DataType::void(), Opcode::Trigger, bundle);
    // Create trigger to the destination module.
    mutator.sys.set_current_module(agent.clone());
    let agent_module = mutator.sys.get_current_module().unwrap();
    let agent_ports = agent_module
      .port_iter()
      .map(|x| x.upcast())
      .collect::<Vec<_>>();
    let cond = if let Some((array, _)) = &handle_tuple {
      let idx_port = agent_ports.get(0).unwrap().clone();
      let new_idx = mutator.sys.create_fifo_peek(idx_port);
      let new_handle = mutator.sys.create_array_ptr(array.clone(), new_idx);
      mutator.sys.create_array_read(new_handle)
    } else {
      mutator.sys.create_array_read(lock_handle)
    };
    let block = mutator.sys.create_block(Some(cond.clone()));
    mutator.sys.set_current_block(block.clone());
    let mut bind = mutator.sys.get_init_bind(dest_module.clone());
    for (i, elem) in agent_ports.iter().enumerate() {
      let value = mutator.sys.create_fifo_pop(elem.clone(), None);
      if i == 0 && handle_tuple.is_some() {
        continue;
      }
      bind = mutator.sys.push_bind(bind, value, false);
    }
    mutator.sys.create_trigger_bound(bind);
    mutator.sys.set_insert_before(block);
    let flip_cond = mutator.sys.create_flip(cond);
    let block = mutator.sys.create_block(Some(flip_cond));
    mutator.sys.set_current_block(block.clone());
    // Send the data from agent to the actual inokee.
    mutator.sys.create_self_trigger();
    mutator.erase_from_parent();
  } else {
    println!("No spin triggers found");
  }
}
