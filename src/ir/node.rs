use std::ops::Deref;

use crate::{
  builder::system::SysBuilder,
  data::{Array, IntImm, Typed},
  ir::{ir_printer::IRPrinter, visitor::Visitor},
  DataType, Module,
};

use super::{block::Block, expr::Expr, port::FIFO};

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
        BaseNode::new(NodeKind::$name, self.key)
      }

      fn into_reference(key: usize) -> BaseNode {
        BaseNode::new(NodeKind::$name, key)
      }

      fn downcast(
        slab: &'sys slab::Slab<Element>,
        node: &BaseNode,
      ) -> Result<&'elem Box<$name>, String> {
        if let NodeKind::$name = node.get_kind() {
          if let Element::$name(res) = &slab[node.get_key()] {
            return Ok(res);
          }
        }
        Err(format!(
          "IsElement::downcast: expecting {}, {:?}({})",
          stringify!($name),
          node.get_kind(),
          node.get_key()
        ))
      }

      fn downcast_mut(
        slab: &'sys mut slab::Slab<Element>,
        node: &BaseNode,
      ) -> Result<&'elem mut Box<$name>, String> {
        if let NodeKind::$name = node.get_kind() {
          if let Element::$name(res) = &mut slab[node.get_key()] {
            return Ok(res);
          }
        }
        Err(format!(
          "IsElement::downcast: expecting {}, {:?}({})",
          stringify!($name),
          node.get_kind(),
          node.get_key()
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
        if let NodeKind::$name = elem.get_kind() {
          $mutator { sys, elem }
        } else {
          panic!("The reference {:?} is not a {}", elem, stringify!($name));
        }
      }
    }

    impl<'elem, 'sys: 'elem> Referencable<'elem, 'sys, $name> for $name {
      type Reference = $reference<'sys>;

      fn reference(sys: &'sys SysBuilder, elem: BaseNode) -> Self::Reference {
        if let NodeKind::$name = elem.get_kind() {
          $reference { sys, elem }
        } else {
          panic!("The reference {:?} is not a {}", elem, stringify!($name));
        }
      }
    }
  };
}

register_element!(Module, ModuleRef, ModuleMut);
register_element!(FIFO, FIFORef, FIFOMut);
register_element!(Expr, ExprRef, ExprMut);
register_element!(Array, ArrayRef, ArrayMut);
register_element!(IntImm, IntImmRef, IntImmMut);
register_element!(Block, BlockRef, BlockMut);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum NodeKind {
  Module,
  FIFO,
  Expr,
  Array,
  IntImm,
  Block,
  Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct BaseNode {
  kind: NodeKind,
  key: usize,
}

impl BaseNode {
  pub fn new(kind: NodeKind, key: usize) -> Self {
    Self { kind, key }
  }

  pub fn unknown() -> Self {
    Self::new(NodeKind::Unknown, 0)
  }

  pub fn get_key(&self) -> usize {
    self.key
  }

  pub fn get_kind(&self) -> NodeKind {
    self.kind.clone()
  }

  pub fn get_dtype(&self, sys: &SysBuilder) -> Option<DataType> {
    match self.kind {
      NodeKind::Module | NodeKind::Array => None,
      NodeKind::IntImm => {
        let int_imm = self.as_ref::<IntImm>(sys).unwrap();
        int_imm.dtype().clone().into()
      }
      NodeKind::FIFO => {
        let input = self.as_ref::<FIFO>(sys).unwrap();
        input.dtype().clone().into()
      }
      NodeKind::Expr => {
        let expr = self.as_ref::<Expr>(sys).unwrap();
        expr.dtype().clone().into()
      }
      NodeKind::Block => None,
      NodeKind::Unknown => {
        panic!("Unknown reference")
      }
    }
  }

  pub fn get_parent(&self, sys: &SysBuilder) -> Option<BaseNode> {
    match self.get_kind() {
      NodeKind::Module => None,
      NodeKind::Array => None,
      NodeKind::IntImm => None,
      NodeKind::FIFO => self.as_ref::<FIFO>(sys).unwrap().get_parent().into(),
      NodeKind::Block => self.as_ref::<Block>(sys).unwrap().get_parent().into(),
      NodeKind::Expr => self.as_ref::<Expr>(sys).unwrap().get_parent().into(),
      NodeKind::Unknown => {
        panic!("Unknown reference")
      }
    }
  }

  pub fn as_ref<'elem, 'sys: 'elem, T: IsElement<'elem, 'sys> + Referencable<'elem, 'sys, T>> (
    &self,
    sys: &'sys SysBuilder,
  ) -> Result<T::Reference, String> {
    Ok(T::reference(sys, self.clone()))
  }
}

impl BaseNode {
  pub fn to_string(&self, sys: &SysBuilder) -> String {
    match self.get_kind() {
      NodeKind::Module => self.as_ref::<Module>(sys).unwrap().get_name().to_string(),
      NodeKind::Array => {
        let array = self.as_ref::<Array>(sys).unwrap();
        format!("{}", array.get_name())
      }
      NodeKind::IntImm => {
        let int_imm = self.as_ref::<IntImm>(sys).unwrap();
        format!(
          "({} as {})",
          int_imm.get_value(),
          int_imm.dtype().to_string()
        )
      }
      NodeKind::FIFO => self.as_ref::<FIFO>(sys).unwrap().get_name().to_string(),
      NodeKind::Unknown => {
        panic!("Unknown reference")
      }
      NodeKind::Block => {
        let block = self.as_ref::<Block>(sys).unwrap();
        IRPrinter::new(sys).visit_block(&block).unwrap()
      }
      NodeKind::Expr => {
        format!("_{}", self.get_key())
      }
    }
  }
}

pub enum Element {
  Module(Box<Module>),
  FIFO(Box<FIFO>),
  Expr(Box<Expr>),
  Array(Box<Array>),
  IntImm(Box<IntImm>),
  Block(Box<Block>),
}
