use std::collections::HashSet;

use crate::builder::SysBuilder;

use super::{expr::OperandOf, node::*, Expr, Module, FIFO};

macro_rules! impl_user_methods {
  ($class:ident) => {
    paste::paste! {
      impl [< $class Mut >] <'_> {
        pub(crate) fn add_user(&mut self, user: OperandOf) {
          assert!(!self.get().users().contains(&user));
          self.get_mut().user_set.insert(user);
        }
        pub(crate) fn remove_user(&mut self, user: &OperandOf) {
          assert!(self.get().users().contains(&user));
          self.get_mut().user_set.remove(user);
        }
      }
      impl [<$class Ref>] <'_> {
        pub fn users(&self) -> &HashSet<OperandOf> {
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
  pub(crate) fn remove_user(&mut self, node: BaseNode, user: OperandOf) {
    match node.get_kind() {
      NodeKind::Module => {
        let mut module_mut = self.get_mut::<Module>(&node).unwrap();
        module_mut.remove_user(&user);
      }
      NodeKind::FIFO => {
        let mut fifo_mut = self.get_mut::<FIFO>(&node).unwrap();
        fifo_mut.remove_user(&user);
      }
      NodeKind::Expr => {
        let mut expr_mut = self.get_mut::<Expr>(&node).unwrap();
        expr_mut.remove_user(&user);
      }
      _ => {}
    }
  }

  pub(crate) fn add_user(&mut self, node: BaseNode, user: OperandOf) {
    match node.get_kind() {
      NodeKind::Module => {
        let mut module_mut = self.get_mut::<Module>(&node).unwrap();
        module_mut.add_user(user);
      }
      NodeKind::FIFO => {
        let mut fifo_mut = self.get_mut::<FIFO>(&node).unwrap();
        fifo_mut.add_user(user);
      }
      NodeKind::Expr => {
        let mut expr_mut = self.get_mut::<Expr>(&node).unwrap();
        expr_mut.add_user(user);
      }
      _ => {}
    }
  }
}
