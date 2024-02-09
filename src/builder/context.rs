use slab;

use crate::{Data, Module};

use super::data::Expr;

pub enum Element {
  Module(Module),
  Data(Data),
  Expr(Expr),
}

impl Element {

  fn set_key(&mut self, key: usize) {
    match self {
      Element::Module(module) => { module.key = key; }
      _ => panic!("Element::set_key: unexpected element type"),
    }
  }

}

pub struct Context {
  slab: slab::Slab<Element>,
}

pub trait IsElement {
  fn into_reference(key: usize) -> Reference;
}

macro_rules! register_element {
  ($name:ident) => {

    impl Into<Element> for $name {
      fn into(self) -> Element {
        Element::$name(self)
      }
    }
    
    impl IsElement for $name {
      fn into_reference(key: usize) -> Reference {
        Reference::Module(key)
      }
    }

  };
}

register_element!(Module);
register_element!(Data);


pub enum Reference {
  Module(usize),
  Data(usize),
  Expr(usize),
}

impl Context {

  pub fn new() -> Self {
    Context {
      slab: slab::Slab::new(),
    }
  }

  pub fn insert<T: Into<Element> + IsElement>(&mut self, elem: T) -> Reference {
    let key = self.slab.insert(elem.into());
    self.slab.get_mut(key).unwrap().set_key(key);
    T::into_reference(key)
  }

}

