use super::{ctx::{Context, Reference}, data::Data};

pub struct Module {
  pub(crate) key: usize,
  name: String,
  subscriber: Vec<i32>,
  inputs: Vec<Data>,
  outputs: Vec<Data>,
}

impl Module {

  pub fn new(ctx: &mut Context, name: &str, inputs: Vec<Data>) -> Reference {
    let mut res = Module {
      key: 0,
      name: name.to_string(),
      subscriber: Vec::new(),
      inputs,
      outputs: Vec::new(),
    };
    ctx.insert(res)
  }


}

