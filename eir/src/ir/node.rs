use std::ops::Deref;

use crate::frontend::*;

use super::super::ir::visitor::Visitor;
use super::ir_printer::IRPrinter;

use paste::paste;

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
  fn mutator(sys: &'sys mut SysBuilder, elem: BaseNode) -> Result<Self::Mutator, String>;
}

pub trait Referencable<'elem, 'sys: 'elem, T: IsElement<'elem, 'sys>> {
  type Reference;
  fn reference(sys: &'sys SysBuilder, elem: BaseNode) -> Result<Self::Reference, String>;
}

macro_rules! emit_elem_impl {
  ($name:ident) => {
    paste! {
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

      pub struct [<$name Mut>] <'a> {
        pub(crate) sys: &'a mut SysBuilder,
        pub(crate) elem: BaseNode,
      }

      pub struct [<$name Ref>] <'a> {
        pub(crate) sys: &'a SysBuilder,
        pub(crate) elem: BaseNode,
      }

      impl<'sys> [<$name Ref>] <'sys> {
        pub fn get<'borrow, 'res>(&'borrow self) -> &'res Box<$name>
        where
          'sys: 'borrow,
          'sys: 'res,
          'borrow: 'res,
        {
          <$name>::downcast(&self.sys.slab, &self.elem).unwrap()
        }
      }

      impl Deref for [<$name Ref>]<'_> {
        type Target = Box<$name>;

        fn deref(&self) -> &Self::Target {
          self.get()
        }
      }

      impl<'sys> [<$name Mut>]<'sys> {
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
        type Mutator = [<$name Mut>]<'sys>;

        fn mutator(sys: &'sys mut SysBuilder, elem: BaseNode) -> Result<Self::Mutator, String> {
          if let NodeKind::$name = elem.get_kind() {
            Ok([<$name Mut>] { sys, elem })
          } else {
            Err(format!(
              "Expecting {}, but {:?} is given",
              stringify!($name),
              elem
            ))
          }
        }
      }

      impl<'elem, 'sys: 'elem> Referencable<'elem, 'sys, $name> for $name {
        type Reference = [<$name Ref>]<'sys>;

        fn reference(sys: &'sys SysBuilder, elem: BaseNode) -> Result<Self::Reference, String> {
          if let NodeKind::$name = elem.get_kind() {
            Ok([<$name Ref>] { sys, elem })
          } else {
            Err(format!(
              "Expecting {}, but {:?} is given",
              stringify!($name),
              elem
            ))
          }
        }
      }
    }
  };
}

macro_rules! register_elements {
  (emit_impl $elem:ident, $($rest:ident),* $(,)?) => {
    emit_elem_impl!($elem);
    register_elements!(emit_impl $($rest),* );
  };

  (emit_impl $elem:ident) => {
    emit_elem_impl!($elem);
  };


  ($($to_register:ident),* $(,)?) => {
    register_elements!(emit_impl $($to_register),* );

    #[derive(Clone, Debug, Eq, PartialEq, Hash)]
    pub enum NodeKind {
      $($to_register,)*
      Unknown,
    }

    pub enum Element {
      $($to_register(Box<$to_register>),)*
    }

  };
}

register_elements!(Module, FIFO, Expr, Array, IntImm, Block, ArrayPtr, Bind);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct BaseNode {
  kind: NodeKind,
  key: usize,
}

/// Cache the nodes in the system.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum CacheKey {
  IntImm((DataType, u64)),
  ArrayPtr((BaseNode, BaseNode)),
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
      NodeKind::Module => {
        let module = self.as_ref::<Module>(sys).unwrap();
        module.dtype().clone().into()
      }
      NodeKind::Array => None,
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
      NodeKind::Block | NodeKind::ArrayPtr | NodeKind::Bind => None,
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
      NodeKind::ArrayPtr => None,
      NodeKind::Bind => None,
      NodeKind::FIFO => self.as_ref::<FIFO>(sys).unwrap().get_parent().into(),
      NodeKind::Block => self.as_ref::<Block>(sys).unwrap().get_parent().into(),
      NodeKind::Expr => self.as_ref::<Expr>(sys).unwrap().get_parent().into(),
      NodeKind::Unknown => {
        panic!("Unknown reference")
      }
    }
  }

  pub fn as_ref<'elem, 'sys: 'elem, T: IsElement<'elem, 'sys> + Referencable<'elem, 'sys, T>>(
    &self,
    sys: &'sys SysBuilder,
  ) -> Result<T::Reference, String> {
    T::reference(sys, self.clone())
  }

  pub fn as_mut<'elem, 'sys: 'elem, T: IsElement<'elem, 'sys> + Mutable<'elem, 'sys, T>>(
    &self,
    sys: &'sys mut SysBuilder,
  ) -> Result<T::Mutator, String> {
    T::mutator(sys, self.clone())
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
      NodeKind::IntImm => IRPrinter::new(sys).dispatch(sys, self, vec![]).unwrap(),
      NodeKind::FIFO => self.as_ref::<FIFO>(sys).unwrap().get_name().to_string(),
      NodeKind::Unknown => {
        panic!("Unknown reference")
      }
      NodeKind::Block => {
        let block = self.as_ref::<Block>(sys).unwrap();
        IRPrinter::new(sys).visit_block(&block).unwrap()
      }
      NodeKind::ArrayPtr => {
        let handle = self.as_ref::<ArrayPtr>(sys).unwrap();
        let array = handle.get_array();
        let idx = handle.get_idx();
        format!("{}[{}]", array.to_string(sys), idx.to_string(sys))
      }
      NodeKind::Expr => {
        format!("_{}", self.get_key())
      }
      NodeKind::Bind => {
        panic!("Bind should not be printed")
      }
    }
  }
}
