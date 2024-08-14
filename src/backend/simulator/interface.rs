use std::collections::HashSet;

use instructions::Bind;
use node::ModuleRef;
use visitor::Visitor;

use crate::backend::simulator::utils::dtype_to_rust_type;
use crate::builder::SysBuilder;
use crate::ir::node::*;
use crate::ir::*;

pub struct Interface {
  interf: BaseNode,
  suffix: String,
  ty: String,
  mutable: bool,
}

impl Interface {
  fn new(interf: BaseNode, suffix: &str, ty: String, mutable: bool) -> Self {
    Self {
      interf,
      suffix: suffix.to_string(),
      ty,
      mutable,
    }
  }

  pub(super) fn to_signature(
    &self,
    sys: &SysBuilder,
    mut dumper: super::elaborate::NodeRefDumper,
  ) -> String {
    format!(
      "{}{}: &{}{}",
      dumper.dispatch(sys, &self.interf, vec![]).unwrap(),
      self.suffix,
      if self.mutable { "mut " } else { "" },
      self.ty,
    )
  }

  pub(super) fn to_arg_feed(
    &self,
    sys: &SysBuilder,
    mut dumper: super::elaborate::NodeRefDumper,
  ) -> String {
    format!(
      "&{}self.{}{}",
      if self.mutable { "mut " } else { "" },
      dumper.dispatch(sys, &self.interf, vec![]).unwrap(),
      self.suffix
    )
  }

  pub(super) fn to_attr_decl(
    &self,
    sys: &SysBuilder,
    mut dumper: super::elaborate::NodeRefDumper,
  ) -> String {
    format!(
      "{}{}: {}",
      dumper.dispatch(sys, &self.interf, vec![]).unwrap(),
      self.suffix,
      self.ty
    )
  }
}

pub(in crate::backend::simulator) fn analyze_module_external_interfaces(
  m: &ModuleRef<'_>,
) -> Vec<Interface> {
  let mut res = Vec::new();
  // All the writes will be done in half a cycle later by events, so no need to feed them
  // to the function signature.
  for (interf, operands) in m.ext_interf_iter() {
    let operations = operands
      .iter()
      .map(|x| {
        let operand = x.as_ref::<Operand>(m.sys).unwrap();
        operand
          .get_parent()
          .as_ref::<Expr>(m.sys)
          .unwrap()
          .get_opcode()
      })
      .collect::<HashSet<_>>();
    if let Ok(array) = interf.as_ref::<Array>(m.sys) {
      if operations.contains(&Opcode::Load) {
        res.push(Interface::new(
          *interf,
          "",
          dtype_to_rust_type(&array.dtype()),
          false,
        ));
      }
      if operations.contains(&Opcode::Store) {
        res.push(Interface::new(
          *interf,
          "_write",
          format!(
            "XEQ<ArrayWrite<{}>>",
            dtype_to_rust_type(&array.scalar_ty())
          ),
          true,
        ));
      }
    } else if let Ok(fifo) = interf.as_ref::<FIFO>(m.sys) {
      assert!(operations.contains(&Opcode::FIFOPush), "{:?}", operations);
      res.push(Interface::new(
        *interf,
        "_push",
        format!("XEQ<FIFOPush<{}>>", dtype_to_rust_type(&fifo.scalar_ty())),
        true,
      ));
    } else if let Ok(bind) = interf.as_expr::<Bind>(m.sys) {
      res.push(Interface::new(
        bind.callee().upcast(),
        "_event",
        "VecDeque<usize>".into(),
        true,
      ));
    }
  }
  res
}
