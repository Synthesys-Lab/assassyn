use std::{
  collections::HashMap,
  fs::{self, File},
  io::Write,
  process::Command,
};

use proc_macro2::Span;
use syn::Ident;

use crate::{
  backend::common::Config,
  builder::system::SysBuilder,
  ir::{bind::get_bind_callee, node::*, visitor::Visitor, *},
};

use super::utils::{
  array_ty_to_id, camelize, dtype_to_rust_type, namify, unwrap_array_ty, user_contains_opcode,
};

use self::{
  ir_printer::IRPrinter,
  module::memory::{module_is_memory, parse_memory_module_name},
};

use super::analysis;

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
        format!("Box::new(EventKind::Module{})", module_name).into()
      }
      _ => Some(format!("_{}", node.get_key())),
    }
  }
}

impl ElaborateModule<'_, '_> {
  fn current_module_id(&self) -> syn::Ident {
    let s = format!("Module{}", camelize(&namify(&self.module_name)));
    syn::Ident::new(&s, Span::call_site())
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
        "  {}: &VecDeque<{}>,\n",
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
    if module_is_memory(&self.module_name) {
      res.push_str("mem: &mut Vec<i32>,");
    }
    res.push_str(") {\n");
    self.indent += 2;

    if let Some(params) = parse_memory_module_name(&self.module_name) {
      res.push_str(format!("// Memory {}\n", params.name).as_str());
      res.push_str(
        format!(
          "// width = {}, depth = {}, lat = [{}, {}]\n",
          params.width, params.depth, params.lat_min, params.lat_max
        )
        .as_str(),
      );
      if let Some(init_file) = params.init_file {
        res.push_str(format!("// init_file =  {}\n\n", init_file).as_str());
        todo!();
      }

      let mut rdata_fifo: Option<String> = None;
      let mut rdata_module: Option<String> = None;
      for node in module.get_body().iter() {
        let expr = node.as_ref::<Expr>(self.sys).unwrap();
        match expr.get_opcode() {
          Opcode::FIFOPush => {
            let fifo = expr
              .get_operand(0)
              .unwrap()
              .get_value()
              .as_ref::<FIFO>(self.sys)
              .unwrap();
            let slab_idx = *self.slab_cache.get(&fifo.upcast()).unwrap();
            let fifo_push = syn::Ident::new(
              &format!("FIFO{}Push", dtype_to_rust_type(&fifo.scalar_ty())),
              Span::call_site(),
            );
            let module_writer = self.current_module_id();
            rdata_fifo = Some(
              quote::quote! {
                q.push(Reverse(Event{
                  stamp: stamp + read_latency * 100 - 50,
                  kind: EventKind::#fifo_push((EventKind::#module_writer.into(), #slab_idx, rdata))
                }));
              }
              .to_string(),
            );
          }
          Opcode::AsyncCall => {
            let bind = expr
              .get_operand(0)
              .unwrap()
              .get_value()
              .as_ref::<Expr>(self.sys)
              .unwrap();
            let module = bind
              .get_operand(bind.get_num_operands() - 1)
              .unwrap()
              .get_value()
              .as_ref::<Module>(self.sys)
              .unwrap();
            let to_trigger = format!("EventKind::Module{}", camelize(&namify(module.get_name())));
            rdata_module = Some(format!(
              "q.push(Reverse(Event{{ stamp: stamp + read_latency * 100, kind: {} }}))",
              to_trigger
            ));
          }
          Opcode::Bind(_) => { /* don't care, processed in corresponding AsyncCall` */ }
          _ => panic!("Unexpected expr of {:?} in memory body", expr.get_opcode()),
        }
      }

      let fifo = module.port_iter().next().unwrap();
      let slab_idx = *self.slab_cache.get(&fifo.upcast()).unwrap();
      let fifo_ty = fifo.scalar_ty();
      let fifo_pop = syn::Ident::new(
        &format!("FIFO{}Pop", dtype_to_rust_type(&fifo_ty)),
        Span::call_site(),
      );
      let module_writer = self.current_module_id();
      let fifo_name = syn::Ident::new(&fifo_name!(fifo), Span::call_site());
      res.push_str(
        quote::quote! {
          let raddr = {
          q.push(Reverse(Event{
            stamp: stamp + 50,
            kind: EventKind::#fifo_pop((EventKind::#module_writer.into(), #slab_idx))
          }));
          #fifo_name.front().unwrap().clone()
        };}
        .to_string()
        .as_str(),
      );

      res.push_str("let rdata = mem[raddr as usize];\n");

      // TODO: truely randomize latency, requires emitting cargo wrapped project
      res.push_str(format!("let read_latency = {};\n", params.lat_min).as_str());

      res.push_str(rdata_fifo.unwrap().as_str());
      res.push_str(rdata_module.unwrap().as_str());
    } else {
      res.push_str(&self.visit_block(&module.get_body()).unwrap());
    }

    self.indent -= 2;
    res.push_str("}\n");
    res.into()
  }

  fn visit_expr(&mut self, expr: &ExprRef<'_>) -> Option<String> {
    let res = if expr.get_opcode().is_binary() {
      let ty = expr.dtype();
      let ty = dtype_to_rust_type(&ty);
      format!(
        "({} as {}) {} ({} as {})",
        dump_ref!(self.sys, &expr.get_operand(0).unwrap().get_value()),
        ty,
        expr.get_opcode().to_string(),
        dump_ref!(self.sys, &expr.get_operand(1).unwrap().get_value()),
        ty,
      )
    } else if expr.get_opcode().is_unary() {
      format!(
        "{}{}",
        expr.get_opcode().to_string(),
        dump_ref!(self.sys, &expr.get_operand(0).unwrap().get_value())
      )
    } else if expr.get_opcode().is_cmp() {
      format!(
        "{} {} {}",
        dump_ref!(self.sys, &expr.get_operand(0).unwrap().get_value()),
        expr.get_opcode().to_string(),
        dump_ref!(self.sys, &expr.get_operand(1).unwrap().get_value()),
      )
    } else {
      match expr.get_opcode() {
        Opcode::Load => {
          let handle = expr
            .get_operand(0)
            .unwrap()
            .get_value()
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
            .get_value()
            .as_ref::<ArrayPtr>(expr.sys)
            .unwrap();
          let array = handle.get_array();
          let slab_idx = *self.slab_cache.get(&array).unwrap();
          let array = array.as_ref::<Array>(expr.sys).unwrap();
          let idx = dump_ref!(expr.sys, handle.get_idx());
          let idx = idx.parse::<proc_macro2::TokenStream>().unwrap();
          let (scalar_ty, size) = unwrap_array_ty(&array.dtype());
          let aid = array_ty_to_id(&scalar_ty, size);
          let id = syn::Ident::new(&format!("Array{}Write", aid), Span::call_site());
          let value = dump_ref!(self.sys, &expr.get_operand(1).unwrap().get_value());
          let value = value.parse::<proc_macro2::TokenStream>().unwrap();
          let module_writer = self.current_module_id();
          quote::quote! {
            q.push(Reverse(Event{
              stamp: stamp + 50,
              kind: EventKind::#id(
                (EventKind::#module_writer.into(), #slab_idx, #idx as usize, #value))
            }))
          }
          .to_string()
        }
        Opcode::AsyncCall => {
          let to_trigger = if let Ok(module) = {
            let callee =
              get_bind_callee(self.sys, expr.get_operand(0).unwrap().get_value().clone());
            callee.as_ref::<Module>(self.sys)
          } {
            format!("EventKind::Module{}", camelize(&namify(module.get_name())))
          } else {
            panic!("AsyncCall target is not a module, did you rewrite the callback?");
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
            .get_value()
            .as_ref::<FIFO>(self.sys)
            .unwrap();
          let slab_idx = *self.slab_cache.get(&fifo.upcast()).unwrap();
          let fifo_ty = fifo.scalar_ty();
          let fifo_pop = syn::Ident::new(
            &format!("FIFO{}Pop", dtype_to_rust_type(&fifo_ty)),
            Span::call_site(),
          );
          let module_writer = self.current_module_id();
          let fifo_name = syn::Ident::new(&fifo_name!(fifo), Span::call_site());
          quote::quote! {{
            q.push(Reverse(Event{
              stamp: stamp + 50,
              kind: EventKind::#fifo_pop((EventKind::#module_writer.into(), #slab_idx))
            }));
            #fifo_name.front().unwrap().clone()
          }}
          .to_string()
        }
        Opcode::FIFOPeek => {
          let fifo = expr
            .get_operand(0)
            .unwrap()
            .get_value()
            .as_ref::<FIFO>(self.sys)
            .unwrap();
          format!("{}.front().unwrap().clone()", fifo_name!(fifo))
        }
        Opcode::FIFOValid => {
          let fifo = expr
            .get_operand(0)
            .unwrap()
            .get_value()
            .as_ref::<FIFO>(self.sys)
            .unwrap();
          format!("!{}.is_empty()", fifo_name!(fifo))
        }
        Opcode::FIFOPush => {
          let fifo = expr
            .get_operand(0)
            .unwrap()
            .get_value()
            .as_ref::<FIFO>(self.sys)
            .unwrap();
          let slab_idx = *self.slab_cache.get(&fifo.upcast()).unwrap();
          let fifo_push = syn::Ident::new(
            &format!("FIFO{}Push", dtype_to_rust_type(&fifo.scalar_ty())),
            Span::call_site(),
          );
          let value = dump_ref!(self.sys, expr.get_operand(1).unwrap().get_value());
          let value = value.parse::<proc_macro2::TokenStream>().unwrap();
          let module_writer = self.current_module_id();
          if !fifo.is_placeholder() {
            quote::quote! {
              q.push(Reverse(Event{
                stamp: stamp + 50,
                kind: EventKind::#fifo_push((EventKind::#module_writer.into(), #slab_idx, #value))
              }))
            }
            .to_string()
          } else {
            panic!(
              "FIFO is a placeholder, cannot push to it! Did you forget to rewrite callbacks?"
            );
          }
        }
        Opcode::Log => {
          let mut res = String::new();
          res.push_str(&format!(
            "print!(\"@line:{{:<5}}\t{{}}:\t[{}]\t\", line!(), cyclize(stamp));",
            self.module_name
          ));
          res.push_str("println!(");
          for elem in expr.operand_iter() {
            res.push_str(&format!("{}, ", dump_ref!(self.sys, elem.get_value())));
          }
          res.push(')');
          res
        }
        Opcode::Slice => {
          let a = dump_ref!(self.sys, &expr.get_operand(0).unwrap().get_value());
          let l = dump_ref!(self.sys, &expr.get_operand(1).unwrap().get_value());
          let r = dump_ref!(self.sys, &expr.get_operand(2).unwrap().get_value());
          let bits = {
            let dtype = expr
              .get_operand(0)
              .unwrap()
              .get_value()
              .get_dtype(self.sys)
              .unwrap();
            dtype.get_bits()
          };
          format!(
            "{{
            let a = {} as u64;
            let l = {} as usize;
            let r = {} as usize;
            let len = r - l + 1;
            let mask = !0 >> ({} - len);
            ((a >> l) & mask) as {}
          }}",
            a,
            l,
            r,
            bits,
            dtype_to_rust_type(&expr.dtype())
          )
        }
        Opcode::Select => {
          let cond = dump_ref!(self.sys, &expr.get_operand(0).unwrap().get_value());
          let true_value = dump_ref!(self.sys, &expr.get_operand(1).unwrap().get_value());
          let false_value = dump_ref!(self.sys, &expr.get_operand(2).unwrap().get_value());
          format!(
            "if {} {{ {} }} else {{ {} }}",
            cond, true_value, false_value
          )
        }
        Opcode::Bind(_) => {
          let callee = {
            let n = expr.get_num_operands();
            let callee = expr.get_operand(n - 1).unwrap().get_value().clone();
            let module = callee.as_ref::<Module>(expr.sys).unwrap();
            format!("EventKind::Module{}", camelize(&namify(module.get_name())))
          };
          format!("let {} = {}", dump_ref!(self.sys, &expr.upcast()), callee)
        }
        _ => {
          if !expr.get_opcode().is_unary()
            && !expr.get_opcode().is_binary()
            && !expr.get_opcode().is_cmp()
          {
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
    match block.get_kind() {
      BlockKind::Condition(cond) => {
        res.push_str(&format!(
          "  if {}{} {{\n",
          dump_ref!(self.sys, &cond),
          if cond.get_dtype(block.sys).unwrap().get_bits() == 1 {
            "".into()
          } else {
            format!(" != 0")
          }
        ));
      }
      BlockKind::Cycle(cycle) => {
        res.push_str(&format!("  if stamp / 100 == {} {{\n", cycle));
      }
      BlockKind::WaitUntil(cond) => {
        let dtype = {
          let cond = cond.as_ref::<Block>(block.sys).unwrap();
          cond.get_value().unwrap().get_dtype(block.sys).unwrap()
        };
        let cond = self.dispatch(block.sys, &cond, vec![]).unwrap();
        res.push_str(&format!(
          "  if {}{} {{\n",
          cond,
          if dtype.get_bits() == 1 {
            "".into()
          } else {
            format!(" != 0")
          }
        ));
      }
      BlockKind::Valued(_) | BlockKind::None => {
        res.push('{');
      }
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
    if let BlockKind::Condition(_) = block.get_kind() {}
    match block.get_kind() {
      BlockKind::Condition(_) | BlockKind::Cycle(_) => {
        res.push_str(&format!("{}}}\n", " ".repeat(self.indent)));
      }
      BlockKind::WaitUntil(_) => {
        res.push_str(&format!("{}}} else {{\n", " ".repeat(self.indent)));
        let module_eventkind_id = self.current_module_id();
        res.push_str(
          &quote::quote! {
            // retry at next cycle
            q.push(Reverse(Event {
              stamp: stamp + 100,
              kind: EventKind::#module_eventkind_id,
            }));
          }
          .to_string(),
        );
        res.push_str(&format!("{}}}\n", " ".repeat(self.indent)));
      }
      BlockKind::Valued(value) => {
        res.push_str(&dump_ref!(self.sys, value));
        res.push('}');
      }
      BlockKind::None => {
        res.push('}');
      }
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
  //   Array{data_type}Write((writer, ref_idx, array, array_idx, value)),
  //   ...
  //   FIFOPush{data_type}((writer, ref_idx, fifo, value)),
  //   ...
  //   FIFO{data_type}Pop((writer, ref_idx, fifo)),
  //   None
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
      "  Array{}Write((Box<EventKind>, usize, usize, {})),\n",
      array_str, scalar_str
    ));
  }
  let fifo_types = analysis::types::fifo_types_used(sys);
  for fty in fifo_types.iter() {
    let ty = dtype_to_rust_type(&fty);
    res.push_str(&format!(
      "  FIFO{}Push((Box<EventKind>, usize, {})),\n",
      ty, ty
    ));
    res.push_str(&format!("  FIFO{}Pop((Box<EventKind>, usize)),\n", ty));
  }
  res.push_str("None, }\n\n");

  res.push_str("impl EventKind {\n");
  res.push_str(
    &quote::quote! {
      fn is_none(&self) -> bool {
        match self {
          EventKind::None => true,
          _ => false,
        }
      }
    }
    .to_string(),
  );
  res.push_str("\n\nfn is_push(&self) -> bool { match self {\n");
  for fty in fifo_types.iter() {
    let ty = dtype_to_rust_type(&fty);
    res.push_str(&format!("  EventKind::FIFO{}Push(_) => true,\n", ty,));
  }
  res.push_str("_ => false, }}\n\n");
  res.push_str("fn is_pop(&self) -> bool { match self {\n");
  for fty in fifo_types.iter() {
    let ty = dtype_to_rust_type(&fty);
    res.push_str(&format!("  EventKind::FIFO{}Pop(_) => true,\n", ty,));
  }
  res.push_str("_ => false, }}\n");
  res.push('}');

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
  // Dump the slab entry struct.
  res.push_str(
    &quote::quote! {
      struct LastOperation {
        operation: Box<EventKind>,
        stamp: usize,
      }
      struct SlabEntry {
        payload: DataSlab,
        last_written: LastOperation,
      }
      impl LastOperation {
        fn update(&mut self, operation: Box<EventKind>, stamp: usize) {
          self.operation = operation;
          self.stamp = stamp;
        }
        fn ok(&mut self, operation: Box<EventKind>, stamp: usize) {
          if self.stamp == stamp {
            if (self.operation.is_none() && self.operation.is_none()) ||
               (self.operation.is_push() && operation.as_ref().is_pop()) ||
               (self.operation.is_pop() && operation.as_ref().is_push()) {
              self.update(operation, stamp);
            } else {
              panic!(
                "{}: Conflict, performing {:?}, but last written by {:?}",
                cyclize(stamp), operation, self.operation);
            }
          } else {
            self.update(operation, stamp);
          }
        }
      }
      trait UnwrapSlab {
        fn unwrap(entry: &SlabEntry) -> &Self;
        fn unwrap_mut(entry: &mut SlabEntry) -> &mut Self;
      }
      impl <'a> SlabEntry {
        fn unwrap_payload<T: UnwrapSlab>(&'a self) -> &'a T {
          T::unwrap(self)
        }
        fn unwrap_payload_mut<T: UnwrapSlab>(&'a mut self) -> &'a mut T {
          T::unwrap_mut(self)
        }
      }
    }
    .to_string(),
  );

  res.push_str(
    &quote::quote! {
      #[derive(Clone, Debug, PartialEq, Eq)]
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

  res.push_str(
    "
macro_rules! impl_unwrap_slab {
  ($ty: ty, $tyid: ident) => {
    impl UnwrapSlab for $ty {
      fn unwrap(entry: &SlabEntry) -> &Self {
        match entry {
          SlabEntry {
            payload: DataSlab::$tyid(data),
            ..
          } => data.as_ref(),
          _ => panic!(\"Invalid slab entry\"),
        }
      }
      fn unwrap_mut(entry: &mut SlabEntry) -> &mut Self {
        match entry {
          SlabEntry {
            payload: DataSlab::$tyid(data),
            ..
          } => data,
          _ => panic!(\"Invalid slab entry\"),
        }
      }
    }
  };
}",
  );

  for array in array_types.iter() {
    let (scalar_ty, size) = unwrap_array_ty(array);
    let aid = array_ty_to_id(&scalar_ty, size);
    let scalar_ty = dtype_to_rust_type(&scalar_ty);
    res.push_str(&format!(
      "impl_unwrap_slab!([{}; {}], Array{});\n",
      scalar_ty, size, aid
    ));
  }

  for fifo in fifo_types.iter() {
    let ty = dtype_to_rust_type(fifo);
    res.push_str(&format!(
      "impl_unwrap_slab!(VecDeque<{}>, FIFO{});\n",
      ty, ty
    ));
  }

  // TODO(@were): Make all arguments of the modules FIFO channels.
  // TODO(@were): Profile the maxium size of all the FIFO channels.
  res.push_str("fn main() {\n");
  res.push_str("  // The global time stamp\n");
  res.push_str("  let mut stamp: usize = 0;\n");
  res.push_str("  // Count the consecutive cycles idled\n");
  res.push_str("  let mut idled: usize = 0;\n");
  res.push_str("  let mut data_slab : Vec<SlabEntry> = vec![\n");
  res.push_str("  // Define global arrays\n");
  let mut slab_cache: HashMap<BaseNode, usize> = HashMap::new();
  for array in sys.array_iter() {
    let (scalar_ty, size) = unwrap_array_ty(&array.dtype());
    let scalar_str = dtype_to_rust_type(&scalar_ty);
    let aty_id = Ident::new(
      &format!("Array{}", array_ty_to_id(&scalar_ty, size)),
      Span::call_site(),
    );
    let init_scalar = if scalar_ty.get_bits() == 1 {
      "false".into()
    } else {
      format!("0 as {}", scalar_str)
    }
    .parse::<proc_macro2::TokenStream>()
    .unwrap();
    res.push_str(&format!(
      "  // {} -> {}\n",
      slab_cache.len(),
      IRPrinter::new(false).visit_array(&array).unwrap(),
    ));
    res.push_str(
      &quote::quote! {
        SlabEntry {
          payload: DataSlab::#aty_id(Box::new([#init_scalar; #size])),
          last_written: LastOperation {
            operation: EventKind::None.into(),
            stamp: 0
          }
        },
      }
      .to_string(),
    );
    slab_cache.insert(array.upcast(), slab_cache.len());
    res.push('\n');
  }
  res.push_str("\n\n  // Define the module FIFOs\n");
  for module in sys.module_iter() {
    for port in module.port_iter() {
      let fifo_ty = dtype_to_rust_type(&port.scalar_ty());
      let fifo_ty = Ident::new(&format!("FIFO{}", fifo_ty), Span::call_site());
      res.push_str(&format!(
        "  // {} -> {}.{}\n",
        slab_cache.len(),
        module.get_name(),
        port.get_name()
      ));
      res.push_str(
        &quote::quote! {
          SlabEntry {
            payload: DataSlab::#fifo_ty(Box::new(VecDeque::new())),
            last_written: LastOperation {
              operation: EventKind::None.into(),
              stamp: 0
            }
          },
        }
        .to_string(),
      );
      slab_cache.insert(port.upcast(), slab_cache.len());
      res.push('\n');
    }
  }
  res.push_str("];\n\n");
  res.push_str("  // Define the event queue\n");
  res.push_str("  let mut q: BinaryHeap<Reverse<Event>> = BinaryHeap::new();\n");
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
  if sys.has_testbench() {
    let testbench_vec = sys
      .module_iter()
      .filter(|m| m.get_name() == "testbench")
      .collect::<Vec<ModuleRef>>();
    let cycles = testbench_vec[0]
      .get_body()
      .iter()
      .filter_map(|n| -> Option<usize> {
        if n.get_kind() == NodeKind::Block {
          let block = n.as_ref::<Block>(sys).unwrap();
          match block.get_kind() {
            BlockKind::Cycle(cycle) => Some(*cycle),
            _ => None,
          }
        } else {
          None
        }
      })
      .collect::<Vec<usize>>();
    // Push the initial events.
    res.push_str(
      &quote::quote! {
        let tb_cycles = vec![#(#cycles, )*];
        for cycle in tb_cycles {
          q.push(Reverse(Event{stamp: cycle * 100, kind: EventKind::ModuleTestbench}));
        }
      }
      .to_string(),
    );
  }

  // memory storage element
  for module in sys.module_iter() {
    if let Some(param) = parse_memory_module_name(&module.get_name().to_string()) {
      res.push_str(
        format!(
          "let mut {}_mem = vec![0i32; {}];\n",
          param.name, param.depth
        )
        .as_str(),
      );
    }
  }

  // generate cycle gatekeeper
  for module in sys.module_iter() {
    let module_gatekeeper = syn::Ident::new(
      &format!("{}_triggered", namify(module.get_name())),
      Span::call_site(),
    );
    res.push_str(
      &quote::quote! {
        let mut #module_gatekeeper = None;
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
    let module_eventkind = &format!("Module{}", camelize(&namify(module.get_name())));
    res.push_str(&format!("      EventKind::{} => {{\n", module_eventkind));
    let module_gatekeeper = syn::Ident::new(
      &format!("{}_triggered", namify(module.get_name())),
      Span::call_site(),
    );
    let module_eventkind_id = syn::Ident::new(&module_eventkind, Span::call_site());
    res.push_str(
      &quote::quote! {
        if #module_gatekeeper.map_or(false, |v| v == event.0.stamp) {
          // retry at next cycle
          q.push(Reverse(Event {
            stamp: event.0.stamp + 100,
            kind: EventKind::#module_eventkind_id,
          }));
          continue;
        }
        #module_gatekeeper = event.0.stamp.into();
      }
      .to_string(),
    );
    // Unpacking the FIFO's from the slab.
    for fifo in module.port_iter() {
      let id = fifo_name!(fifo);
      let slab_idx = *slab_cache.get(&fifo.upcast()).unwrap();
      res.push_str(&format!(
        "let {} = data_slab[{}].unwrap_payload();",
        id, slab_idx,
      ));
    }
    let ext_interf_args = module
      .ext_interf_iter()
      .filter(|(v, ops)| {
        v.get_kind() == NodeKind::Array && user_contains_opcode(module.sys, ops, vec![Opcode::Load])
      })
      .map(|(elem, _)| {
        let id = dump_ref!(sys, elem);
        let slab_idx = *slab_cache.get(&elem).unwrap();
        res.push_str(&format!(
          "let {} = data_slab[{}].unwrap_payload();",
          id, slab_idx,
        ));
        id
      })
      .collect::<Vec<_>>();
    // Dump the function call.
    let callee = namify(module.get_name());
    res.push_str(&format!("{}(event.0.stamp, &mut q,", callee,));
    for fifo in module.port_iter() {
      res.push_str(&format!("{},", fifo_name!(fifo)));
    }
    for elem in ext_interf_args {
      res.push_str(&elem);
      res.push(',');
    }
    if let Some(param) = parse_memory_module_name(&module.get_name().to_string()) {
      res.push_str(format!("&mut {}_mem", param.name).as_str());
    }
    res.push_str(");\n");
    if !module.get_name().eq("driver") {
      res.push_str("idled = 0; stamp = event.0.stamp; }\n");
    } else {
      res.push_str("idled += 1; stamp = event.0.stamp; }\n");
    }
  }
  for aty in array_types.iter() {
    let (scalar_ty, size) = unwrap_array_ty(aty);
    let aid = array_ty_to_id(&scalar_ty, size);
    let array_write = syn::Ident::new(&format!("Array{}Write", aid), Span::call_site());
    let scalar_ty = dtype_to_rust_type(&scalar_ty)
      .parse::<proc_macro2::TokenStream>()
      .unwrap();
    res.push_str(
      &quote::quote! {
        EventKind::#array_write((_, slab_idx, idx, value)) => {
          data_slab[slab_idx].last_written.ok(event.0.kind.into(), event.0.stamp);
          data_slab[slab_idx].unwrap_payload_mut::<[#scalar_ty; #size]>()[idx] = value;
          stamp = event.0.stamp;
        }
      }
      .to_string(),
    );
  }
  for fifo_scalar_ty in fifo_types.iter() {
    let ty = dtype_to_rust_type(fifo_scalar_ty);
    let fifo_push_event = syn::Ident::new(&format!("FIFO{}Push", ty), Span::call_site());
    let fifo_pop_event = syn::Ident::new(&format!("FIFO{}Pop", ty), Span::call_site());
    let ty = dtype_to_rust_type(fifo_scalar_ty)
      .parse::<proc_macro2::TokenStream>()
      .unwrap();
    res.push_str(
      &quote::quote! {
        EventKind::#fifo_push_event((_, slab_idx, value)) => {
          data_slab[slab_idx].last_written.ok(event.0.kind.into(), event.0.stamp);
          data_slab[slab_idx].unwrap_payload_mut::<VecDeque<#ty>>().push_back(value);
          stamp = event.0.stamp;
        }
        EventKind::#fifo_pop_event((_, slab_idx)) => {
          data_slab[slab_idx].last_written.ok(event.0.kind.into(), event.0.stamp);
          data_slab[slab_idx].unwrap_payload_mut::<VecDeque<#ty>>().pop_front();
          stamp = event.0.stamp;
        }
      }
      .to_string(),
    );
  }
  res.push_str("EventKind::None => panic!(\"Unexpected event kind, None\"),\n");
  res.push_str("}\n");
  let threshold = config.idle_threshold;
  res.push_str(
    &quote::quote! {
      if idled > #threshold {
        println!("Idled more than {} cycles, exit @{}!", #threshold, cyclize(stamp));
        break;
      }
    }
    .to_string(),
  );
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
    use std::cmp::{Ord, Reverse};
  };
  fd.write(src.to_string().as_bytes())?;
  fd.write("\n\n\n".as_bytes())
}

pub fn elaborate_impl(sys: &SysBuilder, config: &Config) -> Result<String, std::io::Error> {
  let fname = config.fname(sys, "rs");
  println!("Writing simulator code to {}", fname);
  let mut fd = fs::File::create(fname.clone())?;
  dump_header(&mut fd)?;
  let (rt_src, ri) = dump_runtime(sys, config);
  dump_module(sys, &mut fd, &ri)?;
  fd.write(rt_src.as_bytes())?;
  fd.flush()?;
  Ok(fname)
}

pub fn elaborate(sys: &SysBuilder, config: &Config) -> Result<String, std::io::Error> {
  let fname = elaborate_impl(sys, config)?;
  let output = Command::new("rustfmt")
    .arg(&fname)
    .arg("--config")
    .arg("max_width=100,tab_spaces=2")
    .output()
    .expect("Failed to format");
  assert!(output.status.success());
  Ok(fname)
}
