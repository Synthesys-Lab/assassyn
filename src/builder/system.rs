use std::{collections::HashMap, fmt::Display, ops::Add};

use crate::{
  data::{Array, Typed},
  expr::{Expr, Opcode},
  ir::{block::Block, ir_printer},
  reference::{Element, IsElement, Parented, Visitor},
  DataType, IntImm, Module, Reference,
};

use super::mutator::Mutable;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InsertPoint(pub Reference, pub Reference, pub Option<usize>);

/// A `SysBuilder` struct not only serves as the data structure of the whole system,
/// but also works as the syntax-sugared IR builder.
pub struct SysBuilder {
  /// The slab to store all the elements in the system. We use a slab to maintain such a
  /// highly redundant, and mutually referenced data structure.
  pub(crate) slab: slab::Slab<Element>,
  /// The data structure caches the constant values.
  const_cache: HashMap<(DataType, u64), Reference>,
  /// The name of the system.
  name: String,
  /// The global symbols in this system, including modules and arrays.
  pub(crate) sym_tab: HashMap<String, Reference>,
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
macro_rules! create_binary_op_impl {
  ($func_name:ident, $opcode: expr) => {
    pub fn $func_name(
      &mut self,
      ty: Option<DataType>,
      a: &Reference,
      b: &Reference,
      pred: Option<&Reference>,
    ) -> Reference {
      let res_ty = if let Some(ty) = ty {
        ty
      } else {
        self.combine_types($opcode, a, b)
      };
      self.create_expr(
        res_ty,
        $opcode,
        vec![a.clone(), b.clone()],
        pred.map(|x| x.clone()),
      )
    }
  };
}

macro_rules! impl_typed_iter {
  ($func_name:ident, $ty: ident) => {
    /// Iterate over all the modules of the system.
    pub fn $func_name<'a>(&'a self) -> impl Iterator<Item = &'a Box<$ty>> {
      self
        .sym_tab
        .iter()
        .filter(|(_, v)| {
          if let Reference::$ty(_) = v {
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
    let mut res = Self {
      name: name.into(),
      sym_tab: HashMap::new(),
      slab: slab::Slab::new(),
      const_cache: HashMap::new(),
      inesert_point: InsertPoint(Reference::Unknown, Reference::Unknown, None),
      unique_ids: HashMap::new(),
    };
    res.create_module("driver", vec![]);
    res
  }

  /// The helper function to get an element of the system and downcast it to its actual
  /// type's immutable reference.
  pub(crate) fn get<'a, T: IsElement<'a>>(&'a self, key: &Reference) -> Result<&'a Box<T>, String> {
    T::downcast(&self.slab, key)
  }

  impl_typed_iter!(module_iter, Module);
  impl_typed_iter!(array_iter, Array);

  /// The helper function to get an element of the system and downcast it to its actual type's
  /// mutable reference.
  pub(crate) fn get_mut<'a, T: IsElement<'a> + Mutable<'a, T>>(
    &'a mut self,
    key: &Reference,
  ) -> Result<T::Mutator, String> {
    Ok(T::mutator(self, key.clone()))
  }

  /// Get the driver module. The driver module is special. It is invoked unconditionally every
  /// cycle.
  pub fn get_driver(&self) -> &Module {
    self.get_module("driver").unwrap()
  }

  /// Get the current module to be built.
  pub fn get_current_module(&self) -> Result<&Box<Module>, String> {
    self.get::<Module>(&self.inesert_point.0)
  }

  /// Get the module by its name.
  pub fn get_module(&self, name: &str) -> Option<&Box<Module>> {
    if let Some(reference) = self.sym_tab.get(name) {
      reference.as_ref::<Module>(self).unwrap().into()
    } else {
      None
    }
  }

  /// Get the array by its name.
  pub fn get_array(&self, name: &str) -> Option<&Box<Array>> {
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
  pub fn set_current_module(&mut self, module: Reference) {
    let block = self
      .get::<Module>(&module)
      .unwrap()
      .get_body(self)
      .unwrap()
      .upcast();
    self.inesert_point = InsertPoint(module, block, None);
  }

  /// Set the insert before of the current builder.
  ///
  /// # Arguments
  ///
  /// * `expr` - The reference of the expression to be set as the insert point. NOTE: This expr
  /// should be a part of the current module to be built. Ohterwise, an assertion failure will be
  /// raised.
  pub fn set_insert_before(&mut self, expr: Reference) {
    let (block, module, at) = {
      let block_ref = {
        let expr = expr.as_ref::<Expr>(self).unwrap();
        expr.get_parent()
      };
      let block = self.get::<Block>(&block_ref).unwrap();
      let module = block.get_parent();
      let at = block.iter().position(|x| *x == expr);
      (block_ref, module, at)
    };
    self.inesert_point = InsertPoint(module, block, at);
  }

  /// Get the insert point of the current builder.
  pub fn get_insert_point(&self) -> InsertPoint {
    self.inesert_point.clone()
  }

  /// The helper function to insert an element into the system's slab.
  /// We adopt a slab to maintain such a highly redundant, and mutually referenced data structure.
  ///
  /// # Arguments
  ///
  /// * `elem` - The element to be inserted. An element can be any component of the system IR.
  pub(crate) fn insert_element<'a, T: IsElement<'a> + Into<Element> + 'a>(
    &'a mut self,
    elem: T,
  ) -> Reference {
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
  pub fn get_const_int(&mut self, dtype: &DataType, value: u64) -> Reference {
    let key = (dtype.clone(), value);
    if let Some(cached) = self.const_cache.get(&key) {
      return cached.clone();
    }
    let cloned_key = key.clone();
    let instance = IntImm::new(key.0, key.1);
    let key = self.insert_element(instance);
    self.const_cache.insert(cloned_key, key.clone());
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
    operands: Vec<Reference>,
    cond: Option<Reference>,
  ) -> Reference {
    match self.inesert_point.0 {
      Reference::Unknown => panic!("No current module is set"),
      _ => {}
    }
    if let Some(cond) = cond {
      let block = self.create_block(cond.into());
      let instance = Expr::new(dtype.clone(), opcode, operands, block.clone());
      let value = self.insert_element(instance);
      self
        .get_mut::<Block>(&block)
        .unwrap()
        .push(value.clone())
    } else {
      let instance = Expr::new(
        dtype.clone(),
        opcode,
        operands,
        self.inesert_point.1.clone(),
      );
      let value = self.insert_element(instance);
      self.insert_at_ip(value)
    }
  }

  /// The helper function to insert an element into the current insert point.
  fn insert_at_ip(&mut self, expr: Reference) -> Reference {
    let InsertPoint(_, block, _) = &self.inesert_point;
    let block = block.clone();
    self.get_mut::<Block>(&block).unwrap().insert_at_ip(expr)
  }

  /// Create a trigger. A trigger sends a signal to invoke the given module.
  /// The source module is the current module, and the destination is given.
  ///
  /// # Arguments
  /// * `dst` - The destination module to be invoked.
  /// * `data` - The data to be sent to the destination module.
  /// * `cond` - The condition of triggering the destination. If None is given, the trigger is
  /// unconditional.
  pub fn create_trigger(
    &mut self,
    dst: &Reference,
    mut data: Vec<Reference>,
    cond: Option<Reference>,
  ) {
    data.insert(0, dst.clone());
    self.create_expr(DataType::void(), Opcode::Trigger, data, cond);
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
  /// * `cond` - The condition to be tested.
  /// * `dst` - The destination module to be invoked.
  /// * `data` - The data to be sent to the destination module.
  pub fn create_spin_trigger(
    &mut self,
    cond: Reference,
    dst: &Box<Module>,
    mut data: Vec<Reference>,
  ) {
    data.insert(0, dst.upcast());
    data.insert(1, cond);
    self.create_expr(DataType::void(), Opcode::SpinTrigger, data, None);
  }

  create_binary_op_impl!(create_add, Opcode::Add);
  create_binary_op_impl!(create_sub, Opcode::Sub);
  create_binary_op_impl!(create_bitwise_and, Opcode::BitwiseAnd);
  create_binary_op_impl!(create_bitwise_or, Opcode::BitwiseOr);
  create_binary_op_impl!(create_bitwise_xor, Opcode::BitwiseXor);
  create_binary_op_impl!(create_mul, Opcode::Mul);
  create_binary_op_impl!(create_igt, Opcode::IGT);
  create_binary_op_impl!(create_ige, Opcode::IGE);
  create_binary_op_impl!(create_ilt, Opcode::ILT);
  create_binary_op_impl!(create_ile, Opcode::ILE);

  /// Create a register array associated to this system.
  /// An array can be a register, or memory.
  ///
  /// # Arguments
  /// * `ty` - The data type of data in the array.
  /// * `name` - The name of the array.
  /// * `size` - The size of the array.
  // TODO(@were): Add array types, memory, register, or signal wire.
  pub fn create_array(&mut self, ty: &DataType, name: &str, size: usize) -> Reference {
    let array_name = self.identifier(name);
    let instance = Array::new(ty.clone(), array_name.clone(), size);
    let key = self.insert_element(instance);
    self.sym_tab.insert(array_name, key.clone());
    key
  }

  /// Create a read operation on an array.
  ///
  /// # Arguments
  /// * `array` - The array to be read.
  /// * `index` - The index to be read.
  /// * `cond` - The condition of reading the array. If None is given, the read is unconditional.
  pub fn create_array_read<'elem>(
    &mut self,
    array: &Reference,
    index: &Reference,
    cond: Option<Reference>,
  ) -> Reference {
    let operands = vec![array.clone(), index.clone()];
    let dtype = self.get::<Array>(&array).unwrap().dtype().clone();
    let res = self.create_expr(dtype, Opcode::Load, operands, cond);
    let cur_mod = self.inesert_point.0.clone();
    self
      .get_mut::<Module>(&cur_mod)
      .unwrap()
      .insert_array_used(array.clone(), Opcode::Load);
    res
  }

  /// Create a write operation on an array.
  ///
  /// # Arguments
  /// * `array` - The array to be written.
  /// * `index` - The index to be written.
  /// * `value` - The value to be written.
  /// * `cond` - The condition of writing the array. If None is given, the write is unconditional.
  pub fn create_array_write(
    &mut self,
    array: &Reference,
    index: &Reference,
    value: &Reference,
    cond: Option<Reference>,
  ) -> Reference {
    let operands = vec![array.clone(), index.clone(), value.clone()];
    let res = self.create_expr(DataType::void(), Opcode::Store, operands, cond);
    let cur_mod = self.inesert_point.0.clone();
    self
      .get_mut::<Module>(&cur_mod)
      .unwrap()
      .insert_array_used(array.clone(), Opcode::Store);
    res
  }

  /// The helper function to combine the data types of two references.
  ///
  /// # Arguments
  /// * `op` - The operation code to be combined.
  /// * `a` - The lhs operand.
  /// * `b` - The rhs operand.
  fn combine_types(&self, op: Opcode, a: &Reference, b: &Reference) -> DataType {
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
    let mut printer = ir_printer::IRPrinter::new(self);
    write!(f, "system {} {{\n", self.name)?;
    for elem in self.array_iter() {
      write!(f, "\n  {};\n", printer.visit_array(elem))?;
    }
    printer.inc_indent();
    for elem in self.module_iter() {
      write!(f, "\n{}\n", printer.visit_module(elem))?;
    }
    printer.dec_indent();
    write!(f, "}}")
  }
}
