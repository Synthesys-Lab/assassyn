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
    // data to new trigger
    let data = mutator
      .get()
      .operand_iter()
      .skip(2)
      .map(|x| x.clone())
      .collect::<Vec<_>>();
    // mutator.sys.create_trigger(dst, data, cond)
    let ports = data
      .iter()
      .enumerate()
      .map(|(i, x)| {
        PortInfo::new(
          format!("arg.{}", i).as_str(),
          x.get_dtype(&mutator.sys).unwrap().clone(),
        )
      })
      .collect::<Vec<_>>();
    let agent = mutator
      .sys
      .create_module(format!("{}.async.agent", parent_name).as_str(), ports);
    // Create trigger to the agent module.
    mutator.sys.set_current_module(parent);
    mutator.sys.set_insert_before(mutator.get().upcast());
    mutator.sys.create_bundled_trigger(agent.clone(), data);
    // Create trigger to the destination module.
    mutator.sys.set_current_module(agent.clone());
    let agent_module = mutator.sys.get_current_module().unwrap();
    let agent_ports = agent_module
      .port_iter()
      .map(|x| x.upcast())
      .collect::<Vec<_>>();
    let cond = mutator.sys.create_array_read(lock_handle);
    let block = mutator.sys.create_block(Some(cond.clone()));
    mutator.sys.set_current_block(block.clone());
    let data_to_dst = agent_ports
      .iter()
      .map(|x| mutator.sys.create_fifo_pop(x.clone(), None))
      .collect::<Vec<_>>();
    mutator.sys.create_bundled_trigger(dest_module, data_to_dst);
    mutator.sys.set_insert_before(block);
    let flip_cond = mutator.sys.create_flip(cond);
    let block = mutator.sys.create_block(Some(flip_cond));
    mutator.sys.set_current_block(block.clone());
    // Send the data from agent to the actual inokee.
    mutator.sys.create_trigger(agent);
    mutator.erase_from_parent();
  } else {
    println!("No spin triggers found");
  }
}
