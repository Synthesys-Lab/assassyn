use std::collections::HashSet;

use crate::{builder::SysBuilder, ir::ir_printer::IRPrinter};

use super::{node::*, visitor::Visitor, Expr, Module, Opcode, FIFO};

/// This node defines a def-use relation between the expression nodes.
/// This is necessary because a node can be used by multiple in other user.
/// For example, c = a + a, even if the (user, def) are the same, two different nodes
/// will be instantiated so that the redundant def-use relation can be maintained.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Operand {
  pub(crate) key: usize,
  def: BaseNode,
  user: BaseNode,
}

impl Parented for Operand {
  fn get_parent(&self) -> BaseNode {
    self.user
  }
  fn set_parent(&mut self, _: BaseNode) {}
}

impl Operand {
  pub fn new(value: BaseNode) -> Operand {
    Operand {
      key: 0,
      user: BaseNode::unknown(),
      def: value,
    }
  }
  pub fn get_value(&self) -> &BaseNode {
    &self.def
  }
  pub fn get_user(&self) -> &BaseNode {
    &self.user
  }
  pub(crate) fn set_user(&mut self, user: BaseNode) {
    self.user = user;
  }
}

impl OperandRef<'_> {
  pub fn get_idx(&self) -> usize {
    let expr = self.user.as_ref::<Expr>(self.sys).unwrap();
    let mut iter = expr.operand_iter();
    iter.position(|x| x.get_key() == self.get_key()).unwrap()
  }
}

impl OperandMut<'_> {
  pub fn erase_self(&mut self) {
    let idx = self.get().get_idx();
    let user = self.get().user;
    let mut expr = user.as_mut::<Expr>(self.sys).unwrap();
    expr.remove_operand(idx);
    self.sys.dispose(self.get().upcast());
  }
}

macro_rules! impl_user_methods {
  ($class:ident) => {
    paste::paste! {
      impl [< $class Mut >] <'_> {
        pub(crate) fn add_user(&mut self, user: BaseNode) {
          assert!(!self.get().users().contains(&user));
          self.get_mut().user_set.insert(user);
        }
        pub(crate) fn remove_user(&mut self, user: &BaseNode) {
          assert!(user.get_kind() == NodeKind::Operand);
          assert!(self.get().users().contains(&user));
          self.get_mut().user_set.remove(user);
        }
      }
      impl [<$class Ref>] <'_> {
        pub fn users(&self) -> &HashSet<BaseNode> {
          &self.get().user_set
        }
      }
    }
  };
}

impl_user_methods!(Module);
impl_user_methods!(Expr);
impl_user_methods!(FIFO);

struct GatherAllUses {
  src: BaseNode,
  dst: BaseNode,
  uses: HashSet<(BaseNode, usize, Option<BaseNode>)>,
}

impl GatherAllUses {
  fn new(src: BaseNode, dst: BaseNode) -> Self {
    Self {
      src,
      dst,
      uses: HashSet::new(),
    }
  }
}

impl Visitor<()> for GatherAllUses {
  fn visit_expr(&mut self, expr: ExprRef<'_>) -> Option<()> {
    if let Opcode::AsyncCall = expr.get_opcode() {
      let bind = expr.get_operand(0).unwrap().get_value().clone();
      self.dispatch(expr.sys, &bind, vec![]);
    }
    for (i, operand) in expr.operand_iter().enumerate() {
      match operand.get_value().get_kind() {
        NodeKind::FIFO => {
          let fifo = operand.get_value().as_ref::<FIFO>(expr.sys).unwrap();
          if fifo.is_placeholder() && fifo.get_parent().eq(&self.src) {
            if let Ok(module) = self.dst.as_ref::<Module>(expr.sys) {
              let new_value = module.get_port(fifo.idx()).unwrap();
              self
                .uses
                .insert((expr.upcast(), i, Some(new_value.upcast())));
            }
          }
        }
        _ => {
          if operand.get_value().eq(&self.src) {
            self.uses.insert((expr.upcast(), i, None));
          }
        }
      }
    }
    None
  }
}

impl SysBuilder {
  pub(crate) fn remove_user(&mut self, operand: BaseNode) {
    if operand.is_unknown() {
      return;
    }
    let operand_ref = operand.as_ref::<Operand>(self).unwrap();
    let def_value = operand_ref.get_value().clone();
    match def_value.get_kind() {
      NodeKind::Module => {
        let mut module_mut = self.get_mut::<Module>(&def_value).unwrap();
        module_mut.remove_user(&operand);
      }
      NodeKind::FIFO => {
        let mut fifo_mut = self.get_mut::<FIFO>(&def_value).unwrap();
        fifo_mut.remove_user(&operand);
      }
      NodeKind::Expr => {
        let mut expr_mut = self.get_mut::<Expr>(&def_value).unwrap();
        expr_mut.remove_user(&operand);
      }
      _ => {}
    }
  }

  pub(crate) fn add_user(&mut self, operand: BaseNode) {
    if operand.is_unknown() {
      return;
    }
    let operand_ref = operand.as_ref::<Operand>(self).unwrap();
    let value = operand_ref.get_value().clone();
    match value.get_kind() {
      NodeKind::Module => {
        let mut module_mut = self.get_mut::<Module>(&value).unwrap();
        module_mut.add_user(operand);
      }
      NodeKind::FIFO => {
        let mut fifo_mut = self.get_mut::<FIFO>(&value).unwrap();
        fifo_mut.add_user(operand);
      }
      NodeKind::Expr => {
        let mut expr_mut = self.get_mut::<Expr>(&value).unwrap();
        expr_mut.add_user(operand);
      }
      _ => {}
    }
  }

  // TODO(@were): I strongly believe we can have a BFS based gatherer to have better performance.
  pub fn replace_all_uses_with(&mut self, src: BaseNode, dst: BaseNode) {
    let mut gather = GatherAllUses::new(src, dst);
    eprintln!(
      "replace {}",
      IRPrinter::new(false).dispatch(self, &src, vec![]).unwrap()
    );
    // eprintln!("by {}", dst.to_string(self));
    for m in self.module_iter() {
      gather.visit_module(m);
    }
    for (expr, i, new_value) in gather.uses {
      let new_value = new_value.map_or(dst.clone(), |x| x);
      eprintln!(
        "use: {}",
        IRPrinter::new(false).dispatch(self, &expr, vec![]).unwrap()
      );
      let mut expr_mut = expr.as_mut::<Expr>(self).unwrap();
      expr_mut.set_operand(i, new_value);
    }
  }
}
