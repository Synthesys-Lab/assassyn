use std::collections::HashSet;

use crate::builder::SysBuilder;

use super::{node::*, visitor::Visitor, Block, Expr, Module, FIFO};

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
  pub fn get_idx(&self) -> Option<usize> {
    if let Ok(expr) = self.user.as_ref::<Expr>(self.sys) {
      let mut iter = expr.operand_iter();
      Some(iter.position(|x| x.get_key() == self.get_key()).unwrap())
    } else {
      None
    }
  }
}

impl OperandMut<'_> {
  pub fn erase_self(&mut self) {
    if let Some(idx) = self.get().get_idx() {
      let user = self.get().user;
      let mut expr = user.as_mut::<Expr>(self.sys).unwrap();
      expr.remove_operand(idx);
      self.sys.dispose(self.get().upcast());
    } else {
      todo!("The user should be a block?")
    }
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
  uses: HashSet<(BaseNode, Option<usize>, Option<BaseNode>)>,
}

impl GatherAllUses {
  fn new(src: BaseNode) -> Self {
    Self {
      src,
      uses: HashSet::new(),
    }
  }
}

impl Visitor<()> for GatherAllUses {
  fn visit_operand(&mut self, operand: OperandRef<'_>) -> Option<()> {
    if operand.get_value().eq(&self.src) {
      self
        .uses
        .insert((operand.get_user().clone(), operand.get_idx(), None));
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
    let mut gather = GatherAllUses::new(src);
    gather.enter(self);
    for (user, i, new_value) in gather.uses {
      let new_value = new_value.map_or(dst.clone(), |x| x);
      if let Some(i) = i {
        let mut expr_mut = user.as_mut::<Expr>(self).unwrap();
        expr_mut.set_operand(i, new_value);
      } else {
        let mut block_mut = user.as_mut::<Block>(self).unwrap();
        block_mut.set_cond(new_value);
      }
    }
  }
}
