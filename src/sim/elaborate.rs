use std::collections::HashSet;

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

fn dump_runtime(sys: &SysBuilder) {
  println!("// Simulation runtime.");
  println!("enum Event {{");
  for module in sys.module_iter() {
    println!("  Module_{}(Vec<u64>),", module.get_name());
  }
  println!("}}\n");

  println!("fn driver_only(q: &VecDeque<Event>) -> bool {{");
  println!("  for event in q.iter() {{");
  println!("    match event {{");
  println!("      Event::Module_driver(_) => continue,");
  println!("      _ => return false,");
  println!("    }}");
  println!("  }}");
  println!("  q.is_empty()");
  println!("}}\n");

  println!("fn main() {{");
  for array in sys.array_iter() {
    println!(
      "  let mut {} = vec![0 as {}; {}];",
      array.get_name(),
      array.dtype().to_string(),
      array.get_size()
    );
  }
  println!("  let mut q: VecDeque<Event> = VecDeque::new();");
  println!("  q.push_back(Event::Module_driver(vec![]));");
  println!("  loop {{");
  println!("    let event = q.pop_front();");
  println!("    match event {{");
  for module in sys.module_iter() {
    print!(
      "      Some(Event::Module_{}(args)) => {}(&mut q, args",
      module.get_name(),
      module.get_name()
    );
    for (array, ops) in module.array_iter(sys) {
      print!(
        ", &{}{}",
        if ops.contains(&Opcode::Store) {
          "mut "
        } else {
          ""
        },
        array.get_name(),
      );
    }
    println!("),");
  }
  println!("      _ => {{");
  println!("        println!(\"Exit @{{}}:{{}}, b/c no event to simulate!\", file!(), line!());");
  println!("        break;");
  println!("      }}");
  println!("    }}");
  println!("    if driver_only(&q) {{");
  println!("      println!(\"Exit @{{}}:{{}}, b/c all driver's to simulate!\", file!(), line!());");
  println!("      break;");
  println!("    }}");
  println!("  }}");
  println!("}}\n");
}

fn dump_module(sys: &SysBuilder) {
  let mut em = ElaborateModule::new(sys);
  let mut res = String::new();
  for module in em.sys.module_iter() {
    res.push_str(em.visit_module(module).as_str());
  }
  println!("{}", res);
}

pub fn elaborate(sys: &SysBuilder) {
  println!("use std::collections::VecDeque;\n");
  dump_module(sys);
  dump_runtime(sys);
}
