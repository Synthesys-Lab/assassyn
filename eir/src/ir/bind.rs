use std::collections::HashMap;

use super::{
  data::DataType,
  node::{BaseNode, BindRef},
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

impl BindRef<'_> {
  pub fn full(&self) -> bool {
    let module_ty = self.module.get_dtype(self.sys);
    match &module_ty {
      Some(DataType::Module(types)) => types.len() == self.bound.len(),
      _ => panic!("A module type excepted, but got {:?}", module_ty),
    }
  }

  pub fn to_args(&self) -> Vec<BaseNode> {
    assert!(self.full());
    self.bound.values().cloned().collect()
  }

  pub fn get_callee(&self) -> BaseNode {
    self.module.clone()
  }

  pub fn get_callee_signature(&self) -> DataType {
    self.module.get_dtype(self.sys).unwrap()
  }
}
