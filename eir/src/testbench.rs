use crate::ir::node::BaseNode;

pub enum Event {
  FIFOPush((usize, BaseNode, usize, BaseNode)),
  Trigger((usize, BaseNode)),
  ArrayWrite((usize, BaseNode, BaseNode)),
}

impl Event {
  pub fn fifo_push(cycle: usize, node: BaseNode, data: usize, data_node: BaseNode) -> Event {
    Event::FIFOPush((cycle, node, data, data_node))
  }

  pub fn trigger(cycle: usize, node: BaseNode) -> Event {
    Event::Trigger((cycle, node))
  }

  pub fn array_write(cycle: usize, node: BaseNode, data_node: BaseNode) -> Event {
    Event::ArrayWrite((cycle, node, data_node))
  }

  pub fn cycle(&self) -> usize {
    match self {
      Event::FIFOPush((cycle, _, _, _)) => *cycle,
      Event::Trigger((cycle, _)) => *cycle,
      Event::ArrayWrite((cycle, _, _)) => *cycle,
    }
  }
}
