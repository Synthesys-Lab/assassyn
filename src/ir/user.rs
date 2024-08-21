use std::collections::{HashMap, HashSet};

use crate::builder::SysBuilder;

use super::{node::*, visitor::Visitor, Array, Block, Expr, Module, Opcode, FIFO};

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

pub(crate) struct ExternalInterface {
  external_interfaces: HashMap<BaseNode, HashSet<BaseNode>>,
}

impl ExternalInterface {
  pub(crate) fn new() -> Self {
    Self {
      external_interfaces: HashMap::new(),
    }
  }

  /// Maintain the redundant information, array used in the module.
  ///
  /// # Arguments
  /// * `ext_node` - The external interface node.
  /// * `operand` - The operand node that uses this external interface.
  pub(crate) fn insert_external_interface(&mut self, ext_node: BaseNode, operand: BaseNode) {
    // assert!(
    //   matches!(ext_node.get_kind(), NodeKind::Array | NodeKind::FIFO),
    //   "Expecting Array or FIFO but got {:?}",
    //   ext_node
    // );
    // Next line is equivalent to the following code:
    // if !self.external_interfaces.contains_key(&ext_node) {
    //   self.external_interfaces.insert(ext_node, HashSet::new());
    // }
    self.external_interfaces.entry(ext_node).or_default();
    let users = self.external_interfaces.get_mut(&ext_node).unwrap();
    users.insert(operand);
  }

  /// Iterate over the external interfaces. External interfaces under the context of this project
  /// typically refers to the arrays (both read and write) and FIFOs (typically push)
  /// that are used by the module.
  pub(crate) fn iter(&self) -> impl Iterator<Item = (&BaseNode, &HashSet<BaseNode>)> {
    self.external_interfaces.iter()
  }

  fn remove_external_interface(&mut self, ext_node: BaseNode, operand: BaseNode) {
    if let Some(operations) = self.external_interfaces.get_mut(&ext_node) {
      assert!(operations.contains(&operand));
      operations.remove(&operand);
      if operations.is_empty() {
        self.external_interfaces.remove(&ext_node);
      }
    }
  }
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

impl<'sys> OperandRef<'sys> {
  pub fn get_idx(&self) -> Option<usize> {
    if let Ok(expr) = self.user.as_ref::<Expr>(self.sys) {
      let mut iter = expr.operand_iter();
      Some(iter.position(|x| x.get_key() == self.get_key()).unwrap())
    } else {
      None
    }
  }

  pub fn get_expr<'borrow, 'res>(&self) -> ExprRef<'res>
  where
    'sys: 'res,
    'borrow: 'res,
    'sys: 'borrow,
  {
    self.user.as_ref::<Expr>(self.sys).unwrap()
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
  ($($class:ident),* $(,)?) => {
    paste::paste! {

      $(
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
      )*

      impl SysBuilder {

        fn remove_user(&mut self, operand: &BaseNode) {
          if operand.is_unknown() {
            return;
          }
          let operand_ref = operand.as_ref::<Operand>(self).unwrap();
          let def_value = *operand_ref.get_value();
          match def_value.get_kind() {
            $( NodeKind::$class => {
              let mut mutator = self.get_mut::<$class>(&def_value).unwrap();
              mutator.remove_user(operand);
            } )*
            _ => {}
          }
        }

        fn add_user(&mut self, operand: BaseNode) {
          if operand.is_unknown() {
            return;
          }
          let operand_ref = operand.as_ref::<Operand>(self).unwrap();
          let value = *operand_ref.get_value();
          match value.get_kind() {
            $( NodeKind::$class => {
              let mut mutator = self.get_mut::<$class>(&value).unwrap();
              mutator.add_user(operand);
            } )*
            _ => {}
          }
        }
      }

    }
  };
}

impl_user_methods!(Module, Expr, FIFO, Array);

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

impl ModuleMut<'_> {
  pub(crate) fn insert_external_interface(&mut self, ext_node: BaseNode, operand: BaseNode) {
    self
      .get_mut()
      .external_interface
      .insert_external_interface(ext_node, operand);
  }
}

impl ExprMut<'_> {
  /// Unify the implementation of setting and removing an operand.
  fn set_operand_impl(&mut self, i: usize, value: Option<BaseNode>) {
    let block = self.sys.get::<Block>(&self.get().get_parent()).unwrap();
    let module = block.get_module();
    // Remove all the external interfaces related to this instruction.
    let expr = self.get().upcast();
    if let Some(old) = self.get().operands.get(i) {
      let old = *old;
      self.sys.cut_operand(&old);
      let operand = value.map(|x| self.sys.insert_element(Operand::new(x)));
      if let Some(operand) = operand {
        self.sys.add_related_externals(module, operand);
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
  ///
  /// Arguments
  /// * `i` - The index of the operand.
  /// * `value` - The value to set. NOTE: This is the raw value, not the operand wrapper.
  pub fn set_operand(&mut self, i: usize, value: BaseNode) {
    self.set_operand_impl(i, Some(value));
  }

  pub fn remove_operand(&mut self, i: usize) {
    self.set_operand_impl(i, None);
  }

  /// Push an operand to the end of the operand list.
  ///
  /// Arguments
  /// * `value` - The value to set. NOTE: This is the raw value, not the operand wrapper.
  pub fn push_operand(&mut self, value: BaseNode) {
    self.get_mut().operands.push(BaseNode::unknown());
    self.set_operand(self.get().operands.len() - 1, value);
  }

  /// Insert an operand to the i-th position.
  ///
  /// Arguments
  /// * `i` - The index of the operand.
  /// * `value` - The value to set. NOTE: This is the raw value, not the operand wrapper.
  pub fn insert_operand(&mut self, i: usize, value: BaseNode) {
    self.get_mut().operands.insert(i, BaseNode::unknown());
    self.set_operand(i, value);
  }
}

impl SysBuilder {
  pub fn user_contains_opcode(&self, users: &HashSet<BaseNode>, opcode: Opcode) -> bool {
    users
      .iter()
      .any(|x| x.as_ref::<Operand>(self).unwrap().get_expr().get_opcode() == opcode)
  }

  pub(crate) fn cut_operand(&mut self, operand: &BaseNode) {
    if operand.is_unknown() {
      return;
    }
    let operand_ref = operand.as_ref::<Operand>(self).unwrap();
    let value = *operand_ref.get_value();
    // TODO(@were): Make this a unified interface.
    let module = match operand_ref.get_user().get_kind() {
      NodeKind::Expr => {
        let expr = operand_ref.get_user().as_ref::<Expr>(self).unwrap();
        expr.get_block().get_module()
      }
      _ => unreachable!(),
    };

    let mut module_mut = self.get_mut::<Module>(&module).unwrap();
    module_mut
      .get_mut()
      .external_interface
      .remove_external_interface(value, *operand);

    self.remove_user(operand);
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

  fn add_related_externals(&mut self, module: BaseNode, operand: BaseNode) {
    // Reconnect the external interfaces if applicable.
    // TODO(@were): Maybe later unify a common interface for this.
    let value = {
      let operand_ref = operand.as_ref::<Operand>(self).unwrap();
      *operand_ref.get_value()
    };
    if match value.get_kind() {
      NodeKind::Module | NodeKind::Array => true,
      NodeKind::FIFO => {
        let fifo = value.as_ref::<FIFO>(self).unwrap();
        fifo.get_parent().get_key() != module.get_key()
      }
      NodeKind::Expr => {
        let expr = value.as_ref::<Expr>(self).unwrap();
        // If this expression is NOT in the same module as the user, then it is an external
        // interface.
        expr.get_block().get_module().get_key() != module.get_key()
      }
      _ => false,
    } {
      module
        .as_mut::<Module>(self)
        .unwrap()
        .insert_external_interface(value, operand);
    }
  }
}
