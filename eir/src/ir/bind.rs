use std::collections::HashMap;

use super::node::BaseNode;

/// This is something like a bind function in functional programming.
/// Say, I first have `foo(a, b)`, and then I do `foo5 = bind(foo, {a: 5})`.
/// Calling `foo5(b)` is equivalent to calling `foo(5, b)`.
pub struct Bind {
  pub(crate) key: usize,
  module: BaseNode,
  bound: HashMap<String, BaseNode>,
}
