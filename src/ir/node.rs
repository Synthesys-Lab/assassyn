use std::ops::Deref;

use crate::builder::SysBuilder;
use crate::ir::*;

use self::instructions::AsExpr;
use self::user::Operand;

use super::super::ir::visitor::Visitor;
use super::ir_printer::IRPrinter;

use instructions::call::LazyBind;
use paste::paste;

pub trait IsElement<'elem, 'sys: 'elem> {
  fn upcast(&self) -> BaseNode;
  fn set_key(&mut self, key: usize);
  fn get_key(&self) -> usize;
  fn into_reference(key: usize) -> BaseNode;
  fn downcast(slab: &'sys slab::Slab<Element>, key: &BaseNode) -> Result<&'elem Self, String>;
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

      impl From<$name> for Element {
        fn from(x: $name) -> Element {
          Element::$name(Box::new(x))
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
        ) -> Result<&'elem $name, String> {
          if let NodeKind::$name = node.get_kind() {
            let key = node.get_key();
            let x = slab.get(key);
            if let Element::$name(res) = x.expect(
              &format!(
                "Invalid slab entry @{} for {}, did you access a disposed value?",
                key,
                stringify!($name)
              )
            ) {
              return Ok(res);
            }
          }
          Err(format!(
            "IsElement::downcast: expecting {}, but {:?}",
            stringify!($name),
            node,
          ))
        }

        fn downcast_mut(
          slab: &'sys mut slab::Slab<Element>,
          node: &BaseNode,
        ) -> Result<&'elem mut Box<$name>, String> {
          if let NodeKind::$name = node.get_kind() {
            let key = node.get_key();
            let x = slab.get_mut(key);
            if let Element::$name(res) = x.expect(
              &format!("Invalid slab entry @{}, did you access a disposed value?", key)
            ) {
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

      #[derive(Clone)]
      pub struct [<$name Ref>] <'a> {
        pub(crate) sys: &'a SysBuilder,
        pub(crate) elem: BaseNode,
      }

      impl<'sys> [<$name Ref>] <'sys> {
        pub fn get<'borrow, 'res>(&'borrow self) -> &'res $name
        where
          'sys: 'borrow,
          'sys: 'res,
          'borrow: 'res,
        {
          <$name>::downcast(&self.sys.slab, &self.elem).unwrap()
        }

      }

      impl Deref for [<$name Ref>]<'_> {
        type Target = $name;

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

        pub fn get<'borrow, 'res>(&'borrow self) -> [<$name Ref>]<'res>
        where
          'sys: 'borrow,
          'sys: 'res,
          'borrow: 'res,
        {
          self.elem.as_ref::<$name>(self.sys).unwrap()
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

    #[derive(Clone, Debug, Eq, PartialEq, Hash, Copy)]
    pub enum NodeKind {
      $($to_register,)*
      Unknown,
    }

    paste! {
      $(

        #[derive(Clone, Debug, Eq, PartialEq, Hash)]
        pub struct [<$to_register Node>] {
          key: usize,
        }

        impl From<BaseNode> for [<$to_register Node>] {
          fn from(x: BaseNode) -> [<$to_register Node>] {
            assert_eq!(x.get_kind(), NodeKind::$to_register);
            [<$to_register Node>] { key: x.get_key() }
          }
        }

        impl From<[<$to_register Node>]> for BaseNode {
          fn from(x: [<$to_register Node>]) -> BaseNode {
            BaseNode::new(NodeKind::$to_register, x.key)
          }
        }

        impl [<$to_register Node>] {

          pub fn as_ref<'elem, 'sys: 'elem>(
            &self,
            sys: &'sys SysBuilder,
          ) -> [<$to_register Ref>]<'sys> {
            $to_register::reference(sys, BaseNode::new(NodeKind::$to_register, self.key)).unwrap()
          }

        }

      )*
    }

    pub enum Element {
      $($to_register(Box<$to_register>),)*
    }
  };

}

register_elements!(Module, FIFO, Expr, Array, IntImm, Block, StrImm, Operand, LazyBind);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Copy)]
pub struct BaseNode {
  kind: NodeKind,
  key: usize,
}

/// Cache the nodes in the system.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum CacheKey {
  IntImm((DataType, u64)),
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
    self.kind
  }

  pub fn is_unknown(&self) -> bool {
    self.kind == NodeKind::Unknown
  }

  pub fn get_dtype(&self, sys: &SysBuilder) -> Option<DataType> {
    match self.kind {
      NodeKind::Module => {
        let module = self.as_ref::<Module>(sys).unwrap();
        module.dtype().clone().into()
      }
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
      NodeKind::StrImm => {
        let str_imm = self.as_ref::<StrImm>(sys).unwrap();
        str_imm.dtype().clone().into()
      }
      NodeKind::Array => None,
      NodeKind::Operand => {
        let operand = self.as_ref::<Operand>(sys).unwrap();
        operand.get_value().get_dtype(sys)
      }
      NodeKind::Unknown | NodeKind::LazyBind => unreachable!(),
    }
  }

  pub fn get_parent(&self, sys: &SysBuilder) -> Option<BaseNode> {
    match self.get_kind() {
      NodeKind::Module => None,
      NodeKind::Array => None,
      NodeKind::IntImm => None,
      NodeKind::StrImm => None,
      NodeKind::FIFO => self.as_ref::<FIFO>(sys).unwrap().get_parent().into(),
      NodeKind::Block => self.as_ref::<Block>(sys).unwrap().get_parent().into(),
      NodeKind::Expr => self.as_ref::<Expr>(sys).unwrap().get_parent().into(),
      NodeKind::Operand => (*self.as_ref::<Operand>(sys).unwrap().get_user()).into(),
      NodeKind::Unknown | NodeKind::LazyBind => unreachable!(),
    }
  }

  pub fn as_ref<'elem, 'sys: 'elem, T: IsElement<'elem, 'sys> + Referencable<'elem, 'sys, T>>(
    &self,
    sys: &'sys SysBuilder,
  ) -> Result<T::Reference, String> {
    T::reference(sys, *self)
  }

  pub fn as_mut<'elem, 'sys: 'elem, T: IsElement<'elem, 'sys> + Mutable<'elem, 'sys, T>>(
    &self,
    sys: &'sys mut SysBuilder,
  ) -> Result<T::Mutator, String> {
    T::mutator(sys, *self)
  }

  pub fn as_expr<'elem, 'sys: 'elem, T: AsExpr<'elem>>(
    &self,
    sys: &'sys SysBuilder,
  ) -> Result<T, String> {
    match self.get_kind() {
      NodeKind::Expr => self.as_ref::<Expr>(sys).unwrap().as_sub::<T>(),
      _ => Err(format!("{:?} is NOT an expression", self)),
    }
  }
}

impl BaseNode {
  pub fn to_string(&self, sys: &SysBuilder) -> String {
    match self.get_kind() {
      NodeKind::Module => self.as_ref::<Module>(sys).unwrap().get_name().to_string(),
      NodeKind::Array => {
        let array = self.as_ref::<Array>(sys).unwrap();
        array.get_name().to_string()
      }
      NodeKind::IntImm => IRPrinter::new(false).dispatch(sys, self, vec![]).unwrap(),
      NodeKind::FIFO => self.as_ref::<FIFO>(sys).unwrap().get_name().to_string(),
      NodeKind::Unknown => {
        panic!("Unknown reference")
      }
      NodeKind::Block => {
        let block = self.as_ref::<Block>(sys).unwrap();
        format!("block: _{}", block.get_key())
      }
      NodeKind::Expr => {
        let expr = self.as_ref::<Expr>(sys).unwrap();
        expr.get_name()
      }
      NodeKind::StrImm => {
        let str_imm = self.as_ref::<StrImm>(sys).unwrap();
        format!("\"{}\"", str_imm.get_value())
      }
      NodeKind::Operand => {
        let operand = self.as_ref::<Operand>(sys).unwrap();
        operand.get_value().to_string(sys)
      }
      NodeKind::LazyBind => unreachable!(),
    }
  }
}
