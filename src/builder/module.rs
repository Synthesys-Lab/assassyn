use std::fmt::{Display, Formatter};

use crate::{context::{IsElement, Reference}, data::Typed, expr::Expr};

use super::{port::{Input, Output}, system::PortInfo};

pub struct Module {
  pub(crate) key: usize,
  name: String,
  inputs: Vec<Box<Input>>,
  dfg: Vec<Expr>,
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
  pub fn new(name: &str, inputs: Vec<PortInfo>) -> Module {
    let inputs = inputs.into_iter().map(|x| Input::new(&x.ty, x.name.as_str()).into()).collect();
    Module {
      key: 0,
      name: name.to_string(),
      inputs,
      dfg: Vec::new(),
      outputs: Vec::new(),
    }
  }

  /// Get the required element from the given vector and cast it to the required type.
  ///
  /// # Arguments
  ///
  /// `v` - The vector of references.
  /// `i` - The index of the element.
  fn get_and_cast<'a, T: IsElement<'a>>(v: &'a Vec<Reference>, i: usize) -> Option<&Box<T>> {
    v.get(i).map(|elem| elem.as_ref::<T>().unwrap())
  }

  /// Get the given input reference.
  ///
  /// # Arguments
  ///
  /// * `i` - The index of the input.
  pub fn get_input(&self, i: usize) -> Option<&Box<Input>> {
    self.inputs.get(i)
  }

  /// Get the given output reference.
  ///
  /// # Arguments
  ///
  /// * `i` - The index of the outout.
  pub fn get_output(&self, i: usize) -> Option<&Box<Output>> {
    Self::get_and_cast(&self.outputs, i)
  }

  // TODO(@were): Check if outputs are set.
  // TODO(@were): Check the given references are with deta.
  // TODO(@were): Check the given references are part of the module.
  pub fn set_outputs(&mut self, outputs: Vec<Reference>) {
    self.outputs = outputs.into_iter().map(|data| { Output::new(data) }).collect();
  }

  // TODO(@were): Later make this implicit.
  pub(crate) fn push(&mut self, expr: Expr) -> Reference {
    self.dfg.push(expr);
    self.dfg.last().unwrap().as_super()
  }

}

impl Display for Module {

  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "  module {}(", self.name)?;
    for elem in self.inputs.iter() {
      write!(f, "{}: {}, ", elem.name(), elem.dtype().to_string())?;
    }
    write!(f, ") -> (")?;
    for elem in self.outputs.iter() {
      write!(f, "{}, ", elem.dtype().unwrap().to_string())?;
    }
    write!(f, ") {{\n")?;
    for elem in self.dfg.iter() {
      write!(f, "    {}\n", elem.to_string())?;
    }
    write!(f, "  }}\n")
  }

}

