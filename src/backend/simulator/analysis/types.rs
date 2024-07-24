use std::collections::{HashMap, HashSet};

use crate::backend::simulator::utils::dtype_to_rust_type;
use crate::builder::SysBuilder;
use crate::ir::node::ModuleRef;
use crate::ir::visitor::Visitor;
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
  res
}

pub(in crate::backend::simulator) fn array_types_used(
  sys: &SysBuilder,
) -> HashMap<String, HashSet<DataType>> {
  let types = sys.array_iter().map(|x| x.dtype()).collect();
  type_gather_impl(types)
}

struct FIFOTypesUsedVisitor {
  res: Vec<DataType>,
}

impl Visitor<()> for FIFOTypesUsedVisitor {
  fn visit_module(&mut self, module: ModuleRef<'_>) -> Option<()> {
    self.res.extend(module.port_iter().map(|x| x.scalar_ty()));
    None
  }
}

pub(in crate::backend::simulator) fn fifo_types_used(
  sys: &SysBuilder,
) -> HashMap<String, HashSet<DataType>> {
  let mut visitor = FIFOTypesUsedVisitor { res: Vec::new() };
  visitor.enter(sys);
  type_gather_impl(visitor.res)
}
