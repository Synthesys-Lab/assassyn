use std::{collections::HashSet, fs::{self, File}, io::Write};

use crate::{
  builder::system::SysBuilder,
  context::{IsElement, Visitor},
  data::{Array, Typed},
  expr::{Expr, Opcode},
  port::Input,
  IntImm, Module,
};

struct ElaborateModule<'a> {
  sys: &'a SysBuilder,
  port_idx: usize,
  ops: Option<&'a HashSet<Opcode>>,
}

impl<'a> ElaborateModule<'a> {
  fn new(sys: &'a SysBuilder) -> Self {
    Self {
      sys,
      port_idx: 0,
      ops: None,
    }
  }
}

impl<'a> Visitor<'a, String> for ElaborateModule<'a> {
  fn visit_module(&mut self, module: &'a Module) -> String {
    let mut res = String::new();
    res.push_str(format!("// Elaborating module {}\n", module.get_name()).as_str());
    res.push_str(
      format!(
        "fn {}(q: &mut VecDeque<Event>, args: Vec<u64>",
        module.get_name()
      )
      .as_str(),
    );
    for (array, ops) in module.array_iter(self.sys) {
      self.ops = Some(ops);
      res.push_str(self.visit_array(array).as_str());
    }
    res.push_str(") {\n");
    for (i, arg) in module.port_iter(self.sys).enumerate() {
      self.port_idx = i;
      res.push_str(self.visit_input(arg).as_str());
    }
    for elem in module.expr_iter(self.sys) {
      res.push_str(self.visit_expr(elem).as_str());
    }
    res.push_str("}\n");
    res
  }

  fn visit_input(&mut self, input: &Input) -> String {
    format!(
      "  let {} = (*args.get({}).unwrap()) as {};\n",
      input.get_name(),
      self.port_idx,
      input.dtype().to_string()
    )
  }

  fn visit_expr(&mut self, expr: &Expr) -> String {
    match expr.get_opcode() {
      Opcode::Add | Opcode::Mul => {
        format!(
          "  let _{} = {} {} {};\n",
          expr.get_key(),
          expr.get_operand(0).unwrap().to_string(self.sys),
          expr.get_opcode().to_string(),
          expr.get_operand(1).unwrap().to_string(self.sys)
        )
      }
      Opcode::Load => {
        format!(
          "  let _{} = {}[{} as usize];\n",
          expr.get_key(),
          expr.get_operand(0).unwrap().to_string(self.sys),
          expr.get_operand(1).unwrap().to_string(self.sys)
        )
      }
      Opcode::Store => {
        format!(
          "  {}[{} as usize] = {};\n",
          expr.get_operand(0).unwrap().to_string(self.sys),
          expr.get_operand(1).unwrap().to_string(self.sys),
          expr.get_operand(2).unwrap().to_string(self.sys)
        )
      }
      Opcode::Trigger => {
        let module_name = expr
          .get_operand(0)
          .unwrap()
          .as_ref::<Module>(self.sys)
          .unwrap()
          .get_name();
        let mut res = format!("q.push_back(Event::Module_{}(vec![", module_name);
        for args in expr.operand_iter().skip(1) {
          res.push_str(format!("{}, ", args.to_string(self.sys)).as_str());
        }
        res.push_str("]);\n");
        res
      }
      _ => {
        format!("  // TODO: Other opcode;\n")
      }
    }
  }

  fn visit_array(&mut self, array: &Array) -> String {
    format!(
      ", {}: &{}Vec<{}>",
      array.get_name(),
      if self.ops.unwrap().contains(&Opcode::Store) {
        "mut "
      } else {
        ""
      },
      array.dtype().to_string()
    )
  }

  fn visit_int_imm(&mut self, int_imm: &IntImm) -> String {
    int_imm.to_string()
  }
}

fn dump_runtime(sys: &SysBuilder, fd: &mut File) -> Result<(), std::io::Error> {
  fd.write("// Simulation runtime.\n".as_bytes())?;
  fd.write("enum Event {\n".as_bytes())?;
  for module in sys.module_iter() {
    fd.write(format!("  Module_{}(Vec<u64>),\n", module.get_name()).as_bytes())?;
  }
  fd.write("}\n\n".as_bytes())?;

  fd.write("fn driver_only(q: &VecDeque<Event>) -> bool {\n".as_bytes())?;
  fd.write("  for event in q.iter() {\n".as_bytes())?;
  fd.write("    match event {\n".as_bytes())?;
  fd.write("      Event::Module_driver(_) => continue,\n".as_bytes())?;
  fd.write("      _ => return false,\n".as_bytes())?;
  fd.write("    }\n".as_bytes())?;
  fd.write("  }\n".as_bytes())?;
  fd.write("  q.is_empty()\n".as_bytes())?;
  fd.write("}\n\n".as_bytes())?;

  fd.write("fn main() {\n".as_bytes())?;
  for array in sys.array_iter() {
    fd.write(format!(
      "  let mut {} = vec![0 as {}; {}];\n",
      array.get_name(),
      array.dtype().to_string(),
      array.get_size()
    ).as_bytes())?;
  }
  fd.write("  let mut q: VecDeque<Event> = VecDeque::new();\n".as_bytes())?;
  fd.write("  q.push_back(Event::Module_driver(vec![]));\n".as_bytes())?;
  fd.write("  loop {\n".as_bytes())?;
  fd.write("    let event = q.pop_front();\n".as_bytes())?;
  fd.write("    match event {\n".as_bytes())?;
  for module in sys.module_iter() {
    fd.write(format!(
      "      Some(Event::Module_{}(args)) => {}(&mut q, args",
      module.get_name(),
      module.get_name()
    ).as_bytes())?;
    for (array, ops) in module.array_iter(sys) {
      fd.write(format!(
        ", &{}{}",
        if ops.contains(&Opcode::Store) {
          "mut "
        } else {
          ""
        },
        array.get_name(),
      ).as_bytes())?;
    }
    fd.write("),\n".as_bytes())?;
  }
  fd.write("      _ => {\n".as_bytes())?;
  fd.write("        println!(\"Exit @{}:{}, b/c no event to simulate!\", file!(), line!());\n".as_bytes())?;
  fd.write("        break;\n".as_bytes())?;
  fd.write("      }\n".as_bytes())?;
  fd.write("    }\n".as_bytes())?;
  fd.write("    if driver_only(&q) {\n".as_bytes())?;
  fd.write("      println!(\"Exit @{}:{}, b/c all driver's to simulate!\", file!(), line!());\n".as_bytes())?;
  fd.write("      break;\n".as_bytes())?;
  fd.write("    }\n".as_bytes())?;
  fd.write("  }\n".as_bytes())?;
  fd.write("}\n\n".as_bytes())?;
  Ok(())
}

fn dump_module(sys: &SysBuilder, fd: &mut File) -> Result<(), std::io::Error> {
  let mut em = ElaborateModule::new(sys);
  for module in em.sys.module_iter() {
    fd.write(em.visit_module(module).as_bytes())?;
  }
  Ok(())
}

pub fn elaborate(sys: &SysBuilder, fname: String) -> Result<(), std::io::Error> {
  let mut fd = fs::File::create(fname)?;
  fd.write("use std::collections::VecDeque;\n".as_bytes())?;
  dump_module(sys, &mut fd)?;
  dump_runtime(sys, &mut fd)
}
