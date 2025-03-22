use std::{collections::VecDeque, fmt::Display};

use super::super::common::namify;
use crate::{
  builder::SysBuilder,
  ir::{
    node::{ArrayRef, BaseNode, FIFORef, ModuleRef},
    DataType, StrImm,
  },
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

pub(super) fn declare_array(prefix: &str, array: &ArrayRef<'_>, id: &String, term: &str) -> String {
  let size = array.get_size();
  let ty = array.scalar_ty();
  format!(
    "  {}logic [{}:0] {}{}\n",
    if prefix.is_empty() {
      "".into()
    } else {
      format!("{} ", prefix)
    },
    (ty.get_bits() * size) - 1,
    id,
    term
  )
}

pub(super) fn connect_top<T: Field, U: Field>(display: &T, edge: &U, fields: &[&str]) -> String {
  let mut res = String::new();
  for field in fields {
    res.push_str(&format!("    .{}({}),\n", display.field(field), edge.field(field)));
  }
  res
}

fn type_to_fmt(ty: &DataType) -> String {
  match ty {
    DataType::Int(_) | DataType::UInt(_) | DataType::Bits(_) => "d",
    _ => panic!("Invalid type for type: {}", ty),
  }
  .to_string()
}

pub(super) fn parse_format_string(args: Vec<BaseNode>, sys: &SysBuilder) -> String {
  let raw = args[0]
    .as_ref::<StrImm>(sys)
    .unwrap()
    .get_value()
    .to_string();
  let mut fmt = raw.chars().collect::<VecDeque<_>>();
  let mut res = String::new();
  let mut arg_idx = 1;
  while let Some(c) = fmt.pop_front() {
    match c {
      '{' => {
        if let Some(c) = fmt.pop_front() {
          if c == '{' {
            res.push('{');
          } else {
            let dtype = args[arg_idx].get_dtype(sys).unwrap();
            let mut substr = c.to_string();
            // handle "{}"
            if substr.eq("}") {
              res.push_str(&format!("%{}", &type_to_fmt(&dtype)));
              arg_idx += 1;
              continue;
            }
            let mut closed = false;
            while let Some(c) = fmt.pop_front() {
              if c == '}' {
                closed = true;
                break;
              }
              substr.push(c);
            }
            assert!(closed, "Invalid format string, because of a single {{, {:?}", raw);
            let new_fmt = if substr.is_empty() {
              type_to_fmt(&dtype)
            } else if matches!(substr.chars().next(), Some(':')) {
              let mut width_idx = 1;
              let pad = if matches!(substr.chars().nth(1), Some('0')) {
                width_idx += 1;
                String::from("0")
              } else {
                String::new()
              };
              let width = if substr
                .chars()
                .nth(width_idx)
                .is_some_and(|c| c.is_ascii_digit())
              {
                width_idx += 1;
                substr
                  .chars()
                  .nth(width_idx - 1)
                  .unwrap()
                  .to_digit(10)
                  .unwrap()
                  .to_string()
              } else {
                String::new()
              };
              let vfmt = if substr
                .chars()
                .nth(width_idx)
                .is_some_and(|c| c.is_alphabetic())
              {
                substr.chars().nth(width_idx).unwrap().to_string()
              } else {
                type_to_fmt(&dtype)
              };
              format!("%{}{}{}", pad, width, vfmt)
            } else {
              panic!("Invalid format string {:?}", raw);
            };
            res.push_str(&new_fmt);
            arg_idx += 1;
          }
        } else {
          panic!("Invalid format string, {:?}", raw);
        }
      }
      '}' => {
        if let Some(c) = fmt.pop_front() {
          if c == '}' {
            res.push('}');
            continue;
          }
        }
        panic!("Invalid format string, because of a single {{");
      }
      _ => res.push(c),
    }
  }
  res
}
