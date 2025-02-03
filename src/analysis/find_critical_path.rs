use crate::builder::system::{ModuleKind, SysBuilder};
use crate::ir::expr::subcode::Binary;
use crate::ir::{expr::subcode, instructions::PureIntrinsic, node::*, visitor::Visitor, *};
use crate::ir::{Block, Expr, Module, Opcode};
use std::collections::{HashMap, HashSet};
use std::vec::Vec;
#[derive(Debug, Clone)]
pub struct NodeData {
  mom: usize,
  childs: usize,
  delay: i32,
}
#[derive(Debug, Clone)]
pub struct Stats {
    count: i32,
    sum_x_1: i64,
    sum_x_2: i64,
}

pub struct DependencyGraph {
  adjacency: HashMap<Opcode, Vec<NodeData>>,
  entry: HashMap<usize, BaseNode>,
  path_attr: HashMap<usize,Stats >,
}
impl Default for DependencyGraph {
  fn default() -> Self {
    Self::new()
  }
}

impl DependencyGraph {
  #[allow(clippy::too_many_arguments)]
  pub fn new() -> Self {
    Self {
      adjacency: HashMap::new(),
      entry: HashMap::new(),
      path_attr: HashMap::new(),
    }
  }

  pub fn add_edge(&mut self, src: usize, dst: usize, edge_info: Opcode, delay: i32) {
    self.adjacency.entry(edge_info).or_default().push(NodeData {
      mom: src,
      childs: dst,
      delay,
    });
  }


  pub fn show_all_paths_with_weights(&mut self, sys: &SysBuilder, show_all: bool) {
    let mut all_paths = vec![];
    let mut last_path: usize = 0;
    let mut last_weight: i32 = 0;
    let mut all_stats = Stats{  count: 0, sum_x_1: 0, sum_x_2: 0};


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
      path_attr: &mut HashMap<usize,Stats >,

    ) {
      path.push(current);

      let mut has_neighbors = false;
      for (edge_info, neighbors) in graph {
        for neighbor in neighbors {
          if neighbor.mom == current {
            has_neighbors = true;
            //let edge_weight = calculate_weight(edge_info);
            let edge_weight = neighbor.delay;
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
              path_attr,

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
        if let Some(&start_node) = path.first() {
          path_attr.entry(start_node).and_modify(|stats| {
            stats.count += 1;
            stats.sum_x_1 += current_weight as i64 ;
            stats.sum_x_2 += (current_weight as i64).pow(2);
        }).or_insert(Stats { count: 1, sum_x_1: current_weight as i64, sum_x_2: (current_weight as i64).pow(2)});
        
        //dbg!(&path_attr);
        }


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
        &mut self.path_attr,

      );
    }

    println!("=== All Paths with Time Weights ===");
    for (path, edges, weight) in all_paths {
      if let Some(&start_node) = path.first() {
        if let Some(base_node) = self.entry.get(&start_node) {
          if let Some(attr) = self.path_attr.get(&start_node)
          {

            let base_node_name = base_node.to_string(sys);
            let path_with_edges: Vec<String> = path
              .windows(2)
              .zip(edges.iter())
              .map(|(nodes, edge)| format!("\"{}\" -> {}", edge, nodes[1]))
              .collect();
            println!(
              "Path: \\{}\\ {}   {} | Time Weight: {} | Variance: {}",
              base_node_name,
              base_node.get_key(),
              path_with_edges.join("    "),
              weight,
              ((attr.sum_x_2/attr.count as i64)-(attr.sum_x_1/attr.count as i64).pow(2))
          );
          all_stats.count += 1;
          all_stats.sum_x_1 += weight as i64 ;
          all_stats.sum_x_2 += (weight as i64).pow(2);
        }
        }
      }
    }
    println!("=== All Paths Var is {}===", ((all_stats.sum_x_2/all_stats.count as i64)-(all_stats.sum_x_1/all_stats.count as i64).pow(2)) );
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
        path_attr: HashMap::new(),
      },

      sys,
    }
  }
}

impl<'sys> Visitor<()> for GraphVisitor<'sys> {
  fn visit_expr(&mut self, expr: ExprRef<'_>) -> Option<()> {
    let expr_opcode = expr.get_opcode();
    let mut is_first_time = true;
    dbg!(&expr.get_parent());

    if (expr_opcode != Opcode::Bind) && (expr_opcode != Opcode::AsyncCall) {
      let mut edge_delay = 0;
      for operand_ref in expr.operand_iter() {
        if is_first_time {
          match operand_ref.get_value().get_dtype(self.sys) {
            Some(DataType::Int(bits)) | Some(DataType::UInt(bits)) | Some(DataType::Bits(bits)) => {
              edge_delay =
                calculate_weight_pro(&expr_opcode, bits as i32, expr.get_num_operands() as u8);
            }
            _ => {
              edge_delay = calculate_weight_pro(&expr_opcode, 1, expr.get_num_operands() as u8);
            }
          }

          is_first_time = false;
        }
        self.graph.add_edge(
          operand_ref.get_value().get_key(),
          expr.get_key(),
          expr_opcode,
          edge_delay,
        );
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

pub fn calculate_weight_pro(opcode: &Opcode, bit_width: i32, op_num: u8) -> i32 {
  match opcode {
    Opcode::Load => 0,
    Opcode::Store => 1,
    Opcode::Binary { binop } => match binop {
      Binary::Add | Binary::Sub => (2 + (bit_width as f64).log2() as i32) * (op_num - 1) as i32,
      Binary::Mul => {
        (bit_width + (bit_width - 1) * (2 + (bit_width as f64).log2() as i32)) * (op_num - 1) as i32
      }
      Binary::BitwiseAnd | Binary::BitwiseOr => (op_num - 1) as i32,
      Binary::BitwiseXor => 2 * (op_num - 1) as i32,
      Binary::Shl | Binary::Shr => (bit_width as f64).log2() as i32,
      Binary::Mod => 4,
    },
    Opcode::Unary { .. } => 0,
    Opcode::Select => 1,
    Opcode::Select1Hot => 1,
    Opcode::Compare { .. } => 2 * (op_num - 1) as i32,
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
