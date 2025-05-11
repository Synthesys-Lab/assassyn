use crate::builder::system::{ModuleKind, SysBuilder};
use std::collections::HashMap;
use std::vec::Vec;
//use crate::ir::node::{BaseNode, IsElement, NodeKind, Parented};
use crate::ir::expr::subcode::Binary;
use crate::ir::Opcode;
use crate::ir::{node::*, visitor::Visitor, *};
#[derive(Debug, Clone)]
pub struct NodeData {
  mom: usize,
  childs: usize,
}

pub struct DependencyGraph {
  adjacency: HashMap<Opcode, Vec<NodeData>>,
  entry: HashMap<usize, BaseNode>,
}
impl Default for DependencyGraph {
  fn default() -> Self {
    Self::new()
  }
}

impl DependencyGraph {
  pub fn new() -> Self {
    Self {
      adjacency: HashMap::new(),
      entry: HashMap::new(),
    }
  }

  pub fn add_edge(&mut self, src: usize, dst: usize, edge_info: Opcode) {
    self.adjacency.entry(edge_info).or_default().push(NodeData {
      mom: src,
      childs: dst,
    });
  }

  pub fn show_all_paths_with_weights(&self, sys: &SysBuilder, show_all: bool) {
    let mut all_paths = vec![];
    let mut last_path: usize = 0;
    let mut last_weight: i32 = 0;

    fn calculate_weight(opcode: &Opcode) -> i32 {
      match opcode {
        Opcode::Load => 0,
        Opcode::Store => 1,
        Opcode::Binary { binop } => match binop {
          Binary::Add | Binary::Sub => 2,
          Binary::Mul => 8,
          Binary::BitwiseAnd | Binary::BitwiseOr | Binary::BitwiseXor => 1,
          Binary::Shl | Binary::Shr => 1,
          Binary::Mod => 4,
        },
        Opcode::Unary { .. } => 0,
        Opcode::Select => 1,
        Opcode::Select1Hot => 1,
        Opcode::Compare { .. } => 0,
        Opcode::Bind => 0,
        Opcode::FIFOPush => 1,
        Opcode::FIFOPop => 0,
        Opcode::AsyncCall => 0,
        Opcode::Slice => 0,
        Opcode::Cast { .. } => 0,
        Opcode::Concat => 0,
        Opcode::BlockIntrinsic { .. } => 0,
        Opcode::PureIntrinsic { .. } => 0,
        Opcode::Log => 0,
      }
    }

    #[allow(clippy::too_many_arguments)]
    fn dfs(
      graph: &HashMap<Opcode, Vec<NodeData>>,
      current: usize,
      path: &mut Vec<usize>,
      edges: &mut Vec<Opcode>,
      all_paths: &mut Vec<(Vec<usize>, Vec<Opcode>, i32)>,
      current_weight: i32,
      show_all: bool,
      last_path: &mut usize,
      last_weight: &mut i32,
    ) {
      path.push(current);

      let mut has_neighbors = false;
      for (edge_info, neighbors) in graph {
        for neighbor in neighbors {
          if neighbor.mom == current {
            has_neighbors = true;
            let edge_weight = calculate_weight(edge_info);

            edges.push(*edge_info);
            dfs(
              graph,
              neighbor.childs,
              path,
              edges,
              all_paths,
              current_weight + edge_weight,
              show_all,
              last_path,
              last_weight,
            );
            edges.pop();
          }
        }
      }

      let mut is_end = false;
      if let Some(&last_opcode) = edges.last() {
        is_end = last_opcode == Opcode::FIFOPush || last_opcode == Opcode::Store;
      }

      let mut has_entry = false;
      if let Some(&first_opcode) = edges.first() {
        has_entry = first_opcode == Opcode::FIFOPop || first_opcode == Opcode::Load;
      }

      if !has_neighbors && path.len() > 1 && is_end && has_entry {
        if show_all {
          all_paths.push((path.clone(), edges.clone(), current_weight));
        } else if let Some(&this_path) = path.first() {
          if this_path == *last_path {
            if current_weight > *last_weight {
              all_paths.pop();
              all_paths.push((path.clone(), edges.clone(), current_weight));
              *last_path = this_path;
              *last_weight = current_weight;
            }
          } else {
            all_paths.push((path.clone(), edges.clone(), current_weight));
            *last_path = this_path;
            *last_weight = current_weight;
          }
        }
      }

      path.pop();
    }

    for node in self.entry.keys() {
      let mut path = vec![];
      let mut edges = vec![];
      dfs(
        &self.adjacency,
        *node,
        &mut path,
        &mut edges,
        &mut all_paths,
        0,
        show_all,
        &mut last_path,
        &mut last_weight,
      );
    }

    println!("=== All Paths with Time Weights ===");
    for (path, edges, weight) in all_paths {
      if let Some(&start_node) = path.first() {
        if let Some(base_node) = self.entry.get(&start_node) {
          let base_node_name = base_node.to_string(sys);
          let path_with_edges: Vec<String> = path
            .windows(2)
            .zip(edges.iter())
            .map(|(nodes, edge)| format!("\"{}\" -> {}", edge, nodes[1]))
            .collect();
          println!(
            "Path: \\{}\\ {}   {} | Time Weight: {}",
            base_node_name,
            base_node.get_key(),
            path_with_edges.join("    "),
            weight
          );
        }
      }
    }
  }
}

pub struct GraphVisitor<'sys> {
  pub graph: DependencyGraph,

  pub sys: &'sys SysBuilder,
}

impl<'sys> GraphVisitor<'sys> {
  pub fn new(sys: &'sys SysBuilder) -> Self {
    Self {
      graph: DependencyGraph {
        adjacency: HashMap::new(),
        entry: HashMap::new(),
      },

      sys,
    }
  }
}

impl Visitor<()> for GraphVisitor<'_> {
  fn visit_expr(&mut self, expr: ExprRef<'_>) -> Option<()> {
    let expr_opcode = expr.get_opcode();

    if (expr_opcode != Opcode::Bind) && (expr_opcode != Opcode::AsyncCall) {
      for operand_ref in expr.operand_iter() {
        self
          .graph
          .add_edge(operand_ref.get_value().get_key(), expr.get_key(), expr_opcode);
        if (expr_opcode == Opcode::Load) || (expr_opcode == Opcode::FIFOPop) {
          if let Some(DataType::UInt(_)) = operand_ref.get_value().get_dtype(self.sys) {
          } else {
            self
              .graph
              .entry
              .insert(operand_ref.get_value().get_key(), *operand_ref.get_value());
          }
        }
      }
    }
    None
  }

  fn enter(&mut self, sys: &SysBuilder) -> Option<()> {
    for elem in sys.module_iter(ModuleKind::All) {
      let res = self.visit_module(elem);
      if res.is_some() {
        return res;
      }
    }
    None
  }
}
