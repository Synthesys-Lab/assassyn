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
        "fn {}(stamp: usize, q: &mut BinaryHeap<Reverse<Event>>, args: Vec<u64>",
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
          let mut res = format!(
            "q.push(Reverse(Event::Module_{}(stamp + 1, vec![",
            module_name
          );
          for args in expr.operand_iter().skip(1) {
            res.push_str(format!("{} as u64, ", args.to_string(self.sys)).as_str());
          }
          res.push_str("])))");
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
  // Dump the event enum. Each event corresponds to a module.
  // Each event instance looks like this:
  //
  // enum Event {
  //   Module_{module.get_name()}(time_stamp, args),
  //   ...
  // }
  fd.write("#[derive(Debug, Eq, PartialEq)]\n".as_bytes())?;
  fd.write("enum Event {\n".as_bytes())?;
  for module in sys.module_iter() {
    fd.write(format!("  Module_{}(usize, Vec<u64>),\n", module.get_name()).as_bytes())?;
  }
  fd.write("}\n".as_bytes())?;

  // Dump the event order functions.
  // impl Event {
  //   fn get_stamp(&self) -> usize {
  //      match self {
  //        Event::Module_{module.get_name()}(stamp, _) => *stamp,
  //        ...
  //      }
  //   }
  // }
  fd.write("impl Event {\n".as_bytes())?;
  fd.write("  fn get_stamp(&self) -> usize {\n".as_bytes())?;
  fd.write("    match self {\n".as_bytes())?;
  for module in sys.module_iter() {
    fd.write(
      format!(
        "      Event::Module_{}(stamp, _) => *stamp,\n",
        module.get_name()
      )
      .as_bytes(),
    )?;
  }
  fd.write("    }\n".as_bytes())?;
  fd.write("  }\n".as_bytes())?;
  fd.write("}\n".as_bytes())?;
  // Dump the event order comparison.
  // impl std::cmp::PartialOrd for Event {
  //   fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
  //     if self.get_stamp() == other.get_stamp() {
  //       match (self, other) {
  //         (Event::Module_driver(_, _), _) => Some(Ordering::Less),
  //         _ => Some(Ordering::Equal),
  //       }
  //     }
  //     Some(self.get_stamp().cmp(&other.get_stamp()))
  //   }
  // }
  // impl std::cmp::Ord for Event {
  //   fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
  //     self.partial_cmp(other).unwrap()
  //   }
  // }
  fd.write("impl PartialOrd for Event {\n".as_bytes())?;
  fd.write("  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {\n".as_bytes())?;
  fd.write("    if self.get_stamp() == other.get_stamp() {\n".as_bytes())?;
  fd.write("      match (self, other) {\n".as_bytes())?;
  fd.write(
    "        (Event::Module_driver(_, _), _) => Some(std::cmp::Ordering::Less),\n".as_bytes(),
  )?;
  fd.write("        _ => Some(std::cmp::Ordering::Equal),\n".as_bytes())?;
  fd.write("      }\n".as_bytes())?;
  fd.write("    } else {\n".as_bytes())?;
  fd.write("      Some(self.get_stamp().cmp(&other.get_stamp()))\n".as_bytes())?;
  fd.write("    }\n".as_bytes())?;
  fd.write("  }\n".as_bytes())?;
  fd.write("}\n".as_bytes())?;
  fd.write("impl Ord for Event {\n".as_bytes())?;
  fd.write("  fn cmp(&self, other: &Self) -> Ordering {\n".as_bytes())?;
  fd.write("    self.partial_cmp(other).unwrap()\n".as_bytes())?;
  fd.write("  }\n".as_bytes())?;
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
  fd.write("  let mut q = BinaryHeap::new();\n".as_bytes())?;
  // Push the initial events.
  fd.write(format!("  for i in 0..{} {{\n", config.sim_threshold).as_bytes())?;
  fd.write("    q.push(Reverse(Event::Module_driver(i, vec![])));\n".as_bytes())?;
  fd.write("  }\n".as_bytes())?;
  // TODO(@were): Dump the time stamp of the simulation.
  fd.write("  while let Some(event) = q.pop() {\n".as_bytes())?;
  fd.write("    match event.0 {\n".as_bytes())?;
  for module in sys.module_iter() {
    fd.write(
      format!(
        "      Event::Module_{}(src_stamp, args) => {{ {}(src_stamp, &mut q, args",
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
      fd.write(" idled += 1; stamp = src_stamp; }\n".as_bytes())?;
    }
  }
  fd.write("    }\n".as_bytes())?;
  fd.write(format!("    if idled > {} {{\n", config.idle_threshold).as_bytes())?;
  fd.write(
    format!(
      "      println!(\"Idled more than {} cycles, exit @{{}}!\", stamp);\n",
      config.idle_threshold
    )
    .as_bytes(),
  )?;
  fd.write("      break;\n".as_bytes())?;
  fd.write("    }\n".as_bytes())?;
  fd.write("  }\n".as_bytes())?;
  fd.write("  println!(\"No event to simulate @{}!\", stamp);\n".as_bytes())?;
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

fn dump_header(fd: &mut File) -> Result<(), std::io::Error> {
  fd.write("use std::collections::BinaryHeap;\n".as_bytes())?;
  fd.write("use std::cmp::{PartialOrd, Ord, Ordering, Reverse};\n".as_bytes())?;
  Ok(())
}

pub fn elaborate(sys: &SysBuilder, config: &Config) -> Result<(), std::io::Error> {
  let mut fd = fs::File::create(config.fname.clone())?;
  dump_header(&mut fd)?;
  dump_module(sys, &mut fd)?;
  dump_runtime(sys, &mut fd, config)
}
