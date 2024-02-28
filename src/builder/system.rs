use std::{collections::HashMap, fmt::Display, ops::Add};

use crate::{
  builder::ir_printer,
  data::{Array, Typed},
  expr::{Expr, Opcode},
  port::Input,
  reference::{Element, IsElement, Visitor},
  DataType, IntImm, Module, Reference,
};

/// A `SysBuilder` struct not only serves as the data structure of the whole system,
/// but also works as the syntax-sugared IR builder.
pub struct SysBuilder {
  /// The slab to store all the elements in the system. We use a slab to maintain such a
  /// highly redundant, and mutually referenced data structure.
  pub(super) slab: slab::Slab<Element>,
  /// The data structure caches the constant values.
  const_cache: HashMap<(DataType, u64), Reference>,
  /// The name of the system.
  name: String,
  /// The arrays of this system.
  arrays: Vec<Reference>,
  /// The modules of this system.
  mods: Vec<Reference>,
  /// The current module to be built.
  cur_mod: Option<Reference>,
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
      a: Reference,
      b: Reference,
      pred: Option<Reference>,
    ) -> Reference {
      let res_ty = if let Some(ty) = ty {
        ty
      } else {
        self.combine_types($opcode, &a, &b)
      };
      self.create_expr(res_ty, $opcode, vec![a, b], pred)
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
      arrays: vec![],
      slab: slab::Slab::new(),
      mods: Vec::new(),
      const_cache: HashMap::new(),
      cur_mod: None,
    };
    let driver = Module::new("driver", vec![]);
    let key = res.slab.insert(Element::Module(driver.into()));
    res.mods.push(Reference::Module(key));
    res
  }

  /// The helper function to get an element of the system and downcast it to its actual
  /// type's immutable reference.
  pub(crate) fn get<'a, T: IsElement<'a>>(&'a self, key: &Reference) -> Result<&'a Box<T>, String> {
    T::downcast(&self.slab, key)
  }

  /// Iterate over all the modules of the system.
  pub fn module_iter<'a>(&'a self) -> impl Iterator<Item = &'a Box<Module>> {
    self.mods.iter().map(|x| x.as_ref::<Module>(self).unwrap())
  }

  /// Iterate over all the arrays of the system.
  pub fn array_iter<'a>(&'a self) -> impl Iterator<Item = &'a Box<Array>> {
    self.arrays.iter().map(|x| x.as_ref::<Array>(self).unwrap())
  }

  /// The helper function to get an element of the system and downcast it to its actual type's
  /// mutable reference.
  pub(crate) fn get_mut<'a, T: IsElement<'a>>(
    &'a mut self,
    key: &Reference,
  ) -> Result<&'a mut Box<T>, String> {
    T::downcast_mut(&mut self.slab, key)
  }

  /// Get the driver module. The driver module is special. It is invoked unconditionally every
  /// cycle.
  pub fn get_driver(&self) -> &Module {
    self.get::<Module>(self.mods.first().unwrap()).unwrap()
  }

  /// Set the current module to be built. All the created elements will be inserted into this
  /// module.
  ///
  /// # Arguments
  ///
  /// * `module` - The reference of the module to be set as the current module.
  pub fn set_current_module(&mut self, module: Reference) {
    self.cur_mod = Some(module);
  }

  /// The helper function to insert an element into the system's slab.
  /// We adopt a slab to maintain such a highly redundant, and mutually referenced data structure.
  ///
  /// # Arguments
  ///
  /// * `elem` - The element to be inserted. An element can be any component of the system IR.
  pub(super) fn insert<'a, T: IsElement<'a> + Into<Element> + 'a>(
    &'a mut self,
    elem: T,
  ) -> Reference {
    let key = self.slab.insert(elem.into());
    let res = T::into_reference(key);
    self.get_mut::<T>(&res).unwrap().set_key(key);
    res
  }

  /// The helper function to create a constant integer.
  ///
  /// # Arguments
  ///
  /// * `dtype` - The data type of the constant.
  /// * `value` - The value of the constant.
  // TODO(@were): What if the data type is bigger than 64 bits?
  pub fn get_const_int(&mut self, dtype: DataType, value: u64) -> Reference {
    let key = (dtype, value);
    if let Some(cached) = self.const_cache.get(&key) {
      return cached.clone();
    }
    let cloned_key = key.clone();
    let instance = IntImm::new(key.0, key.1);
    let key = self.insert(instance);
    self.const_cache.insert(cloned_key, key.clone());
    key
  }

  /// Create a new module, and set it as the current module to be built.
  ///
  /// # Arguments
  ///
  /// * `name` - The name of the module.
  /// * `inputs` - The inputs' information to the module. Refer to `PortInfo` for more details.
  // TODO(@were): Rename the module while this name is occupied.
  pub fn create_module(&mut self, name: &str, inputs: Vec<PortInfo>) -> Reference {
    let ports = inputs
      .into_iter()
      .map(|x| self.insert(Input::new(&x.ty, x.name.as_str())))
      .collect::<Vec<_>>();
    let module = Module::new(name, ports);
    // Set the parents of the inputs after instantiating the parent module.
    for i in 0..module.get_num_inputs() {
      let input = module.get_input(i).unwrap();
      self.get_mut::<Input>(input).unwrap().parent = module.upcast();
    }
    let key = self.insert(module);
    self.mods.push(key.clone());
    self.cur_mod = Some(key.clone());
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
  pub fn create_expr(
    &mut self,
    dtype: DataType,
    opcode: Opcode,
    operands: Vec<Reference>,
    cond: Option<Reference>,
  ) -> Reference {
    let cur_mod = if let Some(cur_mod) = &self.cur_mod {
      cur_mod.clone()
    } else {
      panic!("No module to insert into!");
    };
    let instance = Expr::new(dtype.clone(), opcode, operands, cur_mod.clone());
    let key = {
      let origin = self.insert(instance);
      if let Some(cond) = cond {
        let predicated = Expr::new(
          DataType::void(),
          Opcode::Predicate,
          vec![cond, origin.clone()],
          cur_mod.clone(),
        );
        let res = self.insert(predicated);
        self.get_mut::<Expr>(&origin).unwrap().parent = (res.clone(), 1);
        res
      } else {
        origin
      }
    };
    self.get_mut::<Module>(&cur_mod).unwrap().push(key)
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
    dst: Reference,
    mut data: Vec<Reference>,
    cond: Option<Reference>,
  ) {
    data.insert(0, dst);
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
  // TODO(@were): Rename the array while this name is occupied.
  // TODO(@were): Add array types, memory, register, or signal wire.
  pub fn create_array(&mut self, ty: DataType, name: &str, size: usize) -> Reference {
    let instance = Array::new(ty, name.into(), size);
    let key = self.insert(instance);
    self.arrays.push(key.clone());
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
    array: Reference,
    index: Reference,
    cond: Option<Reference>,
  ) -> Reference {
    let operands = vec![array.clone(), index];
    let dtype = self.get::<Array>(&array).unwrap().dtype().clone();
    let res = self.create_expr(dtype, Opcode::Load, operands, cond);
    let cur_mod = self.cur_mod.as_ref().unwrap().clone();
    self
      .get_mut::<Module>(&cur_mod)
      .unwrap()
      .insert_array_used(array, Opcode::Load);
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
    array: Reference,
    index: Reference,
    value: Reference,
    cond: Option<Reference>,
  ) -> Reference {
    let operands = vec![array.clone(), index, value];
    let res = self.create_expr(DataType::void(), Opcode::Store, operands, cond);
    let cur_mod = self.cur_mod.as_ref().unwrap().clone();
    self
      .get_mut::<Module>(&cur_mod)
      .unwrap()
      .insert_array_used(array, Opcode::Store);
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
      Opcode::Add | Opcode::Sub => {
        match (&aty, &bty) {
          (DataType::Int(a), DataType::Int(b)) => {
            DataType::Int(*a.max(b))
          }
          (DataType::UInt(a), DataType::UInt(b)) => {
            DataType::UInt(*a.max(b))
          }
          _ => panic!("Cannot combine types {} and {}", aty.to_string(), bty.to_string()),
        }
      }
      Opcode::Mul => {
        match (&aty, &bty) {
          (DataType::Int(a), DataType::Int(b)) => {
            DataType::Int(a + b)
          }
          (DataType::UInt(a), DataType::UInt(b)) => {
            DataType::UInt(a.add(b))
          }
          _ => panic!("Cannot combine types {} and {}", aty.to_string(), bty.to_string()),
        }
      }
      Opcode::IGT | Opcode::IGE | Opcode::ILT | Opcode::ILE => {
        DataType::uint(1)
      }
      _ => panic!("Unsupported opcode {:?}", op),
    }
  }
}

impl Display for SysBuilder {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut printer = ir_printer::IRPrinter::new(self);

    write!(f, "system {} {{\n", self.name)?;
    for elem in self.arrays.iter() {
      let array = elem.as_ref::<Array>(self).unwrap();
      write!(f, "\n  {};\n", printer.visit_array(array))?;
    }
    printer.inc_indent();
    for elem in self.mods.iter() {
      let module = elem.as_ref::<Module>(self).unwrap();
      write!(f, "\n{}\n", printer.visit_module(module))?;
    }
    printer.dec_indent();
    write!(f, "}}")
  }
}
