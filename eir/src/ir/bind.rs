use crate::ir::{node::Parented, Operand};

use super::{
  data::DataType,
  node::{BaseNode, BindMut, BindRef, IsElement, OperandRef},
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

  pub fn get_kind(&self) -> BindKind {
    self.kind.clone()
  }
}

impl BindMut<'_> {
  fn operandize(&mut self, arg: BaseNode) -> BaseNode {
    let mut operand = Operand::new(arg);
    operand.set_user(self.get().upcast());
    operand.set_parent(self.get().upcast());
    self.sys.insert_element(operand)
  }

  /// Set the i-th argument of the bind function.
  pub fn set_ith_arg(&mut self, i: usize, arg: BaseNode) {
    assert!(self.get().kind == BindKind::Unknown || self.get().kind == BindKind::Sequential);
    self.get_mut().kind = BindKind::Sequential;
    assert!(self.get().args[i].is_none(), "Argument {} already set!", i);
    let arg = self.operandize(arg);
    self.sys.add_user(arg);
    self.get_mut().args[i] = Some(arg);
  }

  /// Set the named argument of the bind function.
  pub fn set_named_arg(&mut self, ky: &String, arg: BaseNode) {
    assert!(self.get().kind == BindKind::Unknown || self.get().kind == BindKind::KVBind);
    let module = self.get().get_callee().as_ref::<Module>(self.sys).unwrap();
    let port = module.get_port_by_name(ky).unwrap_or_else(|| {
      panic!(
        "No such port \"{}\" in module \"{}\"",
        ky,
        module.get_name()
      );
    });
    let idx = port.idx();
    assert!(
      self.get().args[idx].is_none(),
      "Argument \"{}\" already set!",
      ky
    );
    let arg = self.operandize(arg);
    self.sys.add_user(arg);
    self.get_mut().args[idx] = Some(arg);
  }

  pub fn set_kind(&mut self, kind: BindKind) {
    self.get_mut().kind = kind;
  }
}

impl BindRef<'_> {
  pub fn full(&self) -> bool {
    self.args.iter().all(|x| x.is_some())
  }

  pub fn get_callee(&self) -> BaseNode {
    self.module
  }

  pub fn get_callee_signature(&self) -> DataType {
    self.module.get_dtype(self.sys).unwrap()
  }

  pub fn arg_iter(&self) -> impl Iterator<Item = Option<OperandRef<'_>>> {
    self
      .args
      .iter()
      .map(|x| x.map(|x| x.as_ref::<Operand>(self.sys).unwrap()))
  }
}
