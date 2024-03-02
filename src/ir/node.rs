use crate::{
  builder::system::SysBuilder,
  data::{Array, IntImm, Typed},
  ir::ir_printer::IRPrinter,
  DataType, Module,
};

use super::{block::Block, expr::Expr, port::Input};

pub trait IsElement<'a> {
  fn upcast(&self) -> BaseNode;
  fn set_key(&'a mut self, key: usize);
  fn get_key(&self) -> usize;
  fn into_reference(key: usize) -> BaseNode;
  fn downcast(slab: &'a slab::Slab<Element>, key: &BaseNode) -> Result<&'a Box<Self>, String>;
  fn downcast_mut(
    slab: &'a mut slab::Slab<Element>,
    key: &BaseNode,
  ) -> Result<&'a mut Box<Self>, String>;
}

pub trait Parented {
  fn get_parent(&self) -> BaseNode;
  fn set_parent(&mut self, parent: BaseNode);
}

pub trait Mutable<'a, T: IsElement<'a>> {
  type Mutator;
  fn mutator(sys: &'a mut SysBuilder, elem: BaseNode) -> Self::Mutator;
}

pub trait Referencable<'a, T: IsElement<'a>> {
  type BaseNode;
  fn reference(sys: &'a SysBuilder, elem: BaseNode) -> Self::BaseNode;
}


macro_rules! register_element {

  ($name:ident, $reference: ident, $mutator: ident) => {
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

      fn upcast(&self) -> BaseNode {
        BaseNode::$name(self.key)
      }

      fn into_reference(key: usize) -> BaseNode {
        BaseNode::$name(key)
      }

      fn downcast(
        slab: &'a slab::Slab<Element>,
        key: &BaseNode,
      ) -> Result<&'a Box<$name>, String> {
        if let BaseNode::$name(key) = key {
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
        key: &BaseNode,
      ) -> Result<&'a mut Box<$name>, String> {
        if let BaseNode::$name(key) = key {
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

    pub struct $mutator<'a> {
      pub(crate) sys: &'a mut SysBuilder,
      pub(crate) elem: BaseNode,
    }

    pub struct $reference<'a> {
      sys: &'a SysBuilder,
      elem: BaseNode,
    }

    impl <'a> $reference<'a> {

      pub fn get(&self) -> &Box<$name> {
        <$name>::downcast(&self.sys.slab, &self.elem).unwrap()
      }

    }

    impl <'a> $mutator<'a> {

      pub fn get_mut(&mut self) -> &mut Box<$name> {
        <$name>::downcast_mut(&mut self.sys.slab, &self.elem).unwrap()
      }

      pub fn get(&self) -> &Box<$name> {
        <$name>::downcast(&self.sys.slab, &self.elem).unwrap()
      }

    }

    impl <'a> Mutable<'a, $name> for $name {
      type Mutator = $mutator<'a>;

      fn mutator(sys: &'a mut SysBuilder, elem: BaseNode) -> Self::Mutator {
        if let BaseNode::$name(_) = elem {
          $mutator { sys, elem }
        } else {
          panic!("The reference {:?} is not a {}", elem, stringify!($name));
        }
      }

    }

    impl <'a> Referencable<'a, $name> for $name {
      type BaseNode = $reference<'a>;

      fn reference(sys: &'a SysBuilder, elem: BaseNode) -> Self::BaseNode {
        if let BaseNode::$name(_) = elem {
          $reference { sys, elem }
        } else {
          panic!("The reference {:?} is not a {}", elem, stringify!($name));
        }
      }

    }
  };
}

register_element!(Module, ModuleRef, ModuleMut);
register_element!(Input, InputRef, InputMut);
register_element!(Expr, ExprRef, ExprMut);
register_element!(Array, ArrayRef, ArrayMut);
register_element!(IntImm, IntImmRef, IntImmMut);
register_element!(Block, BlockRef, BlockMut);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum BaseNode {
  Module(usize),
  Input(usize),
  Expr(usize),
  Array(usize),
  IntImm(usize),
  Block(usize),
  Unknown,
}

impl BaseNode {
  pub fn get_key(&self) -> usize {
    match self {
      BaseNode::Module(key)
      | BaseNode::Input(key)
      | BaseNode::Expr(key)
      | BaseNode::Array(key)
      | BaseNode::Block(key)
      | BaseNode::IntImm(key) => *key,
      BaseNode::Unknown => unreachable!("Unknown reference"),
    }
  }

  pub fn get_dtype(&self, sys: &SysBuilder) -> Option<DataType> {
    match self {
      BaseNode::Module(_) | BaseNode::Array(_) => None,
      BaseNode::IntImm(_) => {
        let int_imm = self.as_ref::<IntImm>(sys).unwrap();
        int_imm.dtype().clone().into()
      }
      BaseNode::Input(_) => {
        let input = self.as_ref::<Input>(sys).unwrap();
        input.dtype().clone().into()
      }
      BaseNode::Expr(_) => {
        let expr = self.as_ref::<Expr>(sys).unwrap();
        expr.dtype().clone().into()
      }
      BaseNode::Block(_) => None,
      BaseNode::Unknown => {
        panic!("Unknown reference")
      }
    }
  }

  pub fn get_parent(&self, sys: &SysBuilder) -> Option<BaseNode> {
    match self {
      BaseNode::Module(_) => None,
      BaseNode::Array(_) => None,
      BaseNode::IntImm(_) => None,
      BaseNode::Input(_) => self.as_ref::<Input>(sys).unwrap().get_parent().into(),
      BaseNode::Block(_) => self.as_ref::<Block>(sys).unwrap().get_parent().into(),
      BaseNode::Expr(_) => self.as_ref::<Expr>(sys).unwrap().get_parent().into(),
      BaseNode::Unknown => {
        panic!("Unknown reference")
      }
    }
  }

  pub fn as_ref<'a, T: IsElement<'a>>(&self, sys: &'a SysBuilder) -> Result<&'a Box<T>, String> {
    T::downcast(&sys.slab, self)
  }
}

impl BaseNode {

  pub fn to_string(&self, sys: &SysBuilder) -> String {
    match self {
      BaseNode::Module(_) => self.as_ref::<Module>(sys).unwrap().get_name().to_string(),
      BaseNode::Array(_) => {
        let array = self.as_ref::<Array>(sys).unwrap();
        format!("{}", array.get_name())
      }
      BaseNode::IntImm(_) => {
        let int_imm = self.as_ref::<IntImm>(sys).unwrap();
        format!(
          "({} as {})",
          int_imm.get_value(),
          int_imm.dtype().to_string()
        )
      }
      BaseNode::Input(_) => self.as_ref::<Input>(sys).unwrap().get_name().to_string(),
      BaseNode::Unknown => {
        panic!("Unknown reference")
      }
      BaseNode::Block(_) => {
        let expr = self.as_ref::<Block>(sys).unwrap();
        IRPrinter::new(sys).visit_block(expr)
      }
      BaseNode::Expr(key) => {
        format!("_{}", key)
      }
    }
  }

}

pub trait Visitor<'a, T: Default> {
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

