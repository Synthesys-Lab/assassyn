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
        .insert((*operand.get_user(), operand.get_idx(), None));
    }
    None
  }
}

impl ModuleRef<'_> {
  /// Gather all the related external interfaces with the given operand. This is typically used to
  /// maintain the redundant information when modifying this IR.
  /// If the given operand is an operand, gather just this specific operand.
  /// If the given operand is a value reference, gather all the operands that `get_value == this
  /// operand`.
  ///
  /// # Arguments
  ///
  /// * `operand` - The operand to gather the related external interfaces.
  pub(crate) fn gather_related_externals(&self, operand: &BaseNode) -> Vec<(BaseNode, BaseNode)> {
    // Remove all the external interfaces related to this instruction.
    let tmp = self
      .get()
      .external_interfaces
      .iter()
      .map(|(ext, users)| {
        (
          *ext,
          users
            .iter()
            .filter(|x| {
              (*x).eq(operand) || {
                let user = (*x).as_ref::<Operand>(self.sys).unwrap();
                user.get_value().eq(operand)
              }
            })
            .cloned()
            .collect::<Vec<_>>(),
        )
      })
      .filter(|(_, users)| !users.is_empty())
      .collect::<Vec<_>>();
    tmp
      .iter()
      .flat_map(|(ext, users)| users.iter().map(|x| (*ext, *x)))
      .collect()
  }
}

impl ModuleMut<'_> {
  /// Remove a specific external interface's usage. If this usage set is empty after the removal,
  /// remove the external interface from the module, too.
  ///
  /// # Arguments
  ///
  /// * `ext_node` - The external interface node.
  /// * `operand` - The operand node that uses this external interface.
  fn remove_external_interface(&mut self, ext_node: BaseNode, operand: BaseNode) {
    if let Some(operations) = self.get_mut().external_interfaces.get_mut(&ext_node) {
      assert!(operations.contains(&operand));
      operations.remove(&operand);
      if operations.is_empty() {
        self.get_mut().external_interfaces.remove(&ext_node);
      }
    }
  }

  /// Remove all the related external interfaces with the given condition.
  fn remove_related_externals(&mut self, operand: &BaseNode) {
    let to_remove = self.get().gather_related_externals(operand);
    to_remove.into_iter().for_each(|(ext, operand)| {
      self.remove_external_interface(ext, operand);
    });
  }

  /// Add related external interfaces to the module.
  fn add_related_externals(&mut self, operand: BaseNode) {
    // Reconnect the external interfaces if applicable.
    // TODO(@were): Maybe later unify a common interface for this.
    let operand_ref = operand.as_ref::<Operand>(self.sys).unwrap();
    let value = *operand_ref.get_value();
    match value.get_kind() {
      NodeKind::Array => {
        self.insert_external_interface(value, operand);
      }
      NodeKind::FIFO => {
        let fifo = value.as_ref::<FIFO>(self.sys).unwrap();
        if fifo.get_parent().get_key() != self.get().get_key() {
          self.insert_external_interface(value, operand);
        }
      }
      _ => {}
    }
  }
}

impl ExprMut<'_> {
  /// Unify the implementation of setting and removing an operand.
  fn set_operand_impl(&mut self, i: usize, value: Option<BaseNode>) {
    let block = self.sys.get::<Block>(&self.get().get_parent()).unwrap();
    let module = block.get_module();
    // Remove all the external interfaces related to this instruction.
    let module = module.upcast();
    let expr = self.get().upcast();
    if let Some(old) = self.get().operands.get(i) {
      let old = *old;
      self.sys.cut_operand(&old);
      let operand = value.map(|x| self.sys.insert_element(Operand::new(x)));
      let mut module_mut = self.sys.get_mut::<Module>(&module).unwrap();
      if let Some(operand) = operand {
        module_mut.add_related_externals(operand);
        self.get_mut().operands[i] = operand;
        operand
          .as_mut::<Operand>(self.sys)
          .unwrap()
          .get_mut()
          .set_user(expr);
        self.sys.add_user(operand);
      } else {
        self.get_mut().operands.remove(i);
      }
    }
  }

  /// Set the i-th operand to the given value.
  /// NOTE: Just the raw value is given, not the operand wrapper.
  pub fn set_operand(&mut self, i: usize, value: BaseNode) {
    self.set_operand_impl(i, Some(value));
  }

  pub fn remove_operand(&mut self, i: usize) {
    self.set_operand_impl(i, None);
  }
}

impl SysBuilder {
  pub(crate) fn cut_operand(&mut self, operand: &BaseNode) {
    if operand.is_unknown() {
      return;
    }
    let operand_ref = operand.as_ref::<Operand>(self).unwrap();
    // TODO(@were): Make this a unified interface.
    let module = match operand_ref.get_user().get_kind() {
      NodeKind::Expr => {
        let expr = operand_ref.get_user().as_ref::<Expr>(self).unwrap();
        expr
          .get_parent()
          .as_ref::<Block>(self)
          .unwrap()
          .get_module()
          .upcast()
      }
      _ => unreachable!(),
    };
    let mut module_mut = self.get_mut::<Module>(&module).unwrap();
    module_mut.remove_related_externals(operand);
    self.remove_user(operand);
  }

  pub(crate) fn remove_user(&mut self, operand: &BaseNode) {
    if operand.is_unknown() {
      return;
    }
    let operand_ref = operand.as_ref::<Operand>(self).unwrap();
    let def_value = *operand_ref.get_value();
    match def_value.get_kind() {
      NodeKind::Module => {
        let mut module_mut = self.get_mut::<Module>(&def_value).unwrap();
        module_mut.remove_user(operand);
      }
      NodeKind::FIFO => {
        let mut fifo_mut = self.get_mut::<FIFO>(&def_value).unwrap();
        fifo_mut.remove_user(operand);
      }
      NodeKind::Expr => {
        let mut expr_mut = self.get_mut::<Expr>(&def_value).unwrap();
        expr_mut.remove_user(operand);
      }
      _ => {}
    }
  }

  pub(crate) fn add_user(&mut self, operand: BaseNode) {
    if operand.is_unknown() {
      return;
    }
    let operand_ref = operand.as_ref::<Operand>(self).unwrap();
    let value = *operand_ref.get_value();
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
      let new_value = new_value.map_or(dst, |x| x);
      if let Some(i) = i {
        let mut expr_mut = user.as_mut::<Expr>(self).unwrap();
        expr_mut.set_operand(i, new_value);
      }
    }
  }
}
