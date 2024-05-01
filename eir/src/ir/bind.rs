use super::{
  data::DataType,
  node::{BaseNode, BindMut, BindRef, ModuleRef},
  Module,
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
  args: Vec<Option<BaseNode>>,
}

impl Bind {
  pub(crate) fn new(module: BaseNode, kind: BindKind, n: usize) -> Self {
    Self {
      key: 0,
      kind,
      module,
      args: vec![None; n],
    }
  }

  pub fn get_args(&self) -> &Vec<Option<BaseNode>> {
    &self.args
  }

  pub fn get_kind(&self) -> BindKind {
    self.kind.clone()
  }
}

impl BindMut<'_> {
  pub fn get_args_mut(&mut self) -> &mut Vec<Option<BaseNode>> {
    &mut self.get_mut().args
  }

  pub fn set_kind(&mut self, kind: BindKind) {
    self.get_mut().kind = kind;
  }
}

impl BindRef<'_> {
  pub fn full(&self) -> bool {
    self.args.iter().all(|x| x.is_some())
  }

  pub fn get_callee(&self) -> ModuleRef<'_> {
    self.module.as_ref::<Module>(self.sys).unwrap()
  }

  pub fn get_callee_signature(&self) -> DataType {
    self.module.get_dtype(self.sys).unwrap()
  }
}
