use super::{ctx::{Element, CONTEXT_SLAB}, data::Data};

pub struct Module {
  name: String,
  subscriber: Vec<i32>,
  inputs: Vec<Data>,
  outputs: Vec<Data>,
}

impl Module {

  pub fn new(name: &str, inputs: Vec<Data>) -> &Self {
    let res = Module {
      name: name.to_string(),
      subscriber: Vec::new(),
      inputs,
      outputs: Vec::new(),
    };
    let key = unsafe { CONTEXT_SLAB.insert(Element::Module(res)) };
    match unsafe { CONTEXT_SLAB.get(key).unwrap() } {
      Element::Module(module) => module,
      _ => panic!("Module::new: unexpected element type"),
    }
  }


}

