use crate::{
  data::{Array, Typed},
  expr::{Expr, Opcode},
  port::Input,
  reference::IsElement,
  IntImm, Module,
};

use super::{reference::Visitor, system::SysBuilder};

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

impl<'a> Visitor<'a, String> for IRPrinter<'a> {


  fn visit_input(&mut self, input: &'a Input) -> String {
    format!("{}: {}, ", input.get_name(), input.dtype().to_string())
  }

  fn visit_array(&mut self, array: &'a Array) -> String {
    format!(
      "Array: {} {}[{}]",
      array.dtype().to_string(),
      array.get_name(),
      array.get_size()
    )
  }

  fn visit_int_imm(&mut self, int_imm: &'a IntImm) -> String {
    format!(
      "({} as {})",
      int_imm.get_value(),
      int_imm.dtype().to_string()
    )
  }

  fn visit_module(&mut self, module: &'a Module) -> String {
    let mut res = String::new();
    res.push_str(format!("{}module {}(", " ".repeat(self.indent), module.get_name()).as_str());
    for elem in module.port_iter(self.sys) {
      res.push_str(self.visit_input(elem).as_str());
    }
    res.push_str(") {\n");
    self.indent += 2;
    if module.get_name().eq("driver") {
      res.push_str(format!("{}while true {{\n", " ".repeat(self.indent)).as_str());
      self.indent += 2;
    }
    let (mod_ref, at) = self.sys.get_insert_point();
    for (i, expr) in module.expr_iter(self.sys).enumerate() {
      if mod_ref == module.upcast() && at.unwrap_or_else(|| module.get_num_expr()) == i {
        res.push_str(format!("{}-----{{Insert Here}}-----\n", " ".repeat(self.indent)).as_str());
      }
      res.push_str(format!("{}\n", self.visit_expr(expr)).as_str());
    }
    if at.is_none() && mod_ref == module.upcast() {
      res.push_str(format!("{}-----{{Insert Here}}-----\n", " ".repeat(self.indent)).as_str());
    }
    if module.get_name().eq("driver") {
      self.indent -= 2;
      res.push_str(format!("{}}}\n", " ".repeat(self.indent)).as_str());
    }
    self.indent -= 2;
    res.push_str(" ".repeat(self.indent).as_str());
    res.push_str("}\n\n");
    res
  }

  fn visit_expr(&mut self, expr: &'a Expr) -> String {
    let mnem = expr.get_opcode().to_string();
    let res = if expr.get_opcode().is_binary() {
      format!(
        "_{} = {} {} {}",
        expr.get_key(),
        expr.get_operand(0).unwrap().to_string(self.sys),
        mnem,
        expr.get_operand(1).unwrap().to_string(self.sys)
      )
    } else if let Opcode::Predicate = expr.get_opcode() {
      let mut tmp = format!(
        "if {} {{\n",
        expr.get_operand(0).unwrap().to_string(self.sys)
      );
      self.indent += 2;
      for operand in expr.operand_iter().skip(1) {
        tmp.push_str(
          format!(
            "{}\n",
            self.visit_expr(operand.as_ref::<Expr>(self.sys).unwrap())
          )
          .as_str(),
        );
      }
      self.indent -= 2;
      tmp.push_str(format!("{}}}", " ".repeat(self.indent)).as_str());
      tmp
    } else {
      match expr.get_opcode() {
        Opcode::Load => {
          format!(
            "_{} = {}[{}]",
            expr.get_key(),
            expr.get_operand(0).unwrap().to_string(self.sys),
            expr.get_operand(1).unwrap().to_string(self.sys)
          )
        }
        Opcode::Store => {
          format!(
            "{}[{}] = {}",
            expr.get_operand(0).unwrap().to_string(self.sys),
            expr.get_operand(1).unwrap().to_string(self.sys),
            expr.get_operand(2).unwrap().to_string(self.sys)
          )
        }
        Opcode::Trigger => {
          let mut res = format!(
            "self.{}(\"{}\", [",
            mnem,
            expr.get_operand(0).unwrap().to_string(self.sys)
          );
          for op in expr.operand_iter().skip(1) {
            res.push_str(op.to_string(self.sys).as_str());
            res.push_str(", ");
          }
          res.push_str("])");
          res
        }
        Opcode::SpinTrigger => {
          format!(
            "{} {}",
            mnem,
            expr.get_operand(0).unwrap().to_string(self.sys)
          )
        }
        _ => {
          panic!("Unimplemented opcode: {:?}", expr.get_opcode());
        }
      }
    };
    format!("{}{}", " ".repeat(self.indent), res)
  }
}
