use std::collections::HashSet;

use crate::builder::SysBuilder;

use super::{node::*, Expr, Module, FIFO};

/// This node defines a def-use relation between the expression nodes.
/// This is necessary because a node can be used by multiple in other user.
/// For example, c = a + a, even if the (user, def) are the same, two different nodes
/// will be instantiated so that the redundant def-use relation can be maintained.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Operand {
  pub(crate) key: usize,
  user: BaseNode,
  def: BaseNode,
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

impl SysBuilder {
  pub(crate) fn remove_user(&mut self, operand: BaseNode) {
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
}
