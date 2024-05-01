use crate::{
  builder::{system::PortInfo, SysBuilder},
  ir::{node::*, *},
};

use self::visitor::Visitor;

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

pub(super) fn rewrite_spin_triggers(sys: &mut SysBuilder) {
  let mut finder = SpinTriggerFinder::new();

  if let Some((parent, spin_trigger)) = finder.enter(sys) {
    let parent_name = {
      let parent = parent.as_ref::<Module>(sys).unwrap();
      parent.get_name().to_string()
    };
    let mut mutator = sys.get_mut::<Expr>(&spin_trigger).unwrap();
    // Conditional lock.
    let lock_handle = mutator.get().get_operand(0).unwrap().get_value().clone();
    // Destination module
    let dest_module = mutator.get().get_operand(1).unwrap().get_value().clone();
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
    // Here we expose the interface in the module signature.
    if let Some((_, idx)) = &handle_tuple {
      ports.push(PortInfo::new("idx", idx.get_dtype(mutator.sys).unwrap()));
    }
    // Create the agent module while leaving it blank.
    let agent = mutator
      .sys
      .create_module(format!("{}.async.agent", parent_name).as_str(), ports);
    // Create the trigger to the agent module.
    mutator.sys.set_current_module(parent);
    mutator.sys.set_insert_before(mutator.get().upcast());
    let mut bundle = mutator
      .get()
      .operand_iter()
      .skip(1)
      .map(|x| x.get_value().clone())
      .collect::<Vec<_>>();
    // Instead of calling the original destination module, we call the agent module.
    bundle[0] = agent;
    for i in 1..bundle.len() {
      let dst_module = agent.as_ref::<Module>(&mutator.sys).unwrap();
      let port = dst_module.get_port(i - 1).unwrap().clone();
      // For each FIFO push in our system, it takes `module`, and `idx` as arguments.
      // Since we no longer call the original module, we just replace the first argument with the
      // agent module.
      let mut bundle_mut = bundle[i].as_mut::<Expr>(mutator.sys).unwrap();
      assert_eq!(bundle_mut.get().get_opcode(), Opcode::FIFOPush);
      bundle_mut.set_operand(0, port);
    }
    if let Some((_, idx_value)) = &handle_tuple {
      let port_idx = agent
        .as_ref::<Module>(mutator.sys)
        .unwrap()
        .get_num_inputs()
        - 1;
      assert_eq!(port_idx, bundle.len() - 1);
      let push_handle = mutator
        .sys
        .create_fifo_push(agent, port_idx, idx_value.clone());
      bundle.push(push_handle);
    }
    mutator
      .sys
      .create_expr(DataType::void(), Opcode::Trigger, bundle);
    // Create trigger to the destination module.
    mutator.sys.set_current_module(agent.clone());
    mutator.sys.set_current_block_wait_until();
    let wait_until = mutator.sys.get_current_block().unwrap().upcast();
    let agent_module = mutator.sys.get_current_module().unwrap();
    let agent_ports = agent_module
      .port_iter()
      .map(|x| x.upcast())
      .collect::<Vec<_>>();
    if let BlockKind::WaitUntil(cond) = wait_until.as_ref::<Block>(mutator.sys).unwrap().get_kind()
    {
      mutator.sys.set_current_block(cond.clone());
      let cond_handle = if let Some((array, _)) = &handle_tuple {
        let idx_port = agent_ports.last().unwrap().clone();
        let new_idx = mutator.sys.create_fifo_peek(idx_port);
        let new_handle = mutator.sys.create_array_ptr(array.clone(), new_idx);
        new_handle
      } else {
        lock_handle
      };
      let value = mutator.sys.create_array_read(cond_handle);
      let cond_block = mutator.sys.get_current_block().unwrap().upcast();
      cond_block
        .as_mut::<Block>(mutator.sys)
        .unwrap()
        .set_value(value);
    } else {
      panic!("Invalid block kind for a wait_until block");
    }
    mutator.sys.set_current_block(wait_until);
    let mut bind = mutator.sys.get_init_bind(dest_module.clone());
    for (i, elem) in agent_ports.iter().enumerate() {
      let value = mutator.sys.create_fifo_pop(elem.clone(), None);
      if i == agent_ports.len() - 1 && handle_tuple.is_some() {
        continue;
      }
      bind = mutator.sys.push_bind(bind, value, false);
    }
    mutator.sys.create_trigger_bound(bind);
    mutator.erase_from_parent();
  }
}
