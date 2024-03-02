use crate::{
  builder::system::SysBuilder,
  expr::{Expr, Opcode},
  ir::block::Block,
  BaseNode,
};

fn iter_over_body<'a>(
  sys: &SysBuilder,
  iter: impl Iterator<Item = &'a BaseNode>,
) -> Option<BaseNode> {
  for elem in iter {
    match elem {
      BaseNode::Block(_) => {
        if let Some(x) = iter_over_body(sys, elem.as_ref::<Block>(sys).unwrap().iter()) {
          return x.into();
        }
      }
      BaseNode::Expr(_) => {
        if let Opcode::SpinTrigger = elem.as_ref::<Expr>(sys).unwrap().get_opcode() {
          return elem.clone().into();
        }
      }
      _ => {
        panic!("Unexpected reference")
      }
    }
  }
  None
}

fn find_spin_trigger(sys: &SysBuilder) -> Option<BaseNode> {
  for module in sys.module_iter() {
    if let Some(x) = iter_over_body(sys, module.get_body().iter()) {
      return x.into()
    }
  }
  None
}

pub fn rewrite_spin_triggers(sys: &mut SysBuilder) {
  if let Some(spin_trigger) = find_spin_trigger(sys) {
  }
}
