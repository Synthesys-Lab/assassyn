use std::collections::HashMap;

use crate::ir::Module;

use super::{
  data::DataType,
  node::{BaseNode, BindMut, BindRef},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BindKind {
  KVBind,
  Sequential,
  Unknown,
}

/// This is something like a bind function in functional programming.
/// Say, I first have `foo(a, b)`, and then I do `foo5 = bind(foo, {a: 5})`.
/// Calling `foo5(b)` is equivalent to calling `foo(5, b)`.
pub struct Bind {
  pub(crate) key: usize,
  kind: BindKind,
  module: BaseNode,
  bound: HashMap<String, BaseNode>,
}

impl Bind {
  pub(crate) fn new(module: BaseNode, bound: HashMap<String, BaseNode>, kind: BindKind) -> Self {
    Self {
      key: 0,
      kind,
      module,
      bound,
    }
  }

  pub fn get_bound(&self) -> &HashMap<String, BaseNode> {
    &self.bound
  }

  pub fn get_kind(&self) -> BindKind {
    self.kind.clone()
  }
}

impl BindMut<'_> {
  pub fn get_bound_mut(&mut self) -> &mut HashMap<String, BaseNode> {
    &mut self.get_mut().bound
  }

  pub fn set_kind(&mut self, kind: BindKind) {
    self.get_mut().kind = kind;
  }
}

impl BindRef<'_> {
  pub fn full(&self) -> bool {
    let module_ty = self.module.get_dtype(self.sys);
    match &module_ty {
      Some(DataType::Module(types)) => types.len() == self.bound.len(),
      _ => panic!("A module type excepted, but got {:?}", module_ty),
    }
  }

  pub fn to_args(&self) -> Vec<BaseNode> {
    assert!(
      self.full(),
      "Not all arguments are bound, callee: {:?}",
      self.module
    );
    match self.get_kind() {
      BindKind::KVBind => {
        let module = self.module.as_ref::<Module>(self.sys).unwrap();
        module
          .port_iter()
          .map(|x| self.bound.get(x.get_name()).unwrap().clone())
          .collect()
      }
      BindKind::Sequential => (0..self.bound.len())
        .map(|x| self.bound.get(&x.to_string()).unwrap().clone())
        .collect(),
      BindKind::Unknown => {
        assert!(self.get_bound().is_empty());
        vec![]
      }
    }
  }

  pub fn get_callee(&self) -> BaseNode {
    self.module.clone()
  }

  pub fn get_callee_signature(&self) -> DataType {
    self.module.get_dtype(self.sys).unwrap()
  }
}
