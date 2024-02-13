use crate::{arith::Expr, context::cur_ctx, data::{Input, Typed}};

use super::{context::{Reference, cur_ctx_mut}, data::Output};

pub struct Module {
  pub(crate) key: usize,
  name: String,
  inputs: Vec<Reference>,
  dfg: Vec<Reference>,
  outputs: Vec<Reference>,
}

pub struct Driver { }

impl Module {

  /// Returns a reference to the created new module.
  ///
  /// # Arguments
  ///
  /// * `name` - The name of the module.
  /// * `inputs` - The inputs to the module.
  ///
  /// # Example
  ///
  /// ```
  /// let a = Input::new("a", 32);
  /// Module::new("a_plus_b", vec![a.clone()]);
  /// ```
  pub fn new(name: &str, inputs: Vec<Reference>) -> &Box<Module> {
    let module = Module {
      key: 0,
      name: name.to_string(),
      inputs,
      dfg: Vec::new(),
      outputs: Vec::new(),
    };
    let res = cur_ctx_mut().insert(module);
    cur_ctx_mut().get::<Module>(&res).unwrap().inputs.iter().for_each(|elem| {
      elem.as_mut::<Input>().unwrap().parent = Some(res.clone());
    });
    cur_ctx().get::<Module>(&res).unwrap()
  }

  /// Get the given input reference.
  ///
  /// # Arguments
  ///
  /// * `i` - The index of the input.
  pub fn get_input(&self, i: usize) -> Option<&Box<Input>> {
    self.inputs.get(i).map(|elem| elem.as_ref::<Input>().unwrap())
  }

  // TODO(@were): Check if outputs are set.
  // TODO(@were): Check the given references are with deta.
  // TODO(@were): Check the given references are part of the module.
  pub fn set_output(&mut self, outputs: Vec<Reference>) {
    self.outputs = outputs.into_iter().map(|data| { Output::new(data) }).collect();
    // eprintln!("[{:x}] num outputs: {}", self as * const _ as usize, self.outputs.len());
  }

  // TODO(@were): Later make this implicit.
  pub fn push(&mut self, expr: Reference) -> Reference {
    self.dfg.push(expr.clone());
    expr
  }

  // TODO(@were): This is a temporary solution for proof of concept.
  pub fn elaborate(&self, data: Vec<usize>) {
    // eprintln!("[{:x}] num outputs: {}", self as * const _ as usize, self.outputs.len());
    println!("fn {}(", self.name);
    for elem in self.inputs.iter() {
      let elem = elem.as_ref::<Input>().unwrap();
      println!("  {}: u{},", elem.name(), elem.dtype().bits());
    }
    print!(") -> (");
    // TODO(@were): Fix this hardcoded stuff.
    for _ in self.outputs.iter() {
      print!("u{}, ", 32);
    }
    println!(") {{");
    for elem in self.dfg.iter() {
      let expr = elem.as_ref::<Expr>().unwrap();
      println!("  {}", expr.to_string());
    }
    println!("}}\n");

    println!("fn main() {{");
    print!("  {}(", self.name);
    for elem in data {
      print!("{}, ", elem);
    }
    println!(");");
    println!("}}");

  }

}

