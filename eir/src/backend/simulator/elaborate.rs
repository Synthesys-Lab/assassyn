use std::{
  collections::HashMap,
  fs::{self, File, OpenOptions},
  io::Write,
  path::Path,
  process::Command,
};

use proc_macro2::Span;
use quote::quote;
use syn::Ident;

use crate::{
  backend::common::Config,
  builder::system::SysBuilder,
  ir::{expr::subcode, instructions::Bind, node::*, visitor::Visitor, *},
};

use super::utils::{
  array_ty_to_id, camelize, dtype_to_rust_type, namify, unwrap_array_ty, user_contains_opcode,
};

use self::{
  expr::subcode::Cast, instructions::GetElementPtr, ir_printer::IRPrinter,
  module::memory::parse_memory_module_name,
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

fn int_imm_dumper_impl(ty: &DataType, value: u64) -> String {
  if ty.get_bits() == 1 {
    return if value == 0 {
      "false".to_string()
    } else {
      "true".to_string()
    };
  }
  if ty.get_bits() <= 64 {
    format!("{}{}", value, dtype_to_rust_type(ty))
  } else {
    let scalar_ty = if ty.is_signed() { "u64" } else { "i64" };
    format!(
      "ValueCastTo::<{}>::cast(&({} as {}))",
      dtype_to_rust_type(ty),
      value,
      scalar_ty
    )
  }
}

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
        Some(int_imm_dumper_impl(&int_imm.dtype(), int_imm.get_value()))
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
      _ => Some(format!("{}", namify(node.to_string(sys).as_str()))),
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
    res.push_str(&format!("pub fn {}(\n", namify(module.get_name())));
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
    if let Some(params) = parse_memory_module_name(&self.module_name) {
      res.push_str(
        format!(
          "mem: &mut Vec<{}>,",
          dtype_to_rust_type(&DataType::Bits(params.width))
        )
        .as_str(),
      );
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
              .as_expr::<Bind>(self.sys)
              .unwrap();
            let module = bind.get_callee().as_ref::<Module>(self.sys).unwrap();
            let to_trigger = format!("EventKind::Module{}", camelize(&namify(module.get_name())));
            rdata_module = Some(format!(
              "q.push(Reverse(Event{{ stamp: stamp + read_latency * 100, kind: {} }}))",
              to_trigger
            ));
          }
          Opcode::Bind => { /* don't care, processed in corresponding AsyncCall` */ }
          _ => panic!("Unexpected expr of {:?} in memory body", expr.get_opcode()),
        }
      }

      let fifos = module.port_iter().collect::<Vec<_>>();

      let module_writer = self.current_module_id();

      let addr_fifo = &fifos[0];
      let addr_fifo_idx = *self.slab_cache.get(&addr_fifo.upcast()).unwrap();
      let addr_fifo_ty = addr_fifo.scalar_ty();
      let addr_fifo_pop = syn::Ident::new(
        &format!("FIFO{}Pop", dtype_to_rust_type(&addr_fifo_ty)),
        Span::call_site(),
      );
      let addr_fifo_name = syn::Ident::new(&fifo_name!(addr_fifo), Span::call_site());
      res.push_str(
        quote::quote! {
          let addr = {
          q.push(Reverse(Event{
            stamp: stamp + 50,
            kind: EventKind::#addr_fifo_pop((EventKind::#module_writer.into(), #addr_fifo_idx))
          }));
          #addr_fifo_name.front().unwrap().clone()
        };}
        .to_string()
        .as_str(),
      );

      let write_fifo = &fifos[1];
      let write_fifo_idx = *self.slab_cache.get(&write_fifo.upcast()).unwrap();
      let write_fifo_ty = write_fifo.scalar_ty();
      let write_fifo_pop = syn::Ident::new(
        &format!("FIFO{}Pop", dtype_to_rust_type(&write_fifo_ty)),
        Span::call_site(),
      );
      let write_fifo_name = syn::Ident::new(&fifo_name!(write_fifo), Span::call_site());
      res.push_str(
        quote::quote! {
          let write = {
          q.push(Reverse(Event{
            stamp: stamp + 50,
            kind: EventKind::#write_fifo_pop((EventKind::#module_writer.into(), #write_fifo_idx))
          }));
          #write_fifo_name.front().unwrap().clone()
        };}
        .to_string()
        .as_str(),
      );

      let wdata_fifo = &fifos[2];
      let wdata_fifo_idx = *self.slab_cache.get(&wdata_fifo.upcast()).unwrap();
      let wdata_fifo_ty = wdata_fifo.scalar_ty();
      let wdata_fifo_pop = syn::Ident::new(
        &format!("FIFO{}Pop", dtype_to_rust_type(&wdata_fifo_ty)),
        Span::call_site(),
      );
      let wdata_fifo_name = syn::Ident::new(&fifo_name!(wdata_fifo), Span::call_site());
      res.push_str(
        quote::quote! {
          let wdata = {
          q.push(Reverse(Event{
            stamp: stamp + 50,
            kind: EventKind::#wdata_fifo_pop((EventKind::#module_writer.into(), #wdata_fifo_idx))
          }));
          #wdata_fifo_name.front().unwrap().clone()
        };}
        .to_string()
        .as_str(),
      );

      res.push_str("let rdata = mem[addr as usize];\n");

      // TODO: truely randomize latency, requires emitting cargo wrapped project
      res.push_str(format!("let read_latency = {};\n", params.lat_min).as_str());

      res.push_str("if write {");
      res.push_str("mem[addr as usize] = wdata;");
      res.push_str("} else {");
      res.push_str(rdata_fifo.unwrap().as_str());
      res.push_str(rdata_module.unwrap().as_str());
      res.push_str("}");
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
        "ValueCastTo::<{}>::cast(&{}) {} ValueCastTo::<{}>::cast(&{})",
        ty,
        dump_ref!(self.sys, &expr.get_operand(0).unwrap().get_value()),
        expr.get_opcode().to_string(),
        ty,
        dump_ref!(self.sys, &expr.get_operand(1).unwrap().get_value()),
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
          let (array, idx) = {
            let gep = expr
              .get_operand(0)
              .unwrap()
              .get_value()
              .as_expr::<GetElementPtr>(expr.sys)
              .unwrap();
            (gep.get_array(), gep.get_index())
          };
          format!(
            "{}[{} as usize].clone()",
            namify(array.get_name()),
            NodeRefDumper.dispatch(expr.sys, &idx, vec![]).unwrap()
          )
        }
        Opcode::Store => {
          let (array, idx) = {
            let gep = expr
              .get_operand(0)
              .unwrap()
              .get_value()
              .as_expr::<GetElementPtr>(expr.sys)
              .unwrap();
            (gep.get_array(), gep.get_index())
          };
          let slab_idx = *self.slab_cache.get(&array.upcast()).unwrap();
          let idx = dump_ref!(expr.sys, &idx);
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
        Opcode::GetElementPtr => {
          format!(
            "\n// To be generated in its load/store user\n// GEP: {}\n",
            IRPrinter::new(false).visit_expr(&expr).unwrap()
          )
        }
        Opcode::AsyncCall => {
          let to_trigger = if let Ok(module) = {
            let bind = expr
              .get_operand(0)
              .unwrap()
              .get_value()
              .as_expr::<Bind>(self.sys)
              .unwrap();
            bind.get_callee().as_ref::<Module>(self.sys)
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
        Opcode::FIFOField { field } => {
          let fifo = expr
            .get_operand(0)
            .unwrap()
            .get_value()
            .as_ref::<FIFO>(self.sys)
            .unwrap();
          match field {
            subcode::FIFO::Peek => {
              format!("{}.front().unwrap().clone()", fifo_name!(fifo))
            }
            subcode::FIFO::Valid => {
              format!("!{}.is_empty()", fifo_name!(fifo))
            }
            _ => {
              panic!("Unsupported FIFO field: {:?}", field);
            }
          }
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
                kind: EventKind::#fifo_push(
                  (EventKind::#module_writer.into(), #slab_idx, #value.clone()))
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
          let l = expr
            .get_operand(1)
            .unwrap()
            .get_value()
            .as_ref::<IntImm>(self.sys)
            .expect("Only const slice supported")
            .get_value();
          let r = expr
            .get_operand(2)
            .unwrap()
            .get_value()
            .as_ref::<IntImm>(self.sys)
            .expect("Only const slice supported")
            .get_value();
          format!(
            "{{
              let a = ValueCastTo::<BigUint>::cast(&({} as u64));
              let mask = BigUint::parse_bytes(\"{}\".as_bytes(), 2).unwrap();
              let res = (a >> {}) & mask;
              ValueCastTo::<{}>::cast(&res)
            }}",
            a,
            "1".repeat((r - l + 1) as usize),
            l,
            dtype_to_rust_type(&expr.dtype()),
          )
        }
        Opcode::Concat => {
          let a = dump_ref!(self.sys, &expr.get_operand(0).unwrap().get_value());
          let b = dump_ref!(self.sys, &expr.get_operand(1).unwrap().get_value());
          let b_bits = expr
            .get_operand(1)
            .unwrap()
            .get_value()
            .get_dtype(expr.sys)
            .unwrap()
            .get_bits();
          format! {
            "{{
              let a = ValueCastTo::<BigUint>::cast(&{});
              let b = ValueCastTo::<BigUint>::cast(&{});
              let c = (a << {}) | b;
              ValueCastTo::<{}>::cast(&c)
            }}",
            a,
            b,
            b_bits,
            dtype_to_rust_type(&expr.dtype()),
          }
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
        Opcode::Cast { cast } => {
          let src_ref = expr.get_operand(0).unwrap();
          let src = src_ref.get_value();
          let src_dtype = src.get_dtype(expr.sys).unwrap();
          let dest_dtype = expr.dtype();
          let a = dump_ref!(self.sys, src);
          match cast {
            Cast::ZExt => {
              // perform zero extension
              format!(
                "ValueCastTo::<{}>::cast(&ValueCastTo::<{}>::cast(&{}))",
                dtype_to_rust_type(&dest_dtype),
                dtype_to_rust_type(&src_dtype).replace("i", "u"),
                a,
              )
            }
            Cast::Cast => {
              format!(
                "ValueCastTo::<{}>::cast(&{})",
                dtype_to_rust_type(&dest_dtype),
                a
              )
            }
            Cast::SExt => {
              format!(
                "ValueCastTo::<{}>::cast(&{})",
                dtype_to_rust_type(&dest_dtype),
                a
              )
            }
          }
          // if src_dtype.is_int()
          //   && src_dtype.is_signed()
          //   && dest_dtype.is_int()
          //   && !dest_dtype.is_signed()
          // {
          // } else {
          // }
        }
        Opcode::Bind => {
          let callee = {
            let bind = expr.upcast().as_expr::<Bind>(expr.sys).unwrap();
            let callee = bind.get_callee();
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
        "{}let {} = {};\n",
        " ".repeat(self.indent),
        namify(expr.upcast().to_string(self.sys).as_str()),
        res
      )
    }
    .into()
  }

  fn visit_int_imm(&mut self, int_imm: &IntImmRef<'_>) -> Option<String> {
    format!(
      "ValueCastTo::<{}>::cast(&{})",
      dtype_to_rust_type(&int_imm.dtype()),
      int_imm.get_value(),
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
        let value = {
          let cond = cond.as_ref::<Block>(block.sys).unwrap();
          let value = cond.get_value().unwrap().clone();
          value
        };
        let cond_block = self.dispatch(block.sys, &cond, vec![]).unwrap();
        let cond_block = cond_block[1..cond_block.len() - 1].to_string();
        res.push_str(&format!(
          "{}  if {} {{\n",
          cond_block,
          dump_ref!(self.sys, &value)
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
      BlockKind::Valued(_) | BlockKind::None => {
        res.push('}');
      }
    }
    res.into()
  }
}

fn dump_runtime(sys: &SysBuilder, config: &Config) -> (String, HashMap<BaseNode, usize>) {
  let mut res = String::new();
  res.push_str(
    &quote! {
      use std::collections::VecDeque;
      use std::collections::BinaryHeap;
      use std::cmp::{Ord, Reverse};
      use num_bigint::{BigInt, BigUint, ToBigInt, ToBigUint};
    }
    .to_string(),
  );

  for module in sys.module_iter() {
    res.push_str(&format!(
      "use super::modules::{};\n",
      namify(module.get_name())
    ));
  }

  res.push_str(
    &quote::quote! {
      pub fn cyclize(stamp: usize) -> String {
        format!("Cycle @{}.{:02}", stamp / 100, stamp % 100)
      }
      pub trait ValueCastTo<T> {
        fn cast(&self) -> T;
      }
    }
    .to_string(),
  );
  res.push_str("impl ValueCastTo<bool> for bool { fn cast(&self) -> bool { self.clone() } }\n");

  let bigints = ["BigInt", "BigUint"];
  for i in 0..2 {
    let bigint = bigints[i];
    let other = bigints[1 - i];
    res.push_str(&format!(
      "impl ValueCastTo<{}> for {} {{ fn cast(&self) -> {} {{ self.clone() }} }}\n",
      bigint, bigint, bigint
    ));
    res.push_str(&format!(
      "impl ValueCastTo<{}> for {} {{ fn cast(&self) -> {} {{ self.to_{}().unwrap() }} }}\n",
      other,
      bigint,
      other,
      other.to_lowercase()
    ));
    res.push_str(&format!(
      "impl ValueCastTo<{}> for bool {{ fn cast(&self) -> {} {{
        if *self {{ 1.to_{}().unwrap() }} else {{ 0.to_{}().unwrap() }}
      }} }}\n",
      bigint,
      bigint,
      bigint.to_lowercase(),
      bigint.to_lowercase()
    ));
    res.push_str(&format!(
      "impl ValueCastTo<bool> for {} {{ fn cast(&self) -> bool {{
        !self.eq(&0.to_{}().unwrap())
      }} }}\n",
      bigint,
      bigint.to_lowercase()
    ));
  }

  // Dump a template based data cast so that big integers are unified in.
  for sign_i in 0..=1 {
    for i in 3..7 {
      let src_ty = format!("{}{}", ['u', 'i'][sign_i], 1 << i);
      res.push_str(&format!(
        "impl ValueCastTo<bool> for {} {{ fn cast(&self) -> bool {{ *self != 0 }} }}\n",
        src_ty
      ));
      res.push_str(&format!(
        "impl ValueCastTo<{}> for bool {{
            fn cast(&self) -> {} {{ if *self {{ 1 }} else {{ 0 }} }}
          }}\n",
        src_ty, src_ty
      ));
      for idx in 0..2 {
        let bigint = bigints[idx];
        res.push_str(&format!(
          "impl ValueCastTo<{}> for {} {{ fn cast(&self) -> {} {{ self.to_{}().unwrap() }} }}\n",
          bigint,
          src_ty,
          bigint,
          bigint.to_lowercase()
        ));
      }
      res.push_str(&format!(
        "impl ValueCastTo<{}> for BigInt {{
            fn cast(&self) -> {} {{
              let (sign, data) = self.to_u64_digits();
              if data.is_empty() {{
                return 0;
              }}
              match sign {{
                num_bigint::Sign::Plus => data[0] as {},
                num_bigint::Sign::Minus => ((!data[0] + 1) & ({}::MAX as u64)) as {},
                num_bigint::Sign::NoSign => data[0] as {},
              }}
            }}
          }}\n",
        src_ty, src_ty, src_ty, src_ty, src_ty, src_ty
      ));
      res.push_str(&format!(
        "impl ValueCastTo<{}> for BigUint {{
            fn cast(&self) -> {} {{
              let data = self.to_u64_digits();
              if data.is_empty() {{
                return 0;
              }} else {{
                return data[0] as {};
              }}
            }}
          }}\n",
        src_ty, src_ty, src_ty
      ));

      for sign_j in 0..=1 {
        for j in 3..7 {
          let dst_ty = format!("{}{}", ['u', 'i'][sign_j], 1 << j);
          if i == j && sign_i == sign_j {
            res.push_str(&format!(
              "impl ValueCastTo<{}> for {} {{ fn cast(&self) -> {} {{ self.clone() }} }}\n",
              dst_ty, src_ty, dst_ty
            ));
          } else {
            res.push_str(&format!(
              "impl ValueCastTo<{}> for {} {{ fn cast(&self) -> {} {{ *self as {} }} }}\n",
              dst_ty, src_ty, dst_ty, dst_ty
            ));
          }
        }
      }
    }
  }
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
  res.push_str("pub enum EventKind {\n");
  for module in sys.module_iter() {
    res.push_str(&format!(
      "  Module{},\n",
      camelize(&namify(module.get_name()))
    ));
  }
  let array_types = analysis::types::array_types_used(sys);
  for (_, dtypes) in array_types.iter() {
    let aty = dtypes.iter().next().unwrap();
    let (scalar_ty, size) = unwrap_array_ty(aty);
    let scalar_str = dtype_to_rust_type(&scalar_ty);
    let array_str = array_ty_to_id(&scalar_ty, size);
    res.push_str(&format!(
      "  Array{}Write((Box<EventKind>, usize, usize, {})),\n",
      array_str, scalar_str
    ));
  }
  let fifo_types = analysis::types::fifo_types_used(sys);
  for (_, dtypes) in fifo_types.iter() {
    let fty = dtypes.iter().next().unwrap();
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
  for (_, dtypes) in fifo_types.iter() {
    let fty = dtypes.iter().next().unwrap();
    let ty = dtype_to_rust_type(&fty);
    res.push_str(&format!("  EventKind::FIFO{}Push(_) => true,\n", ty,));
  }
  res.push_str("_ => false, }}\n\n");
  res.push_str("fn is_pop(&self) -> bool { match self {\n");
  for (_, dtypes) in fifo_types.iter() {
    let fty = dtypes.iter().next().unwrap();
    let ty = dtype_to_rust_type(&fty);
    res.push_str(&format!("  EventKind::FIFO{}Pop(_) => true,\n", ty,));
  }
  res.push_str("_ => false, }}\n");
  res.push('}');

  // Dump the universal set of data types used in this simulation.
  res.push_str("pub enum DataSlab {");
  for (_, dtypes) in array_types.iter() {
    let array = dtypes.iter().next().unwrap();
    let (scalar_ty, size) = unwrap_array_ty(array);
    res.push_str(&format!(
      "  Array{}(Box<{}>),\n",
      array_ty_to_id(&scalar_ty, size),
      dtype_to_rust_type(&array),
    ));
  }
  for (_, dtypes) in fifo_types.iter() {
    let fifo = dtypes.iter().next().unwrap();
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
      pub struct LastOperation {
        pub operation: Box<EventKind>,
        pub stamp: usize,
      }
      pub struct SlabEntry {
        pub payload: DataSlab,
        pub last_written: LastOperation,
      }
      impl LastOperation {
        fn update(&mut self, operation: Box<EventKind>, stamp: usize) {
          self.operation = operation;
          self.stamp = stamp;
        }
        pub fn ok(&mut self, operation: Box<EventKind>, stamp: usize) {
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
      pub trait UnwrapSlab {
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
      pub struct Event {
        pub stamp: usize,
        pub kind: EventKind,
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

  for (_, dtypes) in array_types.iter() {
    let array = dtypes.iter().next().unwrap();
    let (scalar_ty, size) = unwrap_array_ty(array);
    let aid = array_ty_to_id(&scalar_ty, size);
    let scalar_ty = dtype_to_rust_type(&scalar_ty);
    res.push_str(&format!(
      "impl_unwrap_slab!([{}; {}], Array{});\n",
      scalar_ty, size, aid
    ));
  }

  for (_, dtypes) in fifo_types.iter() {
    let fifo = dtypes.iter().next().unwrap();
    let ty = dtype_to_rust_type(fifo);
    res.push_str(&format!(
      "impl_unwrap_slab!(VecDeque<{}>, FIFO{});\n",
      ty, ty
    ));
  }

  // TODO(@were): Make all arguments of the modules FIFO channels.
  // TODO(@were): Profile the maxium size of all the FIFO channels.
  res.push_str("pub fn simulate() {\n");
  res.push_str("  // The global time stamp\n");
  res.push_str("  let mut stamp: usize = 0;\n");
  res.push_str("  // Count the consecutive cycles idled\n");
  res.push_str("  let mut idled: usize = 0;\n");
  res.push_str("  let mut data_slab : Vec<SlabEntry> = vec![\n");
  res.push_str("  // Define global arrays\n");
  let mut slab_cache: HashMap<BaseNode, usize> = HashMap::new();
  for array in sys.array_iter() {
    let (scalar_ty, size) = unwrap_array_ty(&array.dtype());
    let aty_id = Ident::new(
      &format!("Array{}", array_ty_to_id(&scalar_ty, size)),
      Span::call_site(),
    );
    let initializer = if let Some(init) = array.get_initializer() {
      let list = init
        .iter()
        .map(|x| dump_ref!(sys, x))
        .collect::<Vec<String>>()
        .join(", ");
      format!("[{}]", list)
    } else {
      format!("[{}; {}]", int_imm_dumper_impl(&scalar_ty, 0), size)
    };
    let initializer = initializer.parse::<proc_macro2::TokenStream>().unwrap();
    res.push_str(&format!(
      "  // {} -> {}\n",
      slab_cache.len(),
      IRPrinter::new(false).visit_array(&array).unwrap(),
    ));
    res.push_str(
      &quote::quote! {
        SlabEntry {
          payload: DataSlab::#aty_id(Box::new(#initializer)),
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
          "let mut {}_mem = vec![0 as {}; {}];\n",
          param.name,
          dtype_to_rust_type(&DataType::Bits(param.width)),
          param.depth
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
  res.push_str("    match &event.0.kind {\n");
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
  for (_, dtypes) in array_types.iter() {
    let aty = dtypes.iter().next().unwrap();
    let (scalar_ty, size) = unwrap_array_ty(aty);
    let aid = array_ty_to_id(&scalar_ty, size);
    let array_write = syn::Ident::new(&format!("Array{}Write", aid), Span::call_site());
    let scalar_ty = dtype_to_rust_type(&scalar_ty)
      .parse::<proc_macro2::TokenStream>()
      .unwrap();
    res.push_str(
      &quote::quote! {
        EventKind::#array_write((_, slab_idx, idx, value)) => {
          let slab_idx = *slab_idx;
          let idx = *idx;
          data_slab[slab_idx].last_written.ok(event.0.kind.into(), event.0.stamp);
          let value = match data_slab[slab_idx].last_written.operation.as_ref() {
            EventKind::#array_write((_, _, _, value)) => value.clone(),
            _ => panic!("Invalid last written operation"),
          };
          data_slab[slab_idx].unwrap_payload_mut::<[#scalar_ty; #size]>()[idx] = value;
          stamp = event.0.stamp;
        }
      }
      .to_string(),
    );
  }
  for (_, dtypes) in fifo_types.iter() {
    let fifo_scalar_ty = dtypes.iter().next().unwrap();
    let ty = dtype_to_rust_type(fifo_scalar_ty);
    let fifo_push_event = syn::Ident::new(&format!("FIFO{}Push", ty), Span::call_site());
    let fifo_pop_event = syn::Ident::new(&format!("FIFO{}Pop", ty), Span::call_site());
    let ty = dtype_to_rust_type(fifo_scalar_ty)
      .parse::<proc_macro2::TokenStream>()
      .unwrap();
    res.push_str(
      &quote::quote! {
        EventKind::#fifo_push_event((_, slab_idx, value)) => {
          let value = value.clone();
          data_slab[*slab_idx].last_written.ok(event.0.kind.clone().into(), event.0.stamp);
          data_slab[*slab_idx].unwrap_payload_mut::<VecDeque<#ty>>().push_back(value);
          stamp = event.0.stamp;
        }
        EventKind::#fifo_pop_event((_, slab_idx)) => {
          data_slab[*slab_idx].last_written.ok(event.0.kind.clone().into(), event.0.stamp);
          data_slab[*slab_idx].unwrap_payload_mut::<VecDeque<#ty>>().pop_front();
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

fn dump_modules(
  sys: &SysBuilder,
  fd: &mut File,
  slab_cache: &HashMap<BaseNode, usize>,
) -> Result<(), std::io::Error> {
  fd.write(
    &quote! {
      use super::runtime::*;
      use std::collections::VecDeque;
      use std::collections::BinaryHeap;
      use std::cmp::Reverse;
      use num_bigint::{BigInt, BigUint};
    }
    .to_string()
    .as_bytes(),
  )?;
  let mut em = ElaborateModule::new(sys, slab_cache);
  for module in em.sys.module_iter() {
    if let Some(buffer) = em.visit_module(&module) {
      fd.write(buffer.as_bytes())?;
    }
  }
  Ok(())
}

fn dump_main(fd: &mut File) -> Result<usize, std::io::Error> {
  let src = quote::quote! {
    mod runtime;
    mod modules;

    fn main() {
      runtime::simulate();
    }
  };
  fd.write(src.to_string().as_bytes())?;
  fd.write("\n\n\n".as_bytes())
}

fn elaborate_impl(sys: &SysBuilder, config: &Config) -> Result<String, std::io::Error> {
  let dir_name = config.dir_name(sys);
  if Path::new(&dir_name).exists() {
    if config.override_dump {
      fs::remove_dir_all(&dir_name)?;
      fs::create_dir_all(&dir_name)?;
    } else {
      eprintln!(
        "Directory {} already exists, may possibly lead to dump failure.",
        dir_name
      );
    }
  } else {
    fs::create_dir_all(&dir_name)?;
  }
  eprintln!("Writing simulator code to rust project: {}", dir_name);
  let output = Command::new("cargo")
    .arg("init")
    .arg(&dir_name)
    .output()
    .expect("Failed to init cargo project");
  assert!(output.status.success());
  // Dump the Cargo.toml and rustfmt.toml
  {
    let mut cargo = OpenOptions::new()
      .write(true)
      .append(true)
      .open(format!("{}/Cargo.toml", dir_name))?;
    writeln!(cargo, "num-bigint = \"0.4\"")?;
    let mut fmt = fs::File::create(format!("{}/rustfmt.toml", dir_name))?;
    writeln!(fmt, "max_width = 100")?;
    writeln!(fmt, "tab_spaces = 2")?;
    fmt.flush()?;
  }
  // eprintln!("Writing simulator source to file: {}", fname);
  let fname = format!("{}/src/main.rs", dir_name);
  let (rt_src, ri) = dump_runtime(sys, config);
  {
    let modules_file = format!("{}/src/modules.rs", dir_name);
    let mut fd = fs::File::create(modules_file).expect("Open failure");
    dump_modules(sys, &mut fd, &ri).expect("Dump module failure");
    fd.flush().expect("Flush modules failure");
  }
  {
    let runtime_file = format!("{}/src/runtime.rs", dir_name);
    let mut fruntime = fs::File::create(runtime_file).expect("Open failure");
    fruntime
      .write(rt_src.as_bytes())
      .expect("Dump runtime failure");
    fruntime.flush().expect("Flush runtime failure");
  }
  {
    let mut fd = fs::File::create(fname.clone()).expect("Open failure");
    dump_main(&mut fd).expect("Dump head failure");
    fd.flush().expect("Flush main failure");
  }
  Ok(fname)
}

pub fn elaborate(sys: &SysBuilder, config: &Config) -> Result<String, std::io::Error> {
  let fname = elaborate_impl(sys, config)?;
  let output = Command::new("cargo")
    .arg("fmt")
    .arg("--manifest-path")
    .arg(&format!("{}/Cargo.toml", config.dir_name(sys)))
    .output()
    .expect("Failed to format");
  assert!(output.status.success(), "Failed to format: {:?}", output);
  Ok(fname)
}
