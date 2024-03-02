use std::ops::Deref;

use crate::{
  builder::system::SysBuilder,
  data::{Array, IntImm, Typed},
  ir::{ir_printer::IRPrinter, visitor::Visitor},
  DataType, Module,
};

use super::{block::Block, expr::Expr, port::Input};

pub trait IsElement<'elem, 'sys: 'elem> {
  fn upcast(&self) -> BaseNode;
  fn set_key(&mut self, key: usize);
  fn get_key(&self) -> usize;
  fn into_reference(key: usize) -> BaseNode;
  fn downcast(slab: &'sys slab::Slab<Element>, key: &BaseNode) -> Result<&'elem Box<Self>, String>;
  fn downcast_mut(
    slab: &'sys mut slab::Slab<Element>,
    key: &BaseNode,
  ) -> Result<&'elem mut Box<Self>, String>;
}

pub trait Parented {
  fn get_parent(&self) -> BaseNode;
  fn set_parent(&mut self, parent: BaseNode);
}

pub trait Mutable<'elem, 'sys: 'elem, T: IsElement<'elem, 'sys>> {
  type Mutator;
  fn mutator(sys: &'sys mut SysBuilder, elem: BaseNode) -> Self::Mutator;
}

pub trait Referencable<'elem, 'sys: 'elem, T: IsElement<'elem, 'sys>> {
  type Reference;
  fn reference(sys: &'sys SysBuilder, elem: BaseNode) -> Self::Reference;
}

macro_rules! register_element {
  ($name:ident, $reference: ident, $mutator: ident) => {
    impl Into<Element> for $name {
      fn into(self) -> Element {
        Element::$name(Box::new(self))
      }
    }

    impl<'elem, 'sys: 'elem> IsElement<'elem, 'sys> for $name {
      fn set_key(&mut self, key: usize) {
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
        slab: &'sys slab::Slab<Element>,
        key: &BaseNode,
      ) -> Result<&'elem Box<$name>, String> {
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
        slab: &'sys mut slab::Slab<Element>,
        key: &BaseNode,
      ) -> Result<&'elem mut Box<$name>, String> {
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

    pub struct $reference<'sys> {
      pub(crate) sys: &'sys SysBuilder,
      elem: BaseNode,
    }

    impl<'sys> $reference<'sys> {
      pub fn get<'borrow, 'res>(&'borrow self) -> &'res Box<$name>
      where
        'sys: 'borrow,
        'sys: 'res,
        'borrow: 'res,
      {
        <$name>::downcast(&self.sys.slab, &self.elem).unwrap()
      }
    }

    impl Deref for $reference<'_> {
      type Target = Box<$name>;

      fn deref(&self) -> &Self::Target {
        self.get()
      }
    }

    impl<'sys> $mutator<'sys> {
      pub fn get_mut<'borrow>(&'borrow mut self) -> &'borrow mut Box<$name>
      where
        'sys: 'borrow,
      {
        <$name>::downcast_mut(&mut self.sys.slab, &self.elem).unwrap()
      }

      pub fn get<'borrow>(&'borrow self) -> &'borrow Box<$name>
      where
        'sys: 'borrow,
      {
        <$name>::downcast(&self.sys.slab, &self.elem).unwrap()
      }
    }

    impl<'elem, 'sys: 'elem> Mutable<'elem, 'sys, $name> for $name {
      type Mutator = $mutator<'sys>;

      fn mutator(sys: &'sys mut SysBuilder, elem: BaseNode) -> Self::Mutator {
        if let BaseNode::$name(_) = elem {
          $mutator { sys, elem }
        } else {
          panic!("The reference {:?} is not a {}", elem, stringify!($name));
        }
      }
    }

    impl<'elem, 'sys: 'elem> Referencable<'elem, 'sys, $name> for $name {
      type Reference = $reference<'sys>;

      fn reference(sys: &'sys SysBuilder, elem: BaseNode) -> Self::Reference {
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

  pub fn as_ref<'elem, 'sys: 'elem, T: IsElement<'elem, 'sys> + Referencable<'elem, 'sys, T>>(
    &self,
    sys: &'sys SysBuilder,
  ) -> Result<T::Reference, String> {
    Ok(T::reference(sys, self.clone()))
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
        let block = self.as_ref::<Block>(sys).unwrap();
        IRPrinter::new(sys).visit_block(&block)
      }
      BaseNode::Expr(key) => {
        format!("_{}", key)
      }
    }
  }
}

pub enum Element {
  Module(Box<Module>),
  Input(Box<Input>),
  Expr(Box<Expr>),
  Array(Box<Array>),
  IntImm(Box<IntImm>),
  Block(Box<Block>),
}
