use std::fmt::Display;

use super::super::common::namify;
use crate::ir::{
  node::{ArrayRef, FIFORef, ModuleRef},
  DataType,
};

#[derive(Debug, Clone)]
pub(super) struct DisplayInstance {
  prefix: &'static str,
  id: String,
}

pub(super) trait Field {
  fn field(&self, attr: &str) -> String;
}

impl DisplayInstance {
  fn new(prefix: &'static str, id: String) -> DisplayInstance {
    DisplayInstance { prefix, id }
  }

  pub(super) fn from_module(module: &ModuleRef<'_>) -> Self {
    DisplayInstance::new("", namify(module.get_name()))
  }

  pub(super) fn from_array(array: &ArrayRef<'_>) -> Self {
    DisplayInstance::new("array", namify(array.get_name()))
  }

  pub(super) fn from_fifo(fifo: &FIFORef<'_>, global: bool) -> Self {
    let raw = namify(fifo.get_name());
    let fifo_name = if global {
      format!("{}_{}", namify(fifo.get_module().get_name()), raw)
    } else {
      raw
    };
    DisplayInstance::new("fifo", fifo_name)
  }
}

impl Field for DisplayInstance {
  fn field(&self, attr: &str) -> String {
    format!("{}_{}", self, attr)
  }
}

impl Display for DisplayInstance {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.prefix.is_empty() {
      write!(f, "{}", self.id)
    } else {
      write!(f, "{}_{}", self.prefix, self.id)
    }
  }
}

pub(super) struct Edge {
  instance: DisplayInstance,
  driver: String,
}

impl Edge {
  pub(super) fn new(instance: DisplayInstance, driver: &ModuleRef<'_>) -> Edge {
    Edge {
      instance,
      driver: namify(driver.get_name()),
    }
  }
}

impl Field for Edge {
  fn field(&self, field: &str) -> String {
    format!("{}_driver_{}_{}", self.instance, self.driver, field)
  }
}

pub(super) fn broadcast(value: String, bits: usize) -> String {
  format!("{{ {} {{ {} }} }}", bits, value)
}

pub(super) fn select_1h(iter: impl Iterator<Item = (String, String)>, bits: usize) -> String {
  reduce(
    iter.map(|(pred, value)| format!("({} & {})", broadcast(pred, bits), value)),
    " | ",
  )
}

pub(super) fn reduce(iter: impl Iterator<Item = String>, concat: &str) -> String {
  let res = iter.collect::<Vec<_>>().join(concat);
  if res.is_empty() {
    "'x".to_string()
  } else {
    res
  }
}

pub(super) fn bool_ty() -> DataType {
  DataType::int_ty(1)
}

fn declare_impl(
  decl_prefix: &'static str,
  ty: DataType,
  id: &String,
  term: &'static str,
) -> String {
  let bits = ty.get_bits() - 1;
  format!("  {} [{}:0] {}{}\n", decl_prefix, bits, id, term)
}

pub(super) fn declare_logic(ty: DataType, id: &String) -> String {
  declare_impl("logic", ty, id, ";")
}

pub(super) fn declare_in(ty: DataType, id: &String) -> String {
  declare_impl("input logic", ty, id, ",")
}

pub(super) fn declare_out(ty: DataType, id: &String) -> String {
  declare_impl("output logic", ty, id, ",")
}

pub(super) fn declare_array(array: &ArrayRef<'_>, id: &String, term: &str) -> String {
  let size = array.get_size();
  let ty = array.scalar_ty();
  format!("  logic [{}:0] {} [0:{}]{}\n", ty.get_bits() - 1, id, size - 1, term)
}

pub(super) fn connect_top<T: Field, U: Field>(display: &T, edge: &U, fields: &[&str]) -> String {
  let mut res = String::new();
  for field in fields {
    res.push_str(&format!("    .{}({}),\n", display.field(field), edge.field(field)));
  }
  res
}
