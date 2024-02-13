use slab;
use once_cell::sync::Lazy;

use crate::{data::{DataType, Typed}, Module};

use super::{ arith::Expr, data::{Input, Output} };

pub(crate) struct Context {
  slab: slab::Slab<Element>,
}

pub trait IsElement<'a> {
  fn as_ref(&self) -> Reference;
  fn into_reference(key: usize) -> Reference;
  fn downcast(slab: &'a slab::Slab<Element>, key: &Reference) -> Result<&'a Box<Self>, String>;
  fn downcast_mut(slab: &'a mut slab::Slab<Element>, key: &Reference) -> Result<&'a mut Box<Self>, String>;
}

pub trait Parented {
  fn parent(&self) -> Option<Reference>;
}

macro_rules! register_element {
  ($name:ident) => {

    impl Into<Element> for $name {
      fn into(self) -> Element {
        Element::$name(Box::new(self))
      }
    }

    impl <'a> IsElement <'a> for $name {

      fn as_ref(&self) -> Reference {
        Reference::$name(self.key)
      }

      fn into_reference(key: usize) -> Reference {
        Reference::$name(key)
      }

      fn downcast(slab: &'a slab::Slab<Element>, key: &Reference) -> Result<&'a Box<$name>, String> {
        if let Reference::$name(key) = key {
          if let Element::$name(res) = &slab[*key] {
            return Ok(res)
          }
        }
        Err(format!("IsElement::downcast: expecting {}, {:?}", stringify!($name), key))
      }

      fn downcast_mut(slab: &'a mut slab::Slab<Element>, key: &Reference)
        -> Result<&'a mut Box<$name>, String> {
        if let Reference::$name(key) = key {
          if let Element::$name(res) = &mut slab[*key] {
            return Ok(res)
          }
        }
        Err(format!("IsElement::downcast: expecting {}, {:?}", stringify!($name), key))
      }


    }

  };
}

register_element!(Module);
register_element!(Input);
register_element!(Output);
register_element!(Expr);

#[derive(Clone, Debug)]
pub enum Reference {
  Module(usize),
  Input(usize),
  Output(usize),
  Expr(usize),
}

impl <'a>Reference {

  pub fn as_ref<T: IsElement<'a>>(&self) -> Result<&'a Box<T>, String> {
    cur_ctx().get::<T>(self)
  }

  pub fn as_mut<T: IsElement<'a>>(&self) -> Result<&'a mut Box<T>, String> {
    cur_ctx_mut().get_mut::<T>(self)
  }

  pub fn dtype(&self) -> Result<DataType, String> {
    match self {
      Reference::Input(_) => {
        Ok(self.as_ref::<Input>().unwrap().dtype().clone())
      }
      Reference::Output(_) => {
        self.as_ref::<Output>().unwrap().data.dtype()
      }
      Reference::Expr(_) => {
        Ok(self.as_ref::<Expr>().unwrap().dtype().clone())
      }
      _ => Err(format!("No type for {:?}", self))
    }
  }

}

impl ToString for Reference {

  fn to_string(&self) -> String {
    match self {
      Reference::Input(_) => {
        self.as_ref::<Input>().unwrap().name().clone()
      }
      Reference::Expr(key) => {
        format!("_{}", key)
      }
      _ => unreachable!("Not supported yet {:?}", *self)
    }
  }

}

pub enum Element {
  Module(Box<Module>),
  Input(Box<Input>),
  Output(Box<Output>),
  Expr(Box<Expr>),
}

impl Element {

  fn set_key(&mut self, key: usize) {
    match self {
      Element::Module(module) => { module.key = key; }
      Element::Input(data) => { data.key = key; }
      Element::Output(data) => { data.key = key; }
      Element::Expr(expr) => { expr.key = key; }
    }
  }

}

impl <'a>Context {

  pub fn new() -> Self {
    Context {
      slab: slab::Slab::new(),
    }
  }

  pub fn insert<T: Into<Element> + IsElement<'a>>(&mut self, elem: T) -> Reference {
    let key = self.slab.insert(elem.into());
    self.slab.get_mut(key).unwrap().set_key(key);
    T::into_reference(key)
  }

  pub fn get<T: IsElement<'a>>(&'a self, key: &Reference) -> Result<&'a Box<T>, String> {
    T::downcast(&self.slab, key)
  }

  pub fn get_mut<T: IsElement<'a>>(&'a mut self, key: &Reference) -> Result<&'a mut Box<T>, String> {
    T::downcast_mut(&mut self.slab, key)
  }

}

static mut CONTEXT: Lazy<Context> = Lazy::new(|| { Context::new() });

pub(crate) fn cur_ctx_mut() -> &'static mut Context {
  unsafe { &mut CONTEXT }
}

pub(crate) fn cur_ctx() -> &'static Context {
  unsafe { &CONTEXT }
}

