use std::collections::{HashMap, HashSet};
use std::vec::Vec;
use crate::builder::system::{ModuleKind, SysBuilder};
//use crate::ir::node::{BaseNode, IsElement, NodeKind, Parented};
use crate::ir::{Block, Expr, Module, Opcode};
use crate::ir::{expr::subcode, instructions::PureIntrinsic, node::*, visitor::Visitor, *};



#[derive(Default, Debug, Clone)]
pub struct NodeData {
    // 当前边的起点
    mom: String,
    // 当前边的终点
    childs: String,
}

pub struct DependencyGraph {
    // adjacency: 键是边的权重信息字符串，值是从该信息相关节点出发的所有边
    adjacency: HashMap<String, Vec<NodeData>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
        }
    }

    pub fn add_edge(&mut self, src: String, dst: String, edge_info: String) {
        self.adjacency
            .entry(edge_info.clone())
            .or_insert_with(Vec::new)
            .push(NodeData {
                mom: src.clone(),
                childs: dst.clone(),
            });
        println!(
            "Added Edge: {} -{}-> {} | Current Adjacency: {:?}",
            src, edge_info, dst, self.adjacency
        );
    }

    pub fn show_all_paths_with_weights(&self) {
        let mut all_paths = vec![];

        // 计算边的权重
        fn calculate_weight(edge_info: &str) -> i32 {
            match edge_info {
                "log" => 0,
                _ => 1,
            }
        }

        // 深度优先搜索 (DFS)
        fn dfs(
            graph: &HashMap<String, Vec<NodeData>>,
            current: &str,
            path: &mut Vec<String>,
            edges: &mut Vec<String>,
            all_paths: &mut Vec<(Vec<String>, Vec<String>, i32)>,
            current_weight: i32,
        ) {
            path.push(current.to_string());

            // 遍历邻接表
            let mut has_neighbors = false;
            for (edge_info, neighbors) in graph {
                for neighbor in neighbors {
                    if neighbor.mom == current {
                        has_neighbors = true;
                        let edge_weight = calculate_weight(edge_info);

                        edges.push(edge_info.clone());
                        dfs(
                            graph,
                            &neighbor.childs,
                            path,
                            edges,
                            all_paths,
                            current_weight + edge_weight,
                        );
                        edges.pop();
                    }
                }
            }

            // 如果没有邻居，则记录路径
            if !has_neighbors && path.len() > 1 {
                all_paths.push((path.clone(), edges.clone(), current_weight));
            }

            path.pop();
        }

        // 遍历所有起点（`mom` 未作为 `childs` 出现的节点）
        let all_nodes: Vec<&String> = self
            .adjacency
            .values()
            .flat_map(|neighbors| neighbors.iter().map(|n| &n.mom))
            .collect();
        let child_nodes: Vec<&String> = self
            .adjacency
            .values()
            .flat_map(|neighbors| neighbors.iter().map(|n| &n.childs))
            .collect();
        let starting_nodes: Vec<&String> = all_nodes
            .iter()
            .filter(|&n| !child_nodes.contains(n))
            .cloned()
            .collect();

        println!("Starting Nodes: {:?}", starting_nodes);

        for node in starting_nodes {
            let mut path = vec![];
            let mut edges = vec![];
            dfs(&self.adjacency, node, &mut path, &mut edges, &mut all_paths, 0);
        }

        // 去重并排序路径
        all_paths.sort_by(|a, b| a.0.cmp(&b.0));
        all_paths.dedup();

        println!("=== All Paths in the Graph with Weights ===");
        for (path, edges, weight) in all_paths {
            let path_with_edges: Vec<String> = path
                .windows(2)
                .zip(edges.iter())
                .map(|(nodes, edge)| format!("{} \"{}\"-> {}", nodes[0], edge, nodes[1]))
                .collect();
            println!("Path: {} | Weight: {}", path_with_edges.join(" "), weight);
        }
    }
}
// 2. 定义我们的 Visitor
pub struct GraphVisitor<'sys> {
    pub graph: DependencyGraph,
    /// 把 sys 存进来，方便后续访问
    pub sys: &'sys SysBuilder,
}

impl<'sys> GraphVisitor<'sys> {
    pub fn new(sys: &'sys SysBuilder) -> Self {
        Self {
            graph: DependencyGraph {
                adjacency: HashMap::new(),
            },
            sys,
        }
    }

    /// 获取一个节点的“名称”或“标识”
    /// 注意：此函数需要你在真实 assassyn 中做相应修改
    fn node_name<N: std::fmt::Debug>(&self, node: &N) -> String {
        // 仅作示例，真实环境可使用 node.id(), node.debug_name() 等
        format!("{:?}", node)
    }
}


// 为了示例，这里让 T = ()
impl<'sys> Visitor<()> for GraphVisitor<'sys> {
    /// 模块访问, 先访问 FIFO，再访问主体 block
    fn visit_module(&mut self, module: ModuleRef<'_>) -> Option<()> {
        // 收集模块名?model=o1
        let module_name = format!("Module({})", module.get_name().to_string());
        println!("Visiting Module: {}", module_name);
        // 这里还可选择把“模块”当作一个图节点
        // self.graph.add_edge("SystemRoot".to_string(), module_name.clone());

        // 访问该 module 所有 FIFO
        for fifo in module.fifo_iter() {
            self.visit_fifo(fifo);
        }
        // 继续访问 module 的 body (block)
        if let Some(x) = self.visit_block(module.get_body()) {
            return x.into();
        }
        None
    }

    /// FIFO访问
    fn visit_fifo(&mut self, fifo: FIFORef<'_>) -> Option<()> {
        let fifo_name = format!("FIFO({})", fifo.get_name().to_string());

        // 如果想把 FIFO 和上下游连接建立关系，可在这里加 add_edge
        // 例如: self.graph.add_edge( ... );
        // 暂时仅示例输出
        println!("Visiting FIFO: {}", fifo_name);

        None
    }
/* 
    /// 访问表达式节点(如 create_sub, create_add 等)
    fn visit_expr(&mut self, expr: ExprRef<'_>) -> Option<()> {
        // 获取表达式自身的名称
        let expr_label = format!("Expr_{:?}({:?})", expr.get_opcode().to_string(),expr.get_name().to_string());

        // 对于 expr 的每个 operand
        for operand_ref in expr.operand_iter() {
            // operand 也是一个节点
            // 这里的 dispatch 会自动调用 visit_operand (或别的)  
            // 但我们先获取一个字符串来代表它
            let operand_name = operand_ref.get_value().to_string(self.sys);
            // 比如记录“operand -> expr”这条边
            self.graph.add_edge(operand_name.clone(), expr_label.clone());

            // 让 visitor 继续深入 operand
            if let Some(x) = self.visit_operand(operand_ref) {
                return x.into();
            }
        }
        None
    }
    */
    fn visit_expr(&mut self, expr: ExprRef<'_>) -> Option<()> {
        // 获取表达式的 opcode 和 name，作为节点的数据
        let expr_opcode = expr.get_opcode().to_string();
        let expr_name = expr.get_name().to_string();
    
        // 遍历表达式的每个 operand
        for operand_ref in expr.operand_iter() {
            // 获取 operand 的名称，作为 mom 的值
            let operand_name = operand_ref.get_value().to_string(self.sys);
    
            // 更新图，记录“operand -> expr”的边
            self.graph.add_edge(
                
                operand_name.clone(),
                expr_name.clone(), 
                 
                expr_opcode.clone(),
                
            );
    
            // 递归访问 operand
            if let Some(x) = self.visit_operand(operand_ref) {
                return x.into();
            }
        }
        None
    }
    

    /// 访问数组
    fn visit_array(&mut self, arr: ArrayRef<'_>) -> Option<()> {
        let arr_name = format!("Array({})", arr.get_name().to_string());
        println!("Visiting Array: {}", arr_name);
        None
    }

    /// 访问整型立即数
    fn visit_int_imm(&mut self, imm: IntImmRef<'_>) -> Option<()> {
        println!("IntImm = {}", imm.get_value().to_string());
        None
    }

    /// 访问字符串立即数
    fn visit_string_imm(&mut self, s: StrImmRef<'_>) -> Option<()> {
        println!("StrImm = {}", s.get_value().to_string());
        None
    }

    /// 访问块（block）
    fn visit_block(&mut self, block: BlockRef<'_>) -> Option<()> {
        // block.body_iter() 是一堆 BaseNode
        for elem in block.body_iter() {
            // dispatch 会根据 node 的 kind 来调用相应 visit_XXX
            if let Some(x) = self.dispatch(block.sys, &elem, vec![]) {
                return Some(x);
            }
        }
        None
    }
/* 
    /// 访问操作数 (operand)
    fn visit_operand(&mut self, operand: OperandRef<'_>) -> Option<()> {
        // 你也可以在这里把 operand 视为节点，
        // 例如 self.graph.add_edge( operand -> ??? ) 等
        // 具体视数据流需求而定

        // 如果 operand 有子节点（比如 constant/array index 等），可继续向下 dispatch
        if let Some(sub_node) = operand. {
            self.dispatch(operand.sys, &sub_node, vec![]);
        }
        None
    }
*/
    /// 进入整个系统（SysBuilder），遍历所有模块
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
