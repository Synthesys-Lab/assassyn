use crate::{
  data::{Array, IntImm},
  Module,
};

use super::{expr::Expr, port::Input, system::SysBuilder};

pub trait IsElement<'a> {
  fn upcast(&self) -> Reference;
  fn set_key(&'a mut self, key: usize);
  fn get_key(&self) -> usize;
  fn into_reference(key: usize) -> Reference;
  fn downcast(slab: &'a slab::Slab<Element>, key: &Reference) -> Result<&'a Box<Self>, String>;
  fn downcast_mut(
    slab: &'a mut slab::Slab<Element>,
    key: &Reference,
  ) -> Result<&'a mut Box<Self>, String>;
}

pub trait Parented {
  fn parent(&self) -> Reference;
}

macro_rules! register_element {
  ($name:ident) => {
    impl Into<Element> for $name {
      fn into(self) -> Element {
        Element::$name(Box::new(self))
      }
    }

    impl<'a> IsElement<'a> for $name {
      fn set_key(&'a mut self, key: usize) {
        self.key = key;
      }

      fn get_key(&self) -> usize {
        self.key
      }

      fn upcast(&self) -> Reference {
        Reference::$name(self.key)
      }

      fn into_reference(key: usize) -> Reference {
        Reference::$name(key)
      }

      fn downcast(
        slab: &'a slab::Slab<Element>,
        key: &Reference,
      ) -> Result<&'a Box<$name>, String> {
        if let Reference::$name(key) = key {
          if let Element::$name(res) = &slab[*key] {
            return Ok(res);
          }
        }
        Err(format!(
          "IsElement::downcast: expecting {}, {:?}",
          stringify!($name),
          key
        ))
      }

      fn downcast_mut(
        slab: &'a mut slab::Slab<Element>,
        key: &Reference,
      ) -> Result<&'a mut Box<$name>, String> {
        if let Reference::$name(key) = key {
          if let Element::$name(res) = &mut slab[*key] {
            return Ok(res);
          }
        }
        Err(format!(
          "IsElement::downcast: expecting {}, {:?}",
          stringify!($name),
          key
        ))
      }
    }
  };
}

register_element!(Module);
register_element!(Input);
register_element!(Expr);
register_element!(Array);
register_element!(IntImm);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Reference {
  Module(usize),
  Input(usize),
  Output(usize),
  Expr(usize),
  Array(usize),
  SysBuilder(usize),
  ArrayRead(usize),
  ArrayWrite(usize),
  IntImm(usize),
  Unknown,
}

impl Reference {
  pub fn get_key(&self) -> usize {
    match self {
      Reference::Module(key)
      | Reference::Input(key)
      | Reference::Output(key)
      | Reference::Expr(key)
      | Reference::Array(key)
      | Reference::SysBuilder(key)
      | Reference::ArrayRead(key)
      | Reference::ArrayWrite(key)
      | Reference::IntImm(key) => *key,
      Reference::Unknown => unreachable!("Unknown reference"),
    }
  }

  pub fn as_ref<'a, T: IsElement<'a>>(&self, sys: &'a SysBuilder) -> Result<&'a Box<T>, String> {
    T::downcast(&sys.slab, self)
  }
}

impl Reference {
  pub fn to_string(&self, sys: &SysBuilder) -> String {
    match self {
      Reference::Module(_) => self.as_ref::<Module>(sys).unwrap().get_name().to_string(),
      Reference::Array(_) => {
        let array = self.as_ref::<Array>(sys).unwrap();
        format!("{}", array.get_name())
      }
      Reference::IntImm(_) => self.as_ref::<IntImm>(sys).unwrap().to_string(),
      Reference::Input(_) => self.as_ref::<Input>(sys).unwrap().get_name().to_string(),
      Reference::Unknown => {
        panic!("Unknown reference")
      }
      _ => {
        format!("_{}", self.get_key())
      }
    }
  }
}

pub trait Visitor<'a, T> {
  fn visit_module(&mut self, module: &'a Module) -> T;
  fn visit_input(&mut self, input: &'a Input) -> T;
  fn visit_expr(&mut self, expr: &'a Expr) -> T;
  fn visit_array(&mut self, array: &'a Array) -> T;
  fn visit_int_imm(&mut self, int_imm: &'a IntImm) -> T;
}

pub enum Element {
  Module(Box<Module>),
  Input(Box<Input>),
  Expr(Box<Expr>),
  Array(Box<Array>),
  IntImm(Box<IntImm>),
}
