// TODO(@were): Remove all the predications and move to blocks.

use std::{collections::HashMap, fmt::Display, hash::Hash};

use crate::ir::{
  bind::{as_bind_expr, get_bind_callee, is_fully_bound},
  instructions::GetElementPtr,
  ir_printer::IRPrinter,
  module::Attribute,
  node::*,
  visitor::Visitor,
  *,
};

use self::user::Operand;

use super::symbol_table::SymbolTable;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InsertPoint(pub BaseNode, pub BaseNode, pub Option<usize>);

impl InsertPoint {
  pub fn next(&self, sys: &SysBuilder) -> Option<Self> {
    let InsertPoint(module, block, at) = self;
    let block = block.as_ref::<Block>(sys).unwrap();
    if let Some(cur_at) = at {
      if cur_at + 1 < block.get_num_exprs() {
        return InsertPoint(module.clone(), block.upcast(), Some(cur_at + 1)).into();
      } else {
        return InsertPoint(module.clone(), block.upcast(), None).into();
      }
    } else {
      if let Some(nxt_block) = block.next() {
        return InsertPoint(module.clone(), nxt_block, Some(0)).into();
      } else {
        if let Ok(block_parent) = block.get_parent().as_ref::<Block>(sys) {
          return InsertPoint(module.clone(), block_parent.upcast(), None).into();
        } else {
          return None;
        }
      }
    };
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
  /// The global symbols in this system, including modules and arrays.
  pub(crate) global_symbols: HashMap<String, BaseNode>,
  /// The current module to be built.
  pub(crate) inesert_point: InsertPoint,
  /// The symbol table to maintain the unique identifiers.
  pub(crate) symbol_table: SymbolTable,
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
/// inferred from the operands.
/// * `a` - The first operand.
/// * `b` - The second operand.
/// * `pred` - The condition of executing this expression. If the condition is not `None`, this
/// is always executed.
macro_rules! create_arith_op_impl {
  (binary, $func_name:ident, $opcode: expr) => {
    pub fn $func_name(&mut self, ty: Option<DataType>, a: BaseNode, b: BaseNode) -> BaseNode {
      let res_ty = if let Some(ty) = ty {
        ty
      } else {
        self.combine_types($opcode, &a, &b)
      };
      self.create_expr(res_ty, $opcode, vec![a, b], true)
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

macro_rules! impl_typed_iter {
  ($func_name:ident, $ty: ident, $ty_ref: ident) => {
    /// Iterate over all the modules of the system.
    pub fn $func_name<'a>(&'a self) -> impl Iterator<Item = $ty_ref<'a>> {
      self
        .global_symbols
        .iter()
        .filter(|(_, v)| {
          if let NodeKind::$ty = v.get_kind() {
            true
          } else {
            false
          }
        })
        .map(|(_, x)| x.as_ref::<$ty>(self).unwrap())
    }
  };
}

impl SysBuilder {
  /// Create a new system.
  /// # Arguments
  ///
  /// * `name` - The name of the system.
  pub fn new(name: &str) -> Self {
    let res = Self {
      name: name.into(),
      global_symbols: HashMap::new(),
      slab: slab::Slab::new(),
      cached_nodes: HashMap::new(),
      inesert_point: InsertPoint(BaseNode::unknown(), BaseNode::unknown(), None),
      symbol_table: SymbolTable::new(),
    };
    res
  }

  pub fn get_name(&self) -> &str {
    &self.name
  }

  /// If this system has a driver.
  pub fn has_driver(&self) -> bool {
    self.module_iter().any(|x| x.get_name().eq("driver"))
  }

  /// If this system has a testbench.
  pub fn has_testbench(&self) -> bool {
    self.module_iter().any(|x| x.get_name().eq("testbench"))
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
    T::reference(self, key.clone())
  }

  impl_typed_iter!(module_iter, Module, ModuleRef);
  impl_typed_iter!(array_iter, Array, ArrayRef);

  /// The helper function to get an element of the system and downcast it to its actual type's
  /// mutable reference.
  pub(crate) fn get_mut<'elem, 'sys: 'elem, T: IsElement<'elem, 'sys> + Mutable<'elem, 'sys, T>>(
    &'sys mut self,
    key: &BaseNode,
  ) -> Result<T::Mutator, String> {
    T::mutator(self, key.clone())
  }

  /// Get the current module to be built.
  pub fn get_current_module<'a>(&'a self) -> Result<ModuleRef<'a>, String> {
    self.get::<Module>(&self.inesert_point.0)
  }

  pub fn get_current_block<'a>(&'a self) -> Result<BlockRef<'a>, String> {
    self.get::<Block>(&self.inesert_point.1)
  }

  /// Get the module by its name.
  pub fn get_module<'a>(&'a self, name: &str) -> Option<ModuleRef<'a>> {
    if let Some(reference) = self.global_symbols.get(name) {
      reference.as_ref::<Module>(self).unwrap().into()
    } else {
      None
    }
  }

  /// Get the array by its name.
  pub fn get_array<'a>(&'a self, name: &str) -> Option<ArrayRef<'a>> {
    if let Some(reference) = self.global_symbols.get(name) {
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
    self.inesert_point = InsertPoint(module, block, None);
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
      block.get_module().upcast()
    };
    self.inesert_point = InsertPoint(module, block, None);
  }

  /// Set the insert before of the current builder.
  ///
  /// # Arguments
  ///
  /// * `expr` - The reference of the expression to be set as the insert point. NOTE: This expr
  /// should be a part of the current module to be built. Ohterwise, an assertion failure will be
  /// raised.
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
        .iter()
        .position(|x| *x == node);
      let module = {
        // TODO(@were): Make this a method function.
        let block = block_ref.as_ref::<Block>(self).unwrap();
        block.get_module().upcast()
      };
      (module, block_ref, at)
    };
    self.inesert_point = InsertPoint(module, block, at);
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
      return cached.clone();
    }
    let instance = IntImm::new(dtype.clone(), value);
    let key = self.insert_element(instance);
    self.cached_nodes.insert(cache_key, key.clone());
    key
  }

  pub fn get_str_literal(&mut self, value: String) -> BaseNode {
    let instance = StrImm::new(value);
    let key = self.insert_element(instance);
    key
  }

  /// The helper function to create a handle to an array access.
  ///
  /// # Arguments
  ///
  /// * `array` - The array to be accessed.
  /// * `idx` - The index to be accessed.
  pub fn create_array_ptr(&mut self, array: BaseNode, idx: BaseNode) -> BaseNode {
    assert_eq!(array.get_kind(), NodeKind::Array);
    match idx.get_dtype(self).unwrap() {
      DataType::Int(_) | DataType::UInt(_) => {}
      _ => panic!("Invalid index type"),
    }
    let res = self.create_expr(
      DataType::void(),
      Opcode::GetElementPtr,
      vec![array, idx],
      true,
    );
    res
  }

  pub fn create_log(&mut self, fmt: BaseNode, mut args: Vec<BaseNode>) -> BaseNode {
    assert_eq!(fmt.get_kind(), NodeKind::StrImm);
    args.insert(0, fmt);
    self.create_expr(DataType::void(), Opcode::Log, args, true)
  }

  pub fn create_select(
    &mut self,
    cond: BaseNode,
    true_val: BaseNode,
    false_val: BaseNode,
  ) -> BaseNode {
    let ty = true_val.get_dtype(self).unwrap();
    assert_eq!(ty, false_val.get_dtype(self).unwrap());
    self.create_expr(ty, Opcode::Select, vec![cond, true_val, false_val], true)
  }

  /// The helper function to create an expression.
  /// An expression is the basic building block of a module.
  ///
  /// # Arguments
  /// * `dtype` - The result's data type of the expression.
  /// * `opcode` - The operation code of the expression.
  /// * `operands` - The operands of the expression.
  /// * `cond` - The condition of executing this expression. If the condition is not `None`, the is
  /// always executed.
  /// * `insert` - If this created node is inserted into the current insert point.
  // TODO(@were): Should I rearrange the insert point based on the predication?
  // If the predication is deeper than the current insert point, the inserted point should be
  // inserted to the deepest predication block.
  pub fn create_expr(
    &mut self,
    dtype: DataType,
    opcode: Opcode,
    operands: Vec<BaseNode>,
    insert: bool,
  ) -> BaseNode {
    self.get_current_module().unwrap();
    // Wrap all the operands into Operand instances.
    let operands = operands
      .into_iter()
      .map(|x| self.insert_element(Operand::new(x)))
      .collect();
    let instance = Expr::new(
      dtype.clone(),
      opcode,
      operands,
      self.inesert_point.1.clone(),
    );
    let res = self.insert_element(instance);
    if insert {
      self.insert_at_ip(res);
    }
    let operands = res
      .as_ref::<Expr>(self)
      .unwrap()
      .operand_iter()
      .map(|x| x.upcast())
      .collect::<Vec<_>>();
    operands.into_iter().for_each(|x| {
      let mut operand = x.as_mut::<Operand>(self).unwrap();
      operand.get_mut().set_user(res);
      self.add_user(x.clone());
    });
    res
  }

  fn insert_external_interface(&mut self, ext: BaseNode, expr: BaseNode, operand_idx: usize) {
    let cur_mod = self.get_current_module().unwrap().upcast();
    let operand = expr
      .as_ref::<Expr>(self)
      .unwrap()
      .get_operand(operand_idx)
      .unwrap()
      .upcast();
    let mut mod_mut = self.get_mut::<Module>(&cur_mod).unwrap();
    mod_mut.insert_external_interface(ext, operand);
  }

  /// The helper function to insert an element into the current insert point.
  fn insert_at_ip(&mut self, expr: BaseNode) -> BaseNode {
    let InsertPoint(_, block, _) = &self.inesert_point;
    let block = block.clone();
    self.get_mut::<Block>(&block).unwrap().insert_at_ip(expr)
  }

  /// Create an async call to the given bind. Push all the values to the corresponding named ports.
  pub fn create_async_call(&mut self, bind: BaseNode) -> BaseNode {
    assert!({
      let expr = self.get::<Expr>(&bind).unwrap();
      match expr.get_opcode() {
        Opcode::Bind => true,
        _ => false,
      }
    });
    let args = vec![bind];
    self.insert_at_ip(bind);
    let res = self.create_expr(DataType::void(), Opcode::AsyncCall, args, true);
    res
  }

  create_arith_op_impl!(binary, create_add, Opcode::Add);
  create_arith_op_impl!(binary, create_sub, Opcode::Sub);
  create_arith_op_impl!(binary, create_bitwise_and, Opcode::BitwiseAnd);
  create_arith_op_impl!(binary, create_bitwise_or, Opcode::BitwiseOr);
  create_arith_op_impl!(binary, create_bitwise_xor, Opcode::BitwiseXor);
  create_arith_op_impl!(binary, create_mul, Opcode::Mul);
  create_arith_op_impl!(binary, create_igt, Opcode::IGT);
  create_arith_op_impl!(binary, create_ige, Opcode::IGE);
  create_arith_op_impl!(binary, create_ilt, Opcode::ILT);
  create_arith_op_impl!(binary, create_ile, Opcode::ILE);
  create_arith_op_impl!(binary, create_eq, Opcode::EQ);
  create_arith_op_impl!(binary, create_neq, Opcode::NEQ);
  create_arith_op_impl!(binary, create_concat, Opcode::Concat);

  create_arith_op_impl!(unary, create_neg, Opcode::Neg);
  create_arith_op_impl!(unary, create_flip, Opcode::Flip);

  /// Create a register array associated to this system.
  /// An array can be a register, or memory.
  ///
  /// # Arguments
  /// * `ty` - The data type of data in the array.
  /// * `name` - The name of the array.
  /// * `size` - The size of the array.
  /// * `init` - A vector of initial values of this array.
  // TODO(@were): Add array types, memory, register, or signal wire.
  pub fn create_array(
    &mut self,
    ty: DataType,
    name: &str,
    size: usize,
    init: Option<Vec<BaseNode>>,
  ) -> BaseNode {
    let array_name = self.symbol_table.identifier(name);
    if let Some(init) = &init {
      assert_eq!(init.len(), size);
      init.iter().for_each(|x| {
        assert_eq!(x.get_dtype(self).unwrap(), ty);
      });
    }
    let instance = Array::new(ty.clone(), array_name.clone(), size, init);
    let key = self.insert_element(instance);
    self.global_symbols.insert(array_name, key.clone());
    key
  }

  pub fn get_init_bind(&mut self, node: BaseNode) -> BaseNode {
    let failure = || {
      panic!(
        "[Bind Init] Either a Module or a Bind is expected, but {:?} got!",
        node
      )
    };
    match node.get_kind() {
      // A module is an empty bind.
      NodeKind::Module => {
        let module = node.as_ref::<Module>(self).unwrap();
        let mut args = vec![BaseNode::unknown(); module.get_num_inputs()];
        args.push(module.upcast());
        self.create_expr(DataType::void(), Opcode::Bind, args, false)
      }
      // An expression should be a module type.
      NodeKind::Expr => {
        let expr = node.as_ref::<Expr>(self).unwrap();
        match expr.get_opcode() {
          Opcode::FIFOPop => {
            let n = {
              let dtype = expr.dtype();
              match dtype {
                DataType::Module(ports) => ports.len(),
                _ => panic!("Invalid data type"),
              }
            };
            let mut args = vec![BaseNode::unknown(); n];
            args.push(node);
            self.create_expr(DataType::void(), Opcode::Bind, args, false)
          }
          Opcode::Bind => node,
          _ => failure(),
        }
      }
      _ => failure(),
    }
  }

  /// Add a bind to the current module.
  pub fn add_bind(
    &mut self,
    bind: BaseNode,
    key: String,
    value: BaseNode,
    eager: Option<bool>,
  ) -> BaseNode {
    let module = {
      let callee = get_bind_callee(self, bind);
      callee.as_ref::<Module>(self).unwrap_or_else(|_| {
        panic!(
          "Only module callee can be used for bind, but {:?} got!",
          callee
        )
      })
    };
    let port = module.get_port_by_name(&key).expect(&format!(
      "\"{}\" is NOT a FIFO of \"{}\" ({:?})",
      key,
      module.get_name(),
      module.upcast()
    ));
    assert_eq!(port.scalar_ty(), value.get_dtype(self).unwrap());
    let port_idx = port.idx();
    let module_name = module.get_name().to_string();
    let module = module.upcast();
    let fifo_push = self.create_fifo_push(module.clone(), port_idx, value);
    let mut bind_mut = bind.as_mut::<Expr>(self).unwrap();
    assert!(
      bind_mut
        .get()
        .get_operand(port_idx)
        .unwrap()
        .get_value()
        .is_unknown(),
      "Port \"{}\", indexed @{}, of Module \"{}\" is already bound!",
      key,
      port_idx,
      module_name
    );
    bind_mut.set_operand(port_idx, fifo_push);
    let eager = eager.unwrap_or(
      self
        .get_current_module()
        .unwrap()
        .get_attrs()
        .contains(&Attribute::EagerBind),
    );
    if eager && is_fully_bound(self, bind) {
      self.create_async_call(bind)
    } else {
      bind
    }
  }

  /// Add a bind to the current module.
  pub fn push_bind(&mut self, bind: BaseNode, value: BaseNode, eager: Option<bool>) -> BaseNode {
    let (callee, signature, port_idx) = {
      let callee = get_bind_callee(self, bind);
      let bind = as_bind_expr(self, bind).unwrap();
      let signature = callee.get_dtype(self).unwrap();
      let port_idx = {
        let mut idx = None;
        for i in 0..bind.get_num_operands() - 1 {
          let operand = bind.get_operand(i).unwrap();
          if operand.get_value().is_unknown() && idx.is_none() {
            idx = Some(i);
          } else if idx.is_some() {
            assert!(operand.get_value().is_unknown());
          }
        }
        idx.expect("All arguments bound!")
      };
      (callee, signature, port_idx)
    };
    match &signature {
      DataType::Module(ports) => {
        assert_eq!(
          ports.get(port_idx).unwrap().as_ref().clone(),
          value.get_dtype(self).unwrap(),
        );
      }
      _ => panic!("Invalid signature"),
    }
    let fifo_push = self.create_fifo_push(callee, port_idx, value);
    let mut bind_mut = bind.as_mut::<Expr>(self).unwrap();
    bind_mut.set_operand(port_idx, fifo_push);
    let eager = eager.unwrap_or(
      self
        .get_current_module()
        .unwrap()
        .get_attrs()
        .contains(&Attribute::EagerBind),
    );
    if eager && is_fully_bound(self, bind) {
      self.create_async_call(bind)
    } else {
      bind
    }
  }

  /// A helper function to create a FIFO push.
  pub(crate) fn create_fifo_push(
    &mut self,
    module: BaseNode,
    idx: usize,
    value: BaseNode,
  ) -> BaseNode {
    match module.get_dtype(self) {
      Some(DataType::Module(_)) => {}
      _ => panic!("Invalid module type"),
    }

    let port = match module.get_kind() {
      NodeKind::Module => module
        .as_ref::<Module>(self)
        .unwrap()
        .get_port(idx)
        .expect("Invalid port index")
        .upcast(),
      _ => {
        let dtype = value.get_dtype(self).unwrap();
        let fifo = FIFO::placeholder(dtype, module.clone(), idx);
        self.insert_element(fifo)
      }
    };

    // Create the expression.
    let res = self.create_expr(DataType::void(), Opcode::FIFOPush, vec![port, value], true);

    // Maintain the external interface redundancy when it is determined.
    if !port.as_ref::<FIFO>(self).unwrap().is_placeholder() {
      self.insert_external_interface(port, res.clone(), 0);
    }

    res
  }

  /// Create a read operation on an array.
  ///
  /// # Arguments
  /// * `ptr` - The pointer to the array element.
  /// * `cond` - The condition of reading the array. If None is given, the read is unconditional.
  pub fn create_array_read<'elem>(&mut self, ptr: BaseNode) -> BaseNode {
    let (array, dtype) = {
      let gep = ptr.as_expr::<GetElementPtr>(self).unwrap();
      let array = gep.get_array();
      let dtype = array.scalar_ty();
      (array.upcast(), dtype)
    };
    let res = self.create_expr(dtype, Opcode::Load, vec![ptr.clone()], true);
    self.insert_external_interface(array.clone(), res.clone(), 0);
    res
  }

  /// Create a write operation on an array.
  ///
  /// # Arguments
  /// * `ptr` - The pointer to the array element.
  /// * `value` - The value to be written.
  /// * `cond` - The condition of writing the array. If None is given, the write is unconditional.
  pub fn create_array_write(&mut self, ptr: BaseNode, value: BaseNode) -> BaseNode {
    let array = {
      let gep = ptr.as_expr::<GetElementPtr>(self).unwrap();
      let array = gep.get_array();
      let dtype = array.scalar_ty();
      assert_eq!(
        value.get_dtype(self).unwrap(),
        dtype,
        "Cannot write {:?} with type {:?} to an array with type {:?}",
        value.to_string(self),
        value.get_dtype(self).unwrap(),
        dtype
      );
      array.upcast()
    };
    let operands = vec![ptr.clone(), value.clone()];
    let res = self.create_expr(DataType::void(), Opcode::Store, operands, true);
    self.insert_external_interface(array, res.clone(), 0);
    res
  }

  /// The helper function to combine the data types of two references.
  ///
  /// # Arguments
  /// * `op` - The operation code to be combined.
  /// * `a` - The lhs operand.
  /// * `b` - The rhs operand.
  fn combine_types(&self, op: Opcode, a: &BaseNode, b: &BaseNode) -> DataType {
    let aty = a.get_dtype(self).unwrap();
    let bty = b.get_dtype(self).unwrap();
    if op.is_cmp() {
      if aty.get_bits() != bty.get_bits() {
        panic!(
          "Cannot compare types {} and {} for {:?}",
          aty.to_string(),
          bty.to_string(),
          op
        );
      }
      return DataType::uint_ty(1);
    }
    match op {
      Opcode::Add | Opcode::Sub => {
        match (&aty, &bty) {
          // TODO(@were): Add one more bit to handle overflow.
          (DataType::Int(a), DataType::Int(b)) => DataType::Int(*a.max(b)),
          (DataType::UInt(a), DataType::UInt(b)) => DataType::UInt(*a.max(b)),
          _ => panic!(
            "Cannot combine types {} and {} for {:?}",
            aty.to_string(),
            bty.to_string(),
            op
          ),
        }
      }
      Opcode::BitwiseAnd => DataType::bits_ty(aty.get_bits().min(bty.get_bits())),
      Opcode::BitwiseOr | Opcode::BitwiseXor => {
        DataType::bits_ty(aty.get_bits().max(bty.get_bits()))
      }
      Opcode::Mul => match (&aty, &bty) {
        (DataType::Int(a), DataType::Int(b)) => DataType::Int(a + b),
        (DataType::UInt(a), DataType::UInt(b)) => DataType::UInt(a + b),
        _ => panic!(
          "Cannot combine types {} and {}",
          aty.to_string(),
          bty.to_string()
        ),
      },
      Opcode::Concat => {
        let a_bits = a.get_dtype(self).unwrap().get_bits();
        let b_bits = b.get_dtype(self).unwrap().get_bits();
        DataType::bits_ty(a_bits + b_bits)
      }
      _ => panic!("Unsupported opcode {:?}", op),
    }
  }

  /// Create a FIFO pop operation.
  ///
  /// # Arguments
  /// * `fifo` - The FIFO to be popped.
  /// * `num_elems` - The number of elements to be popped. If None is given, the number of elements
  /// is one.
  /// * `cond` - The condition of popping the FIFO. If None is given, the pop is unconditional.
  pub fn create_fifo_pop(&mut self, fifo: BaseNode, num_elems: Option<BaseNode>) -> BaseNode {
    let num_elems = if let Some(num_elems) = num_elems {
      num_elems
    } else {
      self.get_const_int(DataType::uint_ty(32), 1)
    };
    let ty = fifo.as_ref::<FIFO>(self).unwrap().scalar_ty();
    let res = self.create_expr(ty, Opcode::FIFOPop, vec![fifo.clone(), num_elems], true);
    res
  }

  /// Create a FIFO peek operation. This is similar to pop, but does not remove the value from the
  /// FIFO.
  pub fn create_fifo_peek(&mut self, fifo: BaseNode) -> BaseNode {
    let ty = fifo.as_ref::<FIFO>(self).unwrap().scalar_ty();
    let res = self.create_expr(ty, Opcode::FIFOPeek, vec![fifo], true);
    res
  }

  pub fn create_fifo_valid(&mut self, fifo: BaseNode) -> BaseNode {
    assert_eq!(fifo.get_kind(), NodeKind::FIFO);
    let res = self.create_expr(DataType::int_ty(1), Opcode::FIFOValid, vec![fifo], true);
    res
  }

  /// Create a slice operation.
  ///
  /// TODO(@were): Should we allow `start` and `end` to be variables?
  /// TODO(@were): Should we use [start, end) or [start, end]? For now, [start, end] used.
  pub fn create_slice(
    &mut self,
    ty: Option<DataType>,
    src: BaseNode,
    start: BaseNode,
    end: BaseNode,
  ) -> BaseNode {
    let ty = if let Some(ty) = ty {
      ty
    } else if let Ok(start) = start.as_ref::<IntImm>(self) {
      if let Ok(end) = end.as_ref::<IntImm>(self) {
        assert!(start.get_value() <= end.get_value());
        let bits = end.get_value() - start.get_value() + 1;
        DataType::int_ty(bits as usize)
      } else {
        src.get_dtype(self).unwrap()
      }
    } else {
      src.get_dtype(self).unwrap()
    };
    let res = self.create_expr(ty, Opcode::Slice, vec![src, start, end], true);
    res
  }

  /// Create a cast operation.
  pub fn create_cast(&mut self, src: BaseNode, dest_ty: DataType) -> BaseNode {
    let res = self.create_expr(dest_ty, Opcode::Cast, vec![src], true);
    res
  }

  /// Create a sext operation.
  pub fn create_sext(&mut self, src: BaseNode, dest_ty: DataType) -> BaseNode {
    let res = self.create_expr(dest_ty, Opcode::Sext, vec![src], true);
    res
  }

  pub(crate) fn dispose(&mut self, node: BaseNode) {
    self.slab.remove(node.get_key());
  }

  pub(crate) fn contains(&self, node: &BaseNode) -> bool {
    self.slab.contains(node.get_key())
  }
}

impl Display for SysBuilder {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut printer = IRPrinter::new(false);
    write!(f, "system {} {{\n", self.name)?;
    for elem in self.array_iter() {
      write!(f, "  {};\n", printer.visit_array(&elem).unwrap())?;
    }
    printer.inc_indent();
    for elem in self.module_iter() {
      write!(f, "\n{}", printer.visit_module(&elem).unwrap())?;
    }
    printer.dec_indent();
    write!(f, "}}")
  }
}
