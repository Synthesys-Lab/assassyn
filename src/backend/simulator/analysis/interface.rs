use std::collections::HashSet;

use instructions::Bind;
use node::ModuleRef;

use crate::backend::simulator::utils::dtype_to_rust_type;
use crate::ir::node::*;
use crate::ir::*;

pub(in crate::backend::simulator) fn analyze_module_external_interfaces(
  m: &ModuleRef<'_>,
) -> Vec<(BaseNode, String, String)> {
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
        res.push((*interf, "".to_string(), dtype_to_rust_type(&array.dtype())));
      }
      if operations.contains(&Opcode::Store) {
        res.push((
          *interf,
          "_write".to_string(),
          format!(
            "&mut VecDeque<(usize, usize, {}, String)>",
            dtype_to_rust_type(&array.scalar_ty())
          ),
        ));
      }
    } else if let Ok(fifo) = interf.as_ref::<FIFO>(m.sys) {
      assert!(operations.contains(&Opcode::FIFOPush), "{:?}", operations);
      res.push((
        *interf,
        "_push".to_string(),
        format!(
          "&mut VecDeque<(usize, {}, String)>",
          dtype_to_rust_type(&fifo.scalar_ty())
        ),
      ));
    } else if let Ok(bind) = interf.as_expr::<Bind>(m.sys) {
      res.push((
        bind.callee().upcast(),
        "_event".to_string(),
        "&mut VecDeque<usize>".into(),
      ));
    }
  }
  res
}
