use std::{
  collections::HashMap,
  fs::{self, File, OpenOptions},
  io::Write,
  path::{Path, PathBuf},
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

use self::{expr::subcode::Cast, instructions, ir_printer::IRPrinter, module::Attribute};

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
  fn visit_module(&mut self, module: ModuleRef<'_>) -> Option<String> {
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
    res.push_str(") {\n");
    self.indent += 2;

    res.push_str(&self.visit_block(module.get_body()).unwrap());

    self.indent -= 2;
    res.push_str("}\n");
    res.into()
  }

  fn visit_expr(&mut self, expr: ExprRef<'_>) -> Option<String> {
    let id = if expr.get_opcode().is_valued() {
      Some(namify(expr.upcast().to_string(self.sys).as_str()))
    } else {
      None
    };
    let mut open_scope = false;
    let res = match expr.get_opcode() {
      Opcode::Binary { .. } => {
        let bin = expr.as_sub::<instructions::Binary>().unwrap();
        let ty = bin.get().dtype();
        let ty = dtype_to_rust_type(&ty);
        let lhs = format!(
          "ValueCastTo::<{}>::cast(&{})",
          ty,
          dump_ref!(self.sys, &bin.a())
        );
        let rhs = format!(
          "ValueCastTo::<{}>::cast(&{})",
          ty,
          dump_ref!(self.sys, &bin.b())
        );
        format!("{} {} {}", lhs, bin.get_opcode().to_string(), rhs)
      }
      Opcode::Unary { .. } => {
        let uop = expr.as_sub::<instructions::Unary>().unwrap();
        format!(
          "{}{}",
          uop.get_opcode().to_string(),
          dump_ref!(self.sys, &uop.x())
        )
      }
      Opcode::Compare { .. } => {
        let cmp = expr.as_sub::<instructions::Compare>().unwrap();
        format!(
          "{} {} {}",
          dump_ref!(self.sys, &cmp.a()),
          cmp.get_opcode().to_string(),
          dump_ref!(self.sys, &cmp.b()),
        )
      }
      Opcode::Load => {
        let load = expr.as_sub::<instructions::Load>().unwrap();
        let (array, idx) = (load.array(), load.idx());
        format!(
          "{}[{} as usize].clone()",
          namify(array.get_name()),
          NodeRefDumper
            .dispatch(load.get().sys, &idx, vec![])
            .unwrap()
        )
      }
      Opcode::Store => {
        let store = expr.as_sub::<instructions::Store>().unwrap();
        let (array, idx) = (store.array(), store.idx());
        let slab_idx = *self.slab_cache.get(&array.upcast()).unwrap();
        let idx = dump_ref!(store.get().sys, &idx);
        let idx = idx.parse::<proc_macro2::TokenStream>().unwrap();
        let (scalar_ty, size) = unwrap_array_ty(&array.dtype());
        let aid = array_ty_to_id(&scalar_ty, size);
        let id = syn::Ident::new(&format!("Array{}Write", aid), Span::call_site());
        let value = dump_ref!(self.sys, &store.value());
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
        let call = expr.as_sub::<instructions::AsyncCall>().unwrap();
        let bind = call.bind();
        let event_kind = camelize(&namify(bind.callee().get_name()));
        let event_kind = format!("EventKind::Module{}", event_kind);
        format!(
          "q.push(Reverse(Event{{ stamp: stamp + 100, kind: {} }}))",
          event_kind
        )
      }
      Opcode::FIFOPop => {
        // TODO(@were): Support multiple pop.
        let pop = expr.as_sub::<instructions::FIFOPop>().unwrap();
        let fifo = pop.fifo();
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
        let get_field = expr.as_sub::<instructions::FIFOField>().unwrap();
        let fifo = get_field.fifo();
        match get_field.get_field() {
          subcode::FIFO::Peek => format!("{}.front().unwrap().clone()", fifo_name!(fifo)),
          subcode::FIFO::Valid => format!("!{}.is_empty()", fifo_name!(fifo)),
          _ => panic!("Unsupported FIFO field: {:?}", field),
        }
      }
      Opcode::FIFOPush => {
        let push = expr.as_sub::<instructions::FIFOPush>().unwrap();
        let fifo = push.fifo();
        let slab_idx = *self.slab_cache.get(&fifo.upcast()).unwrap();
        let fifo_push = syn::Ident::new(
          &format!("FIFO{}Push", dtype_to_rust_type(&fifo.scalar_ty())),
          Span::call_site(),
        );
        let value = dump_ref!(self.sys, &push.value());
        let value = value.parse::<proc_macro2::TokenStream>().unwrap();
        let module_writer = self.current_module_id();
        quote::quote! {
          q.push(Reverse(Event{
            stamp: stamp + 50,
            kind: EventKind::#fifo_push(
              (EventKind::#module_writer.into(), #slab_idx, #value.clone()))
          }))
        }
        .to_string()
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
        let slice = expr.as_sub::<instructions::Slice>().unwrap();
        let a = dump_ref!(self.sys, &slice.x());
        let l = slice.l();
        let r = slice.r();
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
          dtype_to_rust_type(&slice.get().dtype()),
        )
      }
      Opcode::Concat => {
        let dtype = expr.dtype();
        let concat = expr.as_sub::<instructions::Concat>().unwrap();
        let a = dump_ref!(self.sys, &concat.msb());
        let b = dump_ref!(self.sys, &concat.lsb());
        let b_bits = concat.lsb().get_dtype(concat.get().sys).unwrap().get_bits();
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
          dtype_to_rust_type(&dtype),
        }
      }
      Opcode::Select => {
        let select = expr.as_sub::<instructions::Select>().unwrap();
        let cond = dump_ref!(self.sys, &select.cond());
        let true_value = dump_ref!(self.sys, &select.true_value());
        let false_value = dump_ref!(self.sys, &select.false_value());
        format!(
          "if {} {{ {} }} else {{ {} }}",
          cond, true_value, false_value
        )
      }
      Opcode::Select1Hot => {
        let select1hot = expr.as_sub::<instructions::Select1Hot>().unwrap();
        let cond = select1hot.cond();
        let mut res = format!(
          "{{ let cond = {}; assert!(cond.count_ones() == 1, \"Select1Hot: condition is not 1-hot\");",
          dump_ref!(self.sys, &cond)
        );
        for (i, value) in select1hot.value_iter().enumerate() {
          if i != 0 {
            res.push_str(" else ");
          }
          res.push_str(&format!(
            "if cond >> {} & 1 != 0 {{ {} }}",
            i,
            dump_ref!(self.sys, &value)
          ));
        }
        res.push_str(" else { unreachable!() } }");
        res
      }
      Opcode::Cast { .. } => {
        let cast = expr.as_sub::<instructions::Cast>().unwrap();
        let src_dtype = cast.src_type();
        let dest_dtype = cast.dest_type();
        let a = dump_ref!(cast.get().sys, &cast.x());
        match cast.get_opcode() {
          Cast::ZExt | Cast::BitCast => {
            // perform zero extension
            format!(
              "ValueCastTo::<{}>::cast(&ValueCastTo::<{}>::cast(&{}))",
              dtype_to_rust_type(&dest_dtype),
              dtype_to_rust_type(&src_dtype).replace("i", "u"),
              a,
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
      }
      Opcode::Bind => {
        let bind = expr.as_sub::<Bind>().unwrap();
        let module = bind.callee();
        format!("EventKind::Module{}", camelize(&namify(module.get_name())))
      }
      Opcode::BlockIntrinsic { intrinsic } => {
        let bi = expr.as_sub::<instructions::BlockIntrinsic>().unwrap();
        let value = dump_ref!(self.sys, &bi.value());
        match intrinsic {
          subcode::BlockIntrinsic::Value => value,
          subcode::BlockIntrinsic::Cycled => {
            open_scope = true;
            format!("if stamp / 100 == ({} as usize) {{", value)
          }
          subcode::BlockIntrinsic::WaitUntil => {
            let module_eventkind_id = self.current_module_id().to_string();
            format!(
              "if !{} {{
                q.push(Reverse(Event {{
                  stamp: stamp + 100,
                  kind: EventKind::{},
                }}));
                return;
              }}",
              value, module_eventkind_id,
            )
          }
          subcode::BlockIntrinsic::Condition => {
            open_scope = true;
            format!("if {} {{", value)
          }
        }
      }
    };
    let res = if let Some(id) = id {
      format!("{}let {} = {};\n", " ".repeat(self.indent), id, res)
    } else {
      format!("{}{};\n", " ".repeat(self.indent), res)
    };
    if open_scope {
      self.indent += 2;
    }
    res.into()
  }

  fn visit_int_imm(&mut self, int_imm: IntImmRef<'_>) -> Option<String> {
    format!(
      "ValueCastTo::<{}>::cast(&{})",
      dtype_to_rust_type(&int_imm.dtype()),
      int_imm.get_value(),
    )
    .into()
  }

  fn visit_block(&mut self, block: BlockRef<'_>) -> Option<String> {
    let mut res = String::new();
    // TODO(@were): Later we support sub-types for blocks.
    let restore_indent = self.indent;
    for elem in block.body_iter() {
      match elem.get_kind() {
        NodeKind::Expr => {
          let expr = elem.as_ref::<Expr>(self.sys).unwrap();
          res.push_str(&self.visit_expr(expr).unwrap());
        }
        NodeKind::Block => {
          let block = elem.as_ref::<Block>(self.sys).unwrap();
          res.push_str(&self.visit_block(block).unwrap());
        }
        _ => {
          panic!("Unexpected reference type: {:?}", elem);
        }
      }
    }
    if restore_indent != self.indent {
      self.indent -= 2;
      res.push_str(&format!("{}}}\n", " ".repeat(self.indent)));
    }
    if block.get_value().is_some() {
      res = format!("{{ {} }}", res);
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
      use num_traits::Num;
      use std::fs::read_to_string;
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
      pub fn init_vec_by_hex_file<T: Num, const N: usize>(array: &mut [T; N], init_file: &str) {
        let mut idx = 0;
        for line in read_to_string(init_file)
          .expect("can not open hex file")
          .lines()
        {
          let line = if let Some(to_strip) = line.find("//") {
            line[..to_strip].trim()
          } else {
            line.trim()
          };
          if line.len() == 0 {
            continue;
          }
          let line = line.replace("_", "");
          if line.starts_with("@") {
            let addr = usize::from_str_radix(&line[1..], 16).unwrap();
            idx = addr;
            continue;
          }
          array[idx] = T::from_str_radix(line.as_str(), 16).ok().unwrap();
          idx += 1;
        }
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
      IRPrinter::new(false).visit_array(array.clone()).unwrap(),
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
  if let Some(testbench) = sys.get_module("testbench") {
    let cycles = testbench
      .get_body()
      .body_iter()
      .filter_map(|n| {
        if let Ok(block) = n.as_ref::<Block>(sys) {
          block.get_cycle()
        } else {
          None
        }
      })
      .collect::<Vec<_>>();
    // Push the initial events.
    res.push_str(
      &quote::quote! {
        let tb_cycles = vec![#(#cycles, )*];
        for cycle in tb_cycles {
          q.push(Reverse(Event{stamp: (cycle as usize) * 100, kind: EventKind::ModuleTestbench}));
        }
      }
      .to_string(),
    );
  }

  // generate memory initializations
  for module in sys.module_iter() {
    for attr in module.get_attrs() {
      match attr {
        Attribute::Memory(param) => {
          if let Some(init_file) = &param.init_file {
            let init_file_path = config.resource_base.join(init_file);
            let init_file_path = init_file_path.to_str().unwrap();
            let array = param.array.as_ref::<Array>(sys).unwrap();
            let slab_idx = slab_cache.get(&param.array).unwrap();
            let (scalar_ty, size) = unwrap_array_ty(&array.dtype());
            let scalar_ty = dtype_to_rust_type(&scalar_ty)
              .parse::<proc_macro2::TokenStream>()
              .unwrap();
            res.push_str(
              format!(
                "\n// initializing array {}, slab_idx = {}, with file {}\n",
                array.get_name(),
                slab_idx,
                init_file
              )
              .as_str(),
            );
            res.push_str(
              &quote::quote! {
                init_vec_by_hex_file(
                  data_slab[#slab_idx].unwrap_payload_mut::<[#scalar_ty; #size]>(),
                  #init_file_path
                );
              }
              .to_string(),
            );
          }
        }
        _ => {}
      }
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
    if let Some(buffer) = em.visit_module(module) {
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

fn elaborate_impl(sys: &SysBuilder, config: &Config) -> Result<PathBuf, std::io::Error> {
  let dir_name = config.dir_name(sys);
  let dir = Path::new(&dir_name);
  if !dir.exists() {
    fs::create_dir_all(&dir_name)?;
  }
  assert!(dir.is_dir());
  let files = fs::read_dir(&dir_name)?;
  if config.override_dump {
    for elem in files {
      let path = elem?.path();
      if path.is_dir() {
        fs::remove_dir_all(path)?;
      } else {
        fs::remove_file(path)?;
      }
    }
  } else {
    assert!(files.count() == 0);
  }
  eprintln!(
    "Writing simulator code to rust project: {}",
    dir_name.to_str().unwrap()
  );
  let output = Command::new("cargo")
    .arg("init")
    .arg(&dir_name)
    .output()
    .expect("Failed to init cargo project");
  assert!(output.status.success());
  let manifest = dir_name.join("Cargo.toml");
  // Dump the Cargo.toml and rustfmt.toml
  {
    let mut cargo = OpenOptions::new()
      .write(true)
      .append(true)
      .open(&manifest)?;
    writeln!(cargo, "num-bigint = \"0.4\"")?;
    writeln!(cargo, "num-traits = \"0.2\"")?;
    let mut fmt = fs::File::create(dir_name.join("rustfmt.toml"))?;
    writeln!(fmt, "max_width = 100")?;
    writeln!(fmt, "tab_spaces = 2")?;
    fmt.flush()?;
  }
  // eprintln!("Writing simulator source to file: {}", fname);
  let fname = dir_name.join("src/main.rs");
  let (rt_src, ri) = dump_runtime(sys, config);
  {
    let modules_file = dir_name.join("src/modules.rs");
    let mut fd = fs::File::create(modules_file).expect("Open failure");
    dump_modules(sys, &mut fd, &ri).expect("Dump module failure");
    fd.flush().expect("Flush modules failure");
  }
  {
    let runtime_file = dir_name.join("src/runtime.rs");
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
  Ok(manifest)
}

pub fn elaborate(sys: &SysBuilder, config: &Config) -> Result<PathBuf, std::io::Error> {
  let manifest = elaborate_impl(sys, config)?;
  let output = Command::new("cargo")
    .arg("fmt")
    .arg("--manifest-path")
    .arg(&manifest)
    .output()
    .expect("Failed to format");
  assert!(output.status.success(), "Failed to format: {:?}", output);
  Ok(manifest)
}
