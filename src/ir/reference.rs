use crate::{
  builder::system::SysBuilder, data::{Array, IntImm, Typed}, ir::ir_printer::IRPrinter, DataType, Module
};

use super::{block::Block, expr::Expr, port::Input};

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
  fn get_parent(&self) -> Reference;
  fn set_parent(&mut self, parent: Reference);
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
register_element!(Block);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Reference {
  Module(usize),
  Input(usize),
  Expr(usize),
  Array(usize),
  IntImm(usize),
  Block(usize),
  Unknown,
}

impl Reference {
  pub fn get_key(&self) -> usize {
    match self {
      Reference::Module(key)
      | Reference::Input(key)
      | Reference::Expr(key)
      | Reference::Array(key)
      | Reference::Block(key)
      | Reference::IntImm(key) => *key,
      Reference::Unknown => unreachable!("Unknown reference"),
    }
  }

  pub fn get_dtype(&self, sys: &SysBuilder) -> Option<DataType> {
    match self {
      Reference::Module(_) | Reference::Array(_) => None,
      Reference::IntImm(_) => {
        let int_imm = self.as_ref::<IntImm>(sys).unwrap();
        int_imm.dtype().clone().into()
      }
      Reference::Input(_) => {
        let input = self.as_ref::<Input>(sys).unwrap();
        input.dtype().clone().into()
      }
      Reference::Expr(_) => {
        let expr = self.as_ref::<Expr>(sys).unwrap();
        expr.dtype().clone().into()
      }
      Reference::Block(_) => None,
      Reference::Unknown => {
        panic!("Unknown reference")
      }
    }
  }

  pub fn get_parent(&self, sys: &SysBuilder) -> Option<Reference> {
    match self {
      Reference::Module(_) => None,
      Reference::Array(_) => None,
      Reference::IntImm(_) => None,
      Reference::Input(_) => {
        self.as_ref::<Input>(sys).unwrap().get_parent().into()
      }
      Reference::Block(_) => {
        self.as_ref::<Block>(sys).unwrap().get_parent().into()
      }
      Reference::Expr(_) => {
        self.as_ref::<Expr>(sys).unwrap().get_parent().into()
      }
      Reference::Unknown => {
        panic!("Unknown reference")
      }
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
      Reference::IntImm(_) => {
        let int_imm = self.as_ref::<IntImm>(sys).unwrap();
        format!("({} as {})", int_imm.get_value(), int_imm.dtype().to_string())
      }
      Reference::Input(_) => self.as_ref::<Input>(sys).unwrap().get_name().to_string(),
      Reference::Unknown => {
        panic!("Unknown reference")
      }
      Reference::Block(_) => {
        let expr = self.as_ref::<Block>(sys).unwrap();
        IRPrinter::new(sys).visit_block(expr)
      }
      Reference::Expr(key) => {
        format!("_{}", key)
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
  fn visit_block(&mut self, block: &'a Block) -> T;
}

pub enum Element {
  Module(Box<Module>),
  Input(Box<Input>),
  Expr(Box<Expr>),
  Array(Box<Array>),
  IntImm(Box<IntImm>),
  Block(Box<Block>),
}
