use std::{
  collections::{HashMap, HashSet, VecDeque},
  fs::File,
  io::{self, Error, Write},
  path::Path,
};

use instructions::FIFOPush;
// use instructions::FIFOPush;
use regex::Regex;

use crate::{
  backend::common::{create_and_clean_dir, namify, Config},
  builder::system::{ModuleKind, SysBuilder},
  ir::{instructions::BlockIntrinsic, node::*, visitor::Visitor, *},
};

use self::{expr::subcode, module::Attribute};

use super::{
  gather::Gather,
  utils::{
    self, bool_ty, connect_top, declare_array, declare_in, declare_logic, declare_out, reduce,
    select_1h, Edge, Field,
  },
  Simulator,
};

macro_rules! fifo_name {
  ($fifo:expr) => {{
    format!("{}", namify($fifo.get_name()))
  }};
}

struct VerilogDumper<'a, 'b> {
  sys: &'a SysBuilder,
  config: &'b Config,
  pred_stack: VecDeque<String>,
  fifo_pushes: HashMap<String, Gather>, // fifo_name -> value
  array_stores: HashMap<String, (Gather, Gather)>, // array_name -> (idx, value)
  triggers: HashMap<String, Gather>,    // module_name -> [pred]
  current_module: String,
}

impl<'a, 'b> VerilogDumper<'a, 'b> {
  fn new(sys: &'a SysBuilder, config: &'b Config) -> Self {
    Self {
      sys,
      config,
      pred_stack: VecDeque::new(),
      fifo_pushes: HashMap::new(),
      array_stores: HashMap::new(),
      triggers: HashMap::new(),
      current_module: String::new(),
    }
  }

  fn get_pred(&self) -> Option<String> {
    if self.pred_stack.is_empty() {
      None
    } else {
      Some(format!(
        "({})",
        self
          .pred_stack
          .iter()
          .map(|s| s.as_str())
          .collect::<Vec<_>>()
          .join(" && ")
      ))
    }
  }

  fn dump_array(&self, array: &ArrayRef, mem_init_path: Option<&String>) -> String {
    let mut res = String::new();
    let display = utils::DisplayInstance::from_array(array);
    // write enable
    let w = display.field("w");
    // write index
    let widx = display.field("widx");
    // write data
    let d = display.field("d");
    // array buffer
    let q = display.field("q");
    res.push_str(&format!("  // {}\n", array));
    res.push_str(&declare_array("", array, &q, ";"));

    let mut seen = HashSet::new();
    let drivers = array
      .users()
      .iter()
      .filter_map(move |x| {
        let expr = x.as_ref::<Operand>(array.sys).unwrap().get_expr();
        if matches!(expr.get_opcode(), Opcode::Store) {
          Some(expr.get_block().get_module())
        } else {
          None
        }
      })
      .filter(|x| seen.insert(x.get_key()))
      .map(|x| Edge::new(display.clone(), &x.as_ref::<Module>(array.sys).unwrap()))
      .collect::<Vec<_>>();

    let scalar_bits = array.scalar_ty().get_bits();

    drivers.iter().for_each(|edge| {
      res.push_str(&declare_logic(array.scalar_ty(), &edge.field("d")));
      res.push_str(&declare_logic(DataType::int_ty(1), &edge.field("w")));
      res.push_str(&declare_logic(array.get_idx_type(), &edge.field("widx")));
    });

    // if w: array[widx] = d;
    // where w is the gathered write enable signal
    // widx/d are 1-hot selected from all the writers
    res.push_str(&declare_logic(array.scalar_ty(), &d));
    res.push_str(&declare_logic(array.get_idx_type(), &widx));
    res.push_str(&declare_logic(DataType::int_ty(1), &w));

    let write_data = select_1h(
      drivers
        .iter()
        .map(|edge| (edge.field("w"), edge.field("d"))),
      scalar_bits,
    );
    res.push_str(&format!("  assign {d} = {};\n", write_data));

    let write_idx = select_1h(
      drivers
        .iter()
        .map(|edge| (edge.field("w"), edge.field("widx"))),
      (array.get_size().ilog2() + 1) as usize,
    );
    res.push_str(&format!("  assign {widx} = {};\n", write_idx));

    let write_enable = reduce(drivers.iter().map(|edge| edge.field("w")), " | ");
    res.push_str(&format!("  assign {w} = {};\n", write_enable));

    res.push_str("  always_ff @(posedge clk or negedge rst_n)\n");
    // Dump the initializer
    res.push_str("    if (!rst_n)\n");
    if mem_init_path.is_some() {
      // Read from memory initialization file
      res.push_str(&format!("      $readmemh(\"{}\", {q});\n", mem_init_path.unwrap()));
    } else if let Some(initializer) = array.get_initializer() {
      // Read from the hardcoded initializer
      res.push_str("    begin\n");
      for (idx, value) in initializer.iter().enumerate() {
        let elem_init = value.as_ref::<IntImm>(self.sys).unwrap().get_value();
        res.push_str(&format!("      {q}[{idx}] <= {elem_init};\n",));
      }
      res.push_str("    end\n");
    } else {
      // Initialize to 0
      res.push_str(&format!("      {q} <= '{{default : {scalar_bits}'d0}};\n",));
    }
    // Dump the array write
    res.push_str(&format!("    else if ({w}) {q}[{widx}] <= {d};\n\n",));

    res
  }

  fn dump_fifo(&self, fifo: &FIFORef) -> String {
    let mut res = String::new();
    let display = utils::DisplayInstance::from_fifo(fifo, true);
    let fifo_name = namify(&format!("{}_{}", fifo.get_module().get_name(), fifo_name!(fifo)));
    let fifo_width = fifo.scalar_ty().get_bits();
    res.push_str(&format!("  // {}\n", fifo));

    let push_valid = display.field("push_valid"); // If external pushers have data to push
    let push_data = display.field("push_data"); // Data to be pushed
    let pop_ready = display.field("pop_ready"); // If the FIFO pops data
    let push_ready = display.field("push_ready"); // If the FIFO is ready to accept data
    let pop_valid = display.field("pop_valid"); // If the popped data is valid
    let pop_data = display.field("pop_data"); // Popped data

    let edges = fifo
      .users()
      .iter()
      .filter_map(|x| {
        x.as_ref::<Operand>(self.sys)
          .unwrap()
          .get_user()
          .as_expr::<FIFOPush>(self.sys)
          .ok()
          .map(|y| y.get().get_block().get_module())
      })
      .collect::<HashSet<_>>()
      .iter()
      .map(|x| Edge::new(display.clone(), &x.as_ref::<Module>(self.sys).unwrap()))
      .collect::<Vec<_>>();

    res.push_str("  // Declare the pop.{data/valid/ready}\n");
    res.push_str(&declare_logic(fifo.scalar_ty(), &pop_data));
    res.push_str(&declare_logic(bool_ty(), &pop_valid));
    res.push_str(&declare_logic(bool_ty(), &pop_ready));

    edges.iter().for_each(|edge| {
      res.push_str(&declare_logic(fifo.scalar_ty(), &edge.field("push_data")));
      res.push_str(&declare_logic(bool_ty(), &edge.field("push_valid")));
      res.push_str(&declare_logic(bool_ty(), &edge.field("push_ready")));
    });

    res.push_str("  // Broadcast the push_ready signal to all the pushers\n");
    res.push_str(&format!("  logic {push_ready};\n"));
    edges
      .iter()
      .for_each(|x| res.push_str(&format!("  assign {} = {push_ready};", x.field("push_ready"))));

    res.push_str("  // Gather all the push signal\n");
    let valid = reduce(edges.iter().map(|x| x.field("push_valid")), " | ");
    res.push_str(&declare_logic(DataType::int_ty(1), &push_valid));
    res.push_str(&format!("  assign {} = {};\n", push_valid, valid));

    res.push_str("  // 1-hot select the push data\n");
    let data = select_1h(
      edges
        .iter()
        .map(|x| (x.field("push_valid"), x.field("push_data"))),
      fifo_width,
    );
    res.push_str(&declare_logic(fifo.scalar_ty(), &push_data));
    res.push_str(&format!("  assign {push_data} = {data};\n"));

    // Instantiate the FIFO
    res.push_str(&format!(
      "
  fifo #({width}) fifo_{name}_i (
    .clk(clk),
    .rst_n(rst_n),
    .push_valid({push_valid}),
    .push_data({push_data}),
    .push_ready({push_ready}),
    .pop_valid({pop_valid}),
    .pop_data({pop_data}),
    .pop_ready({pop_ready}));\n\n",
      name = fifo_name,
      width = fifo_width,
    ));

    res
  }

  /// Dump the trigger event state machine's instantiation.
  fn dump_trigger(&self, module: &ModuleRef) -> String {
    let mut res = String::new();
    let module_name = namify(module.get_name());
    let display = utils::DisplayInstance::from_module(module);
    res.push_str(&format!("  // Trigger SM of Module: {}\n", module.get_name()));
    let delta_value = display.field("counter_delta");
    let pop_ready = display.field("counter_pop_ready");
    let pop_valid = display.field("counter_pop_valid");
    let delta_ready = display.field("counter_delta_ready");

    let callers = module
      .callers()
      .map(|x| Edge::new(display.clone(), &x))
      .collect::<Vec<_>>();

    if module_name != "driver" && module_name != "testbench" {
      callers.iter().for_each(|edge| {
        res.push_str(&declare_logic(
          DataType::int_ty(8 /*FIXME(@were): Do not hardcode*/),
          &edge.field("counter_delta"),
        ));
        res.push_str(&declare_logic(bool_ty(), &edge.field("counter_delta_ready")));
      });
    }
    res.push_str(&declare_logic(bool_ty(), &delta_ready));
    res.push_str(&declare_logic(
      DataType::int_ty(8 /*FIXME(@were): Do not hardcode*/),
      &delta_value,
    ));

    res.push_str("  // Gather all the push signal\n");
    if module_name != "driver" && module_name != "testbench" {
      res.push_str(&format!(
        "  assign {delta_value} = {};\n",
        reduce(callers.iter().map(|x| x.field("counter_delta")), " + ")
      ));
    }
    res.push_str("  // Broadcast the push_ready signal to all the pushers\n");
    res.push_str(&declare_logic(bool_ty(), &pop_ready));
    if module_name != "driver" && module_name != "testbench" {
      callers.iter().for_each(|x| {
        res.push_str(&format!("  assign {} = {};\n", x.field("counter_delta_ready"), pop_ready));
      });
    }
    res.push_str(&declare_logic(bool_ty(), &pop_valid));
    res.push_str(&format!(
      "
  trigger_counter #(8) {}_trigger_i (
    .clk(clk),
    .rst_n(rst_n),
    .delta({delta_value}),
    .delta_ready({delta_ready}),
    .pop_valid({pop_valid}),
    .pop_ready({pop_ready}));\n",
      module_name
    ));
    res
  }

  fn dump_module_instance(&self, module: &ModuleRef) -> String {
    let mut res = String::new();
    let module_name = namify(module.get_name());
    res.push_str(&format!(
      "
  // {module}
  {module} {module}_i (
    .clk(clk),
    .rst_n(rst_n),
",
      module = module_name
    ));
    for port in module.fifo_iter() {
      let local = utils::DisplayInstance::from_fifo(&port, false);
      let global = utils::DisplayInstance::from_fifo(&port, true);
      res.push_str(&connect_top(&local, &global, &["pop_ready", "pop_data", "pop_valid"]));
    }
    for (interf, ops) in module.ext_interf_iter() {
      match interf.get_kind() {
        NodeKind::FIFO => {
          let fifo = interf.as_ref::<FIFO>(self.sys).unwrap();
          let fifo = utils::DisplayInstance::from_fifo(&fifo, true);
          let edge = Edge::new(fifo.clone(), module);
          res.push_str(&connect_top(&fifo, &edge, &["push_valid", "push_data", "push_ready"]));
        }
        NodeKind::Array => {
          let array_ref = interf.as_ref::<Array>(self.sys).unwrap();
          let display = utils::DisplayInstance::from_array(&array_ref);
          let edge = Edge::new(display.clone(), module);
          if self.sys.user_contains_opcode(ops, Opcode::Load) {
            res.push_str(&format!("    .{q}({q}),\n", q = display.field("q"),));
          }
          if self.sys.user_contains_opcode(ops, Opcode::Store) {
            res.push_str(&connect_top(&display, &edge, &["w", "widx", "d"]));
          }
        }
        NodeKind::Module => {
          let interf = interf.as_ref::<Module>(self.sys).unwrap();
          let display = utils::DisplayInstance::from_module(&interf);
          let edge = Edge::new(display.clone(), module);
          res.push_str(&connect_top(&display, &edge, &["counter_delta_ready", "counter_delta"]));
        }
        NodeKind::Expr => {
          // TODO(@were): Implement this for downstreams.
        }
        _ => panic!("Unknown interf kind {:?}", interf.get_kind()),
      }
    }

    let display = utils::DisplayInstance::from_module(module);
    res.push_str(&format!(
      "    .counter_delta_ready({}),\n",
      display.field("counter_delta_ready")
    ));
    res.push_str(&format!("    .counter_pop_ready({}),\n", display.field("counter_pop_ready")));
    res.push_str(&format!("    .counter_pop_valid({}));\n", display.field("counter_pop_valid")));
    res
  }

  fn dump_runtime(
    self: VerilogDumper<'a, 'b>,
    mut fd: File,
    sim_threshold: usize,
  ) -> Result<(), Error> {
    // runtime
    let mut res = String::new();

    res.push_str(
      "
module top (
  input logic clk,
  input logic rst_n
);\n\n",
    );

    // memory initializations map
    let mut mem_init_map: HashMap<BaseNode, String> = HashMap::new(); // array -> init_file_path
    for module in self.sys.module_iter(ModuleKind::Module) {
      for attr in module.get_attrs() {
        if let Attribute::Memory(param) = attr {
          if let Some(init_file) = &param.init_file {
            let mut init_file_path = self.config.resource_base.clone();
            init_file_path.push(init_file);
            let init_file_path = init_file_path.to_str().unwrap();
            let array = param.array.as_ref::<Array>(self.sys).unwrap();
            mem_init_map.insert(array.upcast(), init_file_path.to_string());
          }
        }
      }
    }

    // array storage element definitions
    for array in self.sys.array_iter() {
      res.push_str(&self.dump_array(&array, mem_init_map.get(&array.upcast())));
    }

    // fifo storage element definitions
    for module in self.sys.module_iter(ModuleKind::Module) {
      for fifo in module.fifo_iter() {
        res.push_str(&self.dump_fifo(&fifo));
      }
    }

    // trigger fifo definitions
    for module in self.sys.module_iter(ModuleKind::Module) {
      res.push_str(&self.dump_trigger(&module));
    }

    // FIXME(@were): Do not hardcode the counter delta width.
    if self.sys.has_testbench() {
      res.push_str("  assign testbench_counter_delta = 8'b1;\n\n");
    }
    if self.sys.has_driver() {
      res.push_str("  assign driver_counter_delta = 8'b1;\n\n");
    }

    // module insts
    for module in self.sys.module_iter(ModuleKind::Module) {
      res.push_str(&self.dump_module_instance(&module));
    }

    res.push_str("endmodule // top\n\n");

    fd.write_all(res.as_bytes()).unwrap();

    let init = match self.config.verilog {
      Simulator::VCS => {
        "
initial begin
  $fsdbDumpfile(\"wave.fsdb\");
  $fsdbDumpvars();
  $fsdbDumpMDA();
end"
      }
      Simulator::Verilator => "",
      Simulator::None => panic!("No simulator specified"),
    };

    fd.write_all(include_str!("./runtime.sv").as_bytes())
      .unwrap();

    let threashold = (sim_threshold + 1) * 100;
    fd.write_all(
      format!(
        "
module tb;

logic clk;
logic rst_n;

initial begin
  clk = 1'b1;
  rst_n = 1'b0;
  #150;
  rst_n = 1'b1;
  #{threashold};
  $finish();
end

always #50 clk <= !clk;

{init}

top top_i (
  .clk(clk),
  .rst_n(rst_n)
);

endmodule
"
      )
      .as_bytes(),
    )?;

    Ok(())
  }
}

fn node_dump_ref(
  sys: &SysBuilder,
  node: &BaseNode,
  _: Vec<NodeKind>,
  immwidth: bool,
) -> Option<String> {
  match node.get_kind() {
    NodeKind::Array => {
      let array = node.as_ref::<Array>(sys).unwrap();
      namify(array.get_name()).into()
    }
    NodeKind::FIFO => namify(node.as_ref::<FIFO>(sys).unwrap().get_name()).into(),
    NodeKind::IntImm => {
      let int_imm = node.as_ref::<IntImm>(sys).unwrap();
      let dbits = int_imm.dtype().get_bits();
      let value = int_imm.get_value();
      if immwidth {
        Some(format!("{}'d{}", dbits, value))
      } else {
        Some(format!("{}", value))
      }
    }
    NodeKind::StrImm => {
      let str_imm = node.as_ref::<StrImm>(sys).unwrap();
      let value = str_imm.get_value();
      quote::quote!(#value).to_string().into()
    }
    NodeKind::Expr => Some(namify(&node.to_string(sys))),
    _ => panic!("Unknown node of kind {:?}", node.get_kind()),
  }
}

macro_rules! dump_ref {
  ($sys:expr, $value:expr) => {
    node_dump_ref($sys, $value, vec![], false).unwrap()
  };
}

macro_rules! dump_ref_immwidth {
  ($sys:expr, $value:expr) => {
    node_dump_ref($sys, $value, vec![], true).unwrap()
  };
}

impl VerilogDumper<'_, '_> {
  fn print_body(&mut self, node: BaseNode) -> String {
    match node.get_kind() {
      NodeKind::Expr => {
        let expr = node.as_ref::<Expr>(self.sys).unwrap();
        self.visit_expr(expr).unwrap()
      }
      NodeKind::Block => {
        let block = node.as_ref::<Block>(self.sys).unwrap();
        self.visit_block(block).unwrap()
      }
      _ => {
        panic!("Unexpected reference type: {:?}", node);
      }
    }
  }
}

impl<'a, 'b> Visitor<String> for VerilogDumper<'a, 'b> {
  fn visit_module(&mut self, module: ModuleRef<'_>) -> Option<String> {
    self.current_module = namify(module.get_name()).to_string();

    let mut res = String::new();

    res.push_str(&format!(
      "
module {} (
  input logic clk,
  input logic rst_n,
",
      self.current_module
    ));

    for port in module.fifo_iter() {
      let name = fifo_name!(port);
      let ty = port.scalar_ty();
      let display = utils::DisplayInstance::from_fifo(&port, false);
      // (pop_valid, pop_data): something like `let front : Optional<T> = FIFO.pop();`.
      // `pop_ready: when enabled, it is something like fifo.pop()
      res.push_str(&format!("  // Port FIFO {name}\n", name = name));
      res.push_str(&declare_in(bool_ty(), &display.field("pop_valid")));
      res.push_str(&declare_in(ty, &display.field("pop_data")));
      res.push_str(&declare_out(bool_ty(), &display.field("pop_ready")));
    }

    for (interf, ops) in module.ext_interf_iter() {
      match interf.get_kind() {
        NodeKind::FIFO => {
          let fifo = interf.as_ref::<FIFO>(self.sys).unwrap();
          let parent_name = fifo.get_module().get_name().to_string();
          let display = utils::DisplayInstance::from_fifo(&fifo, true);
          // TODO(@were): Support `push_ready` for backpressures.
          // (push_valid, push_data, push_ready) works like
          // `if push_valid && push_ready: FIFO.push()`
          res.push_str(&format!("  // External FIFO {}.{}\n", parent_name, fifo.get_name()));
          res.push_str(&declare_out(bool_ty(), &display.field("push_valid")));
          res.push_str(&declare_out(fifo.scalar_ty(), &display.field("push_data")));
          res.push_str(&declare_in(bool_ty(), &display.field("push_ready")));
        }
        NodeKind::Array => {
          let array = interf.as_ref::<Array>(self.sys).unwrap();
          let display = utils::DisplayInstance::from_array(&array);
          res.push_str(&format!("  // {}\n", array));
          if self.sys.user_contains_opcode(ops, Opcode::Load) {
            res.push_str(&declare_array("input", &array, &display.field("q"), ","));
          }
          // (w, widx, d): something like `array[widx] = d;`
          if self.sys.user_contains_opcode(ops, Opcode::Store) {
            res.push_str(&declare_out(bool_ty(), &display.field("w")));
            res.push_str(&declare_out(array.get_idx_type(), &display.field("widx")));
            res.push_str(&declare_out(array.scalar_ty(), &display.field("d")));
          }
        }
        NodeKind::Module => {
          let module = interf.as_ref::<Module>(self.sys).unwrap();
          let display = utils::DisplayInstance::from_module(&module);
          res.push_str(&format!("  // Module {}\n", module.get_name()));
          // FIXME(@were): Do not hardcode the counter delta width.
          res.push_str(&declare_out(DataType::int_ty(8), &display.field("counter_delta")));
          res.push_str(&declare_in(bool_ty(), &display.field("counter_delta_ready")));
        }
        NodeKind::Expr => {
          // TODO(@were): Handle this later.
        }
        _ => panic!("Unknown interf kind {:?}", interf.get_kind()),
      }
      res.push('\n');
    }

    res.push_str("  // self.event_q\n");
    res.push_str("  input logic counter_pop_valid,\n");
    res.push_str("  input logic counter_delta_ready,\n");
    res.push_str("  output logic counter_pop_ready);\n\n");

    let mut wait_until: String = "".to_string();

    let skip = if let Some(wu_intrin) = module.get_body().get_wait_until() {
      let mut skip = 0;
      let body = module.get_body();
      let body_iter = body.body_iter();
      for (i, elem) in body_iter.enumerate() {
        if elem == wu_intrin {
          skip = i + 1;
          break;
        }
        res.push_str(&self.print_body(elem));
      }
      let bi = wu_intrin.as_expr::<BlockIntrinsic>(self.sys).unwrap();
      let value = bi.value();
      wait_until = format!(" && ({} != '0)", namify(&value.to_string(self.sys)));
      skip
    } else {
      0
    };

    res.push_str("  logic executed;\n");

    if self.current_module == "testbench" {
      res.push_str(
        "
  int cycle_cnt;
  always_ff @(posedge clk or negedge rst_n) if (!rst_n) cycle_cnt <= 0;
  else if (executed) cycle_cnt <= cycle_cnt + 1;
",
      );
    }

    self.fifo_pushes.clear();
    self.array_stores.clear();
    self.triggers.clear();
    for elem in module.get_body().body_iter().skip(skip) {
      res.push_str(&self.print_body(elem));
    }

    for (m, g) in self.triggers.drain() {
      res.push_str(&format!(
        "  assign {}_counter_delta = executed ? {} : 0;\n\n",
        m,
        if g.is_conditional() {
          g.condition
            .iter()
            .map(|x| format!("{{ {}'b0, |{} }}", g.bits - 1, x))
            .collect::<Vec<_>>()
            .join(" + ")
        } else {
          "1".into()
        }
      ));
    }

    res.push_str("  // Gather FIFO pushes\n");

    for (fifo, g) in self.fifo_pushes.drain() {
      res.push_str(&format!(
        "
  assign fifo_{fifo}_push_valid = {cond};
  assign fifo_{fifo}_push_data = {value};
",
        fifo = fifo,
        cond = g.and("executed", " || "),
        value = g.select_1h()
      ));
    }

    res.push_str("  // Gather Array writes\n");

    for (a, (idx, data)) in self.array_stores.drain() {
      res.push_str(&format!(
        "
  assign array_{a}_w = {cond};
  assign array_{a}_d = {data};
  assign array_{a}_widx = {idx};
",
        a = a,
        cond = idx.and("executed", " || "),
        idx = idx.select_1h(),
        data = data.select_1h()
      ));
    }

    res.push_str(&format!(
      "
  assign executed = counter_pop_valid{};
  assign counter_pop_ready = executed;
endmodule // {}
",
      wait_until, self.current_module
    ));

    Some(res)
  }

  fn visit_block(&mut self, block: BlockRef<'_>) -> Option<String> {
    let mut res = String::new();
    let skip = if let Some(cond) = block.get_condition() {
      self
        .pred_stack
        .push_back(if cond.get_dtype(block.sys).unwrap().get_bits() == 1 {
          dump_ref!(self.sys, &cond)
        } else {
          format!("({} != '0)", dump_ref!(self.sys, &cond))
        });
      1
    } else if let Some(cycle) = block.get_cycle() {
      self
        .pred_stack
        .push_back(format!("(cycle_cnt == {})", cycle));
      1
    } else {
      0
    };
    for elem in block.body_iter().skip(skip) {
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
    self.pred_stack.pop_back();
    res.into()
  }

  fn visit_expr(&mut self, expr: ExprRef<'_>) -> Option<String> {
    let decl = if expr.get_opcode().is_valued()
      && !matches!(expr.get_opcode(), Opcode::FIFOPop | Opcode::Bind)
    {
      Some((namify(&expr.upcast().to_string(self.sys)), expr.dtype()))
    } else {
      None
    };

    let body = match expr.get_opcode() {
      Opcode::Binary { .. } => {
        let bin = expr.as_sub::<instructions::Binary>().unwrap();
        format!(
          "{} {} {}",
          dump_ref!(self.sys, &bin.a()),
          bin.get_opcode(),
          dump_ref!(self.sys, &bin.b())
        )
      }

      Opcode::Unary { ref uop } => {
        let dump = match uop {
          subcode::Unary::Flip => "~",
          subcode::Unary::Neg => "-",
        };
        let uop = expr.as_sub::<instructions::Unary>().unwrap();
        format!("{}{}", dump, dump_ref!(self.sys, &uop.x()))
      }

      Opcode::Compare { .. } => {
        let cmp = expr.as_sub::<instructions::Compare>().unwrap();
        format!(
          "{} {} {}",
          dump_ref!(self.sys, &cmp.a()),
          cmp.get_opcode(),
          dump_ref!(self.sys, &cmp.b())
        )
      }

      Opcode::FIFOPop => {
        let id = namify(&expr.upcast().to_string(self.sys));
        let pop = expr.as_sub::<instructions::FIFOPop>().unwrap();
        let fifo = pop.fifo();
        let display = utils::DisplayInstance::from_fifo(&fifo, false);
        format!(
          "{}\n  assign {} = {};\n  assign {} = executed{};\n",
          declare_logic(fifo.scalar_ty(), &id),
          id,
          display.field("pop_data"),
          display.field("pop_ready"),
          self
            .get_pred()
            .map(|p| format!(" && {}", p))
            .unwrap_or("".to_string())
        )
      }

      Opcode::Log => {
        let mut format_str = dump_ref!(self.sys, expr.operand_iter().next().unwrap().get_value());

        let re = Regex::new(r"\{(:.[bxXo]?)?\}").unwrap();

        let dtypes = expr
          .operand_iter()
          .skip(1)
          .map(|elem| elem.get_value().get_dtype(self.sys).unwrap())
          .collect::<Vec<_>>();

        let mut dtype_index = 0;
        format_str = re
          .replace_all(&format_str, |caps: &regex::Captures| {
            let result = if let Some(format_spec) = caps.get(1) {
              match format_spec.as_str() {
                ":b" => "%b",
                ":x" => "%x",
                ":X" => "%X",
                ":o" => "%o",
                ":" => {
                  if let Some(dtype) = dtypes.get(dtype_index) {
                    match dtype {
                      DataType::Int(_) | DataType::UInt(_) | DataType::Bits(_) => "%d",
                      DataType::Str => "%s",
                      _ => "?",
                    }
                  } else {
                    "?"
                  }
                }
                _ => {
                  println!("Unrecognized format specifier: {}", format_spec.as_str());
                  "?"
                }
              }
            } else if let Some(dtype) = dtypes.get(dtype_index) {
              match dtype {
                DataType::Int(_) | DataType::UInt(_) | DataType::Bits(_) => "%d",
                DataType::Str => "%s",
                _ => "?",
              }
            } else {
              "?"
            };
            dtype_index += 1;
            result
          })
          .into_owned();
        format_str = format_str.replace('"', "");

        let mut res = String::new();
        res.push_str(&format!(
          "  always_ff @(posedge clk iff executed{}) ",
          self
            .get_pred()
            .map(|p| format!(" && {}", p))
            .unwrap_or("".to_string())
        ));
        res.push_str("$display(\"%t\\t");
        res.push_str(&format_str);
        res.push_str("\", $time - 200, ");
        for elem in expr.operand_iter().skip(1) {
          res.push_str(&format!("{}, ", dump_ref!(self.sys, elem.get_value())));
        }
        res.pop();
        res.pop();
        res.push_str(");\n");
        res.push('\n');
        res
      }

      Opcode::Load => {
        let load = expr.as_sub::<instructions::Load>().unwrap();
        let (array_ref, array_idx) = (load.array(), load.idx());
        format!("array_{}_q[{}]", namify(array_ref.get_name()), dump_ref!(self.sys, &array_idx))
      }

      Opcode::Store => {
        let store = expr.as_sub::<instructions::Store>().unwrap();
        let (array_ref, array_idx) = (store.array(), store.idx());
        let array_name = namify(array_ref.get_name());
        let pred = self.get_pred().unwrap_or("".to_string());
        let idx = dump_ref_immwidth!(self.sys, &array_idx);
        let idx_bits = store.idx().get_dtype(self.sys).unwrap().get_bits();
        let value = dump_ref!(self.sys, &store.value());
        let value_bits = store.value().get_dtype(self.sys).unwrap().get_bits();
        match self.array_stores.get_mut(&array_name) {
          Some((g_idx, g_value)) => {
            g_idx.push(pred.clone(), idx, idx_bits);
            g_value.push(pred, value, value_bits);
          }
          None => {
            self.array_stores.insert(
              array_name.clone(),
              (Gather::new(pred.clone(), idx, idx_bits), Gather::new(pred, value, value_bits)),
            );
          }
        }
        "".to_string()
      }

      Opcode::FIFOPush => {
        let push = expr.as_sub::<instructions::FIFOPush>().unwrap();
        let fifo = push.fifo();
        let fifo_name = format!("{}_{}", namify(fifo.get_module().get_name()), fifo_name!(fifo));
        let pred = self.get_pred().unwrap_or("".to_string());
        let value = dump_ref!(self.sys, &push.value());
        match self.fifo_pushes.get_mut(&fifo_name) {
          Some(fps) => fps.push(pred, value, fifo.scalar_ty().get_bits()),
          None => {
            self
              .fifo_pushes
              .insert(fifo_name.clone(), Gather::new(pred, value, fifo.scalar_ty().get_bits()));
          }
        }
        "".to_string()
      }

      Opcode::PureIntrinsic { intrinsic } => {
        let call = expr.as_sub::<instructions::PureIntrinsic>().unwrap();
        let fifo = call
          .get()
          .get_operand_value(0)
          .unwrap()
          .as_ref::<FIFO>(self.sys)
          .unwrap();
        let fifo_name = fifo_name!(fifo);
        match intrinsic {
          subcode::PureIntrinsic::FIFOValid => format!("fifo_{}_pop_valid", fifo_name),
          subcode::PureIntrinsic::FIFOPeek => format!("fifo_{}_pop_data", fifo_name),
          _ => todo!(),
        }
      }

      Opcode::AsyncCall => {
        let call = expr.as_sub::<instructions::AsyncCall>().unwrap();
        let callee = {
          let bind = call.bind();
          bind.callee().get_name().to_string()
        };
        let callee = namify(&callee);
        let pred = self.get_pred().unwrap_or("".to_string());
        // FIXME(@were): Do not hardcode the counter delta width.
        match self.triggers.get_mut(&callee) {
          Some(trgs) => trgs.push(pred, "".into(), 8),
          None => {
            self
              .triggers
              .insert(callee, Gather::new(pred, "".into(), 8));
          }
        }
        "".to_string()
      }

      Opcode::Slice => {
        let slice = expr.as_sub::<instructions::Slice>().unwrap();
        let a = dump_ref!(self.sys, &slice.x());
        let l = dump_ref!(self.sys, &slice.l_intimm().upcast());
        let r = dump_ref!(self.sys, &slice.r_intimm().upcast());
        format!("{}[{}:{}]", a, r, l)
      }

      Opcode::Concat => {
        let concat = expr.as_sub::<instructions::Concat>().unwrap();
        let a = dump_ref_immwidth!(self.sys, &concat.msb());
        let b = dump_ref_immwidth!(self.sys, &concat.lsb());
        format!("{{{}, {}}}", a, b)
      }

      Opcode::Cast { .. } => {
        let dbits = expr.dtype().get_bits();
        let cast = expr.as_sub::<instructions::Cast>().unwrap();
        let a = dump_ref!(self.sys, &cast.x());
        let src_dtype = cast.src_type();
        let pad = dbits - src_dtype.get_bits();
        match cast.get_opcode() {
          subcode::Cast::BitCast => a,
          subcode::Cast::ZExt => format!("{{{}'b0, {}}}", pad, a),
          subcode::Cast::SExt => {
            let dest_dtype = cast.dest_type();
            if src_dtype.is_int()
              && src_dtype.is_signed()
              && dest_dtype.is_int()
              && dest_dtype.is_signed()
              && dest_dtype.get_bits() > src_dtype.get_bits()
            {
              // perform sext
              format!("{{{}'{{{}[{}]}}, {}}}", pad, a, src_dtype.get_bits() - 1, a)
            } else {
              format!("{{{}'b0, {}}}", pad, a)
            }
          }
        }
      }

      Opcode::Select => {
        let select = expr.as_sub::<instructions::Select>().unwrap();
        let cond = dump_ref!(self.sys, &select.cond());
        let true_value = dump_ref!(self.sys, &select.true_value());
        let false_value = dump_ref!(self.sys, &select.false_value());
        format!("{} ? {} : {}", cond, true_value, false_value)
      }

      Opcode::Bind => {
        // currently handled in AsyncCall
        "".to_string()
      }

      Opcode::Select1Hot => {
        let dbits = expr.dtype().get_bits();
        let select1hot = expr.as_sub::<instructions::Select1Hot>().unwrap();
        let cond = dump_ref!(self.sys, &select1hot.cond());
        select1hot
          .value_iter()
          .enumerate()
          .map(|(i, elem)| {
            let value = dump_ref!(self.sys, &elem);
            format!("({{{}{{{}[{}] == 1'b1}}}} & {})", dbits, cond, i, value)
          })
          .collect::<Vec<_>>()
          .join(" | ")
      }

      _ => panic!("Unknown OP: {:?}", expr.get_opcode()),
    };

    if let Some((id, ty)) = decl {
      format!("{}  assign {} = {};\n", declare_logic(ty, &id), id, body)
    } else {
      body
    }
    .into()
  }
}

pub fn generate_cpp_testbench(dir: &Path, sys: &SysBuilder, config: &Config) -> io::Result<()> {
  if matches!(config.verilog, Simulator::Verilator) {
    let main_fname = dir.join("main.cpp");
    let mut main_fd = File::create(main_fname)?;
    main_fd.write_all(include_str!("main.cpp").as_bytes())?;
    let make_fname = dir.join("Makefile");
    let mut make_fd = File::create(make_fname).unwrap();
    make_fd.write_all(format!(include_str!("Makefile"), sys.get_name()).as_bytes())?;
  }
  Ok(())
}

pub fn elaborate(sys: &SysBuilder, config: &Config) -> Result<(), Error> {
  if matches!(config.verilog, Simulator::None) {
    return Err(Error::new(
      io::ErrorKind::Other,
      "No simulator specified for verilog generation",
    ));
  }

  create_and_clean_dir(config.dirname(sys, "verilog"), config.override_dump);
  let verilog_name = config.dirname(sys, "verilog");
  let fname = verilog_name.join(format!("{}.sv", sys.get_name()));

  eprintln!("Writing verilog rtl to {}", fname.to_str().unwrap());

  generate_cpp_testbench(&verilog_name, sys, config)?;

  let mut vd = VerilogDumper::new(sys, config);

  let mut fd = File::create(fname)?;

  for module in vd.sys.module_iter(ModuleKind::Module) {
    fd.write_all(vd.visit_module(module).unwrap().as_bytes())?;
  }

  vd.dump_runtime(fd, config.sim_threshold)?;

  Ok(())
}
