use std::fmt::Display;

use super::super::common::namify;
use crate::ir::node::{ArrayRef, FIFORef, ModuleRef};

#[derive(Debug, Clone)]
pub(super) struct DisplayInstance {
  prefix: &'static str,
  id: String,
}

impl DisplayInstance {
  fn new(prefix: &'static str, id: String) -> DisplayInstance {
    DisplayInstance { prefix, id }
  }

  pub(super) fn field(&self, field: &str) -> String {
    format!("{}_{}", self, field)
  }

  pub(super) fn from_array(array: &ArrayRef<'_>) -> Self {
    DisplayInstance::new("array", namify(array.get_name()))
  }

  pub(super) fn from_fifo(fifo: &FIFORef<'_>) -> Self {
    DisplayInstance::new("fifo", namify(fifo.get_name()))
  }
}

impl Display for DisplayInstance {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}_{}", self.prefix, self.id)
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

  pub(super) fn field(&self, field: &str) -> String {
    format!("{}_driver_{}_{}", self.instance, self.driver, field)
  }
}

pub(super) fn broadcast(value: String, bits: usize) -> String {
  format!("{{ {} {{ {} }} }}", bits, value)
}

pub(super) fn select_1h(iter: impl Iterator<Item = (String, String)>, bits: usize) -> String {
  iter
    .map(|(pred, value)| format!("({} & {})", broadcast(pred, bits), value))
    .collect::<Vec<_>>()
    .join(" | ")
}
