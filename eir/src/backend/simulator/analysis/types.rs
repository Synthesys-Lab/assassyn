use std::collections::HashSet;

use crate::builder::SysBuilder;
use crate::ir::{DataType, Typed};

// TODO(@were): Is it possible to unify both arrays and fifos?

pub(in crate::backend::simulator) fn array_types_used(sys: &SysBuilder) -> HashSet<DataType> {
  let mut res = HashSet::new();
  for array in sys.array_iter() {
    res.insert(array.dtype());
  }
  return res;
}

pub(in crate::backend::simulator) fn fifo_types_used(sys: &SysBuilder) -> HashSet<DataType> {
  let mut res = HashSet::new();
  for array in sys.module_iter() {
    for fifo in array.port_iter() {
      res.insert(fifo.scalar_ty());
    }
  }
  return res;
}
