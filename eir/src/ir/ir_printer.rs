use std::collections::HashSet;

use crate::frontend::*;

use super::{block::Block, visitor::Visitor};

pub struct IRPrinter<'a> {
  indent: usize,
  sys: &'a SysBuilder,
}

impl IRPrinter<'_> {
  pub fn new(sys: &SysBuilder) -> IRPrinter {
    IRPrinter { indent: 0, sys }
  }
  pub fn inc_indent(&mut self) {
    self.indent += 2;
  }
  pub fn dec_indent(&mut self) {
    self.indent -= 2;
  }
}

struct ExtInterDumper<'a>(&'a HashSet<Opcode>);

impl Visitor<String> for ExtInterDumper<'_> {
  fn visit_input(&mut self, input: &FIFORef<'_>) -> Option<String> {
    let module = input.get_parent().as_ref::<Module>(input.sys).unwrap();
    let mut res = format!(
      "{}.{}: fifo<{}>, {{ ",
      module.get_name(),
      input.get_name(),
      input.scalar_ty().to_string()
    );
    for op in self.0.iter() {
      res.push_str(format!("{:?}, ", op).as_str());
    }
    res.push_str("}");
    res.into()
  }

  fn visit_array(&mut self, array: &ArrayRef<'_>) -> Option<String> {
    let mut res = format!(
      "Array: {}[{} x {}], {{ ",
      array.get_name(),
      array.get_size(),
      array.scalar_ty().to_string(),
    );
    for op in self.0.iter() {
      res.push_str(format!("{:?}, ", op).as_str());
    }
    res.push_str("}");
    res.into()
  }

  fn visit_module(&mut self, module: &ModuleRef<'_>) -> Option<String> {
    format!("Module: {}", module.get_name()).into()
  }
}

impl Visitor<String> for IRPrinter<'_> {
  fn visit_input(&mut self, input: &FIFORef<'_>) -> Option<String> {
    format!(
      "{}: fifo<{}>, ",
      input.get_name(),
      input.scalar_ty().to_string()
    )
    .into()
  }

  fn visit_array(&mut self, array: &ArrayRef<'_>) -> Option<String> {
    format!(
      "Array: {}[{} x {}]",
      array.get_name(),
      array.get_size(),
      array.scalar_ty().to_string(),
    )
    .into()
  }

  fn visit_int_imm(&mut self, int_imm: &IntImmRef<'_>) -> Option<String> {
    format!("({}:{})", int_imm.get_value(), int_imm.dtype().to_string()).into()
  }

  fn visit_module(&mut self, module: &ModuleRef<'_>) -> Option<String> {
    let mut res = String::new();
    for (elem, ops) in module.ext_interf_iter() {
      res.push_str(
        format!(
          "{}// {}\n",
          " ".repeat(self.indent),
          ExtInterDumper(ops)
            .dispatch(module.sys, elem, vec![])
            .unwrap()
        )
        .as_str(),
      );
    }
    res.push_str(format!("{}module {}(", " ".repeat(self.indent), module.get_name()).as_str());
    module.get();
    for elem in module.port_iter() {
      res.push_str(self.visit_input(&elem).unwrap().as_str());
    }
    res.push_str(") {\n");
    self.indent += 2;
    if module.get_name().eq("driver") {
      res.push_str(format!("{}while true {{\n", " ".repeat(self.indent)).as_str());
      self.indent += 2;
    }
    let InsertPoint(cur_mod, _, at) = self.sys.get_insert_point();
    for (i, elem) in module.get_body().iter().enumerate() {
      if cur_mod == module.upcast() && at.unwrap_or_else(|| module.get_num_exprs()) == i {
        res.push_str(format!("{}-----{{Insert Here}}-----\n", " ".repeat(self.indent)).as_str());
      }
      match elem.get_kind() {
        NodeKind::Expr => {
          let expr = elem.as_ref::<Expr>(self.sys).unwrap();
          res.push_str(format!("{}\n", self.visit_expr(&expr).unwrap()).as_str());
        }
        NodeKind::Block => {
          let block = elem.as_ref::<Block>(self.sys).unwrap();
          res.push_str(format!("{}\n", self.visit_block(&block).unwrap()).as_str());
        }
        _ => {
          panic!("Not an block-able element: {:?}", elem);
        }
      }
    }
    if at.is_none() && cur_mod == module.upcast() {
      res.push_str(format!("{}-----{{Insert Here}}-----\n", " ".repeat(self.indent)).as_str());
    }
    if module.get_name().eq("driver") {
      self.indent -= 2;
      res.push_str(format!("{}}}\n", " ".repeat(self.indent)).as_str());
    }
    self.indent -= 2;
    res.push_str(" ".repeat(self.indent).as_str());
    res.push_str("}\n");
    res.into()
  }

  fn visit_expr(&mut self, expr: &ExprRef<'_>) -> Option<String> {
    let mnem = expr.get_opcode().to_string();
    let res = if expr.get_opcode().is_binary() {
      format!(
        "_{} = {} {} {}",
        expr.get_key(),
        expr.get_operand(0).unwrap().to_string(self.sys),
        mnem,
        expr.get_operand(1).unwrap().to_string(self.sys)
      )
    } else if expr.get_opcode().is_unary() {
      format!(
        "_{} = {} {}",
        expr.get_key(),
        mnem,
        expr.get_operand(0).unwrap().to_string(self.sys)
      )
    } else {
      match expr.get_opcode() {
        Opcode::Load => {
          format!(
            "_{} = {}",
            expr.get_key(),
            expr.get_operand(0).unwrap().to_string(self.sys),
          )
        }
        Opcode::Store => {
          format!(
            "{} = {} // handle: _{}",
            expr.get_operand(0).unwrap().to_string(self.sys),
            expr.get_operand(1).unwrap().to_string(self.sys),
            expr.get_key()
          )
        }
        Opcode::Trigger => {
          let mut res = format!(
            "async call {}, timing [",
            expr.get_operand(0).unwrap().to_string(self.sys)
          );
          for op in expr.operand_iter().skip(1) {
            res.push('_');
            res.push_str(op.get_key().to_string().as_str());
            res.push(',');
            res.push(' ');
          }
          res.push(']');
          res
        }
        Opcode::SpinTrigger => {
          let mut res = String::new();
          self.indent += 2;
          res.push_str(
            format!(
              "async {{\n{}while !{} {{ }} // DO NOT move on until this is true\n",
              " ".repeat(self.indent),
              expr.get_operand(0).unwrap().to_string(self.sys),
            )
            .as_str(),
          );
          res.push_str(
            format!(
              "{}call {}(",
              " ".repeat(self.indent),
              expr.get_operand(1).unwrap().to_string(self.sys)
            )
            .as_str(),
          );
          for op in expr.operand_iter().skip(2) {
            res.push_str(op.to_string(self.sys).as_str());
            res.push_str(", ");
          }
          res.push_str(")\n");
          self.indent -= 2;
          res.push_str(format!("{}}}", " ".repeat(self.indent)).as_str());
          res
        }
        Opcode::FIFOPop => {
          // TODO(@were): Support multiple pop.
          let fifo = expr
            .get_operand(0)
            .unwrap()
            .as_ref::<FIFO>(self.sys)
            .unwrap();
          let module_name = {
            let parent = fifo.get_parent();
            let module = parent.as_ref::<Module>(self.sys).unwrap();
            module.get_name().to_string()
          };
          let fifo_name = fifo.get_name();
          format!("_{} = {}.{}.pop()", expr.get_key(), module_name, fifo_name)
        }
        Opcode::FIFOPush => {
          let module_name = expr.get_operand(0).unwrap().to_string(self.sys);
          let fifo_idx = expr
            .get_operand(1)
            .unwrap()
            .as_ref::<IntImm>(self.sys)
            .unwrap();
          let idx = fifo_idx.get_value();
          let module = expr.get_operand(0).unwrap();
          let fifo_name = if let Ok(module) = module.as_ref::<Module>(self.sys) {
            let fifo = module
              .get_input(idx as usize)
              .unwrap()
              .as_ref::<FIFO>(self.sys)
              .unwrap();
            fifo.get_name().clone()
          } else {
            "".to_string()
          };
          let to_push = format!("{}.{}", module_name, idx);
          let value = expr.get_operand(2).unwrap().to_string(self.sys);
          format!(
            "{}.push({}) // handle: _{}, fifo: {}",
            to_push,
            value,
            expr.get_key(),
            fifo_name,
          )
        }
        Opcode::CallbackTrigger => {
          let mut res = format!(
            "async call ({})(",
            expr.get_operand(0).unwrap().to_string(self.sys),
          );
          for op in expr.operand_iter().skip(1) {
            res.push_str(op.to_string(self.sys).as_str());
            res.push_str(", ");
          }
          res.push(')');
          res
        }
        _ => {
          panic!("Unimplemented opcode: {:?}", expr.get_opcode());
        }
      }
    };
    format!("{}{}", " ".repeat(self.indent), res).into()
  }
  fn visit_block(&mut self, block: &BlockRef<'_>) -> Option<String> {
    let mut res = String::new();
    if let Some(cond) = block.get_pred() {
      res.push_str(
        format!(
          "{}if {} {{\n",
          " ".repeat(self.indent),
          cond.to_string(self.sys)
        )
        .as_str(),
      );
      self.inc_indent();
    } else {
      res.push_str(format!("{}\n", block.get_key()).as_str());
    }
    for elem in block.iter() {
      match elem.get_kind() {
        NodeKind::Expr => {
          let expr = elem.as_ref::<Expr>(self.sys).unwrap();
          res.push_str(format!("{}\n", self.visit_expr(&expr).unwrap()).as_str());
        }
        NodeKind::Block => {
          let block = elem.as_ref::<Block>(self.sys).unwrap();
          res.push_str(format!("{}\n", self.visit_block(&block).unwrap()).as_str());
        }
        _ => {
          panic!("Not an block-able element: {:?}", elem);
        }
      }
    }
    if block.get_pred().is_some() {
      self.dec_indent();
      res.push_str(format!("{}}}", " ".repeat(self.indent)).as_str());
    }
    res.into()
  }
}
