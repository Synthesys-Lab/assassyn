use crate::{context::{IsElement, Reference}, data::Typed, expr::Expr};

use super::{port::Input, system::SysBuilder};

pub struct Module {
  pub(crate) key: usize,
  name: String,
  inputs: Vec<Reference>,
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
  pub fn new(name: &str, inputs: Vec<Reference>) -> Module {
    Module {
      key: 0,
      name: name.to_string(),
      inputs,
      dfg: Vec::new(),
      outputs: Vec::new(),
    }
  }

  /// Get the given input reference.
  ///
  /// # Arguments
  ///
  /// * `i` - The index of the input.
  pub fn get_input(&self, i: usize) -> Option<&Reference> {
    self.inputs.get(i)
  }


  // TODO(@were): Later make this implicit.
  pub(crate) fn push(&mut self, expr: Expr) -> Reference {
    self.dfg.push(expr);
    self.dfg.last().unwrap().upcast()
  }

  pub fn to_string(&self, sys: &SysBuilder, ident: usize) -> String {
    let ident = "  ".repeat(ident);
    let mut res = String::new();
    res.push_str(format!("{}module {}(", ident, self.name).as_str());
    for elem in self.inputs.iter() {
      let elem = elem.as_ref::<Input>(sys).unwrap();
      res.push_str(format!("{}: {}, ", elem.name(), elem.dtype().to_string()).as_str());
    }
    res.push_str(") {{\n");
    for elem in self.dfg.iter() {
      res.push_str(format!("{}{}\n", ident, elem.to_string()).as_str());
    }
    res.push_str("  }}\n");
    res
  }

}

