use std::collections::HashSet;

use crate::ir::node::IsElement;
use crate::ir::*;

use self::{
  instructions::AsExpr,
  node::{ExprMut, ExprRef, OperandRef, Parented},
  user::Operand,
};

use super::{block::Block, node::BaseNode};

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
  ( $( $opcode:ident $( ( $subcode:ident ) )? ( $fe_method: literal $mn:literal $arity:expr ) => { $($ky:ident),* } ),* $(,)? ) => {

    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum Opcode {
      $( $opcode ),*
    }

    impl ToString for Opcode {
      fn to_string(&self) -> String {
        match self {
          $( Opcode::$opcode => $mn.into() ),*
        }
      }
    }

    impl Opcode {
      pub fn is_valued(&self) -> bool {
        match self {
          $( Opcode::$opcode => find_opcode_attr!(valued; $($ky),*) ),*
        }
      }
      pub fn is_cmp(&self) -> bool {
        match self {
          $( Opcode::$opcode => find_opcode_attr!(cmp; $($ky),*) ),*
        }
      }
      pub fn is_binary(&self) -> bool {
        match self {
          $( Opcode::$opcode => find_opcode_attr!(binary; $($ky),*) ),*
        }
      }
      pub fn is_unary(&self) -> bool {
        match self {
          $( Opcode::$opcode => find_opcode_attr!(unary; $($ky),*) ),*
        }
      }
      pub fn has_side_effect(&self) -> bool {
        match self {
          $( Opcode::$opcode => find_opcode_attr!(side_effect; $($ky),*) ),*
        }
      }
      pub fn fifo_related(&self) -> bool {
        match self {
          $( Opcode::$opcode => find_opcode_attr!(fifo_related; $($ky),*) ),*
        }
      }
      pub fn arity(&self) -> Option<usize> {
        let res = match self {
          $( Opcode::$opcode => $arity ),*
        };
        if res == usize::MAX {
          None
        } else {
          Some(res)
        }
      }
      pub fn from_str(s: &str) -> Option<Opcode> {
        match s {
          $( $fe_method => Some(Opcode::$opcode) ),*,
          _ => None
        }
      }

    }

  };
}

register_opcodes!(
  GetElementPtr("_gep" "gep" 2 /*array, idx*/) => { valued },
  // Memory operations
  Load("_load" "load" 1 /*gep*/) => { valued },
  Store("_store" "store" 2 /*gep value*/) => { side_effect },
  // Binary operations
  Add("add" "+" 2 /*lhs rhs*/) => { binary, valued },
  Sub("sub" "-" 2 /*lhs rhs*/) => { binary, valued },
  Mul("mul" "*" 2 /*lhs rhs*/) => { binary, valued },
  Shl("shl" "<<" 2 /*lhs rhs*/) => { binary, valued },
  Shr("shr" ">>" 2 /*lhs rhs*/) => { binary, valued },
  BitwiseAnd("bitwise_and" "&" 2/*lhs rhs*/) => { binary, valued },
  BitwiseOr("bitwise_or" "|" 2/*lhs rhs*/) => { binary, valued },
  BitwiseXor("bitwise_xor" "^" 2/*lhs rhs*/) => { binary, valued },
  Concat("concat" "concat" 2/*msb lsb*/) => { valued },
  // Comparison operations
  IGT("igt" ">" 2 /*lhs rhs*/) => { cmp, valued },
  ILT("ilt" "<" 2 /*lhs rhs*/) => { cmp, valued },
  IGE("ige" ">=" 2 /*lhs rhs*/) => { cmp, valued },
  ILE("ile" "<=" 2 /*lhs rhs*/) => { cmp, valued },
  EQ("eq" "==" 2 /*lhs rhs*/) => { cmp, valued },
  NEQ("neq" "!=" 2 /*lhs rhs*/) => { cmp, valued },
  // Unary operations
  Neg("neg" "-" 1 /*value*/) => { unary, valued },
  Flip("flip" "!" 1 /*value*/) => { unary, valued },
  // Triary operations
  Select("select" "select" 3 /*cond true_val false_val*/) => { valued },
  // Eventual operations
  Bind("_bind" "bind" usize::MAX /* N/A */) => { valued },
  FIFOPush("push" "push" 2 /*fifo value*/ ) => { side_effect, fifo_related },
  FIFOPop("pop" "pop" 1 /*fifo*/) => { side_effect, valued, fifo_related },
  FIFOPeek("peek" "peek" 1 /*fifo*/) => { valued, fifo_related },
  FIFOValid("valid" "valid" 1 /*fifo*/) => { valued, fifo_related },
  AsyncCall("_async_call" "async_call" usize::MAX /* N/A */) => { side_effect },
  // Other synthesizable operations
  Slice("slice" "slice" 3 /*op [lo, hi]*/) => { valued },
  Cast("cast" "cast" 1 /*value*/) => { valued },
  Sext("sext" "sext" 1 /*value*/) => { valued },
  // Non-synthesizable operations
  Log("_log" "log" usize::MAX /*N/A*/) => { side_effect }
);

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

  pub fn get_name(&self) -> Option<&String> {
    self.name.as_ref()
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
    let parent = self.get().get_parent();
    let block = self.sys.get::<Block>(&parent).unwrap();
    let pos = block.iter().position(|x| *x == self.upcast());
    block.get().get(pos.unwrap()).map(|x| x.clone())
  }
}

impl ExprMut<'_> {
  pub fn move_to_new_parent(&mut self, new_parent: BaseNode, at: Option<usize>) {
    let old_parent = self.get().get_parent();
    let expr = self.get().upcast();
    let mut block_mut = self.sys.get_mut::<Block>(&old_parent).unwrap();
    block_mut.erase(&expr);
    let mut new_parent_mut = self.sys.get_mut::<Block>(&new_parent).unwrap();
    new_parent_mut.insert_at(at, expr);
    self.get_mut().set_parent(new_parent)
  }

  /// Erase the expression from its parent block
  pub fn erase_from_parent(&mut self) {
    // eprintln!(
    //   "erasing {}",
    //   IRPrinter::new(false)
    //     .dispatch(self.sys, &self.get().upcast(), vec![])
    //     .unwrap()
    // );
    // for elem in self.get().users().iter() {
    //   let operand = elem.as_ref::<Operand>(self.sys).unwrap();
    //   let user = IRPrinter::new(false)
    //     .dispatch(self.sys, operand.get_user(), vec![])
    //     .unwrap();
    //   eprintln!("user: {} {:?}", elem.to_string(self.sys), user);
    // }

    assert!(self.get().users().is_empty());
    let parent = self.get().get_parent();
    let expr = self.get().upcast();
    let block = self.sys.get::<Block>(&parent).unwrap();
    let operands = self.get().operands.clone();

    // Remove all the external interfaces related to this instruction.
    let module = block.get_module().upcast();
    let mut module_mut = self.sys.get_mut::<Module>(&module).unwrap();
    module_mut.remove_related_externals(expr);
    for operand in operands.iter() {
      module_mut.remove_related_externals(operand.clone());
    }
    for operand in operands.iter() {
      self.sys.remove_user(operand.clone());
    }

    let mut block_mut = self.sys.get_mut::<Block>(&parent).unwrap();
    block_mut.erase(&expr);

    // Recycle the memory.
    self.sys.dispose(expr);
  }

  /// Unify the implementation of setting and removing an operand.
  fn set_operand_impl(&mut self, i: usize, value: Option<BaseNode>) {
    let block = self.sys.get::<Block>(&self.get().get_parent()).unwrap();
    let module = block.get_module();
    // Remove all the external interfaces related to this instruction.
    let module = module.upcast();
    let expr = self.get().upcast();
    let old = self.get().get_operand(i).unwrap().upcast();
    let operand = value.map(|x| self.sys.insert_element(Operand::new(x)));
    self.sys.remove_user(old);
    let mut module_mut = self.sys.get_mut::<Module>(&module).unwrap();
    module_mut.remove_related_externals(expr);
    if let Some(operand) = operand {
      module_mut.add_related_externals(operand);
      self.get_mut().operands[i] = operand;
      operand
        .as_mut::<Operand>(self.sys)
        .unwrap()
        .get_mut()
        .set_user(expr);
      self.sys.add_user(operand);
    } else {
      self.get_mut().operands.remove(i);
    }
  }

  /// Set the i-th operand to the given value.
  /// NOTE: Just the raw value is given, not the operand wrapper.
  pub fn set_operand(&mut self, i: usize, value: BaseNode) {
    self.set_operand_impl(i, Some(value));
  }

  pub fn remove_operand(&mut self, i: usize) {
    self.set_operand_impl(i, None);
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
