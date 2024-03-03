use crate::{
  builder::system::{PortInfo, SysBuilder},
  expr::{Expr, Opcode},
  ir::visitor::Visitor,
  node::{BlockRef, ExprRef, IsElement, ModuleRef},
  BaseNode, Module,
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
    // dest module
    let dest_module = mutator.get().get_operand(0).unwrap().clone();
    // cond array
    let cond_array = mutator.get().get_operand(1).unwrap().clone();
    // data array index
    let cond_array_idx = mutator.get().get_operand(2).unwrap().clone();
    // data to new trigger
    let data = mutator
      .get()
      .operand_iter()
      .skip(3)
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
    mutator.sys.set_current_module(parent.clone());
    mutator.sys.set_insert_before(mutator.get().upcast());
    mutator.sys.create_trigger(&agent, data, None);
    // Create trigger to the destination module.
    mutator.sys.set_current_module(agent.clone());
    let agent_module = mutator.sys.get_current_module().unwrap();
    let data_to_dst = agent_module.port_iter().map(|x| x.upcast()).collect();
    let cond = mutator.sys.create_array_read(&cond_array, &cond_array_idx, None);
    mutator.sys.create_trigger(&dest_module, data_to_dst, Some(cond.clone()));
    let flip_cond = mutator.sys.create_flip(&cond, None);
    mutator.sys.create_trigger(&agent, vec![], Some(flip_cond));
    mutator.erase_from_parent();
  } else {
    println!("No spin triggers found");
  }
}
