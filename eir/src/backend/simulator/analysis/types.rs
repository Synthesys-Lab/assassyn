use std::collections::{HashMap, HashSet};

use crate::backend::simulator::utils::dtype_to_rust_type;
use crate::builder::SysBuilder;
use crate::ir::{DataType, Typed};

// TODO(@were): Is it possible to unify both arrays and fifos?

fn type_gather_impl(types: Vec<DataType>) -> HashMap<String, HashSet<DataType>> {
  let mut res = HashMap::new();
  for ty in types.into_iter() {
    let key = dtype_to_rust_type(&ty);
    let value_mut = if let Some(value) = res.get_mut(&key) {
      value
    } else {
      res.insert(key.clone(), HashSet::new());
      res.get_mut(&key).unwrap()
    };
    value_mut.insert(ty);
  }
  return res;
}

pub(in crate::backend::simulator) fn array_types_used(
  sys: &SysBuilder,
) -> HashMap<String, HashSet<DataType>> {
  let types = sys.array_iter().map(|x| x.dtype()).collect();
  return type_gather_impl(types);
}

pub(in crate::backend::simulator) fn fifo_types_used(
  sys: &SysBuilder,
) -> HashMap<String, HashSet<DataType>> {
  let types = sys
    .module_iter()
    .flat_map(|x| {
      x.port_iter()
        .map(|x| x.scalar_ty())
        .collect::<Vec<_>>()
        .into_iter()
    })
    .collect();
  return type_gather_impl(types);
}
