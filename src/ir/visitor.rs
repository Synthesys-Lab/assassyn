use std::collections::HashSet;

use crate::{
  builder::system::SysBuilder,
  data::Array,
  expr::Expr,
  node::{ArrayRef, BlockRef, ExprRef, InputRef, IntImmRef, ModuleRef},
  port::Input,
  BaseNode, IntImm, Module,
};

use super::block::Block;

pub trait Visitor<T: Default> {
  fn visit_module(&mut self, module: &ModuleRef<'_>) -> T {
    for input in module.port_iter() {
      self.visit_input(&input);
    }
    self.visit_block(&module.get_body());
    T::default()
  }

  fn visit_input(&mut self, _: &InputRef<'_>) -> T {
    T::default()
  }

  fn visit_expr(&mut self, expr: &ExprRef<'_>) -> T {
    for elem in expr.operand_iter() {
      match elem {
        &BaseNode::Array(_) => self.visit_array(&elem.as_ref::<Array>(expr.sys).unwrap()),
        &BaseNode::IntImm(_) => self.visit_int_imm(&elem.as_ref::<IntImm>(expr.sys).unwrap()),
        &BaseNode::Block(_) => self.visit_block(&elem.as_ref::<Block>(expr.sys).unwrap()),
        &BaseNode::Expr(_) => T::default(),
        &BaseNode::Module(_) => T::default(),
        &BaseNode::Input(_) => T::default(),
        &BaseNode::Unknown => {
          panic!("Unknown node type")
        }
      };
    }
    T::default()
  }

  fn visit_array(&mut self, _: &ArrayRef<'_>) -> T {
    T::default()
  }

  fn visit_int_imm(&mut self, _: &IntImmRef<'_>) -> T {
    T::default()
  }

  fn visit_block(&mut self, block: &BlockRef<'_>) -> T {
    if let Some(_) = block.get_pred() {}
    for elem in block.iter() {
      match elem {
        BaseNode::Expr(_) => self.visit_expr(&elem.as_ref::<Expr>(block.sys).unwrap()),
        BaseNode::Block(_) => self.visit_block(&elem.as_ref::<Block>(block.sys).unwrap()),
        _ => T::default(),
      };
    }
    T::default()
  }

  fn dispatch(&mut self, sys: &SysBuilder, node: &BaseNode, non_recur: &HashSet<String>) -> T {
    match node {
      BaseNode::Expr(_) => {
        if non_recur.contains(&String::from("Expr")) {
          T::default()
        } else {
          self.visit_expr(&node.as_ref::<Expr>(sys).unwrap())
        }
      }
      BaseNode::Block(_) => {
        if non_recur.contains(&String::from("Block")) {
          T::default()
        } else {
          self.visit_block(&node.as_ref::<Block>(sys).unwrap())
        }
      }
      BaseNode::Module(_) => {
        if non_recur.contains(&String::from("Module")) {
          T::default()
        } else {
          self.visit_module(&node.as_ref::<Module>(sys).unwrap())
        }
      }
      BaseNode::Input(_) => {
        if non_recur.contains(&String::from("Input")) {
          T::default()
        } else {
          self.visit_input(&node.as_ref::<Input>(sys).unwrap())
        }
      }
      BaseNode::Array(_) => {
        if non_recur.contains(&String::from("Array")) {
          T::default()
        } else {
          self.visit_array(&node.as_ref::<Array>(sys).unwrap())
        }
      }
      BaseNode::IntImm(_) => {
        if non_recur.contains(&String::from("IntImm")) {
          T::default()
        } else {
          self.visit_int_imm(&node.as_ref::<IntImm>(sys).unwrap())
        }
      }
      BaseNode::Unknown => {
        panic!("Unknown node type")
      }
    }
  }
}
