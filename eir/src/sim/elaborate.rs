use std::{
  collections::HashSet,
  fs::{self, File},
  io::Write,
  process::Command,
};

use crate::{
  builder::system::SysBuilder,
  ir::{node::*, visitor::Visitor, *},
};

use super::{analysis::common_module::CommonModuleCache, Config};

struct ElaborateModule<'a> {
  sys: &'a SysBuilder,
  indent: usize,
  module_name: String,
  fpc: CommonModuleCache,
}

impl<'a> ElaborateModule<'a> {
  fn new(sys: &'a SysBuilder) -> Self {
    Self {
      sys,
      indent: 0,
      module_name: String::new(),
      fpc: CommonModuleCache::new(sys),
    }
  }
}

struct InterfDecl<'a>(&'a HashSet<Opcode>);

macro_rules! fifo_name {
  ($fifo:expr) => {{
    let module = $fifo.get_parent().as_ref::<Module>($fifo.sys).unwrap();
    format!("{}.{}", namify(module.get_name()), $fifo.idx())
  }};
}

macro_rules! dump_ref {
  ($sys:expr, $value:expr) => {
    NodeRefDumper.dispatch($sys, $value, vec![]).unwrap()
  };
}

impl<'a> Visitor<String> for InterfDecl<'a> {
  fn visit_array(&mut self, array: &ArrayRef<'_>) -> Option<String> {
    format!(
      "  {}: &{}Vec<{}>,",
      namify(array.get_name()),
      if self.0.contains(&Opcode::Store) {
        "mut "
      } else {
        ""
      },
      dtype_to_rust_type(&array.scalar_ty())
    )
    .into()
  }

  fn visit_input(&mut self, _: &FIFORef<'_>) -> Option<String> {
    String::from("").into()
  }

  fn visit_module(&mut self, _: &ModuleRef<'_>) -> Option<String> {
    String::from("").into()
  }
}

struct InterfArgFeeder<'ops>(&'ops HashSet<Opcode>);

struct NodeRefDumper;

impl Visitor<String> for NodeRefDumper {
  fn dispatch(&mut self, sys: &SysBuilder, node: &BaseNode, _: Vec<NodeKind>) -> Option<String> {
    match node.get_kind() {
      NodeKind::Array => {
        let array = node.as_ref::<Array>(sys).unwrap();
        namify(array.get_name()).into()
      }
      NodeKind::FIFO => namify(node.as_ref::<FIFO>(sys).unwrap().get_name()).into(),
      NodeKind::IntImm => {
        let int_imm = node.as_ref::<IntImm>(sys).unwrap();
        Some(format!(
          "({} as {})",
          int_imm.get_value(),
          dtype_to_rust_type(&int_imm.dtype())
        ))
      }
      NodeKind::StrImm => {
        let str_imm = node.as_ref::<StrImm>(sys).unwrap();
        let value = str_imm.get_value();
        quote::quote!(#value).to_string().into()
      }
      NodeKind::Module => {
        let module_name = namify(node.as_ref::<Module>(sys).unwrap().get_name());
        format!("Box::new(EventKind::Module_{})", module_name).into()
      }
      _ => Some(format!("_{}", node.get_key())),
    }
  }
}

impl<'ops> Visitor<String> for InterfArgFeeder<'ops> {
  fn visit_array(&mut self, array: &ArrayRef<'_>) -> Option<String> {
    format!(
      ", /*Ext.Intef.Array*/&{}{}",
      if self.0.contains(&Opcode::Store) {
        "mut "
      } else {
        ""
      },
      namify(array.get_name()),
    )
    .into()
  }
  fn visit_input(&mut self, _: &FIFORef<'_>) -> Option<String> {
    format!("").into()
  }
  fn visit_module(&mut self, _: &ModuleRef<'_>) -> Option<String> {
    format!("").into()
  }
}

impl Visitor<String> for ElaborateModule<'_> {
  fn visit_module(&mut self, module: &ModuleRef<'_>) -> Option<String> {
    // let master = self.fpc.get_master(&module.upcast());
    // if master != module.upcast() {
    //   return format!(
    //     "// Module {} unified to its master {}\n",
    //     module.get_name(),
    //     master.as_ref::<Module>(module.sys).unwrap().get_name()
    //   )
    //   .into();
    // }

    self.module_name = module.get_name().to_string();
    let mut res = String::new();
    res.push_str(format!("// Elaborating module {}\n", namify(module.get_name())).as_str());
    res.push_str(format!("fn {}(\n", namify(module.get_name())).as_str());
    res.push_str("  stamp: usize,\n");
    res.push_str("  module_name: &str,\n");
    res.push_str("  q: &mut BinaryHeap<Reverse<Event>>,\n");
    res.push_str("  inputs: &mut (");
    for port in module.port_iter() {
      res.push_str("VecDeque<");
      res.push_str(dtype_to_rust_type(&port.scalar_ty()).as_str());
      res.push_str(">, ");
    }
    res.push_str("),\n");
    for (array, ops) in module.ext_interf_iter() {
      let mut ie = InterfDecl(ops);
      let array_str = ie.dispatch(module.sys, array, vec![]).unwrap();
      res.push_str(format!("{} // external interface\n", array_str).as_str());
    }
    res.push_str(") {\n");
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
        dump_ref!(self.sys, expr.get_operand(0).unwrap()),
        expr.get_opcode().to_string(),
        dump_ref!(self.sys, expr.get_operand(1).unwrap()),
      )
    } else if expr.get_opcode().is_unary() {
      format!(
        "{}{}",
        expr.get_opcode().to_string(),
        dump_ref!(self.sys, expr.get_operand(0).unwrap())
      )
    } else {
      match expr.get_opcode() {
        Opcode::Load => {
          let handle = expr
            .get_operand(0)
            .unwrap()
            .as_ref::<ArrayPtr>(expr.sys)
            .unwrap();
          format!(
            "{}[{} as usize]",
            NodeRefDumper
              .dispatch(expr.sys, &handle.get_array(), vec![])
              .unwrap(),
            NodeRefDumper
              .dispatch(expr.sys, &handle.get_idx(), vec![])
              .unwrap()
          )
        }
        Opcode::Store => {
          let handle = expr
            .get_operand(0)
            .unwrap()
            .as_ref::<ArrayPtr>(expr.sys)
            .unwrap();
          format!(
            "q.push(Reverse(Event{{ stamp: stamp + 50, kind: EventKind::Array_commit_{}({} as usize, {}) }}))",
            NodeRefDumper.dispatch(expr.sys, &handle.get_array(), vec![]).unwrap(),
            NodeRefDumper.dispatch(expr.sys, &handle.get_idx(), vec![]).unwrap(),
            dump_ref!(expr.sys, expr.get_operand(1).unwrap()),
          )
        }
        Opcode::Trigger => {
          let to_trigger =
            if let Ok(module) = expr.get_operand(0).unwrap().as_ref::<Module>(self.sys) {
              format!("EventKind::Module_{}", namify(module.get_name()))
            } else {
              format!(
                "{}.as_ref().clone()",
                dump_ref!(self.sys, expr.get_operand(0).unwrap())
              )
            };
          format!(
            "q.push(Reverse(Event{{ stamp: stamp + 100, kind: {} }}))",
            to_trigger
          )
        }
        Opcode::FIFOPop => {
          // TODO(@were): Support multiple pop.
          let fifo = expr
            .get_operand(0)
            .unwrap()
            .as_ref::<FIFO>(self.sys)
            .unwrap();
          format!("inputs.{}.pop_front().unwrap()", fifo.idx())
        }
        Opcode::FIFOPeek => {
          let fifo = expr
            .get_operand(0)
            .unwrap()
            .as_ref::<FIFO>(self.sys)
            .unwrap();
          format!("*inputs.{}.front().unwrap()", fifo.idx())
        }
        Opcode::FIFOPush => {
          let value = dump_ref!(self.sys, expr.get_operand(2).unwrap());
          let fifo_idx = expr
            .get_operand(1)
            .unwrap()
            .as_ref::<IntImm>(self.sys)
            .unwrap()
            .get_value();
          if let Ok(module) = expr.get_operand(0).unwrap().as_ref::<Module>(self.sys) {
            format!(
              "q.push(Reverse(Event{{ stamp: stamp + 50, kind: EventKind::FIFO_push_{}_{}({}) }}))",
              namify(&module.get_name()),
              fifo_idx,
              value
            )
          } else {
            let module = dump_ref!(self.sys, expr.get_operand(0).unwrap());
            format!(
              "q.push(Reverse(Event{{ stamp: stamp + 50, kind: to_push({}.as_ref().clone(), {}, {} as u64) }}))",
              module, fifo_idx, value
            )
          }
        }
        Opcode::Log => {
          let mut res = String::new();
          res
            .push_str("print!(\"@line:{:<5} [{}] {}:   \", line!(), module_name, cyclize(stamp));");
          res.push_str("println!(");
          for elem in expr.operand_iter() {
            res.push_str(format!("{}, ", dump_ref!(self.sys, elem)).as_str());
          }
          res.push(')');
          res
        }
        _ => {
          if !(expr.get_opcode().is_unary() || expr.get_opcode().is_binary()) {
            panic!("Unknown opcode: {:?}", expr.get_opcode());
          }
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
      dtype_to_rust_type(&int_imm.dtype())
    )
    .into()
  }

  fn visit_block(&mut self, block: &BlockRef<'_>) -> Option<String> {
    let mut res = String::new();
    if let Some(cond) = block.get_pred() {
      res.push_str(
        format!(
          "  if {}{} {{\n",
          dump_ref!(self.sys, &cond),
          if cond.get_dtype(block.sys).unwrap().bits() == 1 {
            "".into()
          } else {
            format!(" != 0")
          }
        )
        .as_str(),
      );
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

fn dtype_to_rust_type(dtype: &DataType) -> String {
  if dtype.is_int() {
    let bits = dtype.bits();
    return if bits.is_power_of_two() && bits >= 8 && bits <= 64 {
      format!("i{}", dtype.bits())
    } else if bits == 1 {
      "bool".to_string()
    } else if bits.is_power_of_two() && bits < 8 {
      "i8".to_string()
    } else {
      format!("i{}", dtype.bits().next_power_of_two())
    };
  }
  match dtype {
    DataType::Module(_) => {
      format!("Box<EventKind>",)
    }
    _ => panic!("Not implemented yet!"),
  }
}

fn namify(name: &str) -> String {
  name.replace(".", "_")
}

fn dump_runtime(
  sys: &SysBuilder,
  fd: &mut File,
  _: &mut CommonModuleCache,
  config: &Config,
) -> Result<(), std::io::Error> {
  // Dump the helper function of cycles.
  fd.write("// Simulation runtime.\n".as_bytes())?;
  {
    let cyclize = quote::quote! {
      fn cyclize(stamp: usize) -> String {
        format!("Cycle @{}.{:02}", stamp / 100, stamp % 100)
      }
    };
    fd.write(cyclize.to_string().as_bytes())?;
    fd.write("\n".as_bytes())?;
  }

  // Dump the event enum. Each event corresponds to a module.
  // Each event instance looks like this:
  //
  // enum EventKind {
  //   Module_{module.get_name()},
  //   ...
  //   FIFO_push_{module.get_name()}_{port.idx()}(value),
  //   ...
  //   Array_commit_{array.get_name()}(idx, value),
  // }
  fd.write("#[derive(Clone, Debug, Eq, PartialEq)]\n".as_bytes())?;
  fd.write("enum EventKind {\n".as_bytes())?;
  for module in sys.module_iter() {
    fd.write(format!("  Module_{},\n", namify(module.get_name())).as_bytes())?;
    for port in module.port_iter() {
      fd.write(
        format!(
          "  FIFO_push_{}({}),\n",
          namify(fifo_name!(port).as_str()),
          dtype_to_rust_type(&port.scalar_ty()),
        )
        .as_bytes(),
      )?;
    }
  }
  for array in sys.array_iter() {
    fd.write(
      format!(
        "  Array_commit_{}(usize, {}),\n",
        namify(array.get_name()),
        dtype_to_rust_type(&array.scalar_ty())
      )
      .as_bytes(),
    )?;
  }
  fd.write("}\n\n".as_bytes())?;
  fd.write("fn to_push(ek: EventKind, idx: usize, value: u64) -> EventKind {\n".as_bytes())?;
  fd.write("  match (ek.clone(), idx) {\n".as_bytes())?;
  for module in sys.module_iter() {
    for (i, port) in module.port_iter().enumerate() {
      if !port.scalar_ty().is_int() {
        continue;
      }
      fd.write(
        format!(
          "    (EventKind::Module_{}, {}) => EventKind::FIFO_push_{}(value as {}),\n",
          namify(module.get_name()),
          i,
          namify(fifo_name!(port).as_str()),
          dtype_to_rust_type(&port.scalar_ty())
        )
        .as_bytes(),
      )?;
    }
  }
  fd.write("    _ => panic!(\"Unknown event to push, {:?}, {}\", ek, idx),\n".as_bytes())?;
  fd.write("  }\n".as_bytes())?;
  fd.write("}\n\n".as_bytes())?;

  fd.write(
    quote::quote! {
      #[derive(Debug, Eq, PartialEq)]
      struct Event {
        stamp: usize,
        kind: EventKind,
      }
      impl std::cmp::Ord for Event {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
          self.stamp.cmp(&other.stamp)
        }
      }
      impl std::cmp::Eq for Event {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
          Some(self.stamp.cmp(&other.stamp))
        }
      }
    }
    .to_string()
    .as_bytes(),
  )?;

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
        "  let mut {} = vec![{}; {}];\n",
        namify(array.get_name()),
        if array.scalar_ty().bits() == 1 {
          "false".into()
        } else {
          format!("0 as {}", dtype_to_rust_type(&array.scalar_ty()))
        },
        array.get_size()
      )
      .as_bytes(),
    )?;
  }
  fd.write("  // Define the module FIFOs\n".as_bytes())?;
  for module in sys.module_iter() {
    fd.write(format!("  let mut {}_fifos : (", namify(module.get_name())).as_bytes())?;
    for port in module.port_iter() {
      fd.write(format!("VecDeque<{}>, ", dtype_to_rust_type(&port.scalar_ty())).as_bytes())?;
    }
    fd.write(") = (".as_bytes())?;
    for _ in module.port_iter() {
      fd.write(format!("VecDeque::new(), ").as_bytes())?;
    }
    fd.write(");\n".as_bytes())?;
  }
  fd.write("  // Define the event queue\n".as_bytes())?;
  fd.write("  let mut q = BinaryHeap::new();\n".as_bytes())?;
  let sim_threshold = config.sim_threshold;
  if sys.has_driver() {
    // Push the initial events.
    fd.write(
      quote::quote! {
        for i in 0..#sim_threshold {
          q.push(Reverse(Event{stamp: i * 100, kind: EventKind::Module_driver}));
        }
      }
      .to_string()
      .as_bytes(),
    )?;
  }
  // TODO(@were): Dump the time stamp of the simulation.
  fd.write("  while let Some(event) = q.pop() {\n".as_bytes())?;
  fd.write(
    quote::quote! {
      if event.0.stamp / 100 > #sim_threshold {
        print!("Exceed the simulation threshold ");
        print!("{}", #sim_threshold);
        println!(", exit!");
        break;
      }
    }
    .to_string()
    .as_bytes(),
  )?;
  fd.write("    match event.0.kind {\n".as_bytes())?;
  for module in sys.module_iter() {
    fd.write(
      format!(
        "      EventKind::Module_{} => {{\n",
        namify(module.get_name())
      )
      .as_bytes(),
    )?;
    let callee = namify(module.get_name());
    // if fpc.get_size(&module.upcast()) != 1{
    //   let master = fpc.get_master(&module.upcast());
    //   let master = master.as_ref::<Module>(sys).unwrap();
    //   fd.write(
    //     format!(
    //       "        // Calling {} merged to calling {}\n",
    //       module.get_name(),
    //       master.get_name()
    //     )
    //     .as_bytes(),
    //   )?;
    //   namify(master.get_name())
    // } else {
    //   namify(module.get_name())
    // };
    fd.write(
      format!(
        "        {}(event.0.stamp, \"{}\", &mut q, &mut {}_fifos",
        callee,
        namify(module.get_name()),
        namify(module.get_name()),
      )
      .as_bytes(),
    )?;
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
          "      EventKind::FIFO_push_{}(value) => {{\n",
          namify(fifo_name!(port).as_str())
        )
        .as_bytes(),
      )?;
      fd.write(
        format!(
          "        {}_fifos.{}.push_back(value);\n",
          namify(module.get_name()),
          port.idx()
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
        namify(array.get_name())
      )
      .as_bytes(),
    )?;
    fd.write(format!("        {}[idx] = value;\n", namify(array.get_name())).as_bytes())?;
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
  fd.write("  println!(\"Finish simulation: {}!\", cyclize(stamp));\n".as_bytes())?;
  fd.write("}\n\n".as_bytes())?;
  Ok(())
}

fn dump_module(sys: &SysBuilder, fd: &mut File) -> Result<CommonModuleCache, std::io::Error> {
  let mut em = ElaborateModule::new(sys);
  for module in em.sys.module_iter() {
    if let Some(buffer) = em.visit_module(&module) {
      fd.write(buffer.as_bytes())?;
    }
  }
  Ok(em.fpc)
}

fn dump_header(fd: &mut File) -> Result<usize, std::io::Error> {
  let src = quote::quote! {
    use std::collections::VecDeque;
    use std::collections::BinaryHeap;
    use std::cmp::{PartialOrd, Ord, Ordering, Reverse};
  };
  fd.write(src.to_string().as_bytes())
}

pub fn elaborate_impl(sys: &SysBuilder, config: &Config) -> Result<(), std::io::Error> {
  println!("Writing simulator code to {}", config.fname);
  let mut fd = fs::File::create(config.fname.clone())?;
  dump_header(&mut fd)?;
  let mut fpc = dump_module(sys, &mut fd)?;
  dump_runtime(sys, &mut fd, &mut fpc, config)?;
  fd.flush()
}

pub fn elaborate(sys: &SysBuilder, config: &Config) -> Result<(), std::io::Error> {
  match elaborate_impl(sys, config) {
    Ok(_) => {}
    Err(e) => {
      eprintln!("Failed to write to file: {}", e);
      std::process::exit(1);
    }
  }
  let output = Command::new("rustfmt")
    .arg(config.fname.as_str())
    .arg("--config")
    .arg("max_width=100,tab_spaces=2")
    .output()
    .expect("Failed to compile");
  assert!(output.status.success());
  Ok(())
}
