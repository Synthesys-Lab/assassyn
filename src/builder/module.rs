use std::fmt::{Display, Formatter};

use crate::{expr::Expr, context::{cur_ctx, IsElement}, data::Typed};

use super::{context::{cur_ctx_mut, Reference}, event::{Event, EventKind}, port::{Input, Output}};

pub struct Module {
  pub(crate) key: usize,
  pub(crate) parent: Option<Reference>,
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
      parent: None,
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
    Self::get_and_cast(&self.inputs, i)
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
  pub fn push(&mut self, expr: Reference) -> Reference {
    self.dfg.push(expr.clone());
    expr
  }

  /// Trigger another module's instance.
  pub fn trigger(&self, other: &Module, data: Vec<Reference>) -> Event {
    Event::new(self.as_super(), other.as_super(), data, EventKind::Trigger)
  }

  /// Test the condition until it is true and then trigger the given module.
  pub fn spin_trigger(&self, other: &Module, data: Vec<Reference>, cond: Reference) -> Event {
    Event::new(self.as_super(), other.as_super(), data, EventKind::Spin(cond))
  }

  /// Test the condition until it is true and then trigger the given module.
  pub fn cond_trigger(&self, other: &Module, data: Vec<Reference>, cond: Reference) -> Event{
    Event::new(self.as_super(), other.as_super(), data, EventKind::Cond(cond))
  }

}

impl Display for Module {

  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "  module {}(", self.name)?;
    for elem in self.inputs.iter() {
      let elem = elem.as_ref::<Input>().unwrap();
      write!(f, "{}: {}, ", elem.name(), elem.dtype().to_string())?;
    }
    write!(f, ") -> (")?;
    for elem in self.outputs.iter() {
      write!(f, "{}, ", elem.dtype().unwrap().to_string())?;
    }
    write!(f, ") {{\n")?;
    for elem in self.dfg.iter() {
      let expr = elem.as_ref::<Expr>().unwrap();
      write!(f, "    {}\n", expr.to_string())?;
    }
    write!(f, "  }}\n")
  }

}

