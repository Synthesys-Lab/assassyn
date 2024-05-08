use std::collections::{HashMap, HashSet};

use crate::{
  builder::{InsertPoint, SysBuilder},
  ir::{
    // ir_printer::IRPrinter,
    node::{IsElement, NodeKind},
    Expr,
    Module,
    Opcode,
    Operand,
  },
};

pub fn unify_common_reads(sys: &mut SysBuilder) {
  let mut to_rewrite = HashMap::new();
  for module in sys.module_iter() {
    let module_key = module.upcast();
    // eprintln!("Module: {}", module.get_name());
    for (ext_node, ops) in module.external_interfaces.iter() {
      // eprintln!("External Node: {:?}", ext_node);
      if ext_node.get_kind() == NodeKind::Array {
        for op in ops.iter() {
          let operand = op.as_ref::<Operand>(sys).unwrap();
          if let Ok(expr) = operand.get_user().as_ref::<Expr>(sys) {
            if expr.get_opcode() == Opcode::Load {
              let ptr = expr.get_operand(0).unwrap().get_value().clone();
              if !to_rewrite.contains_key(&(module_key, ptr)) {
                to_rewrite.insert((module_key, ptr).clone(), HashSet::new());
              }
              let operators = to_rewrite.get_mut(&(module_key, ptr)).unwrap();
              operators.insert(expr.upcast());
              // eprintln!(
              //   "ArrayPtr: {:?}, Expr: {}",
              //   ptr,
              //   IRPrinter::new(false).visit_expr(&expr).unwrap()
              // );
            }
          }
        }
      }
    }
  }

  for ((module, ptr), loaders) in to_rewrite.iter() {
    if loaders.len() > 1 {
      let module = module.as_ref::<Module>(sys).unwrap();
      let ip = InsertPoint(module.upcast(), module.get_body().upcast(), Some(0));
      sys.set_current_ip(ip);
      let mut name = format!("read.{}", ptr.get_key());
      for loader in loaders.iter() {
        name.push_str(&format!(".{}", loader.to_string(sys)));
      }
      let unified = sys.create_array_read(ptr.clone());
      unified.as_mut::<Expr>(sys).unwrap().set_name(name);

      for loader in loaders.iter() {
        sys.replace_all_uses_with(loader.clone(), unified.clone());
        let mut loader_mut = loader.as_mut::<Expr>(sys).unwrap();
        loader_mut.erase_from_parent();
      }
    }
  }
}
