use std::{
  collections::HashMap,
  fs::{self, File},
  io::Write,
  process::Command,
};

use crate::{
  builder::system::SysBuilder,
  ir::{node::*, visitor::Visitor, *},
};

use super::utils::{
  array_ty_to_id, camelize, dtype_to_rust_type, namify, unwrap_array_ty, user_contains_opcode,
};

use self::ir_printer::IRPrinter;

use super::{analysis, Config};

struct ElaborateModule<'a, 'b> {
  sys: &'a SysBuilder,
  indent: usize,
  module_name: String,
  slab_cache: &'b HashMap<BaseNode, usize>,
}

impl<'a, 'b> ElaborateModule<'a, 'b> {
  fn new(sys: &'a SysBuilder, ri: &'b HashMap<BaseNode, usize>) -> Self {
    Self {
      sys,
      indent: 0,
      module_name: String::new(),
      slab_cache: ri,
    }
  }
}

macro_rules! fifo_name {
  ($fifo:expr) => {{
    let module = $fifo.get_parent().as_ref::<Module>($fifo.sys).unwrap();
    format!("{}_{}", namify(module.get_name()), $fifo.idx())
  }};
}

macro_rules! dump_ref {
  ($sys:expr, $value:expr) => {
    NodeRefDumper.dispatch($sys, $value, vec![]).unwrap()
  };
}

struct NodeRefDumper;

impl Visitor<String> for NodeRefDumper {
  fn dispatch(&mut self, sys: &SysBuilder, node: &BaseNode, _: Vec<NodeKind>) -> Option<String> {
    match node.get_kind() {
      NodeKind::Array => {
        let array = node.as_ref::<Array>(sys).unwrap();
        namify(array.get_name()).into()
      }
      NodeKind::FIFO => fifo_name!(node.as_ref::<FIFO>(sys).unwrap()).into(),
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

impl Visitor<String> for ElaborateModule<'_, '_> {
  fn visit_module(&mut self, module: &ModuleRef<'_>) -> Option<String> {
    self.module_name = module.get_name().to_string();
    let mut res = String::new();
    res.push_str(&format!(
      "\n// Elaborating module {}\n",
      namify(module.get_name())
    ));
    // Dump the function signature.
    // First, some common function parameters are dumped.
    res.push_str(&format!("fn {}(\n", namify(module.get_name())));
    res.push_str("  stamp: usize,\n");
    res.push_str("  q: &mut BinaryHeap<Reverse<Event>>,\n");
    for port in module.port_iter() {
      res.push_str(&format!(
        "  {}: &mut VecDeque<{}>,\n",
        fifo_name!(port),
        dtype_to_rust_type(&port.scalar_ty())
      ));
    }
    // All the writes will be done in half a cycle later by events, so no need to feed them
    // to the function signature.
    for (interf, _) in module.ext_interf_iter().filter(|(v, ops)| {
      v.get_kind() == NodeKind::Array && user_contains_opcode(module.sys, ops, vec![Opcode::Load])
    }) {
      let array = interf.as_ref::<Array>(module.sys).unwrap();
      res.push_str(
        format!(
          "{}: &{}, // external array read\n",
          dump_ref!(module.sys, interf),
          dtype_to_rust_type(&array.dtype())
        )
        .as_str(),
      )
    }
    res.push_str(") -> HashSet<usize> { let mut reg_write = HashSet::new();\n");
    self.indent += 2;
    for elem in module.get_body().iter() {
      match elem.get_kind() {
        NodeKind::Expr => {
          let expr = elem.as_ref::<Expr>(self.sys).unwrap();
          res.push_str(&self.visit_expr(&expr).unwrap());
        }
        NodeKind::Block => {
          let block = elem.as_ref::<Block>(self.sys).unwrap();
          res.push_str(&self.visit_block(&block).unwrap());
        }
        _ => {
          panic!("Unexpected reference type: {:?}", elem);
        }
      }
    }
    self.indent -= 2;
    res.push_str(" reg_write }\n");
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
          let array = handle.get_array();
          let slab_idx = *self.slab_cache.get(&array).unwrap();
          let array = array.as_ref::<Array>(expr.sys).unwrap();
          let idx = dump_ref!(expr.sys, handle.get_idx());
          let (scalar_ty, size) = unwrap_array_ty(&array.dtype());
          let aid = array_ty_to_id(&scalar_ty, size);
          format!(
            "reg_write.insert({}); q.push(Reverse(Event{{ stamp: stamp + 50, kind: EventKind::Array{}Write(({}, {} as usize, {})) }}))",
            slab_idx,
            aid,
            slab_idx,
            idx,
            dump_ref!(expr.sys, expr.get_operand(1).unwrap()),
          )
        }
        Opcode::Trigger => {
          let to_trigger =
            if let Ok(module) = expr.get_operand(0).unwrap().as_ref::<Module>(self.sys) {
              format!("EventKind::Module{}", camelize(&namify(module.get_name())))
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
          format!("{}.pop_front().unwrap()", fifo_name!(fifo))
        }
        Opcode::FIFOPeek => {
          let fifo = expr
            .get_operand(0)
            .unwrap()
            .as_ref::<FIFO>(self.sys)
            .unwrap();
          format!("{}.front().unwrap().clone()", fifo_name!(fifo))
        }
        Opcode::FIFOPush => {
          let value = dump_ref!(self.sys, expr.get_operand(2).unwrap());
          let module = expr
            .get_operand(0)
            .unwrap()
            .as_ref::<Module>(self.sys)
            .unwrap();
          let fifo_idx = expr
            .get_operand(1)
            .unwrap()
            .as_ref::<IntImm>(self.sys)
            .unwrap()
            .get_value();
          let port = module
            .get_input(fifo_idx as usize)
            .unwrap()
            .as_ref::<FIFO>(self.sys)
            .unwrap();
          let slab_idx = *self.slab_cache.get(&port.upcast()).unwrap();
          if expr.get_operand(0).unwrap().get_kind() == NodeKind::Module {
            format!(
              "reg_write.insert({}); q.push(Reverse(Event{{ stamp: stamp + 50, kind: EventKind::FIFO{}Push(({}, {})) }}))",
              slab_idx,
              dtype_to_rust_type(&port.scalar_ty()),
              slab_idx,
              value
            )
          } else {
            let module = dump_ref!(self.sys, expr.get_operand(0).unwrap());
            format!(
              "// q.push(Reverse(Event{{ stamp: stamp + 50, kind: to_push({}.as_ref().clone(), {}, {} as u64) }}))",
              module, fifo_idx, value
            )
          }
        }
        Opcode::Log => {
          let mut res = String::new();
          res.push_str(&format!(
            "print!(\"@line:{{:<5}} {{}}: [{}]\t\", line!(), cyclize(stamp));",
            self.module_name
          ));
          res.push_str("println!(");
          for elem in expr.operand_iter() {
            res.push_str(&format!("{}, ", dump_ref!(self.sys, elem)));
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
      res.push_str(&format!(
        "  if {}{} {{\n",
        dump_ref!(self.sys, &cond),
        if cond.get_dtype(block.sys).unwrap().bits() == 1 {
          "".into()
        } else {
          format!(" != 0")
        }
      ));
    }
    self.indent += 2;
    for elem in block.iter() {
      match elem.get_kind() {
        NodeKind::Expr => {
          let expr = elem.as_ref::<Expr>(self.sys).unwrap();
          res.push_str(&self.visit_expr(&expr).unwrap());
        }
        NodeKind::Block => {
          let block = elem.as_ref::<Block>(self.sys).unwrap();
          res.push_str(&self.visit_block(&block).unwrap());
        }
        _ => {
          panic!("Unexpected reference type: {:?}", elem);
        }
      }
    }
    self.indent -= 2;
    if block.get_pred().is_some() {
      res.push_str(&format!("{}}}\n", " ".repeat(self.indent)));
    }
    res.into()
  }
}

fn dump_runtime(sys: &SysBuilder, config: &Config) -> (String, HashMap<BaseNode, usize>) {
  let mut res = String::new();
  // Dump the helper function of cycles.
  res.push_str("// Simulation runtime.\n");
  res.push_str(
    &quote::quote! {
      fn cyclize(stamp: usize) -> String {
        format!("Cycle @{}.{:02}", stamp / 100, stamp % 100)
      }
    }
    .to_string(),
  );
  res.push('\n');

  // Dump the event enum. Each event corresponds to a module.
  // Each event instance looks like this:
  //
  // enum EventKind {
  //   Module{camelize(module.get_name())},
  //   ...
  //   Array{data_type}Write((ref_idx, array, array_idx, value)),
  //   ...
  //   FIFOPush{data_type}((ref_idx, fifo, value)),
  // }
  res.push_str("#[derive(Clone, Debug, Eq, PartialEq)]\n");
  res.push_str("enum EventKind {\n");
  for module in sys.module_iter() {
    res.push_str(&format!(
      "  Module{},\n",
      camelize(&namify(module.get_name()))
    ));
  }
  let array_types = analysis::types::array_types_used(sys);
  for aty in array_types.iter() {
    let (scalar_ty, size) = unwrap_array_ty(aty);
    let scalar_str = dtype_to_rust_type(&scalar_ty);
    let array_str = array_ty_to_id(&scalar_ty, size);
    res.push_str(&format!(
      "  Array{}Write((usize, usize, {})),\n",
      array_str, scalar_str
    ));
  }
  let fifo_types = analysis::types::fifo_types_used(sys);
  for fty in fifo_types.iter() {
    let ty = dtype_to_rust_type(&fty);
    res.push_str(&format!("  FIFO{}Push((usize, {})),\n", ty, ty));
  }
  res.push_str("}\n\n");

  // Dump the universal set of data types used in this simulation.
  res.push_str("enum DataSlab {");
  for array in array_types.iter() {
    let (scalar_ty, size) = unwrap_array_ty(array);
    res.push_str(&format!(
      "  Array{}(Box<{}>),\n",
      array_ty_to_id(&scalar_ty, size),
      dtype_to_rust_type(&array),
    ));
  }
  for fifo in fifo_types.iter() {
    res.push_str(&format!(
      "  FIFO{}(Box<VecDeque<{}>>),\n",
      dtype_to_rust_type(&fifo),
      dtype_to_rust_type(&fifo)
    ));
  }
  res.push_str("}\n\n");
  // Dump the state of ownership of each register.
  res.push_str(
    &quote::quote! {
      #[derive(Clone, Debug, PartialEq, Eq)]
      enum Ownership {
        None,
        Writingby(EventKind),
      }
    }
    .to_string(),
  );
  // Dump the slab entry struct.
  res.push_str(
    &quote::quote! {
      struct SlabEntry {
        payload: DataSlab,
        ownership: Ownership,
      }
    }
    .to_string(),
  );

  // res.push_str("fn to_push(ek: EventKind, idx: usize, value: u64) -> EventKind {\n");
  // res.push_str("  match (ek.clone(), idx) {\n");
  // for module in sys.module_iter() {
  //   for (i, port) in module.port_iter().enumerate() {
  //     if !port.scalar_ty().is_int() {
  //       continue;
  //     }
  //     res.push_str(&format!(
  //       "    (EventKind::Module_{}, {}) => EventKind::FIFO_push_{}(value as {}),\n",
  //       namify(module.get_name()),
  //       i,
  //       &fifo_name!(port),
  //       dtype_to_rust_type(&port.scalar_ty())
  //     ));
  //   }
  // }
  // res.push_str("    _ => panic!(\"Unknown event to push, {:?}, {}\", ek, idx),\n");
  // res.push_str("  }\n");
  // res.push_str("}\n\n");

  res.push_str(
    &quote::quote! {
      #[derive(Debug, PartialEq, Eq)]
      struct Event {
        stamp: usize,
        kind: EventKind,
      }
      impl std::cmp::Ord for Event {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
          self.stamp.cmp(&other.stamp)
        }
      }
      impl std::cmp::PartialOrd for Event {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
          Some(self.stamp.cmp(&other.stamp))
        }
      }
    }
    .to_string(),
  );

  // TODO(@were): Make all arguments of the modules FIFO channels.
  // TODO(@were): Profile the maxium size of all the FIFO channels.
  res.push_str("fn main() {\n");
  res.push_str("  // The global time stamp\n");
  res.push_str("  let mut stamp: usize = 0;\n");
  res.push_str("  // Count the consecutive cycles idled\n");
  res.push_str("  let mut idled: usize = 0;\n");
  res.push_str("  let mut data_slab : Vec<Option<SlabEntry>> = vec![\n");
  res.push_str("  // Define global arrays\n");
  let mut slab_cache: HashMap<BaseNode, usize> = HashMap::new();
  for array in sys.array_iter() {
    let (scalar_ty, size) = unwrap_array_ty(&array.dtype());
    let scalar_str = dtype_to_rust_type(&scalar_ty);
    res.push_str(&format!(
      "Some(SlabEntry {{ payload: DataSlab::Array{}(Box::new([{}; {}])), ownership: Ownership::None }}), // {} -> {}\n",
      array_ty_to_id(&scalar_ty, size),
      if scalar_ty.bits() == 1 {
        "false".into()
      } else {
        format!("0 as {}", scalar_str)
      },
      size,
      slab_cache.len(),
      IRPrinter::new().visit_array(&array).unwrap()
    ));
    slab_cache.insert(array.upcast(), slab_cache.len());
  }
  res.push_str("\n\n  // Define the module FIFOs\n");
  for module in sys.module_iter() {
    for port in module.port_iter() {
      res.push_str(&format!(
        "Some(SlabEntry {{ payload: DataSlab::FIFO{}(Box::new(VecDeque::new())), ownership: Ownership::None }}), // {} -> {}.{}\n",
        dtype_to_rust_type(&port.scalar_ty()),
        slab_cache.len(),
        module.get_name(),
        port.idx()
      ));
      slab_cache.insert(port.upcast(), slab_cache.len());
    }
  }
  res.push_str("];\n\n");
  res.push_str("  // Define the event queue\n");
  res.push_str("  let mut q = BinaryHeap::new();\n");
  let sim_threshold = config.sim_threshold;
  if sys.has_driver() {
    // Push the initial events.
    res.push_str(
      &quote::quote! {
        for i in 0..#sim_threshold {
          q.push(Reverse(Event{stamp: i * 100, kind: EventKind::ModuleDriver}));
        }
      }
      .to_string(),
    );
  }
  // TODO(@were): Dump the time stamp of the simulation.
  res.push_str("  while let Some(event) = q.pop() {\n");
  res.push_str(
    &quote::quote! {
      if event.0.stamp / 100 > #sim_threshold {
        print!("Exceed the simulation threshold ");
        print!("{}", #sim_threshold);
        println!(", exit!");
        break;
      }
    }
    .to_string(),
  );
  res.push_str("    match event.0.kind {\n");
  for module in sys.module_iter() {
    res.push_str(&format!(
      "      EventKind::Module{} => {{\n",
      camelize(&namify(module.get_name()))
    ));
    // Unpacking the FIFO's from the slab.
    for fifo in module.port_iter() {
      let id = fifo_name!(fifo);
      let slab_idx = *slab_cache.get(&fifo.upcast()).unwrap();
      res.push_str(&format!(
        "let mut {} = match std::mem::replace(&mut data_slab[{}], None) {{",
        id, slab_idx,
      ));
      res.push_str(&format!(
        "Some(SlabEntry {{ payload: DataSlab::FIFO{}(x), .. }}) => x,",
        dtype_to_rust_type(&fifo.scalar_ty())
      ));
      res.push_str("_ => panic!(\"Unexpected slab type\"),");
      res.push_str("};\n");
    }
    let ext_interf_args = module
      .ext_interf_iter()
      .filter(|(v, ops)| {
        v.get_kind() == NodeKind::Array && user_contains_opcode(module.sys, ops, vec![Opcode::Load])
      })
      .map(|(elem, _)| {
        let id = dump_ref!(sys, elem);
        let slab_idx = *slab_cache.get(&elem).unwrap();
        res.push_str(&format!("let {} = match &data_slab[{}] {{", id, slab_idx,));
        let aty = elem.as_ref::<Array>(sys).unwrap().dtype();
        let aty = unwrap_array_ty(&aty);
        res.push_str(&format!(
          "Some(SlabEntry {{ payload: DataSlab::Array{}(x), .. }}) => x,",
          array_ty_to_id(&aty.0, aty.1)
        ));
        res.push_str("_ => panic!(\"Unexpected slab type\"),");
        res.push_str("};\n");
        id
      })
      .collect::<Vec<_>>();
    // Dump the function call.
    let callee = namify(module.get_name());
    res.push_str(&format!(
      "        let reg_write = {}(event.0.stamp, &mut q,",
      callee,
    ));
    for fifo in module.port_iter() {
      res.push_str(&format!(" &mut {},", fifo_name!(fifo)));
    }
    for elem in ext_interf_args {
      res.push_str(&elem);
      res.push(',');
    }
    res.push_str(");\n");
    // Return the ownership.
    for fifo in module.port_iter() {
      let id = fifo_name!(fifo);
      let idx = *slab_cache.get(&fifo.upcast()).unwrap();
      res.push_str(format!("assert!(data_slab[{}].is_none());", idx).as_str());
      res.push_str(&format!(
        "data_slab[{}] = Some(SlabEntry{{ payload: DataSlab::FIFO{}({}), ownership: Ownership::None}});\n",
        idx,
        dtype_to_rust_type(&fifo.scalar_ty()),
        id
      ));
    }
    for (elem, op) in module.ext_interf_iter() {
      let idx = *slab_cache.get(&elem).unwrap();
      if user_contains_opcode(module.sys, op, vec![Opcode::Store, Opcode::FIFOPush]) {
        res.push_str(&format!(
          "if reg_write.contains(&{}) {{ match &mut data_slab[{}] {{ // {}\n",
          idx,
          idx,
          dump_ref!(sys, elem)
        ));
        res.push_str("  Some(SlabEntry { ownership: x, .. } ) => {\n");
        res.push_str(&format!(
          "assert_eq!(x, &Ownership::None); *x = Ownership::Writingby(EventKind::Module{});\n",
          camelize(&callee)
        ));
        res.push_str("  }\n");
        res.push_str("  _ => panic!(\"Unexpected slab type\"),\n");
        res.push_str("}}\n");
      }
    }
    if !module.get_name().eq("driver") {
      res.push_str("idled = 0; continue; }\n");
    } else {
      res.push_str("idled += 1; stamp = event.0.stamp; }\n");
    }
  }
  // match &mut data_slab[slab_idx] {
  //   Some(SlabEntry { payload: DataSlab::Array{}(array), .. }) => {
  //     array[idx] = value;
  //   }
  //   _ => panic!("Unexpected slab type"),
  // }
  for aty in array_types.iter() {
    let (scalar_ty, size) = unwrap_array_ty(aty);
    let aid = array_ty_to_id(&scalar_ty, size);
    res.push_str(&format!(
      "EventKind::Array{}Write((slab_idx, idx, value)) => {{\n",
      aid
    ));
    res.push_str("match &mut data_slab[slab_idx] {\n");
    res.push_str(&format!(
      "Some(SlabEntry {{ payload: DataSlab::Array{}(array), ownership: o }}) => {{\n",
      aid
    ));
    res.push_str("array[idx] = value; *o = Ownership::None; }\n");
    res.push_str("_ => panic!(\"Unexpected slab type\"),\n");
    res.push_str("}}\n");
  }
  for fifo_ty in fifo_types.iter() {
    let ty = dtype_to_rust_type(fifo_ty);
    res.push_str(&format!(
      "EventKind::FIFO{}Push((slab_idx, value)) => {{\n",
      ty
    ));
    res.push_str("match &mut data_slab[slab_idx] {\n");
    res.push_str(&format!(
      "Some(SlabEntry {{ payload: DataSlab::FIFO{}(fifo), ownership: o }}) => {{\n",
      ty
    ));
    res.push_str("fifo.push_back(value); *o = Ownership::None; }\n");
    res.push_str("_ => panic!(\"Unexpected slab type\"),\n");
    res.push_str("}}\n");
  }
  res.push_str("}\n");
  res.push_str(&format!("    if idled > {} {{\n", config.idle_threshold));
  res.push_str(&format!(
    "      println!(\"Idled more than {} cycles, exit @{{}}!\", cyclize(stamp));\n",
    config.idle_threshold
  ));
  res.push_str("      break;\n");
  res.push_str("    }\n");
  res.push_str("  }\n");
  res.push_str("  println!(\"Finish simulation: {}!\", cyclize(stamp));\n");
  res.push_str("}\n\n");
  (res, slab_cache)
}

fn dump_module(
  sys: &SysBuilder,
  fd: &mut File,
  slab_cache: &HashMap<BaseNode, usize>,
) -> Result<(), std::io::Error> {
  let mut em = ElaborateModule::new(sys, slab_cache);
  for module in em.sys.module_iter() {
    if let Some(buffer) = em.visit_module(&module) {
      fd.write(buffer.as_bytes())?;
    }
  }
  Ok(())
}

fn dump_header(fd: &mut File) -> Result<usize, std::io::Error> {
  let src = quote::quote! {
    use std::collections::VecDeque;
    use std::collections::BinaryHeap;
    use std::collections::HashSet;
    use std::cmp::{Ord, Reverse};
  };
  fd.write(src.to_string().as_bytes())?;
  fd.write("\n\n\n".as_bytes())
}

pub fn elaborate_impl(sys: &SysBuilder, config: &Config) -> Result<(), std::io::Error> {
  println!("Writing simulator code to {}", config.fname);
  let mut fd = fs::File::create(config.fname.clone())?;
  dump_header(&mut fd)?;
  let (rt_src, ri) = dump_runtime(sys, config);
  dump_module(sys, &mut fd, &ri)?;
  fd.write(rt_src.as_bytes())?;
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
    .arg(&config.fname)
    .arg("--config")
    .arg("max_width=100,tab_spaces=2")
    .output()
    .expect("Failed to compile");
  assert!(output.status.success());
  Ok(())
}
