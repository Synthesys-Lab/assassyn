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
  backend::common::{create_and_clean_dir, Config},
  builder::system::{ModuleKind, SysBuilder},
  ir::{node::*, visitor::Visitor, *},
};

use self::{expr::subcode, module::Attribute};

use super::{gather::Gather, Simulator};

fn namify(name: &str) -> String {
  name.replace('.', "_")
}

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
  simulator: Simulator,
}

impl<'a, 'b> VerilogDumper<'a, 'b> {
  fn new(sys: &'a SysBuilder, config: &'b Config, simulator: Simulator) -> Self {
    Self {
      sys,
      config,
      pred_stack: VecDeque::new(),
      fifo_pushes: HashMap::new(),
      array_stores: HashMap::new(),
      triggers: HashMap::new(),
      current_module: String::new(),
      simulator,
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
          .cloned()
          .collect::<Vec<_>>()
          .join(" && "),
      ))
    }
  }

  fn dump_array(&self, array: &ArrayRef, mem_init_path: Option<&String>) -> String {
    let mut res = String::new();
    let array_name = namify(array.get_name());
    let scalar_bits = array.scalar_ty().get_bits();
    res.push_str(&format!(
      "
  // array: {name}[{size}]
  logic [{ty_bits}:0] array_{name}_q[0:{decl_size}];
",
      name = array_name,
      size = array.get_size(),
      ty_bits = scalar_bits - 1,
      decl_size = array.get_size() - 1
    ));

    let drivers = array
      .users()
      .iter()
      .map(|x| {
        x.as_ref::<Operand>(array.sys)
          .unwrap()
          .get_expr()
          .get_block()
          .get_module()
      })
      .collect::<HashSet<_>>()
      .into_iter()
      .map(|x| namify(x.as_ref::<Module>(array.sys).unwrap().get_name()))
      .collect::<HashSet<_>>();

    let scalar_bits = array.scalar_ty().get_bits();
    let decl_bits = scalar_bits - 1;
    let idx_bits = array.get_size().ilog2();
    drivers.iter().for_each(|driver| {
      res.push_str(&format!(
        "
  logic [{decl_bits}:0] array_{name}_driver_{driver}_d;
  logic [{idx_bits}:0] array_{name}_driver_{driver}_widx;
  logic array_{name}_driver_{driver}_w;
",
        name = array_name,
        driver = driver,
      ))
    });

    res.push_str(&format!("
  logic [{decl_bits}:0] array_{array_name}_d;
  assign array_{array_name}_d = \n{};
  logic [{idx_bits}:0] array_{array_name}_widx;
  assign array_{array_name}_widx = \n{};
  logic array_{array_name}_w;
  assign array_{array_name}_w = \n{};
",
      // FIXME(@were): Make sure these drivers are write-only ones?
      drivers // one-hot select driver write-data
        .iter()
        .map(|driver| format!(
          "  ({{{scalar_bits}{{array_{name}_driver_{driver}_w}}}} & array_{name}_driver_{driver}_d)",
          name = array_name,
        ))
        .collect::<Vec<String>>()
        .join("|"),
      drivers // one-hot select driver write-index
        .iter()
        .map(|driver| format!(
          "  ({{{}{{array_{}_driver_{}_w}}}} & array_{}_driver_{}_widx)",
          array.get_size().ilog2() + 1,
          array_name,
          driver,
          array_name,
          driver
        ))
        .collect::<Vec<String>>()
        .join(" |\n"),
      drivers // gather all the write-enable signals
        .iter()
        .map(|driver| format!("array_{}_driver_{}_w", array_name, driver))
        .collect::<Vec<String>>()
        .join(" | ")
    ));
    res.push_str("always_ff @(posedge clk or negedge rst_n)\n");
    if mem_init_path.is_some() {
      res.push_str(&format!(
        "if (!rst_n) $readmemh(\"{}\", array_{array_name}_q);\n",
        mem_init_path.unwrap(),
      ));
    } else if let Some(initializer) = array.get_initializer() {
      res.push_str("if (!rst_n) begin\n");
      for (idx, _) in initializer.iter().enumerate() {
        let elem_init = initializer[idx]
          .as_ref::<IntImm>(self.sys)
          .unwrap()
          .get_value();
        res.push_str(&format!(
          "    array_{array_name}_q[{idx}] <= {elem_init};\n"
        ));
      }
      res.push_str("end\n");
    } else {
      res.push_str(&format!(
        "if (!rst_n) array_{array_name}_q <= '{{default : {scalar_bits}'d0}};\n",
      ));
    }
    res.push_str(&format!(
      "else if (array_{n}_w) array_{n}_q[array_{n}_widx] <= array_{n}_d;\n",
      n = array_name,
    ));

    res.push('\n');

    res
  }

  fn dump_fifo(&self, fifo: &FIFORef) -> String {
    let mut res = String::new();
    let fifo_name = namify(&format!(
      "{}_{}",
      fifo.get_module().get_name(),
      fifo_name!(fifo)
    ));
    let fifo_width = fifo.scalar_ty().get_bits();
    res.push_str(&format!("// fifo: {fifo_name}\n"));

    let drivers = fifo
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
      .collect::<HashSet<_>>();

    let mut pusher_valid = vec![];
    let mut pusher_data = vec![];
    let mut pusher_ready = vec![];
    drivers.iter().for_each(|x| {
      let name = &fifo_name;
      let module = x.as_ref::<Module>(self.sys).unwrap();
      let driver = namify(module.get_name());
      res.push_str(&format!(
        "
  logic fifo_{name}_driver_{driver}_push_valid;\n
  logic [{ty_width}:0] fifo_{name}_driver_{driver}_push_data;\n
  logic fifo_{name}_driver_{driver}_push_ready;\n",
        driver = driver,
        ty_width = fifo_width - 1
      ));
      pusher_valid.push(format!("  fifo_{name}_driver_{driver}_push_valid"));
      pusher_data.push(format!(
          "  ({{{fifo_width}{{fifo_{name}_driver_{driver}_push_valid}}}} & fifo_{name}_driver_{driver}_push_data)"));
      pusher_ready.push(
        format!("  assign fifo_{name}_driver_{driver}_push_ready = fifo_{name}_push_ready;")
      );
    });

    res.push_str(&format!(
      "
  logic fifo_{name}_push_valid;
  assign fifo_{name}_push_valid = {pusher_valid};
  logic fifo_{name}_push_ready;
  {pusher_readiness}
  logic fifo_{name}_pop_valid;
  logic [{ty_width}:0] fifo_{name}_push_data;
  assign fifo_{name}_push_data = {pusher_data};
  logic [{ty_width}:0] fifo_{name}_pop_data;
  logic fifo_{name}_pop_ready;\n",
      name = fifo_name,
      ty_width = fifo_width - 1,
      pusher_readiness = pusher_ready.join("\n"),
      pusher_valid = pusher_valid.join(" | "),
      pusher_data = pusher_data.join("\n")
    ));

    res.push_str(&format!(
      "
  fifo #({width}) fifo_{name}_i (
    .clk(clk),
    .rst_n(rst_n),
    .push_valid(fifo_{name}_push_valid),
    .push_data(fifo_{name}_push_data),
    .push_ready(fifo_{name}_push_ready),
    .pop_valid(fifo_{name}_pop_valid),
    .pop_data(fifo_{name}_pop_data),
    .pop_ready(fifo_{name}_pop_ready));\n",
      name = fifo_name,
      width = fifo_width,
    ));

    res
  }

  fn dump_trigger(&self, module: &ModuleRef) -> String {
    let mut res = String::new();
    let module_name = namify(module.get_name());
    res.push_str(&format!("// {} trigger\n", module_name));
    if module_name != "driver" && module_name != "testbench" {
      module.callers().for_each(|x| {
        let driver = namify(x.get_name());
        res.push_str(&format!(
          "
  logic {module}_driver_{driver}_trigger_push_valid;
  logic {module}_driver_{driver}_trigger_push_ready;\n",
          module = module_name,
          driver = driver
        ));
      });
    }
    res.push_str(&format!("  logic {}_trigger_push_valid;\n", module_name));
    if module_name != "driver" && module_name != "testbench" {
      res.push_str(&format!(
        "  assign {}_trigger_push_valid = {};\n",
        module_name,
        module
          .callers()
          .map(|x| {
            let driver = namify(x.get_name());
            format!("  {}_driver_{}_trigger_push_valid", module_name, driver)
          })
          .collect::<Vec<String>>()
          .join(" |\n")
      ));
    }
    res.push_str(&format!("  logic {}_trigger_push_ready;\n", module_name));
    if module_name != "driver" && module_name != "testbench" {
      module.callers().for_each(|x| {
        let driver = namify(x.get_name());
        res.push_str(&format!(
          "  assign {}_driver_{}_trigger_push_ready = {}_trigger_push_ready;\n",
          module_name, driver, module_name
        ));
      });
    }
    res.push_str(&format!(
      "
  logic {module}_trigger_pop_valid;
  logic {module}_trigger_pop_ready;
  fifo #(1) {module}_trigger_i (
    .clk(clk),
    .rst_n(rst_n),
    .push_valid({module}_trigger_push_valid),
    .push_data(1'b1),
    .push_ready({module}_trigger_push_ready),
    .pop_valid({module}_trigger_pop_valid),
    .pop_data(),
    .pop_ready({module}_trigger_pop_ready));",
      module = module_name
    ));
    res
  }

  fn dump_module_inst(&self, module: &ModuleRef) -> String {
    let mut res = String::new();
    let module_name = namify(module.get_name());
    res.push_str(&format!(
      "
  // {module}
  {module} {module}_i (
    .clk(clk),
    .rst_n(rst_n),",
      module = module_name
    ));
    for port in module.fifo_iter() {
      let port_name = fifo_name!(port);
      res.push_str(&format!(
        "
    .fifo_{fifo}_pop_valid(fifo_{module}_{fifo}_pop_valid),
    .fifo_{fifo}_pop_data(fifo_{module}_{fifo}_pop_data),
    .fifo_{fifo}_pop_ready(fifo_{module}_{fifo}_pop_ready),",
        fifo = port_name,
        module = module_name
      ));
    }
    for (interf, _) in module.ext_interf_iter() {
      match interf.get_kind() {
        NodeKind::FIFO => {
          let fifo = interf.as_ref::<FIFO>(self.sys).unwrap();
          let fifo_name = namify(&format!(
            "{}_{}",
            fifo.get_module().get_name(),
            fifo_name!(fifo)
          ));
          res.push_str(&format!(
            "
    .fifo_{fifo}_push_valid(fifo_{fifo}_driver_{module}_push_valid),
    .fifo_{fifo}_push_data(fifo_{fifo}_driver_{module}_push_data),
    .fifo_{fifo}_push_ready(fifo_{fifo}_driver_{module}_push_ready),\n",
            fifo = fifo_name,
            module = module_name
          ));
        }
        NodeKind::Array => {
          let array_ref = interf.as_ref::<Array>(self.sys).unwrap();
          let array_name = namify(array_ref.get_name());
          res.push_str(&format!(
            "
    .array_{name}_q(array_{name}_q),
    .array_{name}_w(array_{name}_driver_{module}_w),
    .array_{name}_widx(array_{name}_driver_{module}_widx),
    .array_{name}_d(array_{name}_driver_{module}_d),\n",
            name = array_name,
            module = module_name
          ));
        }
        NodeKind::Module | NodeKind::Expr => {
          // TODO(@were): Skip this for now. I am 100% sure we need this later.
        }
        _ => panic!("Unknown interf kind {:?}", interf.get_kind()),
      }
    }

    let trigger_modules = get_triggered_modules(module);

    for trigger_module in trigger_modules {
      res.push_str(&format!(
        "
    .{trigger}_trigger_push_valid({trigger}_driver_{module}_trigger_push_valid),
    .{trigger}_trigger_push_ready({trigger}_driver_{module}_trigger_push_ready),",
        trigger = trigger_module,
        module = module_name
      ));
    }
    res.push_str(&format!(
      "
    .trigger_pop_valid({module}_trigger_pop_valid),
    .trigger_pop_ready({module}_trigger_pop_ready));\n",
      module = module_name
    ));
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

    if self.sys.has_testbench() {
      res.push_str("  assign testbench_trigger_push_valid = 1'b1;\n\n");
    }
    if self.sys.has_driver() {
      res.push_str("  assign driver_trigger_push_valid = 1'b1;\n\n");
    }

    // module insts
    for module in self.sys.module_iter(ModuleKind::Module) {
      res.push_str(&self.dump_module_inst(&module));
    }

    res.push_str("endmodule // top\n\n");

    fd.write_all(res.as_bytes()).unwrap();

    let init = match self.simulator {
      Simulator::VCS => {
        "
initial begin
  $fsdbDumpfile(\"wave.fsdb\");
  $fsdbDumpvars();
  $fsdbDumpMDA();
end"
      }
      Simulator::Verilator => "",
    };

    fd.write_all(include_str!("fifo_impl.sv").as_bytes())
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

fn get_triggered_modules(m: &ModuleRef<'_>) -> HashSet<String> {
  m.callees()
    .map(|x| namify(x.get_name()))
    .collect::<HashSet<_>>()
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
      let bits = port.scalar_ty().get_bits();
      let name = fifo_name!(port);
      res.push_str(&format!(
        "
  // port {name}
  input logic fifo_{name}_pop_valid,
  input logic [{ty_bits}:0] fifo_{name}_pop_data,
  output logic fifo_{name}_pop_ready,
",
        name = name,
        ty_bits = bits - 1
      ));
    }

    for (interf, _ops) in module.ext_interf_iter() {
      match interf.get_kind() {
        NodeKind::FIFO => {
          let fifo = interf.as_ref::<FIFO>(self.sys).unwrap();
          let fifo_name = format!(
            "{}_{}",
            namify(fifo.get_module().get_name()),
            fifo_name!(fifo)
          );
          res.push_str(&format!(
            "
  // port {name}
  output logic fifo_{name}_push_valid,
  output logic [{ty_bits}:0] fifo_{name}_push_data,
  input logic fifo_{name}_push_ready,
",
            name = fifo_name,
            ty_bits = fifo.scalar_ty().get_bits() - 1
          ));
        }
        NodeKind::Array => {
          let array_ref = interf.as_ref::<Array>(self.sys).unwrap();
          let name = namify(array_ref.get_name());
          res.push_str(&format!(
            "
  // array {name}
  input logic [{ty_bits}:0] array_{name}_q[0:{size}],
  output logic array_{name}_w,
  output logic [{idx_size}:0] array_{name}_widx,
  output logic [{ty_bits}:0] array_{name}_d,
",
            name = name,
            ty_bits = array_ref.scalar_ty().get_bits() - 1,
            size = array_ref.get_size() - 1,
            idx_size = array_ref.get_size().ilog2()
          ));
        }
        NodeKind::Module | NodeKind::Expr => {
          // Handled somewhere else. See `callers` and `callees`.
        }
        _ => panic!("Unknown interf kind {:?}", interf.get_kind()),
      }
      res.push('\n');
    }

    let trigger_modules = get_triggered_modules(&module);
    for trigger_module in trigger_modules {
      res.push_str(&format!(
        "
  output logic {}_trigger_push_valid,
  input logic {}_trigger_push_ready,
",
        trigger_module, trigger_module
      ));
    }

    res.push_str(
      "
  // trigger
  input logic trigger_pop_valid,
  output logic trigger_pop_ready
);\n",
    );

    let mut wait_until: Option<String> = None;

    let skip = if let Some(wu_intrin) = module.get_body().get_wait_until() {
      let mut skip = 0;
      let body = module.get_body();
      let body_iter = body.body_iter();
      for (i, elem) in body_iter.enumerate() {
        if elem == wu_intrin {
          skip = i + 1;
          break;
        }
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
      let bi = wu_intrin
        .as_expr::<instructions::BlockIntrinsic>(self.sys)
        .unwrap();
      let value = bi.value();
      wait_until = Some(format!(
        " && ({}{})",
        namify(&value.to_string(self.sys)),
        if value.get_dtype(self.sys).unwrap().get_bits() == 1 {
          "".into()
        } else {
          " != '0".to_string()
        }
      ));
      skip
    } else {
      0
    };

    res.push_str(
      "
  logic trigger;
  assign trigger_pop_ready = trigger;
",
    );

    if self.current_module == "testbench" {
      res.push_str(
        "
  int cycle_cnt;
  always_ff @(posedge clk or negedge rst_n) if (!rst_n) cycle_cnt <= 0;
  else if (trigger) cycle_cnt <= cycle_cnt + 1;
",
      );
    }

    self.fifo_pushes.clear();
    self.array_stores.clear();
    self.triggers.clear();
    for elem in module.get_body().body_iter().skip(skip) {
      res.push_str(&match elem.get_kind() {
        NodeKind::Expr => {
          let expr = elem.as_ref::<Expr>(self.sys).unwrap();
          self.visit_expr(expr).unwrap()
        }
        NodeKind::Block => {
          let block = elem.as_ref::<Block>(self.sys).unwrap();
          self.visit_block(block).unwrap()
        }
        _ => {
          panic!("Unexpected reference type: {:?}", elem);
        }
      });
    }

    for (m, cond) in self.triggers.drain() {
      res.push_str(&format!(
        "  assign {}_trigger_push_valid = {};\n\n",
        m,
        cond.condition.and("trigger".into())
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
        cond = g.condition.and("trigger".into()),
        value = g.value
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
        cond = idx.condition.and("trigger".into()),
        idx = idx.value,
        data = data.value
      ));
    }

    // tie off array store port
    for (interf, ops) in module.ext_interf_iter() {
      if interf.get_kind() == NodeKind::Array {
        let array_ref = interf.as_ref::<Array>(self.sys).unwrap();
        let array_name = namify(array_ref.get_name());
        let read_only = !ops.iter().any(|x| {
          matches!(
            x.as_ref::<Operand>(self.sys)
              .unwrap()
              .get_user()
              .as_ref::<Expr>(self.sys)
              .unwrap()
              .get_opcode(),
            Opcode::Store
          )
        });
        if read_only {
          res.push_str(&format!(
            "  assign array_{name}_w = '0;\nassign array_{name}_d = '0;\nassign array_{name}_widx = '0;\n\n",
            name = array_name
          ));
        }
      }
    }

    res.push_str(&format!(
      "
  assign trigger = trigger_pop_valid{};
endmodule // {}
",
      wait_until.unwrap_or("".to_string()),
      self.current_module
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
      Some((
        namify(&expr.upcast().to_string(self.sys)),
        expr.dtype().get_bits() - 1,
      ))
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

      Opcode::Unary { .. } => {
        let uop = expr.as_sub::<instructions::Unary>().unwrap();
        format!("{}{}", uop.get_opcode(), dump_ref!(self.sys, &uop.x()))
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
        let name = namify(&expr.upcast().to_string(self.sys));
        let pop = expr.as_sub::<instructions::FIFOPop>().unwrap();
        let fifo = pop.fifo();
        let fifo_name = fifo_name!(fifo);
        format!(
          "
  logic [{}:0] {};
  assign {} = fifo_{}_pop_data;
  assign fifo_{}_pop_ready = trigger{};
",
          fifo.scalar_ty().get_bits() - 1,
          name,
          name,
          fifo_name,
          fifo_name,
          self
            .get_pred()
            .map(|p| format!(" && {}", p))
            .unwrap_or("".to_string())
        )
      }

      Opcode::Log => {
        let mut format_str = dump_ref!(self.sys, expr.operand_iter().next().unwrap().get_value());

        let re = Regex::new(r"\{(:.[bxXo]?)?\}").unwrap();

        let dtypes: Vec<_> = expr
          .operand_iter()
          .skip(1)
          .map(|elem| elem.get_value().get_dtype(self.sys).unwrap())
          .collect();

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
          "  always_ff @(posedge clk iff trigger{}) ",
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
        format!(
          "array_{}_q[{}]",
          namify(array_ref.get_name()),
          dump_ref!(self.sys, &array_idx)
        )
      }

      Opcode::Store => {
        let store = expr.as_sub::<instructions::Store>().unwrap();
        let (array_ref, array_idx) = (store.array(), store.idx());
        let array_name = namify(array_ref.get_name());
        let pred = self.get_pred().unwrap_or("".to_string());
        let idx = dump_ref!(self.sys, &array_idx);
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
              (
                Gather::new(pred.clone(), idx, idx_bits),
                Gather::new(pred, value, value_bits),
              ),
            );
          }
        }
        "".to_string()
      }

      Opcode::FIFOPush => {
        let push = expr.as_sub::<instructions::FIFOPush>().unwrap();
        let fifo = push.fifo();
        let fifo_name = format!(
          "{}_{}",
          namify(fifo.get_module().get_name()),
          fifo_name!(fifo)
        );
        let pred = self.get_pred().unwrap_or("".to_string());
        let value = dump_ref!(self.sys, &push.value());
        match self.fifo_pushes.get_mut(&fifo_name) {
          Some(fps) => fps.push(pred, value, fifo.scalar_ty().get_bits()),
          None => {
            self.fifo_pushes.insert(
              fifo_name.clone(),
              Gather::new(pred, value, fifo.scalar_ty().get_bits()),
            );
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
        match self.triggers.get_mut(&callee) {
          Some(trgs) => trgs.push(pred, "".into(), 0),
          None => {
            self
              .triggers
              .insert(callee, Gather::new(pred, "".into(), 0));
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

    if let Some((id, bits)) = decl {
      format!("  logic [{}:0] {}; assign {} = {};\n", bits, id, id, body)
    } else {
      body
    }
    .into()
  }
}

pub fn generate_cpp_testbench(
  dir: &Path,
  sys: &SysBuilder,
  simulator: &Simulator,
) -> io::Result<()> {
  if matches!(simulator, Simulator::Verilator) {
    let main_fname = dir.join("main.cpp");
    let mut main_fd = File::create(main_fname)?;
    main_fd.write_all(include_str!("main.cpp").as_bytes())?;
    let make_fname = dir.join("Makefile");
    let mut make_fd = File::create(make_fname).unwrap();
    make_fd.write_all(format!(include_str!("Makefile"), sys.get_name()).as_bytes())?;
  }
  Ok(())
}

pub fn elaborate(sys: &SysBuilder, config: &Config, simulator: Simulator) -> Result<(), Error> {
  create_and_clean_dir(config.dirname(sys, "verilog"), config.override_dump);
  let verilog_name = config.dirname(sys, "verilog");
  let fname = verilog_name.join(format!("{}.sv", sys.get_name()));

  eprintln!("Writing verilog rtl to {}", fname.to_str().unwrap());

  generate_cpp_testbench(&verilog_name, sys, &simulator)?;

  let mut vd = VerilogDumper::new(sys, config, simulator);

  let mut fd = File::create(fname)?;

  for module in vd.sys.module_iter(ModuleKind::Module) {
    fd.write_all(vd.visit_module(module).unwrap().as_bytes())?;
  }

  vd.dump_runtime(fd, config.sim_threshold)?;

  Ok(())
}
