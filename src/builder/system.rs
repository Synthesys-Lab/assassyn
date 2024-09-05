use std::{collections::HashMap, fmt::Display, hash::Hash};

use instructions::call::LazyBind;

use crate::ir::{ir_printer::IRPrinter, node::*, visitor::Visitor, *};

use self::expr::subcode::{self, Binary};

use super::symbol_table::SymbolTable;

pub enum ModuleKind {
  Module,
  Downstream,
  All,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InsertPoint {
  pub module: BaseNode,
  pub block: BaseNode,
  pub at: Option<usize>,
}

impl InsertPoint {
  pub fn next(&self, sys: &SysBuilder) -> Option<Self> {
    let (module, block, at) = { (self.module, self.block, self.at) };
    let block = block.as_ref::<Block>(sys).unwrap();
    if let Some(cur_at) = at {
      if cur_at + 1 < block.get_num_exprs() {
        Some(InsertPoint {
          module,
          block: block.upcast(),
          at: (cur_at + 1).into(),
        })
      } else {
        Some(InsertPoint {
          module,
          block: block.upcast(),
          at: None,
        })
      }
    } else if let Some(nxt_block) = block.next() {
      Some(InsertPoint {
        module,
        block: nxt_block,
        at: 0.into(),
      })
    } else if let Ok(block_parent) = block.get_parent().as_ref::<Block>(sys) {
      return Some(InsertPoint {
        module,
        block: block_parent.upcast(),
        at: None,
      });
    } else {
      return None;
    }
  }
}

/// A `SysBuilder` struct not only serves as the data structure of the whole system,
/// but also works as the syntax-sugared IR builder.
pub struct SysBuilder {
  /// The slab to store all the elements in the system. We use a slab to maintain such a
  /// highly redundant, and mutually referenced data structure.
  pub(crate) slab: slab::Slab<Element>,
  /// The data structure caches the constant values.
  cached_nodes: HashMap<CacheKey, BaseNode>,
  /// The name of the system.
  name: String,
  /// The current module to be built.
  pub(crate) inesert_point: InsertPoint,
  /// The symbol table to maintain the unique identifiers.
  pub(crate) symbol_table: SymbolTable,
  /// The set of finalized binds. A lazy bind will not be instantiated as a bind expression until
  /// it is called. Key: LazyBind, Value: Expr::Bind.
  finalized_binds: HashMap<BaseNode, BaseNode>,
}

/// The information of an input of a module.
/// We do not want to expose port constructors to the user, because ports are meaningless
/// without a module.
pub struct PortInfo {
  pub name: String,
  pub ty: DataType,
}

impl PortInfo {
  /// Create a new port info.
  /// # Arguments
  ///
  /// * `name` - The name of the port.
  /// * `ty` - The data type of the port.
  pub fn new(name: &str, ty: DataType) -> Self {
    Self {
      name: name.into(),
      ty,
    }
  }
}

/// Create a binary operation expression.
///
/// # Arguments
/// * `ty` - The result's data type of the expression. If None is given, the data type will be
///    inferred from the operands.
/// * `a` - The first operand.
/// * `b` - The second operand.
/// * `pred` - The condition of executing this expression. If the condition is not `None`, this
///    is always executed.
macro_rules! create_arith_op_impl {
  (binary, $func_name:ident, $opcode: expr) => {
    pub fn $func_name(&mut self, a: BaseNode, b: BaseNode) -> BaseNode {
      match self.combine_types($opcode, &a, &b) {
        Ok(res_ty) => self.create_expr(res_ty, $opcode, vec![a, b], true),
        Err(msg) => panic!("{}", msg),
      }
    }
  };

  (unary, $func_name:ident, $opcode: expr) => {
    pub fn $func_name(&mut self, x: BaseNode) -> BaseNode {
      let res_ty = x.get_dtype(self).unwrap_or_else(|| {
        panic!("{} has no type!", x.to_string(self));
      });
      self.create_expr(res_ty, $opcode, vec![x.clone()], true)
    }
  };
}

impl SysBuilder {
  /// Create a new system.
  /// # Arguments
  ///
  /// * `name` - The name of the system.
  pub fn new(name: &str) -> Self {
    Self {
      name: name.into(),
      slab: slab::Slab::new(),
      cached_nodes: HashMap::new(),
      inesert_point: InsertPoint {
        module: BaseNode::unknown(),
        block: BaseNode::unknown(),
        at: None,
      },
      symbol_table: SymbolTable::new(),
      finalized_binds: HashMap::new(),
    }
  }

  pub fn get_name(&self) -> &str {
    &self.name
  }

  /// If this system has a driver.
  pub fn has_driver(&self) -> bool {
    self
      .module_iter(ModuleKind::Module)
      .any(|x| x.get_name().eq("driver"))
  }

  /// If this system has a testbench.
  pub fn has_testbench(&self) -> bool {
    self
      .module_iter(ModuleKind::Module)
      .any(|x| x.get_name().eq("testbench"))
  }

  /// The helper function to get an element of the system and downcast it to its actual
  /// type's immutable reference.
  pub(crate) fn get<
    'elem,
    'sys: 'elem,
    T: IsElement<'elem, 'sys> + Referencable<'elem, 'sys, T>,
  >(
    &'sys self,
    key: &BaseNode,
  ) -> Result<T::Reference, String> {
    T::reference(self, *key)
  }

  pub fn array_iter(&self) -> impl Iterator<Item = ArrayRef<'_>> {
    self
      .symbol_table
      .symbols()
      .filter_map(|v| v.as_ref::<Array>(self).ok())
  }

  /// Get the iterator of the modules.
  ///
  /// # Arguments
  /// * `downstream` - If true, the iterator will only return downstream modules.
  ///   If false, the iterator will only return upstream modules.
  ///   If None, the iterator will return both modules.
  pub fn module_iter(&self, kind: ModuleKind) -> impl Iterator<Item = ModuleRef<'_>> {
    self.symbol_table.symbols().filter_map(move |v| {
      v.as_ref::<Module>(self).ok().filter(|m| match kind {
        ModuleKind::Module => !m.is_downstream(),
        ModuleKind::Downstream => m.is_downstream(),
        ModuleKind::All => true,
      })
    })
  }

  /// The helper function to get an element of the system and downcast it to its actual type's
  /// mutable reference.
  pub(crate) fn get_mut<'elem, 'sys: 'elem, T: IsElement<'elem, 'sys> + Mutable<'elem, 'sys, T>>(
    &'sys mut self,
    key: &BaseNode,
  ) -> Result<T::Mutator, String> {
    T::mutator(self, *key)
  }

  /// Get the current module to be built.
  pub fn get_current_module(&self) -> Result<ModuleRef<'_>, String> {
    self.get::<Module>(&self.inesert_point.module)
  }

  pub fn get_current_block(&self) -> Result<BlockRef<'_>, String> {
    self.get::<Block>(&self.inesert_point.block)
  }

  /// Get the module by its name.
  pub fn get_module<'a>(&'a self, name: &str) -> Option<ModuleRef<'a>> {
    if let Some(reference) = self.symbol_table.get(name) {
      reference.as_ref::<Module>(self).unwrap().into()
    } else {
      None
    }
  }

  /// Get the array by its name.
  pub fn get_array<'a>(&'a self, name: &str) -> Option<ArrayRef<'a>> {
    if let Some(reference) = self.symbol_table.get(name) {
      reference.as_ref::<Array>(self).unwrap().into()
    } else {
      None
    }
  }

  /// Set the current module to be built. All the created elements will be inserted into this
  /// module.
  ///
  /// # Arguments
  ///
  /// * `module` - The reference of the module to be set as the current module.
  pub fn set_current_module(&mut self, module: BaseNode) {
    let block = self.get::<Module>(&module).unwrap().get_body().upcast();
    self.inesert_point = InsertPoint {
      module,
      block,
      at: None,
    };
  }

  /// Get the current insert point of this builder.
  pub fn get_current_ip(&self) -> InsertPoint {
    self.inesert_point.clone()
  }

  /// Set the current insert point.
  pub fn set_current_ip(&mut self, ip: InsertPoint) {
    self.inesert_point = ip;
  }

  /// Set the current insert point to the given block.
  ///
  /// # Arguments
  ///
  /// * `block` - The reference of the block to be set as the insert point.
  pub fn set_current_block(&mut self, block: BaseNode) {
    let module = {
      let block = block.as_ref::<Block>(self).unwrap();
      block.get_module()
    };
    self.inesert_point = InsertPoint {
      module,
      block,
      at: None,
    };
  }

  /// Set the insert before of the current builder.
  ///
  /// # Arguments
  ///
  /// * `expr` - The reference of the expression to be set as the insert point. NOTE: This expr
  ///    should be a part of the current module to be built. Ohterwise, an assertion failure will
  ///    be raised.
  pub fn set_insert_before(&mut self, node: BaseNode) {
    // Make this more general, the insert before point can also be a block.
    // Which leads to something like this:
    // module-body [
    //   // something here...
    //   // [insert-point]
    //   block a[
    //   ]
    // ]
    let (module, block, at) = {
      let block_ref = {
        let parent = node.get_parent(self).unwrap();
        assert_eq!(parent.get_kind(), NodeKind::Block);
        parent
      };
      let at = block_ref
        .as_ref::<Block>(self)
        .unwrap()
        .body_iter()
        .position(|x| x.eq(&node));
      let module = {
        // TODO(@were): Make this a method function.
        let block = block_ref.as_ref::<Block>(self).unwrap();
        block.get_module()
      };
      (module, block_ref, at)
    };
    self.inesert_point = InsertPoint { module, block, at };
  }

  /// Get the insert point of the current builder.
  pub fn get_insert_point(&self) -> InsertPoint {
    self.inesert_point.clone()
  }

  /// Set the insert point of the current builder.
  ///
  /// # Arguments
  /// * `ip` - The insert point to be set.
  pub fn set_insert_point(&mut self, ip: InsertPoint) {
    self.inesert_point = ip;
  }

  /// The helper function to insert an element into the system's slab.
  /// We adopt a slab to maintain such a highly redundant, and mutually referenced data structure.
  ///
  /// # Arguments
  ///
  /// * `elem` - The element to be inserted. An element can be any component of the system IR.
  pub(crate) fn insert_element<
    'elem,
    'sys: 'elem,
    T: IsElement<'elem, 'sys> + Into<Element> + 'sys,
  >(
    &'sys mut self,
    elem: T,
  ) -> BaseNode {
    let key = self.slab.insert(elem.into());
    let res = T::into_reference(key);
    T::downcast_mut(&mut self.slab, &res).unwrap().set_key(key);
    res
  }

  /// The helper function to create a constant integer.
  ///
  /// # Arguments
  ///
  /// * `dtype` - The data type of the constant.
  /// * `value` - The value of the constant.
  // TODO(@were): What if the data type is bigger than 64 bits?
  pub fn get_const_int(&mut self, dtype: DataType, value: u64) -> BaseNode {
    let cache_key = CacheKey::IntImm((dtype.clone(), value));
    if let Some(cached) = self.cached_nodes.get(&cache_key) {
      return *cached;
    }
    let instance = IntImm::new(dtype.clone(), value);
    let key = self.insert_element(instance);
    self.cached_nodes.insert(cache_key, key);
    key
  }

  /// Create a string literal node.
  pub fn get_str_literal(&mut self, value: String) -> BaseNode {
    let instance = StrImm::new(value);
    self.insert_element(instance)
  }

  /// Create a log command.
  pub fn create_log(&mut self, fmt: BaseNode, mut args: Vec<BaseNode>) -> BaseNode {
    assert_eq!(fmt.get_kind(), NodeKind::StrImm);
    args.insert(0, fmt);
    self.create_expr(DataType::void(), Opcode::Log, args, true)
  }

  pub fn create_select_1hot(&mut self, cond: BaseNode, values: Vec<BaseNode>) -> BaseNode {
    let cond_ty = cond.get_dtype(self).unwrap();
    assert_eq!(cond_ty.get_bits(), values.len(), "Select1Hot value count mismatch!",);
    let v0type = values[0].get_dtype(self).unwrap();
    for elem in values.iter().skip(1) {
      let vitype = elem.get_dtype(self).unwrap();
      assert_eq!(v0type, vitype, "Select1Hot value type mismatch {:?} != {:?}", v0type, vitype,);
    }
    let mut args = vec![cond];
    args.extend(values);
    self.create_expr(v0type, Opcode::Select1Hot, args, true)
  }

  pub fn create_select(
    &mut self,
    cond: BaseNode,
    true_val: BaseNode,
    false_val: BaseNode,
  ) -> BaseNode {
    let t_ty = true_val.get_dtype(self).unwrap();
    let f_ty = false_val.get_dtype(self).unwrap();
    assert_eq!(t_ty, f_ty, "Select value type mismatch: {:?} and {:?}", t_ty, f_ty);
    self.create_expr(f_ty, Opcode::Select, vec![cond, true_val, false_val], true)
  }

  /// The helper function to create an expression.
  /// An expression is the basic building block of a module.
  ///
  /// # Arguments
  /// * `dtype` - The result's data type of the expression.
  /// * `opcode` - The operation code of the expression.
  /// * `operands` - The operands of the expression.
  /// * `cond` - The condition of executing this expression. If the condition is not `None`, the is
  ///    always executed.
  /// * `insert` - If this created node is inserted into the current insert point.
  pub fn create_expr(
    &mut self,
    dtype: DataType,
    opcode: Opcode,
    operands: Vec<BaseNode>,
    insert: bool,
  ) -> BaseNode {
    // TODO(@were): Should I rearrange the insert point based on the predication?
    // If the predication is deeper than the current insert point, the inserted point should be
    // inserted to the deepest predication block.
    self.get_current_module().unwrap();
    // Wrap all the operands into Operand instances.
    let instance = Expr::new(
      dtype.clone(),
      opcode,
      vec![BaseNode::unknown(); operands.len()],
      if insert {
        self.inesert_point.block
      } else {
        // If this expression is not inserted at all, leave its parent block as unknown.
        BaseNode::unknown()
      },
    );
    let res = self.insert_element(instance);
    if insert {
      self.insert_at_ip(res);
    }
    let mut expr_mut = self.get_mut::<Expr>(&res).unwrap();
    operands.into_iter().enumerate().for_each(|(i, x)| {
      expr_mut.set_operand(i, x);
    });
    res
  }

  /// The helper function to insert an element into the current insert point.
  fn insert_at_ip(&mut self, expr: BaseNode) -> BaseNode {
    let block = self.inesert_point.block;
    self.get_mut::<Block>(&block).unwrap().insert_at_ip(expr)
  }

  /// Create an async call to the given bind. Push all the values to the corresponding named ports.
  pub fn create_async_call(&mut self, lazy_bind: BaseNode) -> BaseNode {
    // A bind will not finalize its place until it is called. This assumption helps to maintain the
    // external interfaces.
    let operands = {
      let bind = lazy_bind.as_ref::<LazyBind>(self).unwrap();
      let mut res = bind.get_bind().values().copied().collect::<Vec<_>>();
      res.push(bind.get_callee());
      res
    };
    let bind = self.create_expr(DataType::void(), Opcode::Bind, operands, true);
    self.finalized_binds.insert(lazy_bind, bind);
    self.create_expr(DataType::void(), Opcode::AsyncCall, vec![bind], true)
  }

  create_arith_op_impl!(binary, create_add, Binary::Add.into());
  create_arith_op_impl!(binary, create_sub, Binary::Sub.into());
  create_arith_op_impl!(binary, create_shl, Binary::Shl.into());
  create_arith_op_impl!(binary, create_shr, Binary::Shr.into());
  create_arith_op_impl!(binary, create_bitwise_and, Binary::BitwiseAnd.into());
  create_arith_op_impl!(binary, create_bitwise_or, Binary::BitwiseOr.into());
  create_arith_op_impl!(binary, create_bitwise_xor, Binary::BitwiseXor.into());
  create_arith_op_impl!(binary, create_mod, Binary::Mod.into());
  create_arith_op_impl!(binary, create_mul, subcode::Binary::Mul.into());
  create_arith_op_impl!(binary, create_igt, subcode::Compare::IGT.into());
  create_arith_op_impl!(binary, create_ige, subcode::Compare::IGE.into());
  create_arith_op_impl!(binary, create_ilt, subcode::Compare::ILT.into());
  create_arith_op_impl!(binary, create_ile, subcode::Compare::ILE.into());
  create_arith_op_impl!(binary, create_eq, subcode::Compare::EQ.into());
  create_arith_op_impl!(binary, create_neq, subcode::Compare::NEQ.into());
  create_arith_op_impl!(binary, create_concat, Opcode::Concat);

  create_arith_op_impl!(unary, create_neg, subcode::Unary::Neg.into());
  create_arith_op_impl!(unary, create_flip, subcode::Unary::Flip.into());

  /// Get an empty bind for the given module.
  ///
  /// # Arguments
  /// * `module` - A `BaseNode` reference to a module.
  ///
  /// # Returns
  /// * A `BaseNode` reference to the returned empty bind.
  pub fn get_init_bind(&mut self, node: BaseNode) -> BaseNode {
    match node.get_kind() {
      // A module is an empty bind.
      NodeKind::Module => {
        node.as_ref::<Module>(self).unwrap();
        let instance = LazyBind::new(node);
        self.insert_element(instance)
      }
      _ => panic!("Only a module can be bound!"),
    }
  }

  /// Add a bound argument to the given bind.
  pub fn bind_arg(&mut self, bind: BaseNode, key: String, value: BaseNode) -> BaseNode {
    let port = {
      let bind = bind.as_ref::<LazyBind>(self).unwrap();
      assert!(bind.get_arg(&key).is_none(), "Argument {} already exists!", key);
      let module = bind.get_callee().as_ref::<Module>(self).unwrap();
      let port = module.get_fifo(&key).unwrap_or_else(|| {
        panic!("\"{}\" is NOT a FIFO of \"{}\" ({:?})", key, module.get_name(), module.upcast())
      });
      assert_eq!(
        port.scalar_ty(),
        value.get_dtype(self).unwrap(),
        "Port \"{}\" requires {}",
        key,
        port.scalar_ty()
      );
      port.upcast()
    };
    let push = self.create_expr(DataType::void(), Opcode::FIFOPush, vec![port, value], true);
    if let Some(bind) = self.finalized_binds.get(&bind).cloned() {
      let mut expr_mut = self.get_mut::<Expr>(&bind).unwrap();
      let n = expr_mut.get().get_num_operands();
      expr_mut.insert_operand(n - 1, push);
    } else {
      self
        .get_mut::<LazyBind>(&bind)
        .unwrap()
        .get_mut()
        .bind_arg(key, push);
    }
    push
  }

  fn indexable(&self, idx: BaseNode) -> bool {
    let dtype = idx.get_dtype(self).unwrap();
    matches!(dtype, DataType::Int(_) | DataType::UInt(_) | DataType::Bits(_))
  }

  /// Create a read operation on an array.
  ///
  /// # Arguments
  /// * `ptr` - The pointer to the array element.
  /// * `cond` - The condition of reading the array. If None is given, the read is unconditional.
  pub fn create_array_read(&mut self, array: BaseNode, idx: BaseNode) -> BaseNode {
    assert!(
      self.indexable(idx),
      "{}'s type, {:?}, is not indexable!",
      idx.to_string(self),
      idx.get_dtype(self).unwrap()
    );
    assert!(matches!(array.get_kind(), NodeKind::Array));
    let dtype = array.as_ref::<Array>(self).unwrap().scalar_ty();

    self.create_expr(dtype, Opcode::Load, vec![array, idx], true)
  }

  /// Create a write operation on an array.
  ///
  /// # Arguments
  /// * `ptr` - The pointer to the array element.
  /// * `value` - The value to be written.
  /// * `cond` - The condition of writing the array. If None is given, the write is unconditional.
  pub fn create_array_write(
    &mut self,
    array: BaseNode,
    idx: BaseNode,
    value: BaseNode,
  ) -> BaseNode {
    assert!(
      self.indexable(idx),
      "{}'s type, {:?}, is not indexable!",
      idx.to_string(self),
      idx.get_dtype(self).unwrap()
    );
    assert!(matches!(array.get_kind(), NodeKind::Array), "Expect an array, but {:?}", array);
    let dtype = array.as_ref::<Array>(self).unwrap().scalar_ty();
    let vtype = value.get_dtype(self).unwrap_or_else(|| {
      panic!("{} has no type!", value.to_string(self));
    });
    assert_eq!(dtype, vtype, "Value type mismatch {:?} != {:?}!", dtype, vtype);
    let operands = vec![array, idx, value];

    self.create_expr(DataType::void(), Opcode::Store, operands, true)
  }

  /// The helper function to combine the data types of two references.
  ///
  /// # Arguments
  /// * `op` - The operation code to be combined.
  /// * `a` - The lhs operand.
  /// * `b` - The rhs operand.
  pub fn combine_types(&self, op: Opcode, a: &BaseNode, b: &BaseNode) -> Result<DataType, String> {
    let aty = a.get_dtype(self).unwrap();
    let bty = b.get_dtype(self).unwrap();
    if op.is_cmp() {
      if aty.get_bits() != bty.get_bits() {
        return Err(format!("Cannot compare types {} and {} for {:?}", aty, bty, op));
      }
      return Ok(DataType::uint_ty(1));
    }
    let res = match op {
      Opcode::Binary { binop } => {
        match binop {
          Binary::Add | Binary::Sub => match (&aty, &bty) {
            // TODO(@were): Add one more bit to handle overflow.
            (DataType::Int(a), DataType::Int(b)) => Some(DataType::Int(*a.max(b))),
            (DataType::UInt(a), DataType::UInt(b)) => Some(DataType::UInt(*a.max(b))),
            _ => None,
          },
          Binary::Shl | Binary::Shr => match (&aty, &bty) {
            (DataType::Int(a), DataType::Int(_)) => Some(DataType::Int(*a)),
            (DataType::UInt(a), DataType::UInt(_)) => Some(DataType::UInt(*a)),
            (DataType::Bits(a), DataType::Bits(_)) => Some(DataType::Bits(*a)),
            _ => None,
          },
          Binary::BitwiseAnd => Some(DataType::bits_ty(aty.get_bits().min(bty.get_bits()))),
          Binary::BitwiseOr | Binary::BitwiseXor => {
            Some(DataType::bits_ty(aty.get_bits().max(bty.get_bits())))
          }
          Binary::Mul => match (&aty, &bty) {
            (DataType::Int(a), DataType::Int(b)) => Some(DataType::Int(a + b)),
            (DataType::UInt(a), DataType::UInt(b)) => Some(DataType::UInt(a + b)),
            _ => None,
          },
          Binary::Mod => match (&aty, &bty) {
            (DataType::Int(a), DataType::Int(b)) => Some(DataType::Int(*a.min(b))),
            (DataType::UInt(a), DataType::UInt(b)) => Some(DataType::UInt(*a.min(b))),
            _ => None,
          },
        }
      }
      Opcode::Concat => {
        let a_bits = a.get_dtype(self).unwrap().get_bits();
        let b_bits = b.get_dtype(self).unwrap().get_bits();
        Some(DataType::bits_ty(a_bits + b_bits))
      }
      _ => panic!("Unsupported opcode {:?}", op),
    };
    if let Some(res) = res {
      Ok(res)
    } else {
      Err(format!("Cannot combine types {} and {} for {:?}", aty, bty, op))
    }
  }

  /// Create a FIFO pop operation.
  ///
  /// # Arguments
  /// * `fifo` - The FIFO to be popped.
  /// * `num_elems` - The number of elements to be popped. If None is given, the number of elements
  ///   is one.
  /// * `cond` - The condition of popping the FIFO. If None is given, the pop is unconditional.
  pub fn create_fifo_pop(&mut self, fifo: BaseNode) -> BaseNode {
    let ty = fifo.as_ref::<FIFO>(self).unwrap().scalar_ty();

    self.create_expr(ty, Opcode::FIFOPop, vec![fifo], true)
  }

  /// The helper function to create a pure intrinsic.
  pub fn create_pure_intrinsic(
    &mut self,
    ty: DataType,
    subcode: subcode::PureIntrinsic,
    operands: Vec<BaseNode>,
  ) -> BaseNode {
    self.create_expr(ty, subcode.into(), operands, true)
  }

  /// Create a FIFO peek operation. This is similar to pop, but does not remove the value from the
  /// FIFO.
  pub fn create_fifo_peek(&mut self, fifo: BaseNode) -> BaseNode {
    let ty = fifo.as_ref::<FIFO>(self).unwrap().scalar_ty();
    self.create_pure_intrinsic(ty, subcode::PureIntrinsic::FIFOPeek, vec![fifo])
  }

  pub fn create_fifo_valid(&mut self, fifo: BaseNode) -> BaseNode {
    assert!(matches!(fifo.get_kind(), NodeKind::FIFO), "Expect FIFO as the operand");
    self.create_pure_intrinsic(DataType::int_ty(1), subcode::PureIntrinsic::FIFOValid, vec![fifo])
  }

  /// Create a slice operation.
  ///
  /// TODO(@were): Should we allow `start` and `end` to be variables?
  /// TODO(@were): Should we use [start, end) or [start, end]? For now, [start, end] used.
  pub fn create_slice(&mut self, src: BaseNode, start: BaseNode, end: BaseNode) -> BaseNode {
    let ty = if let Ok(start) = start.as_ref::<IntImm>(self) {
      if let Ok(end) = end.as_ref::<IntImm>(self) {
        assert!(start.get_value() <= end.get_value());
        let bits = end.get_value() - start.get_value() + 1;
        DataType::bits_ty(bits as usize)
      } else {
        panic!("End is NOT a constant!");
      }
    } else {
      panic!("Start is NOT a constant!");
    };

    self.create_expr(ty, Opcode::Slice, vec![src, start, end], true)
  }

  /// Create a value.valid operation, which checks if this value is validly produced in this cycle.
  /// NOTE: This operation is only valid in a downstream module.
  ///
  /// # Arguments
  /// * `value` - The value to be checked.
  pub fn create_value_valid(&mut self, value: BaseNode) -> BaseNode {
    assert!(
      self.get_current_module().unwrap().is_downstream(),
      "`value.valid` is only meaningful in a downstream module!"
    );
    assert!(
      matches!(value.get_kind(), NodeKind::Expr),
      "A value expected for a validity check!"
    );
    self.create_pure_intrinsic(
      DataType::uint_ty(1),
      subcode::PureIntrinsic::ValueValid,
      vec![value],
    )
  }

  /// Create a module.triggered operation, which checks if this module is triggered in this cycle.
  /// NOTE: This operation is only valid in a downstream module.
  ///
  /// # Arguments
  /// * `module` - The module to be checked.
  pub fn create_module_triggered(&mut self, module: BaseNode) -> BaseNode {
    assert!(
      self.get_current_module().unwrap().is_downstream(),
      "`module.valid` is only meaningful in a downstream module!"
    );
    module.as_ref::<Module>(self).unwrap();
    self.create_pure_intrinsic(
      DataType::uint_ty(1),
      subcode::PureIntrinsic::ModuleTriggered,
      vec![module],
    )
  }

  fn retype_imm(&mut self, src: BaseNode, dest_ty: DataType) -> BaseNode {
    // When dealing with immediates,
    // currently there's no difference between zext and sext,
    // because we don't have negtive immediates.
    // And we convert the immediates without checking for src/dest type width,
    // because whether a type can hold an imm is checked in verifier.
    self.get_const_int(dest_ty, src.as_ref::<IntImm>(self).unwrap().get_value())
  }

  fn create_cast_impl(
    &mut self,
    src: BaseNode,
    dest_ty: DataType,
    subcode: subcode::Cast,
  ) -> BaseNode {
    match src.get_kind() {
      NodeKind::IntImm => self.retype_imm(src, dest_ty),
      _ => self.create_expr(dest_ty, Opcode::Cast { cast: subcode }, vec![src], true),
    }
  }

  /// Create a cast operation.
  pub fn create_bitcast(&mut self, src: BaseNode, dest_ty: DataType) -> BaseNode {
    self.create_cast_impl(src, dest_ty, subcode::Cast::BitCast)
  }

  /// Create a sext operation.
  pub fn create_sext(&mut self, src: BaseNode, dest_ty: DataType) -> BaseNode {
    self.create_cast_impl(src, dest_ty, subcode::Cast::SExt)
  }

  /// Create a zext operation.
  pub fn create_zext(&mut self, src: BaseNode, dest_ty: DataType) -> BaseNode {
    self.create_cast_impl(src, dest_ty, subcode::Cast::ZExt)
  }

  /// Create a halt operation to terminate the program.
  pub fn create_finish(&mut self) -> BaseNode {
    let intrinsic = subcode::BlockIntrinsic::Finish;
    self.create_expr(DataType::void(), Opcode::BlockIntrinsic { intrinsic }, vec![], true)
  }

  pub(crate) fn dispose(&mut self, node: BaseNode) {
    self.slab.remove(node.get_key());
  }

  pub(crate) fn contains(&self, node: &BaseNode) -> bool {
    self.slab.contains(node.get_key())
  }

  pub fn move_to_new_parent(&mut self, node: BaseNode, new_parent: BaseNode, at: Option<usize>) {
    let old_parent = node.get_parent(self).unwrap();
    let mut block_mut = self.get_mut::<Block>(&old_parent).unwrap();
    block_mut.erase(&node);
    let mut new_parent_mut = self.get_mut::<Block>(&new_parent).unwrap();
    new_parent_mut.insert_at(at, node);
    match node.get_kind() {
      NodeKind::Block => {
        node
          .as_mut::<Block>(self)
          .unwrap()
          .get_mut()
          .set_parent(new_parent);
      }
      NodeKind::Expr => {
        node
          .as_mut::<Expr>(self)
          .unwrap()
          .get_mut()
          .set_parent(new_parent);
      }
      _ => panic!("Unsupported node kind!"),
    }
  }
}

impl Display for SysBuilder {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut printer = IRPrinter::new(false);
    writeln!(f, "system {} {{", self.name)?;
    for elem in self.array_iter() {
      writeln!(f, "  {};", printer.visit_array(elem).unwrap())?;
    }
    printer.inc_indent();
    for elem in self.module_iter(ModuleKind::All) {
      write!(f, "\n{}", printer.visit_module(elem).unwrap())?;
    }
    printer.dec_indent();
    write!(f, "}}")
  }
}
