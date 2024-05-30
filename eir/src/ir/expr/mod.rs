use std::collections::HashSet;

use node::NodeKind;

use crate::ir::node::IsElement;
use crate::ir::*;

use self::{
  instructions::AsExpr,
  node::{ExprMut, ExprRef, OperandRef, Parented},
  user::Operand,
};

use super::{block::Block, node::BaseNode};

pub mod subcode;

#[derive(Clone, Debug, Eq, PartialEq, Copy, Hash)]
pub enum BindKind {
  KVBind,
  Sequential,
  Unknown,
}

macro_rules! find_opcode_attr {
  ( $target:ident; $($ky:ident),* ) => {
    find_opcode_attr!(@find $target ; $($ky),*)
  };

  (@find $target:ident ; $first:ident, $($rest:ident),*) => {
    stringify!($target) == stringify!($first) || find_opcode_attr!(@find $target ; $($rest),*)
  };

  (@find $target:expr ; $ky:ident) => {
    stringify!($target) == stringify!($ky)
  };
}

macro_rules! register_opcodes {
  ( $( $var_id:ident ( $( { $subcode:ident : $subty:ty }, )? $( $mn:ident, )? $arity:literal ) =>
       { $( $attrs:ident ),* $(,)? } ),* ) => {

    #[derive(Clone, Debug, Eq, PartialEq, Copy, Hash)]
    pub enum Opcode {
      $( $var_id $( { $subcode : $subty } )? ),*
    }

    impl ToString for Opcode {
      fn to_string(&self) -> String {
        match self {
          $( Opcode::$var_id $( { $subcode } )?  => {
            $( $subcode.to_string() )?
            $( stringify!($mn).to_string() )?
          }),*
        }
      }
    }

    impl ToString for ExprRef<'_> {
      fn to_string(&self) -> String {
        match self.get_opcode() {
          $( Opcode::$var_id $( { $subcode } )? => {
            $( let _ = $subcode; )?
            let sub = self.clone().as_sub::<instructions::$var_id>().unwrap();
            sub.to_string()
          })*
        }
      }
    }


    paste::paste!{
      impl Opcode {
        pub fn from_str(s: &str) -> Option<Opcode> {
          if let Some(general) = match s {
            $( $( stringify!($mn) => Some(Opcode::$var_id), )? )*
            _ => None,
          } {
            return Some(general)
          }
          $( $(if let Some(sub) = $subty::from_str(s) {
            return Some(sub.into())
          })? )*
          None
        }

        pub fn is_valued(&self) -> bool {
          match self {
            $( Opcode::$var_id $( { $subcode } )? => {
              $( let _ = $subcode; )?
              find_opcode_attr!(valued; $($attrs),*)
            })*
          }
        }
        pub fn has_side_effect(&self) -> bool {
          match self {
            $( Opcode::$var_id $( { $subcode } )? => {
              $( let _ = $subcode; )?
              find_opcode_attr!(side_effect; $($attrs),*)
            }),*
          }
        }
        pub fn arity(&self) -> Option<usize> {
          let res : i64 = match self {
            $( Opcode::$var_id $( { $subcode } )? => {
              $( let _ = $subcode; )?
              $arity
            } ),*
          };
          if res == -1 {
            None
          } else {
            Some(res as usize)
          }
        }
      }
    }

    $($(
      impl From<$subty> for Opcode {
        fn from(s: $subty) -> Self {
          Opcode::$var_id {  $subcode: s }
        }
      }
    )?)*

  };
}

// A declared opcode should have the following fields:
// variant_name: ident
// (
//   { subcode:ident : subty:ty }? // Optional 1
//   { mnemonic:ident, }?          // Optional 2
//   arity:literal                 // Mandatory, the number of operands, -1 for variadic
//   =>
//   { valued, side_effect }       // Optional 3
// )
//
// NOTE: Optional 1 and 2 are exclusive but one of them is mandatory!
register_opcodes!(
  // Memory operations
  Load(load, 1 /*gep*/) => { valued },
  Store(store, 2 /*gep value*/) => { side_effect },
  // Arith operations
  Binary({ binop: subcode::Binary }, 2 /*lhs rhs*/) => { valued },
  Unary({ uop: subcode::Unary }, 1 /*value*/) => { valued },
  Select(select, 3 /*cond true_val false_val*/) => { valued },
  Select1Hot(select_1hot, -1) => { valued },
  Compare({ cmp: subcode::Compare }, 2 /*lhs rhs*/) => { valued },
  // Eventual operations
  Bind(bind, 1 /*value*/) => { valued },
  FIFOPush(push, 2 /*fifo value*/) => { side_effect },
  FIFOPop(pop, 1 /*fifo*/) => { side_effect, valued },
  FIFOField({ field: subcode::FIFO }, 1 /*fifo*/) => { valued },
  AsyncCall(async_call, -1 /* N/A */) => { side_effect },
  // Other synthesizable operations
  Slice(slice, 3 /*op [lo, hi]*/) => { valued },
  Cast({ cast: subcode::Cast }, 1 /*value*/) => { valued },
  Concat(concat, 2/*msb lsb*/) => { valued },
  // Block intrinsics
  BlockIntrinsic({ intrinsic: subcode::BlockIntrinsic }, -1 /*N/A*/) => { side_effect },
  // Non-synthesizable operations
  Log(log, -1 /*N/A*/) => { side_effect }
);

impl Opcode {
  pub fn is_binary(&self) -> bool {
    match self {
      &Opcode::Binary { .. } => true,
      _ => false,
    }
  }

  pub fn is_cmp(&self) -> bool {
    match self {
      &Opcode::Compare { .. } => true,
      _ => false,
    }
  }

  pub fn is_unary(&self) -> bool {
    match self {
      &Opcode::Unary { .. } => true,
      _ => false,
    }
  }
}

pub struct Expr {
  name: Option<String>,
  pub(super) key: usize,
  parent: BaseNode,
  dtype: DataType,
  opcode: Opcode,
  pub(crate) operands: Vec<BaseNode>,
  pub(crate) user_set: HashSet<BaseNode>,
}

impl Expr {
  pub(crate) fn new(
    dtype: DataType,
    opcode: Opcode,
    operands: Vec<BaseNode>,
    parent: BaseNode,
  ) -> Self {
    Self {
      name: None,
      key: 0,
      parent,
      dtype,
      opcode,
      operands,
      user_set: HashSet::new(),
    }
  }

  pub fn get_opcode(&self) -> Opcode {
    self.opcode.clone()
  }

  pub fn get_num_operands(&self) -> usize {
    self.operands.len()
  }

  pub fn get_name(&self) -> String {
    if let Some(x) = self.name.clone() {
      x
    } else {
      format!("_{}", self.key)
    }
  }
}

impl Typed for Expr {
  fn dtype(&self) -> DataType {
    self.dtype.clone()
  }
}

impl Parented for Expr {
  fn get_parent(&self) -> BaseNode {
    self.parent.clone()
  }

  fn set_parent(&mut self, parent: BaseNode) {
    self.parent = parent;
  }
}

impl<'a> ExprRef<'a> {
  pub fn as_sub<T: AsExpr<'a>>(self) -> Result<T, String> {
    T::downcast(self)
  }
}

impl ExprRef<'_> {
  pub fn get_operand_value(&self, i: usize) -> Option<BaseNode> {
    self.get_operand(i).map(|x| x.get_value().clone())
  }

  pub fn get_operand(&self, i: usize) -> Option<OperandRef<'_>> {
    self
      .operands
      .get(i)
      .map(|x| x.as_ref::<Operand>(self.sys).unwrap())
  }

  pub fn operand_iter(&self) -> impl Iterator<Item = OperandRef<'_>> {
    self
      .operands
      .iter()
      .map(|x| x.as_ref::<Operand>(self.sys).unwrap())
  }

  // Get the next expression in the block
  pub fn next(&self) -> Option<BaseNode> {
    let idx = self.idx();
    let block = self.get_parent().as_ref::<Block>(self.sys).unwrap();
    block.get().get(idx).map(|x| x.clone())
  }

  // The index of the instruction in its parent block
  pub fn idx(&self) -> usize {
    let parent = self.get().get_parent();
    let block = self.sys.get::<Block>(&parent).unwrap();
    let mut iter = block.body_iter();
    iter
      .position(|x| self.get_key() == x.get_key() && matches!(x.get_kind(), NodeKind::Expr))
      .unwrap()
  }
}

impl ExprMut<'_> {
  /// Erase the expression from its parent block
  pub fn erase_from_parent(&mut self) {
    assert!(self.get().users().is_empty());
    let parent = self.get().get_parent();
    let expr = self.get().upcast();
    let operands = self.get().operands.clone();

    // Remove all the external interfaces related to this instruction.
    for operand in operands.iter() {
      self.sys.cut_operand(operand);
    }

    let mut block_mut = self.sys.get_mut::<Block>(&parent).unwrap();
    block_mut.erase(&expr);

    // Recycle the memory.
    self.sys.dispose(expr);
  }

  pub fn set_name(&mut self, name: String) {
    let name = {
      let module = self
        .get()
        .get_parent()
        .as_ref::<Block>(self.sys)
        .unwrap()
        .get_module()
        .upcast();
      let mut module_mut = module.as_mut::<Module>(self.sys).unwrap();
      module_mut.get_mut().symbol_table.identifier(&name)
    };
    self.get_mut().name = Some(name);
  }
}
