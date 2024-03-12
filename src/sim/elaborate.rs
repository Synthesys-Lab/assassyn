use std::{
  collections::HashSet,
  fs::{self, File},
  io::Write,
};

use crate::{
  builder::system::SysBuilder,
  data::{Handle, Typed},
  expr::{Expr, Opcode},
  ir::{block::Block, port::FIFO, visitor::Visitor},
  node::{
    ArrayRef, BlockRef, ExprRef, FIFORef, IntImmRef, IsElement, ModuleRef, NodeKind, Parented,
  },
  Module,
};

use super::Config;

struct ElaborateModule<'a> {
  sys: &'a SysBuilder,
  indent: usize,
}

impl<'a> ElaborateModule<'a> {
  fn new(sys: &'a SysBuilder) -> Self {
    Self { sys, indent: 0 }
  }
}

struct InterfDecl<'a>(&'a HashSet<Opcode>);

impl<'a> Visitor<String> for InterfDecl<'a> {
  fn visit_array(&mut self, array: &ArrayRef<'_>) -> Option<String> {
    format!(
      "  {}: &{}Vec<{}>,",
      array.get_name(),
      if self.0.contains(&Opcode::Store) {
        "mut "
      } else {
        ""
      },
      array.scalar_ty().to_string()
    )
    .into()
  }

  fn visit_input(&mut self, fifo: &FIFORef<'_>) -> Option<String> {
    let module = fifo.get_parent().as_ref::<Module>(fifo.sys).unwrap();
    format!(
      "  {}_{}: &mut VecDeque<{}>,",
      module.get_name(),
      fifo.get_name(),
      fifo.scalar_ty().to_string()
    )
    .into()
  }
}

struct InterfArgFeeder<'ops>(&'ops HashSet<Opcode>);

impl<'ops> Visitor<String> for InterfArgFeeder<'ops> {
  fn visit_array(&mut self, array: &ArrayRef<'_>) -> Option<String> {
    format!(
      ", /*Ext.Intef.Array*/&{}{}",
      if self.0.contains(&Opcode::Store) {
        "mut "
      } else {
        ""
      },
      array.get_name(),
    )
    .into()
  }
  fn visit_input(&mut self, fifo: &FIFORef<'_>) -> Option<String> {
    let module = fifo.get_parent().as_ref::<Module>(fifo.sys).unwrap();
    format!(
      ", /*Ext.Interf.FIFO*/&mut {}_{}",
      module.get_name(),
      fifo.get_name(),
    )
    .into()
  }
}

impl Visitor<String> for ElaborateModule<'_> {
  fn visit_module(&mut self, module: &ModuleRef<'_>) -> Option<String> {
    let mut res = String::new();
    res.push_str(format!("// Elaborating module {}\n", module.get_name()).as_str());
    res.push_str(format!("fn {}(\n", module.get_name()).as_str());
    res.push_str("  stamp: usize,\n");
    res.push_str("  q: &mut BinaryHeap<Reverse<Event>>,\n");
    for port in module.port_iter() {
      res.push_str(
        format!(
          "  {}_{}: &mut VecDeque<{}>, // input\n",
          module.get_name(),
          port.get_name(),
          port.scalar_ty().to_string()
        )
        .as_str(),
      );
    }
    for (array, ops) in module.ext_interf_iter() {
      let mut ie = InterfDecl(ops);
      let array_str = ie.dispatch(module.sys, array, vec![]).unwrap();
      res.push_str(format!("{} // external interface\n", array_str).as_str());
    }
    res.push_str(") {\n");
    res.push_str(
      format!(
        "  println!(\"@line:{{:<6}} {{}}: Simulating module {}\", line!(), cyclize(stamp));\n",
        module.get_name()
      )
      .as_str(),
    );
    self.indent += 2;
    for elem in module.get_body().iter() {
      match elem.get_kind() {
        NodeKind::Expr => {
          let expr = elem.as_ref::<Expr>(self.sys).unwrap();
          res.push_str(self.visit_expr(&expr).unwrap().as_str());
        }
        NodeKind::Block => {
          let block = elem.as_ref::<Block>(self.sys).unwrap();
          res.push_str(self.visit_block(&block).unwrap().as_str());
        }
        _ => {
          panic!("Unexpected reference type: {:?}", elem);
        }
      }
    }
    self.indent -= 2;
    res.push_str("}\n");
    res.into()
  }

  fn visit_expr(&mut self, expr: &ExprRef<'_>) -> Option<String> {
    let res = if expr.get_opcode().is_binary() {
      format!(
        "{} {} {}",
        expr.get_operand(0).unwrap().to_string(self.sys),
        expr.get_opcode().to_string(),
        expr.get_operand(1).unwrap().to_string(self.sys)
      )
    } else if expr.get_opcode().is_unary() {
      format!(
        "{}{}",
        expr.get_opcode().to_string(),
        expr.get_operand(0).unwrap().to_string(self.sys)
      )
    } else {
      match expr.get_opcode() {
        Opcode::Load => {
          let handle = expr
            .get_operand(0)
            .unwrap()
            .as_ref::<Handle>(expr.sys)
            .unwrap();
          format!(
            "{}[{} as usize]",
            handle.get_array().to_string(expr.sys),
            handle.get_idx().to_string(expr.sys)
          )
        }
        Opcode::Store => {
          let handle = expr
            .get_operand(0)
            .unwrap()
            .as_ref::<Handle>(expr.sys)
            .unwrap();
          format!(
            "q.push(Reverse(Event{{ stamp: stamp + 50, kind: EventKind::Array_commit_{}({} as usize, {}) }}))",
            handle.get_array().to_string(expr.sys),
            handle.get_idx().to_string(expr.sys),
            expr.get_operand(1).unwrap().to_string(self.sys),
          )
        }
        Opcode::Trigger => {
          let module_ref = expr
            .get_operand(0)
            .unwrap()
            .as_ref::<Module>(self.sys)
            .unwrap();
          let module_name = module_ref.get_name();
          format!(
            "q.push(Reverse(Event{{ stamp: stamp + 100, kind: EventKind::Module_{} }}))",
            module_name
          )
        }
        Opcode::FIFOPop => {
          // TODO(@were): Support multiple pop.
          let fifo = expr
            .get_operand(0)
            .unwrap()
            .as_ref::<FIFO>(self.sys)
            .unwrap();
          let module_name = fifo
            .get_parent()
            .as_ref::<Module>(self.sys)
            .unwrap()
            .get_name()
            .to_string();
          let fifo_name = fifo.get_name();
          format!("{}_{}.pop_front().unwrap()", module_name, fifo_name)
        }
        Opcode::FIFOPush => {
          let fifo = expr
            .get_operand(0)
            .unwrap()
            .as_ref::<FIFO>(self.sys)
            .unwrap();
          let value = expr.get_operand(1).unwrap().to_string(self.sys);
          let module_name = fifo
            .get_parent()
            .as_ref::<Module>(self.sys)
            .unwrap()
            .get_name()
            .to_string();
          let fifo_name = fifo.get_name();
          format!(
            "q.push(Reverse(Event{{ stamp: stamp + 50, kind: EventKind::FIFO_push_{}_{}({}) }}))",
            module_name, fifo_name, value
          )
        }
        _ => {
          assert!(expr.get_opcode().is_unary() || expr.get_opcode().is_binary());
          format!("// TODO: opcode: {}\n", expr.get_opcode().to_string())
        }
      }
    };
    if expr.dtype().is_void() {
      format!("{}{};\n", " ".repeat(self.indent), res)
    } else {
      format!(
        "{}let _{} = {};\n",
        " ".repeat(self.indent),
        expr.get_key(),
        res
      )
    }
    .into()
  }

  fn visit_int_imm(&mut self, int_imm: &IntImmRef<'_>) -> Option<String> {
    format!(
      "({} as {})",
      int_imm.get_value(),
      int_imm.dtype().to_string()
    )
    .into()
  }

  fn visit_block(&mut self, block: &BlockRef<'_>) -> Option<String> {
    let mut res = String::new();
    if let Some(cond) = block.get_pred() {
      res.push_str(format!("  if {} {{\n", cond.to_string(self.sys)).as_str());
    }
    self.indent += 2;
    for elem in block.iter() {
      match elem.get_kind() {
        NodeKind::Expr => {
          let expr = elem.as_ref::<Expr>(self.sys).unwrap();
          res.push_str(self.visit_expr(&expr).unwrap().as_str());
        }
        NodeKind::Block => {
          let block = elem.as_ref::<Block>(self.sys).unwrap();
          res.push_str(self.visit_block(&block).unwrap().as_str());
        }
        _ => {
          panic!("Unexpected reference type: {:?}", elem);
        }
      }
    }
    self.indent -= 2;
    if block.get_pred().is_some() {
      res.push_str(format!("{}}}\n", " ".repeat(self.indent)).as_str());
    }
    res.into()
  }
}

fn dump_runtime(sys: &SysBuilder, fd: &mut File, config: &Config) -> Result<(), std::io::Error> {
  fd.write("// Simulation runtime.\n".as_bytes())?;
  // Dump the helper function of cycles.
  // fn cyclize(stamp: usize) -> String {
  //   format!("{}.{:02}", stamp / 100, stamp % 100")
  // }
  fd.write("fn cyclize(stamp: usize) -> String {\n".as_bytes())?;
  fd.write("  format!(\"Cycle @{}.{:02}\", stamp / 100, stamp % 100)\n".as_bytes())?;
  fd.write("}\n\n".as_bytes())?;

  // Dump the event enum. Each event corresponds to a module.
  // Each event instance looks like this:
  //
  // enum EventKind {
  //   Module_{module.get_name()},
  //   ...
  //   FIFO_push_{module.get_name()}_{port.get_name()}(value),
  //   ...
  //   Array_commit_{array.get_name()}(idx, value),
  // }
  fd.write("#[derive(Debug, Eq, PartialEq)]\n".as_bytes())?;
  fd.write("enum EventKind {\n".as_bytes())?;
  for module in sys.module_iter() {
    fd.write(format!("  Module_{},\n", module.get_name()).as_bytes())?;
    for port in module.port_iter() {
      fd.write(
        format!(
          "  FIFO_push_{}_{}({}),\n",
          module.get_name(),
          port.get_name(),
          port.scalar_ty().to_string(),
        )
        .as_bytes(),
      )?;
    }
  }
  for array in sys.array_iter() {
    fd.write(
      format!(
        "  Array_commit_{}(usize, {}),\n",
        array.get_name(),
        array.scalar_ty().to_string()
      )
      .as_bytes(),
    )?;
  }
  fd.write("}\n\n".as_bytes())?;

  // Dump the event runtime.
  // #[derive(Debug, Eq, PartialEq)]
  // struct Event {
  //   stamp: usize,
  //   kind: EventKind,
  // }
  fd.write("#[derive(Debug, Eq, PartialEq)]\n".as_bytes())?;
  fd.write("struct Event {\n".as_bytes())?;
  fd.write("  stamp: usize,\n".as_bytes())?;
  fd.write("  kind: EventKind,\n".as_bytes())?;
  fd.write("}\n\n".as_bytes())?;

  // Dump the event order comparison.
  // impl std::cmp::Ord for Event {
  //   fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
  //     self.partial_cmp(other).unwrap()
  //   }
  // }
  // impl std::cmp::Eq for Event {
  //   fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
  //     self.partial_cmp(other).unwrap()
  //   }
  // }
  fd.write("impl Ord for Event {\n".as_bytes())?;
  fd.write("  fn cmp(&self, other: &Self) -> Ordering {\n".as_bytes())?;
  fd.write("    self.partial_cmp(other).unwrap()\n".as_bytes())?;
  fd.write("  }\n".as_bytes())?;
  fd.write("}\n\n".as_bytes())?;
  fd.write("impl PartialOrd for Event {\n".as_bytes())?;
  fd.write("  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {\n".as_bytes())?;
  fd.write("    Some(self.stamp.cmp(&other.stamp))\n".as_bytes())?;
  fd.write("  }\n".as_bytes())?;
  fd.write("}\n\n".as_bytes())?;

  // TODO(@were): Make all arguments of the modules FIFO channels.
  // TODO(@were): Profile the maxium size of all the FIFO channels.
  fd.write("fn main() {\n".as_bytes())?;
  fd.write("  // The global time stamp\n".as_bytes())?;
  fd.write("  let mut stamp: usize = 0;\n".as_bytes())?;
  fd.write("  // Count the consecutive cycles idled\n".as_bytes())?;
  fd.write("  let mut idled: usize = 0;\n".as_bytes())?;
  fd.write("  // Define global arrays\n".as_bytes())?;
  for array in sys.array_iter() {
    fd.write(
      format!(
        "  let mut {} = vec![0 as {}; {}];\n",
        array.get_name(),
        array.scalar_ty().to_string(),
        array.get_size()
      )
      .as_bytes(),
    )?;
  }
  fd.write("  // Define the module FIFOs\n".as_bytes())?;
  for module in sys.module_iter() {
    for port in module.port_iter() {
      fd.write(
        format!(
          "  let mut {}_{} : VecDeque<{}> = VecDeque::new();\n",
          module.get_name(),
          port.get_name(),
          port.scalar_ty().to_string()
        )
        .as_bytes(),
      )?;
    }
  }
  fd.write("  // Define the event queue\n".as_bytes())?;
  fd.write("  let mut q = BinaryHeap::new();\n".as_bytes())?;
  // Push the initial events.
  fd.write(format!("  for i in 0..{} {{\n", config.sim_threshold).as_bytes())?;
  fd.write(
    "    q.push(Reverse(Event{stamp: i * 100, kind: EventKind::Module_driver}));\n".as_bytes(),
  )?;
  fd.write("  }\n".as_bytes())?;
  // TODO(@were): Dump the time stamp of the simulation.
  fd.write("  while let Some(event) = q.pop() {\n".as_bytes())?;
  fd.write("    match event.0.kind {\n".as_bytes())?;
  for module in sys.module_iter() {
    fd.write(format!("      EventKind::Module_{} => {{\n", module.get_name(),).as_bytes())?;
    fd.write(format!("        {}(event.0.stamp, &mut q", module.get_name()).as_bytes())?;
    for (i, port) in module.port_iter().enumerate() {
      fd.write(
        format!(
          ", /*FIFO.{}*/ &mut {}_{}",
          i,
          module.get_name(),
          port.get_name()
        )
        .as_bytes(),
      )?;
    }
    for (array, ops) in module.ext_interf_iter() {
      fd.write(
        InterfArgFeeder(ops)
          .dispatch(sys, array, vec![])
          .unwrap()
          .as_bytes(),
      )?;
    }
    fd.write(");\n".as_bytes())?;
    if !module.get_name().eq("driver") {
      fd.write("        idled = 0;\n".as_bytes())?;
      fd.write("        continue;\n".as_bytes())?;
      fd.write("      }\n".as_bytes())?;
    } else {
      fd.write("        idled += 1;\n".as_bytes())?;
      fd.write("        stamp = event.0.stamp;\n".as_bytes())?;
      fd.write("      }\n".as_bytes())?;
    }
  }
  for module in sys.module_iter() {
    for port in module.port_iter() {
      fd.write(
        format!(
          "      EventKind::FIFO_push_{}_{}(value) => {{\n",
          module.get_name(),
          port.get_name()
        )
        .as_bytes(),
      )?;
      fd.write(
        format!(
          "        println!(\"@line:{{:<6}} {{}}: Commit FIFO {}.{} push {{}}\", line!(), cyclize(event.0.stamp), value);\n",
          module.get_name(),
          port.get_name()
        )
        .as_bytes(),
      )?;
      fd.write(
        format!(
          "        {}_{}.push_back(value);\n",
          module.get_name(),
          port.get_name()
        )
        .as_bytes(),
      )?;
      fd.write("      }\n".as_bytes())?;
    }
  }
  for array in sys.array_iter() {
    fd.write(
      format!(
        "      EventKind::Array_commit_{}(idx, value) => {{\n",
        array.get_name()
      )
      .as_bytes(),
    )?;
    fd.write(
      format!(
        "        println!(\"@line:{{:<6}} {{}}: Commit array {} write\", line!(), cyclize(event.0.stamp));\n",
        array.get_name()
      )
      .as_bytes(),
    )?;
    fd.write(format!("        {}[idx] = value;\n", array.get_name()).as_bytes())?;
    fd.write("      }\n".as_bytes())?;
  }
  fd.write("    }\n".as_bytes())?;
  fd.write(format!("    if idled > {} {{\n", config.idle_threshold).as_bytes())?;
  fd.write(
    format!(
      "      println!(\"Idled more than {} cycles, exit @{{}}!\", cyclize(stamp));\n",
      config.idle_threshold
    )
    .as_bytes(),
  )?;
  fd.write("      break;\n".as_bytes())?;
  fd.write("    }\n".as_bytes())?;
  fd.write("  }\n".as_bytes())?;
  fd.write("  println!(\"No event to simulate {}!\", cyclize(stamp));\n".as_bytes())?;
  fd.write("}\n\n".as_bytes())?;
  Ok(())
}

fn dump_module(sys: &SysBuilder, fd: &mut File) -> Result<(), std::io::Error> {
  let mut em = ElaborateModule::new(sys);
  for module in em.sys.module_iter() {
    fd.write(em.visit_module(&module).unwrap().as_bytes())?;
  }
  Ok(())
}

fn dump_header(fd: &mut File) -> Result<(), std::io::Error> {
  fd.write("use std::collections::VecDeque;\n".as_bytes())?;
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
