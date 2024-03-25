use std::collections::HashMap;

use crate::frontend::FIFO;

use super::{
  module::Module,
  node::{BaseNode, BindMut},
};

/// This is something like a bind function in functional programming.
/// Say, I first have `foo(a, b)`, and then I do `foo5 = bind(foo, {a: 5})`.
/// Calling `foo5(b)` is equivalent to calling `foo(5, b)`.
pub struct Bind {
  pub(crate) key: usize,
  module: BaseNode,
  bound: HashMap<String, BaseNode>,
}

impl Bind {
  pub(crate) fn new(module: BaseNode, bound: HashMap<String, BaseNode>) -> Self {
    Self {
      key: 0,
      module,
      bound,
    }
  }

  pub fn get_module(&self) -> &BaseNode {
    &self.module
  }

  pub fn get_bound(&self) -> &HashMap<String, BaseNode> {
    &self.bound
  }
}

impl BindMut<'_> {
  pub fn bind(&mut self, key: String, value: BaseNode) -> BaseNode {
    let mut new_bind = self.get().get_bound().clone();
    let module = self.get().get_module().clone();
    if new_bind.contains_key(&key) {
      panic!("Key {} already exists in bind", key);
    }
    if let Ok(module) = module.as_ref::<Module>(self.sys) {
      let port = module.get_input_by_name(&key).expect(
        format!(
          "\"{}\" is NOT an input of Module \"{}\"",
          key,
          module.get_name()
        )
        .as_str(),
      );
      let port = port.as_ref::<FIFO>(self.sys).unwrap();
      assert_eq!(port.scalar_ty(), value.get_dtype(self.sys).unwrap());
    }
    new_bind.insert(key, value);
    self.sys.create_bind(module, new_bind)
  }
}
