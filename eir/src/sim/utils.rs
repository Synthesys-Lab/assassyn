use std::collections::HashSet;

use crate::{
  builder::SysBuilder,
  ir::{expr::OperandOf, DataType, Expr, Opcode},
};

pub(super) fn namify(name: &str) -> String {
  name.replace(".", "_")
}

pub(super) fn camelize(name: &str) -> String {
  let mut result = String::new();
  let mut capitalize = true;
  for c in name.chars() {
    if c == '_' {
      capitalize = true;
    } else if capitalize {
      result.push(c.to_ascii_uppercase());
      capitalize = false;
    } else {
      result.push(c);
    }
  }
  result
}

pub(super) fn unwrap_array_ty(dty: &DataType) -> (DataType, usize) {
  match dty {
    DataType::ArrayType(ty, size) => (ty.as_ref().clone(), *size),
    _ => panic!("Expected array type, found {:?}", dty),
  }
}

pub(super) fn dtype_to_rust_type(dtype: &DataType) -> String {
  if dtype.is_int() {
    let prefix = if dtype.is_signed() { "i" } else { "u" };
    let bits = dtype.bits();
    return if bits.is_power_of_two() && bits >= 8 && bits <= 64 {
      format!("{}{}", prefix, dtype.bits())
    } else if bits == 1 {
      "bool".to_string()
    } else if bits.is_power_of_two() && bits < 8 {
      format!("{}8", prefix)
    } else {
      format!("{}{}", prefix, dtype.bits().next_power_of_two())
    };
  }
  match dtype {
    DataType::Module(_) => {
      format!("Box<EventKind>",)
    }
    DataType::ArrayType(ty, size) => {
      format!("[{}; {}]", dtype_to_rust_type(ty), size)
    }
    _ => panic!("Not implemented yet, {:?}!", dtype),
  }
}

pub(super) fn array_ty_to_id(scalar_ty: &DataType, size: usize) -> String {
  format!("{}x{}", dtype_to_rust_type(scalar_ty), size)
}

pub(super) fn user_contains_opcode(
  sys: &SysBuilder,
  users: &HashSet<OperandOf>,
  ops: Vec<Opcode>,
) -> bool {
  users
    .iter()
    .any(|operand_of| ops.contains(&operand_of.user.as_ref::<Expr>(sys).unwrap().get_opcode()))
}
