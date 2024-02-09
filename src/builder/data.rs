use crate::Context;

use super::ctx::Reference;

pub struct Data {
  parent: Option<Reference>,
  name: String,
  bits: usize,
}

impl Data {

  pub fn new(ctx: &mut Context, name: &str, bits: usize) -> Reference {
    let res = Data {
      parent: None,
      name: name.to_string(),
      bits,
    };
    ctx.insert(res)
  }

}

