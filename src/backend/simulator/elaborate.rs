use std::{
  collections::HashSet,
  fs::{self, File},
  io::{self, Write},
  path::PathBuf,
  process::Command,
};

use module::Attribute;
use proc_macro2::Span;
use quote::quote;

use crate::{
  backend::common::{create_and_clean_dir, Config},
  builder::system::{ModuleKind, SysBuilder},
  ir::{expr::subcode, node::*, visitor::Visitor, *},
};

use super::utils::{dtype_to_rust_type, namify};

use self::{expr::subcode::Cast, instructions};

struct ElaborateModule<'a> {
  sys: &'a SysBuilder,
  indent: usize,
  module_name: String,
  module_ctx: BaseNode,
}

impl<'a> ElaborateModule<'a> {
  fn new(sys: &'a SysBuilder) -> Self {
    Self {
      sys,
      indent: 0,
      module_name: String::new(),
      module_ctx: BaseNode::unknown(),
    }
  }
}

macro_rules! fifo_name {
  ($fifo:expr) => {{
    let module = $fifo.get_parent().as_ref::<Module>($fifo.sys).unwrap();
    format!("{}_{}", namify(module.get_name()), namify($fifo.get_name()))
  }};
}

macro_rules! dump_ref {
  ($sys:expr, $value:expr) => {
    NodeRefDumper::new(None)
      .dispatch($sys, $value, vec![])
      .unwrap()
  };

  ($ctx: expr, $sys:expr, $value:expr) => {
    NodeRefDumper::new(Some($ctx))
      .dispatch($sys, $value, vec![])
      .unwrap()
  };
}

pub(super) struct NodeRefDumper {
  module_ctx: Option<BaseNode>,
}

impl NodeRefDumper {
  pub fn new(module_ctx: Option<BaseNode>) -> Self {
    Self { module_ctx }
  }
}

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
    format!("ValueCastTo::<{}>::cast(&({} as {}))", dtype_to_rust_type(ty), value, scalar_ty)
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
      NodeKind::Module => Some(namify(node.as_ref::<Module>(sys).unwrap().get_name())),
      NodeKind::Expr => {
        let expr = node.as_ref::<Expr>(sys).unwrap();
        let id = {
          let raw = namify(&expr.get_name());
          if self
            .module_ctx
            .map_or(false, |x| x != expr.get_parent().as_ref::<Block>(sys).unwrap().get_module())
          {
            format!("sim.{}_value.unwrap()", raw)
          } else {
            raw
          }
        };

        id.into()
      }
      _ => Some(namify(node.to_string(sys).as_str()).to_string()),
    }
  }
}

fn needs_validity(expr: &ExprRef<'_>) -> bool {
  let res = expr
    .users()
    .iter()
    .filter_map(|x| {
      x.as_ref::<Operand>(expr.sys)
        .unwrap()
        .get_parent()
        .as_ref::<Expr>(expr.sys)
        .ok()
    })
    .any(|user| {
      if let Opcode::PureIntrinsic { intrinsic } = user.get_opcode() {
        matches!(intrinsic, subcode::PureIntrinsic::ValueValid)
      } else {
        false
      }
    });
  res
}

impl Visitor<String> for ElaborateModule<'_> {
  fn visit_module(&mut self, module: ModuleRef<'_>) -> Option<String> {
    self.module_name = module.get_name().to_string();
    self.module_ctx = module.upcast();
    let mut res = String::new();
    res.push_str(&format!("\n// Elaborating module {}\n", namify(module.get_name())));
    // Dump the function signature.
    // First, some common function parameters are dumped.
    res.push_str(&format!("pub fn {}(sim: &mut Simulator) {{\n", namify(module.get_name())));
    self.indent += 2;

    res.push_str(&self.visit_block(module.get_body()).unwrap());

    self.indent -= 2;
    res.push_str("}\n");
    res.into()
  }

  fn visit_expr(&mut self, expr: ExprRef<'_>) -> Option<String> {
    let (id, expose_validity) = if expr.get_opcode().is_valued() {
      let need = needs_validity(&expr);
      (Some(namify(expr.upcast().to_string(self.sys).as_str())), need)
    } else {
      (None, false)
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
          dump_ref!(self.module_ctx, self.sys, &bin.a())
        );
        let rhs = format!(
          "ValueCastTo::<{}>::cast(&{})",
          ty,
          dump_ref!(self.module_ctx, self.sys, &bin.b())
        );
        format!("{} {} {}", lhs, bin.get_opcode(), rhs)
      }
      Opcode::Unary { .. } => {
        let uop = expr.as_sub::<instructions::Unary>().unwrap();
        format!("{}{}", uop.get_opcode(), dump_ref!(self.module_ctx, self.sys, &uop.x()))
      }
      Opcode::Compare { .. } => {
        let cmp = expr.as_sub::<instructions::Compare>().unwrap();
        format!(
          "{} {} {}",
          dump_ref!(self.module_ctx, self.sys, &cmp.a()),
          cmp.get_opcode(),
          dump_ref!(self.module_ctx, self.sys, &cmp.b()),
        )
      }
      Opcode::Load => {
        let load = expr.as_sub::<instructions::Load>().unwrap();
        let (array, idx) = (load.array(), load.idx());
        format!(
          "sim.{}.payload[{} as usize].clone()",
          namify(array.get_name()),
          NodeRefDumper::new(None)
            .dispatch(load.get().sys, &idx, vec![])
            .unwrap()
        )
      }
      Opcode::Store => {
        let store = expr.as_sub::<instructions::Store>().unwrap();
        let (array, idx) = (store.array(), store.idx());
        let idx = dump_ref!(self.module_ctx, store.get().sys, &idx);
        let idx = idx.parse::<proc_macro2::TokenStream>().unwrap();
        let value = dump_ref!(self.module_ctx, self.sys, &store.value());
        let value = value.parse::<proc_macro2::TokenStream>().unwrap();
        let module_writer = &self.module_name;
        let array = syn::Ident::new(
          &dump_ref!(self.module_ctx, store.get().sys, &array.upcast()),
          Span::call_site(),
        );
        quote::quote! {
          let stamp = sim.stamp;
          sim.#array.write.push(
            ArrayWrite::new(stamp + 50, #idx as usize, #value.clone(), #module_writer.to_string()));
        }
        .to_string()
      }
      Opcode::AsyncCall => {
        let call = expr.as_sub::<instructions::AsyncCall>().unwrap();
        let bind = call.bind();
        let event_q = namify(bind.callee().get_name());
        format!("{{ let stamp = sim.stamp; sim.{}_event.push_back(stamp + 100) }}", event_q)
      }
      Opcode::FIFOPop => {
        let pop = expr.as_sub::<instructions::FIFOPop>().unwrap();
        let fifo = pop.fifo();
        let fifo_id = syn::Ident::new(&fifo_name!(fifo), Span::call_site());
        let module_name = &self.module_name;
        quote::quote! {{
          let stamp = sim.stamp;
          sim.#fifo_id.pop.push(FIFOPop::new(stamp + 50, #module_name.to_string()));
          sim.#fifo_id.payload.front().unwrap().clone()
        }}
        .to_string()
      }
      Opcode::PureIntrinsic { intrinsic } => {
        let call = expr.as_sub::<instructions::PureIntrinsic>().unwrap();
        match intrinsic {
          subcode::PureIntrinsic::FIFOPeek => {
            let port_self =
              dump_ref!(self.module_ctx, self.sys, &call.get().get_operand_value(0).unwrap());
            format!("sim.{}.front().unwrap().clone()", port_self)
          }
          subcode::PureIntrinsic::FIFOValid => {
            let port_self =
              dump_ref!(self.module_ctx, self.sys, &call.get().get_operand_value(0).unwrap());
            format!("!sim.{}.is_empty()", port_self)
          }
          subcode::PureIntrinsic::FIFOReady => {
            todo!()
          }
          subcode::PureIntrinsic::ValueValid => {
            let value = call.get().get_operand_value(0).unwrap();
            let expr = value
              .as_ref::<Expr>(self.sys)
              .expect("A value expected for validity");
            format!("sim.{}_value.is_some()", namify(&expr.get_name()))
          }
          subcode::PureIntrinsic::ModuleTriggered => {
            let port_self =
              dump_ref!(self.module_ctx, self.sys, &call.get().get_operand_value(0).unwrap());
            format!("sim.{}_triggered", port_self)
          }
        }
      }
      Opcode::FIFOPush => {
        let push = expr.as_sub::<instructions::FIFOPush>().unwrap();
        let fifo = push.fifo();
        let fifo_id = syn::Ident::new(&fifo_name!(fifo), Span::call_site());
        let value = dump_ref!(self.module_ctx, self.sys, &push.value());
        let value = value.parse::<proc_macro2::TokenStream>().unwrap();
        let module_writer = &self.module_name;
        quote::quote! {
          let stamp = sim.stamp;
          sim.#fifo_id.push.push(
            FIFOPush::new(stamp + 50, #value.clone(), #module_writer.to_string()));
        }
        .to_string()
      }
      Opcode::Log => {
        let mut res = String::new();
        res.push_str(&format!(
          "print!(\"@line:{{:<5}}\t{{}}:\t[{}]\t\", line!(), cyclize(sim.stamp));",
          self.module_name
        ));
        res.push_str("println!(");
        for elem in expr.operand_iter() {
          res.push_str(&format!("{}, ", dump_ref!(self.module_ctx, self.sys, elem.get_value())));
        }
        res.push(')');
        res
      }
      Opcode::Slice => {
        let slice = expr.as_sub::<instructions::Slice>().unwrap();
        let a = dump_ref!(self.module_ctx, self.sys, &slice.x());
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
          "1".repeat(r - l + 1),
          l,
          dtype_to_rust_type(&slice.get().dtype()),
        )
      }
      Opcode::Concat => {
        let dtype = expr.dtype();
        let concat = expr.as_sub::<instructions::Concat>().unwrap();
        let a = dump_ref!(self.module_ctx, self.sys, &concat.msb());
        let b = dump_ref!(self.module_ctx, self.sys, &concat.lsb());
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
        let cond = dump_ref!(self.module_ctx, self.sys, &select.cond());
        let true_value = dump_ref!(self.module_ctx, self.sys, &select.true_value());
        let false_value = dump_ref!(self.module_ctx, self.sys, &select.false_value());
        format!("if {} {{ {} }} else {{ {} }}", cond, true_value, false_value)
      }
      Opcode::Select1Hot => {
        let select1hot = expr.as_sub::<instructions::Select1Hot>().unwrap();
        let cond = select1hot.cond();
        let mut res = format!(
          "{{ let cond = {}; assert!(cond.count_ones() == 1, \"Select1Hot: condition is not 1-hot\");",
          dump_ref!(self.module_ctx, self.sys, &cond)
        );
        for (i, value) in select1hot.value_iter().enumerate() {
          if i != 0 {
            res.push_str(" else ");
          }
          res.push_str(&format!(
            "if cond >> {} & 1 != 0 {{ {} }}",
            i,
            dump_ref!(self.module_ctx, self.sys, &value)
          ));
        }
        res.push_str(" else { unreachable!() } }");
        res
      }
      Opcode::Cast { .. } => {
        let cast = expr.as_sub::<instructions::Cast>().unwrap();
        let src_dtype = cast.src_type();
        let dest_dtype = cast.dest_type();
        let a = dump_ref!(self.module_ctx, cast.get().sys, &cast.x());
        match cast.get_opcode() {
          Cast::ZExt | Cast::BitCast => {
            // perform zero extension
            format!(
              "ValueCastTo::<{}>::cast(&ValueCastTo::<{}>::cast(&{}))",
              dtype_to_rust_type(&dest_dtype),
              dtype_to_rust_type(&src_dtype).replace('i', "u"),
              a,
            )
          }
          Cast::SExt => {
            format!("ValueCastTo::<{}>::cast(&{})", dtype_to_rust_type(&dest_dtype), a)
          }
        }
      }
      Opcode::Bind => "()".into(),
      Opcode::BlockIntrinsic { intrinsic } => {
        let bi = expr.as_sub::<instructions::BlockIntrinsic>().unwrap();
        let value = dump_ref!(self.module_ctx, self.sys, &bi.value());
        match intrinsic {
          subcode::BlockIntrinsic::Value => value,
          subcode::BlockIntrinsic::Cycled => {
            open_scope = true;
            format!("if sim.stamp / 100 == ({} as usize) {{", value)
          }
          subcode::BlockIntrinsic::WaitUntil => {
            format!(
              "if !{} {{ let stamp = sim.stamp; sim.{}_event.push_back(stamp + 100); return; }}",
              value,
              namify(&self.module_name),
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
      let valid = if expose_validity {
        format!("sim.{}_value = Some({}.clone());", id, id)
      } else {
        "".into()
      };
      format!("let {} = {{ {} }}; {}\n", id, res, valid)
    } else {
      format!("{};\n", res)
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

fn dump_simulator(sys: &SysBuilder, config: &Config, fd: &mut std::fs::File) -> io::Result<()> {
  // TODO(@were): Make all arguments of the modules FIFO channels.
  // TODO(@were): Profile the maxium size of all the FIFO channels.
  fd.write_all("use std::collections::VecDeque;\n".as_bytes())?;
  fd.write_all("use super::runtime::*;\n".as_bytes())?;
  fd.write_all("use num_bigint::{BigInt, BigUint};\n".as_bytes())?;
  fd.write_all("use rand::seq::SliceRandom;\n".as_bytes())?;

  let mut simulator_init = vec![];
  let mut downstream_reset = vec![];
  let mut registers = vec![];
  // Dump the memebers of the simulator struct.
  // struct Simuator {
  //   ... arrays
  //   ... module
  //     ... modules' ports
  // }
  //
  // Along with the struct, the constructor will also be dumped lazily.
  // By "lazily", it means that constructor will be pre-stored in the simulator_init vector,
  // and will be filled in the `Simulator::new` function dumping.
  fd.write_all("pub struct Simulator { pub stamp: usize, ".as_bytes())?;
  for array in sys.array_iter() {
    let name = namify(array.get_name());
    let dtype = dtype_to_rust_type(&array.scalar_ty());
    fd.write_all(format!("pub {} : Array<{}>,", name, dtype,).as_bytes())?;
    if let Some(init) = array.get_initializer() {
      let init = init
        .iter()
        .map(|x| dump_ref!(sys, x))
        .collect::<Vec<_>>()
        .join(", ");
      simulator_init.push(format!("{} : Array::new_with_init(vec![{}]),", name, init));
    } else {
      simulator_init.push(format!("{} : Array::new({}),", name, array.get_size()));
    }
    registers.push(name);
  }
  let mut expr_validities = HashSet::new();
  for module in sys.module_iter(ModuleKind::All) {
    let module_name = namify(module.get_name());
    fd.write_all(format!("pub {}_triggered : bool,", module_name).as_bytes())?;
    simulator_init.push(format!("{}_triggered : false,", module_name));
    downstream_reset.push(format!("self.{}_triggered = false;", module_name));
    if !module.is_downstream() {
      fd.write_all(format!("pub {}_event : VecDeque<usize>,", module_name).as_bytes())?;
      simulator_init.push(format!("{}_event : VecDeque::new(),", module_name));
      for fifo in module.fifo_iter() {
        let name = fifo_name!(fifo);
        let ty = dtype_to_rust_type(&fifo.scalar_ty());
        fd.write_all(format!("pub {} : FIFO<{}>,", name, ty).as_bytes())?;
        simulator_init.push(format!("{} : FIFO::new(),", name));
        registers.push(name);
      }
    } else {
      // TODO(@were): Make this a gather.
      for expr in module
        .ext_interf_iter()
        .filter_map(|(x, _)| x.as_ref::<Expr>(sys).ok())
        .filter(|x| needs_validity(x))
      {
        expr_validities.insert(expr.upcast());
      }
    }
  }
  for expr in expr_validities
    .into_iter()
    .map(|x| x.as_ref::<Expr>(sys).unwrap())
  {
    let name = namify(&expr.get_name());
    let dtype = dtype_to_rust_type(&expr.dtype());
    fd.write_all(format!("pub {}_value : Option<{}>,", name, dtype).as_bytes())?;
    simulator_init.push(format!("{}_value : None,", name));
    downstream_reset.push(format!("self.{}_value = None;", name));
  }
  fd.write_all("}".as_bytes())?;

  fd.write_all("impl Simulator {".as_bytes())?;

  // Constructor.
  fd.write_all("pub fn new() -> Self {".as_bytes())?;
  fd.write_all("Simulator {".as_bytes())?;
  fd.write_all("stamp: 0,".as_bytes())?;
  for elem in simulator_init {
    fd.write_all(elem.as_bytes())?;
  }
  fd.write_all("}}".as_bytes())?;

  fd.write_all("fn event_valid(&self, event: &VecDeque<usize>) -> bool {".as_bytes())?;
  fd.write_all("event.front().map_or(false, |x| *x <= self.stamp)".as_bytes())?;
  fd.write_all("}".as_bytes())?;

  // Reset the downstream ports every cycle.
  fd.write_all("pub fn reset_downstream(&mut self) {".as_bytes())?;
  for elem in downstream_reset {
    fd.write_all(elem.as_bytes())?;
  }
  fd.write_all("}".as_bytes())?;

  // Tick the registers.
  fd.write_all("pub fn tick_registers(&mut self) {".as_bytes())?;
  for elem in registers {
    fd.write_all(format!("self.{}.tick(self.stamp);", elem).as_bytes())?;
  }
  fd.write_all("}".as_bytes())?;

  let mut simulators = vec![];
  let mut downstreams = vec![];
  for module in sys.module_iter(ModuleKind::All) {
    let module_name = namify(module.get_name());
    fd.write_all(format!("fn simulate_{}(&mut self) {{", module_name).as_bytes())?;
    if !module.is_downstream() {
      fd.write_all(format!("if self.event_valid(&self.{}_event) {{", module_name).as_bytes())?;
      fd.write_all(format!("self.{}_event.pop_front();", module_name).as_bytes())?;
    } else {
      let mut conds = HashSet::new();
      for (interf, _) in module.ext_interf_iter() {
        if let Ok(expr) = interf.as_ref::<Expr>(sys) {
          conds.insert(namify(
            expr
              .get_parent()
              .as_ref::<Block>(sys)
              .unwrap()
              .get_module()
              .as_ref::<Module>(sys)
              .unwrap()
              .get_name(),
          ));
        }
      }
      let conds = conds
        .into_iter()
        .map(|x| format!("self.{}_triggered", x))
        .collect::<Vec<_>>();
      fd.write_all("if ".as_bytes())?;
      fd.write_all(conds.join(" && ").as_bytes())?;
      fd.write_all(" {".as_bytes())?;
    }
    fd.write_all(format!("super::modules::{}(self);", module_name).as_bytes())?;
    if !module.is_downstream() {
      simulators.push(module_name.clone());
    } else {
      downstreams.push(module_name.clone());
    }
    fd.write_all(format!("self.{}_triggered = true;\n", module_name).as_bytes())?;
    fd.write_all("} // close event condition\n".as_bytes())?;
    fd.write_all("} // close function\n".as_bytes())?;
  }

  fd.write_all("}".as_bytes())?;

  fd.write_all("pub fn simulate() {\n".as_bytes())?;

  // TODO(@were): Later we allow some randomization of the simulation, these functions can be
  // shuffled.
  fd.write_all("let mut sim = Simulator::new();\n".as_bytes())?;

  if config.random {
    fd.write_all("let mut rng = rand::thread_rng();\n".as_bytes())?;
    fd.write_all("let mut simulators : Vec<fn(&mut Simulator)> = vec![".as_bytes())?;
  } else {
    fd.write_all("let simulators : Vec<fn(&mut Simulator)> = vec![".as_bytes())?;
  }
  for sim in simulators {
    fd.write_all(format!("Simulator::simulate_{},", sim).as_bytes())?;
  }
  fd.write_all("];\n".as_bytes())?;

  // TODO(@were): A downstream module can be recursively dependent to another downstream module.
  // A topological order among these downstream modules is needed.
  fd.write_all("let downstreams : Vec<fn(&mut Simulator)> = vec![".as_bytes())?;
  for downstream in downstreams {
    fd.write_all(format!("Simulator::simulate_{},", downstream).as_bytes())?;
  }
  fd.write_all("];\n".as_bytes())?;

  // generate memory initializations
  for module in sys.module_iter(ModuleKind::Module) {
    for attr in module.get_attrs() {
      if let Attribute::Memory(param) = attr {
        if let Some(init_file) = &param.init_file {
          let init_file_path = config.resource_base.join(init_file);
          let init_file_path = init_file_path.to_str().unwrap();
          let array = param.array.as_ref::<Array>(sys).unwrap();
          let array_name = syn::Ident::new(&namify(array.get_name()), Span::call_site());
          fd.write_all(
            quote::quote! { load_hex_file(&mut sim.#array_name.payload, #init_file_path); }
              .to_string()
              .as_bytes(),
          )?;
        }
      }
    }
  }

  let sim_threshold = config.sim_threshold;

  // Push the initial events from driver.
  if sys.has_driver() {
    fd.write_all(
      quote::quote! {
        for i in 1..=#sim_threshold {
          sim.driver_event.push_back(i * 100);
        }
      }
      .to_string()
      .as_bytes(),
    )?;
  }

  // Push the initial events from testbench.
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
    fd.write_all(
      quote::quote! {
        let tb_cycles = vec![#(#cycles, )*];
        for cycle in tb_cycles {
          sim.testbench_event.push_back(cycle * 100);
        }
      }
      .to_string()
      .as_bytes(),
    )?;
  }

  let randomization = if config.random {
    quote! { simulators.shuffle(&mut rng); }
  } else {
    quote! {}
  };

  fd.write_all(
    quote! {
      for i in 1..=#sim_threshold {
        sim.stamp = i * 100;
        sim.reset_downstream();
        #randomization;
        for simulate in simulators.iter() {
          simulate(&mut sim);
        }
        for simulate in downstreams.iter() {
          simulate(&mut sim);
        }
        sim.stamp += 50;
        sim.tick_registers();
      }
    }
    .to_string()
    .as_bytes(),
  )?;

  fd.write_all("}".as_bytes())?;

  Ok(())
}

fn dump_modules(sys: &SysBuilder, fd: &mut File) -> Result<(), std::io::Error> {
  fd.write_all(
    quote! {
      use super::runtime::*;
      use super::simulator::Simulator;
      use std::collections::VecDeque;
      use num_bigint::{BigInt, BigUint};
    }
    .to_string()
    .as_bytes(),
  )?;
  let mut em = ElaborateModule::new(sys);
  for module in em.sys.module_iter(ModuleKind::All) {
    if let Some(buffer) = em.visit_module(module) {
      fd.write_all(buffer.as_bytes())?;
    }
  }
  Ok(())
}

fn dump_main(fd: &mut File) -> Result<(), std::io::Error> {
  let src = quote::quote! {
    mod runtime;
    mod modules;
    mod simulator;

    fn main() {
      simulator::simulate();
    }
  };
  fd.write_all(src.to_string().as_bytes())?;
  fd.write_all("\n\n\n".as_bytes())
}

fn elaborate_impl(sys: &SysBuilder, config: &Config) -> Result<PathBuf, std::io::Error> {
  // Create and clean the simulator directory.
  let simulator_name = config.dirname(sys, "simulator");
  create_and_clean_dir(simulator_name.clone(), config.override_dump);
  fs::create_dir_all(simulator_name.join("src")).unwrap();
  eprintln!("Writing simulator code to rust project: {}", simulator_name.to_str().unwrap());
  let manifest = simulator_name.join("Cargo.toml");
  // Dump the Cargo.toml and rustfmt.toml
  {
    let mut cargo = fs::File::create(simulator_name.join("Cargo.toml"))?;
    writeln!(cargo, "[package]")?;
    writeln!(cargo, "name = \"{}_simulator\"", sys.get_name())?;
    writeln!(cargo, "version = \"0.1.0\"")?;
    writeln!(cargo, "edition = \"2021\"")?;
    writeln!(cargo, "[dependencies]")?;
    writeln!(cargo, "num-bigint = \"0.4\"")?;
    writeln!(cargo, "num-traits = \"0.2\"")?;
    writeln!(cargo, "rand = \"0.8\"")?;
    let mut fmt = fs::File::create(simulator_name.join("rustfmt.toml"))?;
    writeln!(fmt, include_str!("../../../rustfmt.toml"))?;
    fmt.flush()?;
  }
  {
    // Dump modules' logic.
    let modules_file = simulator_name.join("src/modules.rs");
    let mut fd = fs::File::create(modules_file).expect("Open failure");
    dump_modules(sys, &mut fd).expect("Dump module failure");
    fd.flush().expect("Flush modules failure");
  };
  {
    // Dump runtime, mainly data casting.
    let runtime_file = simulator_name.join("src/runtime.rs");
    let mut fruntime = fs::File::create(runtime_file).expect("Open failure");
    super::runtime::dump_runtime(&mut fruntime);
    fruntime.flush().expect("Flush runtime failure");
  }
  {
    // Dump the simulator host.
    let simulator_file = simulator_name.join("src/simulator.rs");
    let mut fsim = fs::File::create(simulator_file).expect("Open failure");
    dump_simulator(sys, config, &mut fsim).expect("Dump simulator failure");
    fsim.flush().expect("Flush simulator failure");
  }
  {
    // Dump the main entrance.
    let fname = simulator_name.join("src/main.rs");
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
