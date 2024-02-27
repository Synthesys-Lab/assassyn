use std::{
  collections::HashSet,
  fs::{self, File},
  io::Write,
};

use crate::{
  builder::system::SysBuilder,
  data::{Array, Typed},
  expr::{Expr, Opcode},
  port::Input,
  reference::{IsElement, Visitor},
  IntImm, Module,
};

use super::Config;

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
        "fn {}(stamp: usize, q: &mut VecDeque<Event>, args: Vec<u64>",
        module.get_name()
      )
      .as_str(),
    );
    for (array, ops) in module.array_iter(self.sys) {
      self.ops = Some(ops);
      res.push_str(self.visit_array(array).as_str());
    }
    res.push_str(") {\n");
    res.push_str(
      format!(
        "  println!(\"{{}}:{{:04}} @Cycle {{}}: Invoking module {}\", file!(), line!(), stamp);\n",
        module.get_name()
      )
      .as_str(),
    );
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
    let res = if expr.get_opcode().is_binary() {
      format!(
        "{} {} {}",
        expr.get_operand(0).unwrap().to_string(self.sys),
        expr.get_opcode().to_string(),
        expr.get_operand(1).unwrap().to_string(self.sys)
      )
    } else {
      match expr.get_opcode() {
        Opcode::Load => {
          format!(
            "{}[{} as usize]",
            expr.get_operand(0).unwrap().to_string(self.sys),
            expr.get_operand(1).unwrap().to_string(self.sys)
          )
        }
        Opcode::Store => {
          format!(
            "{}[{} as usize] = {}",
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
          let mut res = format!("q.push_back(Event::Module_{}(stamp + 1, vec![", module_name);
          for args in expr.operand_iter().skip(1) {
            res.push_str(format!("{} as u64, ", args.to_string(self.sys)).as_str());
          }
          res.push_str("]))");
          res
        }
        _ => {
          format!("  // TODO: Other opcode;\n")
        }
      }
    };
    // TODO(@were): Propagate the predications of the expressions.
    let pred = if let Some(pred) = expr.get_pred() {
      Some(pred.to_string(self.sys))
    } else {
      None
    };
    if expr.dtype().is_void() {
      if let Some(pred) = pred {
        format!("  if {} {{\n    {};\n  }}\n", pred, res)
      } else {
        format!("  {};\n", res)
      }
    } else {
      if let Some(pred) = pred {
        format!(
          "  let _{} = if {} {{ Some({}) }} else {{ None }};\n",
          expr.get_key(),
          pred,
          res
        )
      } else {
        format!("  let _{} = {};\n", expr.get_key(), res)
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

fn dump_runtime(sys: &SysBuilder, fd: &mut File, config: &Config) -> Result<(), std::io::Error> {
  fd.write("// Simulation runtime.\n".as_bytes())?;
  fd.write("enum Event {\n".as_bytes())?;
  for module in sys.module_iter() {
    fd.write(format!("  Module_{}(usize, Vec<u64>),\n", module.get_name()).as_bytes())?;
  }
  fd.write("}\n\n".as_bytes())?;

  fd.write("fn driver_only(q: &VecDeque<Event>) -> bool {\n".as_bytes())?;
  fd.write("  for event in q.iter() {\n".as_bytes())?;
  fd.write("    match event {\n".as_bytes())?;
  fd.write("      Event::Module_driver(_, _) => continue,\n".as_bytes())?;
  fd.write("      _ => return false,\n".as_bytes())?;
  fd.write("    }\n".as_bytes())?;
  fd.write("  }\n".as_bytes())?;
  fd.write("  q.is_empty()\n".as_bytes())?;
  fd.write("}\n\n".as_bytes())?;

  // TODO(@were): Make all arguments of the modules FIFO channels.
  // TODO(@were): Profile the maxium size of all the FIFO channels.
  fd.write("fn main() {\n".as_bytes())?;
  fd.write("  let mut stamp: usize = 0;\n".as_bytes())?;
  fd.write("  let mut idled: usize = 0;\n".as_bytes())?;
  for array in sys.array_iter() {
    fd.write(
      format!(
        "  let mut {} = vec![0 as {}; {}];\n",
        array.get_name(),
        array.dtype().to_string(),
        array.get_size()
      )
      .as_bytes(),
    )?;
  }
  fd.write("  let mut q: VecDeque<Event> = VecDeque::new();\n".as_bytes())?;
  fd.write("  q.push_back(Event::Module_driver(stamp, vec![]));\n".as_bytes())?;
  // TODO(@were): Dump the time stamp of the simulation.
  fd.write("  loop {\n".as_bytes())?;
  fd.write("    let event = q.pop_front();\n".as_bytes())?;
  fd.write("    match event {\n".as_bytes())?;
  for module in sys.module_iter() {
    fd.write(
      format!(
        "      Some(Event::Module_{}(src_stamp, args)) => {{ {}(src_stamp, &mut q, args",
        module.get_name(),
        module.get_name()
      )
      .as_bytes(),
    )?;
    for (array, ops) in module.array_iter(sys) {
      fd.write(
        format!(
          ", &{}{}",
          if ops.contains(&Opcode::Store) {
            "mut "
          } else {
            ""
          },
          array.get_name(),
        )
        .as_bytes(),
      )?;
    }
    fd.write(");".as_bytes())?;
    if !module.get_name().eq("driver") {
      fd.write(" idled = 0; continue; }\n".as_bytes())?;
    } else {
      fd.write(" }\n".as_bytes())?;
    }
  }
  fd.write("      _ => {\n".as_bytes())?;
  fd.write("        println!(\"No event to simulate @{}!\", stamp);\n".as_bytes())?;
  fd.write("        break;\n".as_bytes())?;
  fd.write("      }\n".as_bytes())?;
  fd.write("    }\n".as_bytes())?;
  fd.write("    if driver_only(&q) {\n".as_bytes())?;
  fd.write("      idled = idled + 1;\n".as_bytes())?;
  fd.write(format!("      if idled > {} {{\n", config.idle_threshold).as_bytes())?;
  fd.write("        println!(\"No event other than drivers, exit @{}!\", stamp);\n".as_bytes())?;
  fd.write("        break;\n".as_bytes())?;
  fd.write("      }\n".as_bytes())?;
  fd.write("    }\n".as_bytes())?;
  fd.write("    stamp = stamp + 1; // tick the time stamp...\n".as_bytes())?;
  fd.write(format!("    if stamp <= {} {{\n", config.sim_threshold).as_bytes())?;
  fd.write("      q.push_back(Event::Module_driver(stamp, vec![]));\n".as_bytes())?;
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

pub fn elaborate(sys: &SysBuilder, config: &Config) -> Result<(), std::io::Error> {
  let mut fd = fs::File::create(config.fname.clone())?;
  fd.write("use std::collections::VecDeque;\n".as_bytes())?;
  dump_module(sys, &mut fd)?;
  dump_runtime(sys, &mut fd, config)
}
