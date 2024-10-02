use std::collections::{HashMap, HashSet};
use std::vec::Vec;

use crate::builder::system::{ModuleKind, SysBuilder};
use crate::ir::node::{BaseNode, IsElement, Parented};
use crate::ir::{Block, Expr};

pub fn topo_sort(sys: &SysBuilder) -> Vec<BaseNode> {
  let mut sorted_nodes = vec![];

  // Collect key and name information for all downstream modules
  let mut module_map: HashMap<usize, BaseNode> = HashMap::new();
  let mut downstream_keys: HashSet<usize> = HashSet::new();
  for module in sys.module_iter(ModuleKind::Downstream) {
    module_map.insert(module.key, module.upcast());
    downstream_keys.insert(module.key);
  }

  type TopoGraph = HashMap<usize, HashSet<usize>>;

  let mut topo_graph: TopoGraph = HashMap::new();
  let mut in_degree: HashMap<usize, usize> = HashMap::new();

  for module in sys.module_iter(ModuleKind::Downstream) {
    in_degree.entry(module.key).or_insert(0);

    topo_graph.entry(module.key).or_default();

    for (ext, _) in module.ext_interf_iter() {
      if let Ok(expr) = ext.as_ref::<Expr>(sys) {
        let parent_module = expr.get_parent().as_ref::<Block>(sys).unwrap().get_module();

        // Add edges only when parent_madule is downstream
        if downstream_keys.contains(&parent_module.get_key()) {
          topo_graph
            .entry(parent_module.get_key())
            .or_default()
            .insert(module.key);

          *in_degree.entry(module.key).or_insert(0) += 1;
        }
      }
    }
  }

  // Topological sort
  let mut queue: Vec<usize> = in_degree
    .iter()
    .filter(|&(_, &deg)| deg == 0)
    .map(|(&node, _)| node)
    .collect();

  while let Some(node_key) = queue.pop() {
    if let Some(node) = module_map.get(&node_key) {
      sorted_nodes.push(*node);
    }

    if let Some(neighbors) = topo_graph.get(&node_key) {
      for &neighbor in neighbors {
        if let Some(in_deg) = in_degree.get_mut(&neighbor) {
          if *in_deg > 0 {
            *in_deg -= 1;
            if *in_deg == 0 {
              queue.push(neighbor);
            }
          }
        }
      }
    }
  }

  // Check for the presence of circles
  if sorted_nodes.len() != module_map.len() {
    panic!("Topological sort failed: graph contains a cycle or unresolved dependencies.");
  }

  sorted_nodes
}
