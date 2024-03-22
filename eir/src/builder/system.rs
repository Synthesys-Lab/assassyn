// TODO(@were): Remove all the predications and move to blocks.

use std::{collections::HashMap, fmt::Display, ops::Add};

use crate::{
  frontend::*,
  ir::{ir_printer::IRPrinter, visitor::Visitor},
};

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
  pub(crate) sym_tab: HashMap<String, BaseNode>,
  /// The symbol table of this system to handle components with same identifiers.
  unique_ids: HashMap<String, usize>,
  /// The current module to be built.
  pub(crate) inesert_point: InsertPoint,
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
      self.create_expr(res_ty, $opcode, vec![a, b])
    }
  };

  (unary, $func_name:ident, $opcode: expr) => {
    pub fn $func_name(&mut self, x: BaseNode) -> BaseNode {
      let res_ty = x.get_dtype(self).unwrap();
      self.create_expr(res_ty, $opcode, vec![x.clone()])
    }
  };
}

macro_rules! impl_typed_iter {
  ($func_name:ident, $ty: ident, $ty_ref: ident) => {
    /// Iterate over all the modules of the system.
    pub fn $func_name<'a>(&'a self) -> impl Iterator<Item = $ty_ref<'a>> {
      self
        .sym_tab
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
      sym_tab: HashMap::new(),
      slab: slab::Slab::new(),
      cached_nodes: HashMap::new(),
      inesert_point: InsertPoint(BaseNode::unknown(), BaseNode::unknown(), None),
      unique_ids: HashMap::new(),
    };
    res
  }

  /// The helper function to get an element of the system and downcast it to its actual
  /// type's immutable reference.
  pub(crate) fn get<
    'elem,
    'sys: 'elem,
    T: IsElement<'sys, 'elem> + Referencable<'sys, 'elem, T>,
  >(
    &'sys self,
    key: &BaseNode,
  ) -> Result<T::Reference, String> {
    Ok(T::reference(self, key.clone()))
  }

  impl_typed_iter!(module_iter, Module, ModuleRef);
  impl_typed_iter!(array_iter, Array, ArrayRef);

  /// The helper function to get an element of the system and downcast it to its actual type's
  /// mutable reference.
  pub(crate) fn get_mut<'elem, 'sys: 'elem, T: IsElement<'sys, 'elem> + Mutable<'sys, 'elem, T>>(
    &'sys mut self,
    key: &BaseNode,
  ) -> Result<T::Mutator, String> {
    Ok(T::mutator(self, key.clone()))
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
    if let Some(reference) = self.sym_tab.get(name) {
      reference.as_ref::<Module>(self).unwrap().into()
    } else {
      None
    }
  }

  /// Get the array by its name.
  pub fn get_array<'a>(&'a self, name: &str) -> Option<ArrayRef<'a>> {
    if let Some(reference) = self.sym_tab.get(name) {
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
    let cached_key = CacheKey::ArrayPtr((array.clone(), idx.clone()));
    if let Some(cached) = self.cached_nodes.get(&cached_key) {
      return cached.clone();
    }
    let instance = ArrayPtr::new(array.clone(), idx.clone());
    let key = self.insert_element(instance);
    self.cached_nodes.insert(cached_key, key.clone());
    key
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
  // TODO(@were): Should I rearrange the insert point based on the predication?
  // If the predication is deeper than the current insert point, the inserted point should be
  // inserted to the deepest predication block.
  pub fn create_expr(
    &mut self,
    dtype: DataType,
    opcode: Opcode,
    operands: Vec<BaseNode>,
  ) -> BaseNode {
    self.get_current_module().unwrap();
    let instance = Expr::new(
      dtype.clone(),
      opcode,
      operands,
      self.inesert_point.1.clone(),
    );
    let value = self.insert_element(instance);
    self.insert_at_ip(value)
  }

  /// The helper function to insert an element into the current insert point.
  fn insert_at_ip(&mut self, expr: BaseNode) -> BaseNode {
    let InsertPoint(_, block, _) = &self.inesert_point;
    let block = block.clone();
    self.get_mut::<Block>(&block).unwrap().insert_at_ip(expr)
  }

  /// Create a bundled trigger. Let the current module invoke the given module (destination)
  /// with all input data ready and pushed to the destination module's port FIFO.
  ///
  /// # Arguments
  /// * `dst` - The destination module to be invoked.
  /// * `data` - The data to be sent to the destination module.
  /// * `cond` - The condition of triggering the destination. If None is given, the trigger is
  /// unconditional.
  pub fn create_bundled_trigger(&mut self, dst: BaseNode, data: Vec<BaseNode>) -> BaseNode {
    let current_module = self.get_current_module().unwrap().upcast();
    let mut args = vec![dst.clone()];

    // Handle callback trigger
    let res = match dst.get_kind() {
      NodeKind::Module => {
        let ports = dst
          .as_ref::<Module>(self)
          .unwrap()
          .port_iter()
          .map(|x| x.upcast())
          .collect::<Vec<_>>();
        assert_eq!(ports.len(), data.len(), "Data size mismatch!");
        for (port, arg) in ports.iter().zip(data.iter()) {
          {
            let port = port.as_ref::<FIFO>(self).unwrap();
            assert_eq!(port.scalar_ty(), arg.get_dtype(self).unwrap());
          }
          let push_handle = self.create_fifo_push(port.clone(), arg.clone());
          args.push(push_handle);
          self
            .get_mut::<Module>(&current_module)
            .unwrap()
            .insert_external_interface(port.clone(), Opcode::FIFOPush);
        }
        self.create_expr(DataType::void(), Opcode::Trigger, args)
      }
      NodeKind::Expr => {
        let expr = dst.as_ref::<Expr>(self).unwrap();
        assert_eq!(expr.get_opcode(), Opcode::FIFOPop);
        let ty = expr.dtype();
        let types = if let DataType::Module(types) = ty {
          types.iter().map(|x| x.as_ref().clone()).collect::<Vec<_>>()
        } else {
          panic!("Invalid destination, {:?}", dst);
        };
        assert_eq!(types.len(), data.len(), "Data size mismatch!");
        for (ty, arg) in types.iter().zip(data.iter()) {
          assert_eq!(ty, &arg.get_dtype(self).unwrap());
        }
        self.create_expr(DataType::void(), Opcode::CallbackTrigger, args)
      }
      _ => panic!("Invalid destination"),
    };

    res
  }

  /// Create a trigger just invoke the given module (destination) without checking the readiness
  /// of the data. This is something more like a state machine trigger.
  ///
  /// # Arguments
  /// * `dst` - The destination module to be invoked.
  /// * `pred` - The condition of triggering the destination. If None is given, the trigger is
  /// always triggered.
  pub fn create_trigger(&mut self, dst: BaseNode) -> BaseNode {
    self.create_expr(DataType::void(), Opcode::Trigger, vec![dst.clone()])
  }

  /// Convert bind format to a vector of BaseNode corresponding to the original port order.
  pub fn bind_to_vec(&mut self, dst: BaseNode, binds: HashMap<String, BaseNode>) -> Vec<BaseNode> {
    let module = dst.as_ref::<Module>(self).unwrap();
    assert_eq!(module.get_num_inputs(), binds.len());
    let mut res = vec![BaseNode::unknown(); binds.len()];
    for (k, v) in binds.iter() {
      let pos = module.port_iter().position(|x| x.get_name() == k).unwrap();
      let fifo = module.get_input(pos).unwrap().as_ref::<FIFO>(self).unwrap();
      assert_eq!(fifo.scalar_ty(), v.get_dtype(self).unwrap());
      res[pos] = v.clone();
    }
    res
  }

  /// Create a spin trigger with bound format of arguments given.
  pub fn create_spin_trigger_bound(
    &mut self,
    handle: BaseNode,
    dst: BaseNode,
    binds: HashMap<String, BaseNode>,
  ) {
    let data = self.bind_to_vec(dst.clone(), binds);
    self.create_spin_trigger(handle, dst, data);
  }

  /// Create a trigger. Push all the values to the corresponding named ports.
  pub fn create_bound_trigger(
    &mut self,
    dst: BaseNode,
    binds: HashMap<String, BaseNode>,
  ) -> BaseNode {
    let current_module = self.get_current_module().unwrap().upcast();
    let mut bundle = vec![dst.clone()];
    assert_eq!(
      binds.len(),
      dst.as_ref::<Module>(self).unwrap().get_num_inputs()
    );
    for (name, value) in binds {
      let dst_ref = dst
        .as_ref::<Module>(self)
        .expect(format!("{:?} is NOT a module", dst).as_str());
      let port = dst_ref
        .get_input_by_name(&name)
        .expect(format!("{} is NOT an input of {:?}", name, dst).as_str());
      assert_eq!(
        port.as_ref::<FIFO>(self).unwrap().scalar_ty(),
        value.get_dtype(self).unwrap()
      );
      bundle.push(self.create_fifo_push(port.clone(), value));
      self
        .get_mut::<Module>(&current_module)
        .unwrap()
        .insert_external_interface(port, Opcode::FIFOPush);
    }
    self.create_expr(DataType::void(), Opcode::Trigger, bundle)
  }

  /// Create a spin trigger. A spin trigger repeats to test the condition
  /// until it is true, and send a signal to invoke the given module module.
  /// The source module is the current module, and the destination is given.
  ///
  /// NOTE: This created expression is more like a syntax sugar. It is equivalent create another
  /// midman module, which trigers the destination module when the condition is true, and triggers
  /// itself when the condition is false.
  ///
  /// # Arguments
  /// * `array` - A pointer to the an array element, which serves as an handle to a "lock".
  /// * `dst` - The destination module to be invoked.
  /// * `data` - The data to be sent to the destination module.
  pub fn create_spin_trigger(&mut self, handle: BaseNode, dst: BaseNode, mut data: Vec<BaseNode>) {
    data.insert(0, handle.clone());
    data.insert(1, dst.clone());
    self.create_expr(DataType::void(), Opcode::SpinTrigger, data);
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

  create_arith_op_impl!(unary, create_neg, Opcode::Neg);
  create_arith_op_impl!(unary, create_flip, Opcode::Flip);

  /// Create a register array associated to this system.
  /// An array can be a register, or memory.
  ///
  /// # Arguments
  /// * `ty` - The data type of data in the array.
  /// * `name` - The name of the array.
  /// * `size` - The size of the array.
  // TODO(@were): Add array types, memory, register, or signal wire.
  pub fn create_array(&mut self, ty: DataType, name: &str, size: usize) -> BaseNode {
    let array_name = self.identifier(name);
    let instance = Array::new(ty.clone(), array_name.clone(), size);
    let key = self.insert_element(instance);
    self.sym_tab.insert(array_name, key.clone());
    key
  }

  pub fn create_fifo_push(&mut self, fifo: BaseNode, value: BaseNode) -> BaseNode {
    let res = self.create_expr(
      DataType::void(),
      Opcode::FIFOPush,
      vec![fifo.clone(), value],
    );
    res
  }

  /// Create a read operation on an array.
  ///
  /// # Arguments
  /// * `ptr` - The pointer to the array element.
  /// * `cond` - The condition of reading the array. If None is given, the read is unconditional.
  pub fn create_array_read<'elem>(&mut self, ptr: BaseNode) -> BaseNode {
    let array = self.get::<ArrayPtr>(&ptr).unwrap().get_array().clone();
    let dtype = self.get::<Array>(&array).unwrap().scalar_ty().clone();
    let res = self.create_expr(dtype, Opcode::Load, vec![ptr.clone()]);
    let cur_mod = self.inesert_point.0.clone();
    self
      .get_mut::<Module>(&cur_mod)
      .unwrap()
      .insert_external_interface(array.clone(), Opcode::Load);
    res
  }

  /// Create a write operation on an array.
  ///
  /// # Arguments
  /// * `ptr` - The pointer to the array element.
  /// * `value` - The value to be written.
  /// * `cond` - The condition of writing the array. If None is given, the write is unconditional.
  pub fn create_array_write(&mut self, ptr: BaseNode, value: BaseNode) -> BaseNode {
    let array = self.get::<ArrayPtr>(&ptr).unwrap().get_array().clone();
    let operands = vec![ptr.clone(), value.clone()];
    let res = self.create_expr(DataType::void(), Opcode::Store, operands);
    let cur_mod = self.inesert_point.0.clone();
    self
      .get_mut::<Module>(&cur_mod)
      .unwrap()
      .insert_external_interface(array.clone(), Opcode::Store);
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
    match op {
      Opcode::Add | Opcode::Sub | Opcode::BitwiseAnd | Opcode::BitwiseOr | Opcode::BitwiseXor => {
        match (&aty, &bty) {
          (DataType::Int(a), DataType::Int(b)) => DataType::Int(*a.max(b)),
          (DataType::UInt(a), DataType::UInt(b)) => DataType::UInt(*a.max(b)),
          _ => panic!(
            "Cannot combine types {} and {}",
            aty.to_string(),
            bty.to_string()
          ),
        }
      }
      Opcode::Mul => match (&aty, &bty) {
        (DataType::Int(a), DataType::Int(b)) => DataType::Int(a + b),
        (DataType::UInt(a), DataType::UInt(b)) => DataType::UInt(a.add(b)),
        _ => panic!(
          "Cannot combine types {} and {}",
          aty.to_string(),
          bty.to_string()
        ),
      },
      Opcode::IGT | Opcode::IGE | Opcode::ILT | Opcode::ILE => DataType::uint(1),
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
      self.get_const_int(DataType::uint(32), 1)
    };
    let ty = fifo.as_ref::<FIFO>(self).unwrap().scalar_ty();
    let res = self.create_expr(ty, Opcode::FIFOPop, vec![fifo.clone(), num_elems]);
    res
  }

  /// The helper function to generate a unique identifier.
  ///
  /// # Arguments
  /// * `id` - The original identifier.
  /// # Returns
  /// The unique identifier.
  // TODO(@were): Overengineer a dedicated module for this later.
  pub(crate) fn identifier(&mut self, id: &str) -> String {
    // If the identifier is already in the symbol table, we append a number to it.
    if let Some(x) = self.unique_ids.get_mut(id.into()) {
      // Append a number after.
      let res = format!("{}.{}", id, x);
      *x += 1;
      // To avoid user to use the appended identifier, we also insert it into the symbol table.
      self.unique_ids.insert(res.clone(), 0);
      return res;
    }
    // If not, we just use itself.
    self.unique_ids.insert(id.into(), 0);
    id.into()
  }
}

impl Display for SysBuilder {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut printer = IRPrinter::new(self);
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
