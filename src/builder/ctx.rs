use slab;

use crate::{Data, Module};

use super::data::Expr;

pub enum Element {
  Module(Module),
  Data(Data),
  Expr(Expr),
}

pub static mut CONTEXT_SLAB: slab::Slab<Element> = slab::Slab::new();

