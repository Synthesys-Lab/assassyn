use std::collections::{HashMap, HashSet};
use std::vec::Vec;

use crate::builder::system::{ModuleKind, SysBuilder};
use crate::ir::node::{BaseNode, IsElement, NodeKind, Parented};
use crate::ir::{Block, Expr, Module, Opcode};

pub fn topo_sort(sys: &SysBuilder) -> Vec<BaseNode> {
  let mut sorted_nodes = vec![];

  // Collect key and name information for all downstream modules

  let mut rev_graph = sys
    .module_iter(ModuleKind::Downstream)
    .map(|m| (m.get_key(), HashSet::new()))
    .collect::<HashMap<_, _>>();

  let mut graph = rev_graph.clone();

  // Build both the forward and reverse dependency graph
  for module in sys.module_iter(ModuleKind::Downstream) {
    for (ext, _) in module.ext_interf_iter() {
      if let Ok(expr) = ext.as_ref::<Expr>(sys) {
        if matches!(expr.get_opcode(), Opcode::FIFOPush) {
          continue;
        }
        let parent_module = expr.get_parent().as_ref::<Block>(sys).unwrap().get_module();
        if let Some(edge) = graph.get_mut(&parent_module.get_key()) {
          edge.insert(module.get_key());
          rev_graph
            .get_mut(&module.get_key())
            .unwrap()
            .insert(parent_module.get_key());
        }
      }
    }
  }

  // For all the nodes without incoming edges in the reverse graph, add them to the queue
  let mut queue: Vec<usize> = rev_graph
    .iter()
    .filter_map(|(key, incoming)| {
      if incoming.is_empty() {
        Some(*key)
      } else {
        None
      }
    })
    .collect();

  while let Some(node) = queue.pop() {
    sorted_nodes.push(BaseNode::new(NodeKind::Module, node));
    // Use the forward graph to remove the edges in the reverse graph
    for neighbour in graph.get(&node).unwrap().iter() {
      let indeg = rev_graph.get_mut(neighbour).unwrap();
      indeg.remove(&node);
      if indeg.is_empty() {
        queue.push(*neighbour);
      }
    }
  }

  // Check for the presence of circles
  if rev_graph.iter().any(|(_, in_degree)| !in_degree.is_empty()) {
    for (node, deg) in rev_graph.iter() {
      if deg.is_empty() {
        let node = BaseNode::new(NodeKind::Module, *node);
        let module = node.as_ref::<Module>(sys).unwrap();
        eprintln!("Unresolved Module `{}' with in-degree {}", module.get_name(), deg.len());
      }
    }
    panic!("Topological sort failed: graph contains a cycle or unresolved dependencies.");
  }

  sorted_nodes
}
