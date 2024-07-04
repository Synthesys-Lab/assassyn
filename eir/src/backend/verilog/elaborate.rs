use std::{
  collections::{HashMap, HashSet, VecDeque},
  fs::File,
  io::{Error, Write},
  path::PathBuf,
};

use crate::{
  backend::common::{create_and_clean_dir, Config},
  builder::system::SysBuilder,
  ir::{node::*, visitor::Visitor, *},
};

use self::{expr::subcode, module::Attribute};

use super::Simulator;

fn namify(name: &str) -> String {
  name.replace(".", "_")
}

macro_rules! fifo_name {
  ($fifo:expr) => {{
    format!("{}", namify($fifo.get_name()))
  }};
}

struct VerilogDumper<'a, 'b> {
  sys: &'a SysBuilder,
  config: &'b Config,
  indent: usize,
  pred_stack: VecDeque<String>,
  fifo_pushes: HashMap<String, Vec<(String, String)>>, // fifo_name -> [(pred, value)]
  array_stores: HashMap<String, Vec<(String, String, String)>>, // array_name -> [(pred, idx, value)]
  triggers: HashMap<String, Vec<String>>,                       // module_name -> [pred]
  current_module: String,
  has_testbench: bool,
  has_driver: bool,
  trigger_drivers: HashMap<String, HashSet<String>>, // module_name -> {driver module}
  array_drivers: HashMap<String, HashSet<String>>,   // array -> {driver module}
  fifo_drivers: HashMap<String, HashSet<String>>,    // fifo -> {driver module}
  simulator: Simulator,
}

impl<'a, 'b> VerilogDumper<'a, 'b> {
  fn new(sys: &'a SysBuilder, config: &'b Config, simulator: Simulator) -> Self {
    Self {
      sys,
      config,
      indent: 0,
      pred_stack: VecDeque::new(),
      fifo_pushes: HashMap::new(),
      array_stores: HashMap::new(),
      triggers: HashMap::new(),
      current_module: String::new(),
      has_testbench: false,
      has_driver: false,
      trigger_drivers: HashMap::new(),
      array_drivers: HashMap::new(),
      fifo_drivers: HashMap::new(),
      simulator,
    }
  }

  fn get_pred(&self) -> Option<String> {
    if self.pred_stack.len() == 0 {
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
    res.push_str(format!("// array: {}[{}]\n", array_name, array.get_size()).as_str());
    res.push_str(
      format!(
        "logic [{}:0] array_{}_q[0:{}];\n",
        array.scalar_ty().get_bits() - 1,
        array_name,
        array.get_size() - 1
      )
      .as_str(),
    );

    for driver in self.array_drivers.get(&array_name).unwrap().into_iter() {
      res.push_str(
        format!(
          "logic [{}:0] array_{}_driver_{}_d;\n",
          array.scalar_ty().get_bits() - 1,
          array_name,
          driver
        )
        .as_str(),
      );
      res.push_str(
        format!(
          "logic [{}:0] array_{}_driver_{}_widx;\n",
          (array.get_size() + 1).ilog2() - 1,
          array_name,
          driver
        )
        .as_str(),
      );
      res.push_str(format!("logic array_{}_driver_{}_w;\n", array_name, driver).as_str());
    }

    res.push_str(
      format!(
        "logic [{}:0] array_{}_d;\n",
        array.scalar_ty().get_bits() - 1,
        array_name
      )
      .as_str(),
    );
    res.push_str(
      format!(
        "assign array_{}_d = \n{};\n",
        array_name,
        self
          .array_drivers
          .get(&array_name)
          .unwrap()
          .into_iter()
          .map(|driver| format!(
            "  ({{{}{{array_{}_driver_{}_w}}}} & array_{}_driver_{}_d)",
            array.scalar_ty().get_bits(),
            array_name,
            driver,
            array_name,
            driver
          ))
          .collect::<Vec<String>>()
          .join(" |\n")
      )
      .as_str(),
    );
    res.push_str(
      format!(
        "logic [{}:0] array_{}_widx;\n",
        (array.get_size() + 1).ilog2() - 1,
        array_name
      )
      .as_str(),
    );
    res.push_str(
      format!(
        "assign array_{}_widx = \n{};\n",
        array_name,
        self
          .array_drivers
          .get(&array_name)
          .unwrap()
          .into_iter()
          .map(|driver| format!(
            "  ({{{}{{array_{}_driver_{}_w}}}} & array_{}_driver_{}_widx)",
            (array.get_size() + 1).ilog2(),
            array_name,
            driver,
            array_name,
            driver
          ))
          .collect::<Vec<String>>()
          .join(" |\n")
      )
      .as_str(),
    );
    res.push_str(format!("logic array_{}_w;\n", array_name).as_str());
    res.push_str(
      format!(
        "assign array_{}_w = {};\n",
        array_name,
        self
          .array_drivers
          .get(&array_name)
          .unwrap()
          .into_iter()
          .map(|driver| format!("array_{}_driver_{}_w", array_name, driver))
          .collect::<Vec<String>>()
          .join(" | ")
      )
      .as_str(),
    );
    res.push_str("always_ff @(posedge clk or negedge rst_n)\n");
    if mem_init_path.is_some() {
      res.push_str(
        format!(
          "if (!rst_n) $readmemh(\"{}\", array_{}_q);\n",
          mem_init_path.unwrap(),
          array_name
        )
        .as_str(),
      );
    } else {
      res.push_str(
        format!(
          "if (!rst_n) array_{}_q <= '{{default : {}'d0}};\n",
          array_name,
          array.scalar_ty().get_bits()
        )
        .as_str(),
      );
    }
    res.push_str(&format!(
      "else if (array_{}_w) array_{}_q[array_{}_widx] <= array_{}_d;\n",
      array_name, array_name, array_name, array_name
    ));

    res.push_str("\n");

    res
  }

  fn dump_fifo(&self, fifo: &FIFORef) -> String {
    let mut res = String::new();
    let fifo_name = namify(
      format!(
        "{}_{}",
        fifo
          .get_parent()
          .as_ref::<Module>(self.sys)
          .unwrap()
          .get_name(),
        fifo_name!(fifo)
      )
      .as_str(),
    );
    let fifo_width = fifo.scalar_ty().get_bits();
    res.push_str(format!("// fifo: {}\n", fifo_name).as_str());
    for driver in self.fifo_drivers.get(&fifo_name).unwrap().into_iter() {
      res.push_str(format!("logic fifo_{}_driver_{}_push_valid;\n", fifo_name, driver).as_str());
      res.push_str(
        format!(
          "logic [{}:0] fifo_{}_driver_{}_push_data;\n",
          fifo_width - 1,
          fifo_name,
          driver
        )
        .as_str(),
      );
      res.push_str(format!("logic fifo_{}_driver_{}_push_ready;\n", fifo_name, driver).as_str());
    }
    res.push_str(format!("logic fifo_{}_push_valid;\n", fifo_name).as_str());
    res.push_str(
      format!(
        "assign fifo_{}_push_valid = {};\n",
        fifo_name,
        self
          .fifo_drivers
          .get(&fifo_name)
          .unwrap()
          .into_iter()
          .map(|driver| format!("fifo_{}_driver_{}_push_valid", fifo_name, driver))
          .collect::<Vec<String>>()
          .join(" | ")
      )
      .as_str(),
    );
    res.push_str(
      format!(
        "logic [{}:0] fifo_{}_push_data;\n",
        fifo_width - 1,
        fifo_name
      )
      .as_str(),
    );
    res.push_str(
      format!(
        "assign fifo_{}_push_data = \n{};\n",
        fifo_name,
        self
          .fifo_drivers
          .get(&fifo_name)
          .unwrap()
          .into_iter()
          .map(|driver| format!(
            "  ({{{}{{fifo_{}_driver_{}_push_valid}}}} & fifo_{}_driver_{}_push_data)",
            fifo_width, fifo_name, driver, fifo_name, driver
          ))
          .collect::<Vec<String>>()
          .join(" |\n")
      )
      .as_str(),
    );
    res.push_str(format!("logic fifo_{}_push_ready;\n", fifo_name).as_str());
    for driver in self.fifo_drivers.get(&fifo_name).unwrap().into_iter() {
      res.push_str(
        format!(
          "assign fifo_{}_driver_{}_push_ready = fifo_{}_push_ready;\n",
          fifo_name, driver, fifo_name
        )
        .as_str(),
      );
    }
    res.push_str(format!("logic fifo_{}_pop_valid;\n", fifo_name).as_str());
    res.push_str(
      format!(
        "logic [{}:0] fifo_{}_pop_data;\n",
        fifo_width - 1,
        fifo_name
      )
      .as_str(),
    );
    res.push_str(format!("logic fifo_{}_pop_ready;\n", fifo_name).as_str());
    res.push_str(format!("fifo #({}) fifo_{}_i (\n", fifo_width, fifo_name).as_str());
    res.push_str(format!("  .clk(clk),\n").as_str());
    res.push_str(format!("  .rst_n(rst_n),\n").as_str());
    res.push_str(format!("  .push_valid(fifo_{}_push_valid),\n", fifo_name).as_str());
    res.push_str(format!("  .push_data(fifo_{}_push_data),\n", fifo_name).as_str());
    res.push_str(format!("  .push_ready(fifo_{}_push_ready),\n", fifo_name).as_str());
    res.push_str(format!("  .pop_valid(fifo_{}_pop_valid),\n", fifo_name).as_str());
    res.push_str(format!("  .pop_data(fifo_{}_pop_data),\n", fifo_name).as_str());
    res.push_str(format!("  .pop_ready(fifo_{}_pop_ready)\n", fifo_name).as_str());
    res.push_str(format!(");\n\n").as_str());

    res
  }

  fn dump_trigger(&self, module: &ModuleRef) -> String {
    let mut res = String::new();
    let module_name = namify(module.get_name());
    res.push_str(format!("// {} trigger\n", module_name).as_str());
    if module_name != "driver" && module_name != "testbench" {
      for driver in self
        .trigger_drivers
        .get(&module_name)
        .unwrap_or_else(|| panic!("{} not found!", module_name))
        .into_iter()
      {
        res.push_str(
          format!(
            "logic {}_driver_{}_trigger_push_valid;\n",
            module_name, driver
          )
          .as_str(),
        );
        res.push_str(
          format!(
            "logic {}_driver_{}_trigger_push_ready;\n",
            module_name, driver
          )
          .as_str(),
        );
      }
    }
    res.push_str(format!("logic {}_trigger_push_valid;\n", module_name).as_str());
    if module_name != "driver" && module_name != "testbench" {
      res.push_str(
        format!(
          "assign {}_trigger_push_valid = \n{};\n",
          module_name,
          self
            .trigger_drivers
            .get(&module_name)
            .unwrap()
            .into_iter()
            .map(|driver| format!("  {}_driver_{}_trigger_push_valid", module_name, driver))
            .collect::<Vec<String>>()
            .join(" |\n")
            .as_str()
        )
        .as_str(),
      );
    }
    res.push_str(format!("logic {}_trigger_push_ready;\n", module_name).as_str());
    if module_name != "driver" && module_name != "testbench" {
      for driver in self.trigger_drivers.get(&module_name).unwrap().into_iter() {
        res.push_str(
          format!(
            "assign {}_driver_{}_trigger_push_ready = {}_trigger_push_ready;\n",
            module_name, driver, module_name
          )
          .as_str(),
        );
      }
    }
    res.push_str(format!("logic {}_trigger_pop_valid;\n", module_name).as_str());
    res.push_str(format!("logic {}_trigger_pop_ready;\n", module_name).as_str());
    res.push_str(format!("fifo #(1) {}_trigger_i (\n", module_name).as_str());
    res.push_str(format!("  .clk(clk),\n").as_str());
    res.push_str(format!("  .rst_n(rst_n),\n").as_str());
    res.push_str(format!("  .push_valid({}_trigger_push_valid),\n", module_name).as_str());
    res.push_str(format!("  .push_data(1'b1),\n").as_str());
    res.push_str(format!("  .push_ready({}_trigger_push_ready),\n", module_name).as_str());
    res.push_str(format!("  .pop_valid({}_trigger_pop_valid),\n", module_name).as_str());
    res.push_str(format!("  .pop_data(),\n").as_str());
    res.push_str(format!("  .pop_ready({}_trigger_pop_ready)\n", module_name).as_str());
    res.push_str(format!(");\n\n").as_str());
    res
  }

  fn dump_module_inst(&self, module: &ModuleRef) -> String {
    let mut res = String::new();
    let module_name = namify(module.get_name());
    res.push_str(format!("// {}\n", module_name).as_str());
    res.push_str(format!("{} {}_i (\n", module_name, module_name).as_str());
    res.push_str(format!("  .clk(clk),\n").as_str());
    res.push_str(format!("  .rst_n(rst_n),\n").as_str());
    for port in module.port_iter() {
      let fifo_name = namify(
        format!(
          "{}_{}",
          port
            .get_parent()
            .as_ref::<Module>(self.sys)
            .unwrap()
            .get_name(),
          fifo_name!(port)
        )
        .as_str(),
      );
      res.push_str(
        format!(
          "  .fifo_{}_pop_valid(fifo_{}_pop_valid),\n",
          namify(port.get_name().as_str()),
          fifo_name
        )
        .as_str(),
      );
      res.push_str(
        format!(
          "  .fifo_{}_pop_data(fifo_{}_pop_data),\n",
          namify(port.get_name().as_str()),
          fifo_name
        )
        .as_str(),
      );
      res.push_str(
        format!(
          "  .fifo_{}_pop_ready(fifo_{}_pop_ready),\n",
          namify(port.get_name().as_str()),
          fifo_name
        )
        .as_str(),
      );
    }
    for (interf, _) in module.ext_interf_iter() {
      if interf.get_kind() == NodeKind::FIFO {
        let fifo = interf.as_ref::<FIFO>(self.sys).unwrap();
        let fifo_name = namify(
          format!(
            "{}_{}",
            fifo
              .get_parent()
              .as_ref::<Module>(self.sys)
              .unwrap()
              .get_name(),
            fifo_name!(fifo)
          )
          .as_str(),
        );
        res.push_str(
          format!(
            "  .fifo_{}_push_valid(fifo_{}_driver_{}_push_valid),\n",
            fifo_name, fifo_name, module_name
          )
          .as_str(),
        );
        res.push_str(
          format!(
            "  .fifo_{}_push_data(fifo_{}_driver_{}_push_data),\n",
            fifo_name, fifo_name, module_name
          )
          .as_str(),
        );
        res.push_str(
          format!(
            "  .fifo_{}_push_ready(fifo_{}_driver_{}_push_ready),\n",
            fifo_name, fifo_name, module_name
          )
          .as_str(),
        );
      } else if interf.get_kind() == NodeKind::Array {
        let array_ref = interf.as_ref::<Array>(self.sys).unwrap();
        res.push_str(
          format!(
            "  .array_{}_q(array_{}_q),\n",
            namify(array_ref.get_name()),
            namify(array_ref.get_name())
          )
          .as_str(),
        );
        res.push_str(
          format!(
            "  .array_{}_w(array_{}_driver_{}_w),\n",
            namify(array_ref.get_name()),
            namify(array_ref.get_name()),
            module_name
          )
          .as_str(),
        );
        res.push_str(
          format!(
            "  .array_{}_widx(array_{}_driver_{}_widx),\n",
            namify(array_ref.get_name()),
            namify(array_ref.get_name()),
            module_name
          )
          .as_str(),
        );
        res.push_str(
          format!(
            "  .array_{}_d(array_{}_driver_{}_d),\n",
            namify(array_ref.get_name()),
            namify(array_ref.get_name()),
            module_name
          )
          .as_str(),
        );
      } else {
        panic!("Unknown interf kind {:?}", interf.get_kind());
      }
    }

    let mut trigger_modules = get_triggered_modules(&module.upcast(), self.sys);
    trigger_modules.sort_unstable();
    trigger_modules.dedup();
    for trigger_module in trigger_modules {
      res.push_str(
        format!(
          "  .{}_trigger_push_valid({}_driver_{}_trigger_push_valid),\n",
          trigger_module, trigger_module, module_name
        )
        .as_str(),
      );
      res.push_str(
        format!(
          "  .{}_trigger_push_ready({}_driver_{}_trigger_push_ready),\n",
          trigger_module, trigger_module, module_name
        )
        .as_str(),
      );
    }
    res.push_str(format!("  .trigger_pop_valid({}_trigger_pop_valid),\n", module_name).as_str());
    res.push_str(format!("  .trigger_pop_ready({}_trigger_pop_ready)\n", module_name).as_str());
    res.push_str(format!(");\n\n").as_str());
    res
  }

  fn dump_runtime(
    self: VerilogDumper<'a, 'b>,
    mut fd: File,
    sim_threshold: usize,
  ) -> Result<(), Error> {
    // runtime
    let mut res = String::new();

    res.push_str("module top (\n");
    res.push_str("  input logic clk,\n");
    res.push_str("  input logic rst_n\n");
    res.push_str(");\n\n");

    // memory initializations map
    let mut mem_init_map: HashMap<BaseNode, String> = HashMap::new(); // array -> init_file_path
    for module in self.sys.module_iter() {
      for attr in module.get_attrs() {
        match attr {
          Attribute::Memory(param) => {
            if let Some(init_file) = &param.init_file {
              let mut init_file_path = self.config.resource_base.clone();
              init_file_path.push(init_file);
              let init_file_path = init_file_path.to_str().unwrap();
              let array = param.array.as_ref::<Array>(self.sys).unwrap();
              mem_init_map.insert(array.upcast(), init_file_path.to_string());
            }
          }
          _ => {}
        }
      }
    }

    // array storage element definitions
    for array in self.sys.array_iter() {
      res.push_str(
        self
          .dump_array(&array, mem_init_map.get(&array.upcast()))
          .as_str(),
      );
    }

    // fifo storage element definitions
    for module in self.sys.module_iter() {
      for fifo in module.port_iter() {
        res.push_str(self.dump_fifo(&fifo).as_str());
      }
    }

    // trigger fifo definitions
    for module in self.sys.module_iter() {
      res.push_str(self.dump_trigger(&module).as_str());
    }

    if self.has_testbench {
      res.push_str("assign testbench_trigger_push_valid = 1'b1;\n\n");
    }
    if self.has_driver {
      res.push_str("assign driver_trigger_push_valid = 1'b1;\n\n");
    }

    // module insts
    for module in self.sys.module_iter() {
      res.push_str(self.dump_module_inst(&module).as_str());
    }

    res.push_str("endmodule // top\n\n");

    fd.write(res.as_bytes()).unwrap();

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

    fd.write(
      format!(
        "
module fifo #(
    parameter WIDTH = 8
) (
  input logic clk,
  input logic rst_n,

  input  logic               push_valid,
  input  logic [WIDTH - 1:0] push_data,
  output logic               push_ready,

  output logic               pop_valid,
  output logic [WIDTH - 1:0] pop_data,
  input  logic               pop_ready
);

logic [WIDTH - 1:0] q[$];

always @(posedge clk or negedge rst_n) begin
  if (!rst_n) begin
    pop_valid <= 1'b0;
    pop_data <= 'x;
  end else begin
    if (pop_ready) q.pop_front();

    if (push_valid) q.push_back(push_data);

    if (q.size() == 0) begin
      pop_valid <= 1'b0;
      pop_data <= 'x;
    end else begin
      pop_valid <= 1'b1;
      pop_data <= q[0];
    end
  end
end

assign push_ready = 1'b1;

endmodule


module tb;

logic clk;
logic rst_n;

initial begin
  clk = 1'b1;
  rst_n = 1'b0;
  #1;
  rst_n = 1'b1;
  #{}00;
  $finish();
end

always #50 clk <= !clk;

{}

top top_i (
  .clk(clk),
  .rst_n(rst_n)
);

endmodule
",
        sim_threshold, init
      )
      .as_bytes(),
    )?;

    Ok(())
  }
}

fn get_triggered_modules(node: &BaseNode, sys: &SysBuilder) -> Vec<String> {
  let mut triggered_modules = Vec::<String>::new();
  match node.get_kind() {
    NodeKind::Module => {
      let module = node.as_ref::<Module>(sys).unwrap();
      for elem in module.get_body().body_iter() {
        if elem.get_kind() == NodeKind::Expr || elem.get_kind() == NodeKind::Block {
          triggered_modules.append(get_triggered_modules(&elem, sys).as_mut());
        }
      }
    }
    NodeKind::Block => {
      let block = node.as_ref::<Block>(sys).unwrap();
      for elem in block.body_iter() {
        if elem.get_kind() == NodeKind::Expr || elem.get_kind() == NodeKind::Block {
          triggered_modules.append(get_triggered_modules(&elem, sys).as_mut());
        }
      }
    }
    NodeKind::Expr => {
      let expr = node.as_ref::<Expr>(sys).unwrap();
      if matches!(expr.get_opcode(), Opcode::AsyncCall) {
        let call = expr.as_sub::<instructions::AsyncCall>().unwrap();
        // let triggered_module = {
        //   let bind = call.bind();
        //   bind.callee()
        // };
        triggered_modules.push(namify(call.bind().callee().get_name()));
      }
    }
    _ => {}
  }
  triggered_modules
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
      if immwidth {
        Some(format!(
          "{}'d{}",
          int_imm.dtype().get_bits(),
          int_imm.get_value()
        ))
      } else {
        Some(format!("{}", int_imm.get_value()))
      }
    }
    NodeKind::StrImm => {
      let str_imm = node.as_ref::<StrImm>(sys).unwrap();
      let value = str_imm.get_value();
      quote::quote!(#value).to_string().into()
    }
    NodeKind::Expr => Some(namify(node.to_string(sys).as_str())),
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
    if self.current_module == "testbench" {
      self.has_testbench = true;
    }

    if self.current_module == "driver" {
      self.has_driver = true;
    }

    let mut res = String::new();

    res.push_str(format!("module {} (\n", self.current_module).as_str());

    self.indent += 2;
    res.push_str(format!("{}input logic clk,\n", " ".repeat(self.indent)).as_str());
    res.push_str(format!("{}input logic rst_n,\n", " ".repeat(self.indent)).as_str());
    res.push_str("\n");
    for port in module.port_iter() {
      res.push_str(format!("{}// port {}\n", " ".repeat(self.indent), fifo_name!(port)).as_str());
      res.push_str(
        format!(
          "{}input logic fifo_{}_pop_valid,\n",
          " ".repeat(self.indent),
          fifo_name!(port)
        )
        .as_str(),
      );
      res.push_str(
        format!(
          "{}input logic [{}:0] fifo_{}_pop_data,\n",
          " ".repeat(self.indent),
          port.scalar_ty().get_bits() - 1,
          fifo_name!(port)
        )
        .as_str(),
      );
      res.push_str(
        format!(
          "{}output logic fifo_{}_pop_ready,\n\n",
          " ".repeat(self.indent),
          fifo_name!(port)
        )
        .as_str(),
      );
    }

    for (interf, _ops) in module.ext_interf_iter() {
      if interf.get_kind() == NodeKind::FIFO {
        let fifo = interf.as_ref::<FIFO>(self.sys).unwrap();
        let fifo_name = namify(
          format!(
            "{}_{}",
            fifo
              .get_parent()
              .as_ref::<Module>(self.sys)
              .unwrap()
              .get_name(),
            fifo_name!(fifo)
          )
          .as_str(),
        );
        res.push_str(format!("{}// port {}\n", " ".repeat(self.indent), fifo_name).as_str());
        res.push_str(
          format!(
            "{}output logic fifo_{}_push_valid,\n",
            " ".repeat(self.indent),
            fifo_name
          )
          .as_str(),
        );
        res.push_str(
          format!(
            "{}output logic [{}:0] fifo_{}_push_data,\n",
            " ".repeat(self.indent),
            fifo.scalar_ty().get_bits() - 1,
            fifo_name
          )
          .as_str(),
        );
        res.push_str(
          format!(
            "{}input logic fifo_{}_push_ready,\n",
            " ".repeat(self.indent),
            fifo_name
          )
          .as_str(),
        );
      } else if interf.get_kind() == NodeKind::Array {
        let array_ref = interf.as_ref::<Array>(self.sys).unwrap();
        res.push_str(
          format!(
            "{}// array {}\n",
            " ".repeat(self.indent),
            namify(array_ref.get_name())
          )
          .as_str(),
        );
        res.push_str(
          format!(
            "{}input logic [{}:0] array_{}_q[0:{}],\n",
            " ".repeat(self.indent),
            array_ref.scalar_ty().get_bits() - 1,
            namify(array_ref.get_name()),
            array_ref.get_size() - 1
          )
          .as_str(),
        );
        res.push_str(
          format!(
            "{}output logic array_{}_w,\n",
            " ".repeat(self.indent),
            namify(array_ref.get_name())
          )
          .as_str(),
        );
        res.push_str(
          format!(
            "{}output logic [{}:0] array_{}_widx,\n",
            " ".repeat(self.indent),
            (array_ref.get_size() + 1).ilog2() - 1,
            namify(array_ref.get_name())
          )
          .as_str(),
        );
        res.push_str(
          format!(
            "{}output logic [{}:0] array_{}_d,\n",
            " ".repeat(self.indent),
            array_ref.scalar_ty().get_bits() - 1,
            namify(array_ref.get_name())
          )
          .as_str(),
        );
      } else {
        panic!("Unknown interf kind {:?}", interf.get_kind());
      }
      res.push_str("\n");
    }

    let mut trigger_modules = get_triggered_modules(&module.upcast(), self.sys);
    trigger_modules.sort_unstable();
    trigger_modules.dedup();
    let mut has_trigger_modules = false;
    for trigger_module in trigger_modules {
      has_trigger_modules = true;
      res.push_str(
        format!(
          "{}output logic {}_trigger_push_valid,\n",
          " ".repeat(self.indent),
          trigger_module
        )
        .as_str(),
      );
      res.push_str(
        format!(
          "{}input logic {}_trigger_push_ready,\n",
          " ".repeat(self.indent),
          trigger_module
        )
        .as_str(),
      );
    }

    if has_trigger_modules {
      res.push_str("\n");
    }

    res.push_str(format!("{}// trigger\n", " ".repeat(self.indent)).as_str());
    res.push_str(
      format!(
        "{}input logic trigger_pop_valid,\n",
        " ".repeat(self.indent)
      )
      .as_str(),
    );
    res.push_str(
      format!(
        "{}output logic trigger_pop_ready\n",
        " ".repeat(self.indent)
      )
      .as_str(),
    );
    self.indent -= 2;
    res.push_str(");\n\n");

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
            res.push_str(self.visit_expr(expr).unwrap().as_str());
          }
          NodeKind::Block => {
            let block = elem.as_ref::<Block>(self.sys).unwrap();
            res.push_str(self.visit_block(block).unwrap().as_str());
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
        namify(value.to_string(self.sys).as_str()),
        if value.get_dtype(self.sys).unwrap().get_bits() == 1 {
          "".into()
        } else {
          format!(" != 0")
        }
      ));
      skip
    } else {
      0
    };

    res.push_str(format!("logic trigger;\n").as_str());
    res.push_str(format!("assign trigger_pop_ready = trigger;\n\n").as_str());

    if self.current_module == "testbench" {
      res.push_str("int cycle_cnt;\n");
      res.push_str("always_ff @(posedge clk or negedge rst_n) if (!rst_n) cycle_cnt <= 0; ");
      res.push_str("else if (trigger) cycle_cnt <= cycle_cnt + 1;\n\n");
    }

    self.fifo_pushes.clear();
    self.array_stores.clear();
    self.triggers.clear();
    for elem in module.get_body().body_iter().skip(skip) {
      match elem.get_kind() {
        NodeKind::Expr => {
          let expr = elem.as_ref::<Expr>(self.sys).unwrap();
          res.push_str(self.visit_expr(expr).unwrap().as_str());
        }
        NodeKind::Block => {
          let block = elem.as_ref::<Block>(self.sys).unwrap();
          res.push_str(self.visit_block(block).unwrap().as_str());
        }
        _ => {
          panic!("Unexpected reference type: {:?}", elem);
        }
      }
    }

    for (m, preds) in self.triggers.drain() {
      let mut valid_conds = Vec::<String>::new();
      let mut has_unconditional_branch = false;
      let mut has_conditional_branch = false;
      for p in preds {
        if p == "" {
          if has_unconditional_branch {
            panic!("multiple unconditional branches for trigger {}", m);
          }
          if has_conditional_branch {
            panic!(
              "mixed conditional and unconditional branches for trigger {}",
              m
            );
          }
          has_unconditional_branch = true;
        } else {
          if has_unconditional_branch {
            panic!(
              "mixed conditional and unconditional branches for trigger {}",
              m
            );
          }
          has_conditional_branch = true;
          valid_conds.push(p.clone());
        }
      }
      if has_conditional_branch {
        res.push_str(
          format!(
            "assign {}_trigger_push_valid = trigger && ({});\n\n",
            m,
            valid_conds.join(" || ")
          )
          .as_str(),
        );
      } else {
        res.push_str(format!("assign {}_trigger_push_valid = trigger;\n\n", m).as_str());
      }
    }

    for (f, branches) in self.fifo_pushes.drain() {
      let mut valid_conds = Vec::<String>::new();
      let mut data_str = String::new();
      let mut has_unconditional_branch = false;
      let mut has_conditional_branch = false;
      for (p, v) in branches {
        if p == "" {
          if has_unconditional_branch {
            panic!("multiple unconditional branches for fifo {}", f);
          }
          if has_conditional_branch {
            panic!(
              "mixed conditional and unconditional branches for fifo {}",
              f
            );
          }
          has_unconditional_branch = true;
          data_str.push_str(format!("{}", v).as_str());
        } else {
          if has_unconditional_branch {
            panic!(
              "mixed conditional and unconditional branches for fifo {}",
              f
            );
          }
          has_conditional_branch = true;
          valid_conds.push(p.clone());
          data_str.push_str(format!("{} ? {} : ", p, v).as_str());
        }
      }
      if has_conditional_branch {
        data_str.push_str(format!("'x").as_str());
      }
      if has_conditional_branch {
        res.push_str(
          format!(
            "assign fifo_{}_push_valid = trigger && ({});\n",
            f,
            valid_conds.join(" || ")
          )
          .as_str(),
        );
      } else {
        res.push_str(format!("assign fifo_{}_push_valid = trigger;\n", f).as_str());
      }
      res.push_str(format!("assign fifo_{}_push_data = {};\n\n", f, data_str).as_str());
    }

    for (a, branches) in self.array_stores.drain() {
      let mut w_conds = Vec::<String>::new();
      let mut idx_str = String::new();
      let mut d_str = String::new();
      let mut has_unconditional_branch = false;
      let mut has_conditional_branch = false;
      for (p, idx, v) in branches {
        if p == "" {
          if has_unconditional_branch {
            panic!("multiple unconditional branches for array {} store", a);
          }
          if has_conditional_branch {
            panic!(
              "mixed conditional and unconditional branches for array {} store",
              a
            );
          }
          has_unconditional_branch = true;
          d_str.push_str(format!("{}", v).as_str());
          idx_str.push_str(format!("{}", idx).as_str());
        } else {
          if has_unconditional_branch {
            panic!(
              "mixed conditional and unconditional branches for array {} store",
              a
            );
          }
          has_conditional_branch = true;
          w_conds.push(p.clone());
          d_str.push_str(format!("{} ? {} : ", p, v).as_str());
          idx_str.push_str(format!("{} ? {} : ", p, idx).as_str());
        }
      }
      if has_conditional_branch {
        d_str.push_str(format!("'x").as_str());
        idx_str.push_str(format!("'x").as_str());
      }
      if has_conditional_branch {
        res.push_str(
          format!(
            "assign array_{}_w = trigger && ({});\n",
            a,
            w_conds.join(" || ")
          )
          .as_str(),
        );
      } else {
        res.push_str(format!("assign array_{}_w = trigger;\n", a).as_str());
      }
      res.push_str(format!("assign array_{}_d = {};\n", a, d_str).as_str());
      res.push_str(format!("assign array_{}_widx = {};\n\n", a, idx_str).as_str());
    }

    // tie off array store port
    for (interf, _ops) in module.ext_interf_iter() {
      if interf.get_kind() == NodeKind::Array {
        let array_ref = interf.as_ref::<Array>(&self.sys).unwrap();
        let array_name = namify(array_ref.get_name());
        let mut read_only = false;
        match self.array_drivers.get_mut(&array_name) {
          Some(ads) => {
            if !ads.contains(&self.current_module) {
              read_only = true;
            }
            ads.insert(self.current_module.clone());
          }
          None => {
            read_only = true;
            self.array_drivers.insert(
              array_name.clone(),
              HashSet::from([self.current_module.clone()]),
            );
          }
        }
        if read_only {
          res.push_str(
            format!(
              "assign array_{}_w = '0;\nassign array_{}_d = '0;\nassign array_{}_widx = '0;\n\n",
              array_name, array_name, array_name
            )
            .as_str(),
          );
        }
      }
    }

    res.push_str(
      format!(
        "assign trigger = trigger_pop_valid{};\n\n",
        wait_until.unwrap_or("".to_string())
      )
      .as_str(),
    );

    res.push_str(format!("endmodule // {}\n\n\n", self.current_module).as_str());

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
          res.push_str(self.visit_expr(expr).unwrap().as_str());
        }
        NodeKind::Block => {
          let block = elem.as_ref::<Block>(self.sys).unwrap();
          res.push_str(self.visit_block(block).unwrap().as_str());
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
    match expr.get_opcode() {
      Opcode::Binary { .. } => {
        let name = namify(&expr.upcast().to_string(self.sys));
        let dbits = expr.dtype().get_bits() - 1;
        let bin = expr.as_sub::<instructions::Binary>().unwrap();
        Some(format!(
          "logic [{}:0] {};\nassign {} = {} {} {};\n\n",
          dbits,
          name,
          name,
          dump_ref!(self.sys, &bin.a()),
          bin.get_opcode().to_string(),
          dump_ref!(self.sys, &bin.b())
        ))
      }

      Opcode::Unary { .. } => {
        let name = namify(&expr.upcast().to_string(self.sys));
        let dbits = expr.dtype().get_bits() - 1;
        let uop = expr.as_sub::<instructions::Unary>().unwrap();
        Some(format!(
          "logic [{}:0] {};\nassign {} = {}{};\n\n",
          dbits,
          name,
          name,
          uop.get_opcode().to_string(),
          dump_ref!(self.sys, &uop.x())
        ))
      }

      Opcode::Compare { .. } => {
        let name = namify(&expr.upcast().to_string(self.sys));
        let dbits = expr.dtype().get_bits() - 1;
        let cmp = expr.as_sub::<instructions::Compare>().unwrap();
        Some(format!(
          "logic [{}:0] {};\nassign {} = {} {} {};\n\n",
          dbits,
          name,
          name,
          dump_ref!(self.sys, &cmp.a()),
          cmp.get_opcode().to_string(),
          dump_ref!(self.sys, &cmp.b())
        ))
      }

      Opcode::FIFOPop => {
        let name = namify(&expr.upcast().to_string(self.sys));
        let pop = expr.as_sub::<instructions::FIFOPop>().unwrap();
        let fifo = pop.fifo();
        Some(format!(
            "logic [{}:0] {};\nassign {} = fifo_{}_pop_data;\nassign fifo_{}_pop_ready = trigger{};\n\n",
            fifo.scalar_ty().get_bits() - 1,
            name,
            name,
            fifo_name!(fifo),
            fifo_name!(fifo),
            (self.get_pred().and_then(|p| Some(format!(" && {}", p)))).unwrap_or("".to_string())
          ))
      }

      Opcode::Log => {
        let mut format_str = dump_ref!(
          self.sys,
          expr
            .operand_iter()
            .collect::<Vec<OperandRef>>()
            .first()
            .unwrap()
            .get_value()
        );
        for elem in expr.operand_iter().skip(1) {
          format_str = format_str.replacen(
            "{}",
            match elem.get_value().get_dtype(self.sys).unwrap() {
              DataType::Int(_) | DataType::UInt(_) | DataType::Bits(_) => "%d",
              DataType::Str => "%s",
              _ => "?",
            },
            1,
          );
        }
        format_str = format_str.replace("\"", "");
        let mut res = String::new();
        res.push_str(
          format!(
            "always_ff @(posedge clk iff trigger{}) ",
            (self.get_pred().and_then(|p| Some(format!(" && {}", p)))).unwrap_or("".to_string())
          )
          .as_str(),
        );
        res.push_str("$display(\"%t\\t");
        res.push_str(format_str.as_str());
        res.push_str("\", $time, ");
        for elem in expr.operand_iter().skip(1) {
          res.push_str(format!("{}, ", dump_ref!(self.sys, elem.get_value())).as_str());
        }
        res.pop();
        res.pop();
        res.push_str(");\n");
        res.push_str("\n");
        Some(res)
      }

      Opcode::Load => {
        let dtype = expr.dtype();
        let name = namify(expr.upcast().to_string(self.sys).as_str());
        let load = expr.as_sub::<instructions::Load>().unwrap();
        let (array_ref, array_idx) = (load.array(), load.idx());
        Some(format!(
          "logic [{}:0] {};\nassign {} = array_{}_q[{}];\n\n",
          dtype.get_bits() - 1,
          name,
          name,
          namify(array_ref.get_name()),
          dump_ref!(self.sys, &array_idx)
        ))
      }

      Opcode::Store => {
        let store = expr.as_sub::<instructions::Store>().unwrap();
        let (array_ref, array_idx) = (store.array(), store.idx());
        let array_name = namify(array_ref.get_name());
        match self.array_drivers.get_mut(&array_name) {
          Some(ads) => {
            ads.insert(self.current_module.clone());
          }
          None => {
            self.array_drivers.insert(
              array_name.clone(),
              HashSet::from([self.current_module.clone()]),
            );
          }
        }
        let pred = self.get_pred().unwrap_or("".to_string());
        match self.array_stores.get_mut(&array_name) {
          Some(ass) => ass.push((
            pred,
            dump_ref!(self.sys, &array_idx),
            dump_ref!(self.sys, &store.value()),
          )),
          None => {
            self.array_stores.insert(
              array_name.clone(),
              vec![(
                pred,
                dump_ref!(self.sys, &array_idx),
                dump_ref!(self.sys, &store.value()),
              )],
            );
          }
        }
        Some("".to_string())
      }

      Opcode::FIFOPush => {
        let push = expr.as_sub::<instructions::FIFOPush>().unwrap();
        let fifo = push.fifo();
        let fifo_name = namify(
          format!(
            "{}_{}",
            fifo
              .get_parent()
              .as_ref::<Module>(self.sys)
              .unwrap()
              .get_name(),
            fifo_name!(fifo)
          )
          .as_str(),
        );
        match self.fifo_drivers.get_mut(&fifo_name) {
          Some(fds) => {
            fds.insert(self.current_module.clone());
          }
          None => {
            self.fifo_drivers.insert(
              fifo_name.clone(),
              HashSet::from([self.current_module.clone()]),
            );
          }
        }
        let pred = self.get_pred().unwrap_or("".to_string());
        match self.fifo_pushes.get_mut(&fifo_name) {
          Some(fps) => fps.push((pred, dump_ref!(self.sys, &push.value()))),
          None => {
            self.fifo_pushes.insert(
              fifo_name.clone(),
              vec![(pred, dump_ref!(self.sys, &push.value()))],
            );
          }
        }
        Some("".to_string())
      }

      Opcode::FIFOField { field } => {
        let name = namify(expr.upcast().to_string(self.sys).as_str());
        let get_field = expr.as_sub::<instructions::FIFOField>().unwrap();
        let fifo = get_field.fifo();
        let fifo_name = fifo_name!(fifo);
        match field {
          subcode::FIFO::Valid => Some(format!(
            "logic {};\nassign {} = fifo_{}_pop_valid;\n\n",
            name, name, fifo_name
          )),
          subcode::FIFO::Peek => Some(format!(
            "logic [{}:0] {};\nassign {} = fifo_{}_pop_data;\n\n",
            fifo.scalar_ty().get_bits() - 1,
            name,
            name,
            fifo_name
          )),
          subcode::FIFO::AlmostFull => todo!(),
        }
      }

      Opcode::AsyncCall => {
        let call = expr.as_sub::<instructions::AsyncCall>().unwrap();
        let module_name = {
          let bind = call.bind();
          bind.callee().get_name().to_string()
        };
        let module_name = namify(&module_name);
        match self.trigger_drivers.get_mut(&module_name) {
          Some(tds) => {
            tds.insert(self.current_module.clone());
          }
          None => {
            self.trigger_drivers.insert(
              module_name.clone(),
              HashSet::from([self.current_module.clone()]),
            );
          }
        }
        let pred = self.get_pred().unwrap_or("".to_string());
        match self.triggers.get_mut(&module_name) {
          Some(trgs) => trgs.push(pred),
          None => {
            self.triggers.insert(module_name.clone(), vec![pred]);
          }
        }
        Some("".to_string())
      }

      Opcode::Slice => {
        let dbits = expr.dtype().get_bits() - 1;
        let name = namify(expr.upcast().to_string(self.sys).as_str());
        let slice = expr.as_sub::<instructions::Slice>().unwrap();
        let a = dump_ref!(self.sys, &slice.x());
        let l = dump_ref!(self.sys, &slice.l_intimm().upcast());
        let r = dump_ref!(self.sys, &slice.r_intimm().upcast());
        Some(format!(
          "logic [{}:0] {};\nassign {} = {}[{}:{}];\n\n",
          dbits, name, name, a, r, l
        ))
      }

      Opcode::Concat => {
        let dbits = expr.dtype().get_bits() - 1;
        let name = namify(expr.upcast().to_string(self.sys).as_str());
        let concat = expr.as_sub::<instructions::Concat>().unwrap();
        let a = dump_ref_immwidth!(self.sys, &concat.msb());
        let b = dump_ref_immwidth!(self.sys, &concat.lsb());
        Some(format!(
          "logic [{}:0] {};\nassign {} = {{{}, {}}};\n\n",
          dbits, name, name, a, b
        ))
      }

      Opcode::Cast { .. } => {
        let dbits = expr.dtype().get_bits();
        let name = namify(expr.upcast().to_string(self.sys).as_str());
        let cast = expr.as_sub::<instructions::Cast>().unwrap();
        let a = dump_ref!(self.sys, &cast.x());
        let pad = dbits - cast.x().get_dtype(self.sys).unwrap().get_bits();
        match cast.get_opcode() {
          subcode::Cast::BitCast => Some(format!(
            "logic [{}:0] {};\nassign {} = {};\n\n",
            dbits - 1,
            name,
            name,
            a
          )),
          subcode::Cast::ZExt => Some(format!(
            "logic [{}:0] {};\nassign {} = {{{}'b0, {}}};\n\n",
            dbits - 1,
            name,
            name,
            pad,
            a
          )),
          subcode::Cast::SExt => {
            let src_dtype = cast.src_type();
            let dest_dtype = cast.dest_type();
            if src_dtype.is_int()
              && src_dtype.is_signed()
              && dest_dtype.is_int()
              && dest_dtype.is_signed()
              && dest_dtype.get_bits() > src_dtype.get_bits()
            {
              // perform sext
              Some(format!(
                "logic [{}:0] {};\nassign {} = {{{}'{{{}[{}]}}, {}}};\n\n",
                dbits,
                name,
                name,
                dest_dtype.get_bits() - src_dtype.get_bits(),
                a,
                src_dtype.get_bits() - 1,
                a
              ))
            } else {
              Some(format!(
                "logic [{}:0] {};\nassign {} = {{{}'b0, {}}};\n\n",
                dbits - 1,
                name,
                name,
                pad,
                a
              ))
            }
          }
        }
      }

      Opcode::Select => {
        let dbits = expr.dtype().get_bits() - 1;
        let name = namify(expr.upcast().to_string(self.sys).as_str());
        let select = expr.as_sub::<instructions::Select>().unwrap();
        let cond = dump_ref!(self.sys, &select.cond());
        let true_value = dump_ref!(self.sys, &select.true_value());
        let false_value = dump_ref!(self.sys, &select.false_value());
        Some(format!(
          "logic [{}:0] {};\nassign {} = {} ? {} : {};\n\n",
          dbits, name, name, cond, true_value, false_value
        ))
      }

      Opcode::Bind => {
        // currently handled in AsyncCall
        Some("".to_string())
      }

      _ => panic!("Unknown OP: {:?}", expr.get_opcode()),
    }
  }
}

pub fn generate_cpp_testbench(dir: &PathBuf, sys: &SysBuilder, simulator: &Simulator) {
  if !matches!(simulator, Simulator::Verilator) {
    return;
  }
  let main_fname = dir.join("main.cpp");
  let mut main_fd = File::create(main_fname).unwrap();
  main_fd
    .write(format!("#include \"Vtb.h\"\n").as_bytes())
    .unwrap();
  main_fd
    .write("#include \"verilated.h\"\n".as_bytes())
    .unwrap();
  main_fd
    .write("#include \"verilated_vcd_c.h\"\n".as_bytes())
    .unwrap();
  main_fd
    .write("vluint64_t main_time = 0;\n".as_bytes())
    .unwrap();
  main_fd
    .write("double sc_time_stamp() { return main_time; }\n".as_bytes())
    .unwrap();
  main_fd
    .write("int main(int argc, char **argv) {\n".as_bytes())
    .unwrap();
  main_fd
    .write("  Verilated::commandArgs(argc, argv);\n".as_bytes())
    .unwrap();
  main_fd
    .write(format!("  auto* top = new Vtb;\n").as_bytes())
    .unwrap();
  main_fd
    .write("  Verilated::traceEverOn(true);\n".as_bytes())
    .unwrap();
  main_fd
    .write("  auto* tfp = new VerilatedVcdC;\n".as_bytes())
    .unwrap();
  main_fd
    .write("  top->trace(tfp, 99);\n".as_bytes())
    .unwrap();
  main_fd
    .write("  tfp->open(\"wave.vcd\");\n".as_bytes())
    .unwrap();
  main_fd
    .write("  while (!Verilated::gotFinish()) {\n".as_bytes())
    .unwrap();
  main_fd.write("    top->eval();\n".as_bytes()).unwrap();
  main_fd
    .write("    tfp->dump(main_time);\n".as_bytes())
    .unwrap();
  main_fd.write("    main_time++;\n".as_bytes()).unwrap();
  main_fd.write("  }\n".as_bytes()).unwrap();
  main_fd.write("  tfp->close();\n".as_bytes()).unwrap();
  main_fd.write("  delete top;\n".as_bytes()).unwrap();
  main_fd.write("  delete tfp;\n".as_bytes()).unwrap();
  main_fd.write("  return 0;\n".as_bytes()).unwrap();
  main_fd.write("}\n".as_bytes()).unwrap();
  let make_fname = dir.join("Makefile");
  let mut make_fd = File::create(make_fname).unwrap();
  make_fd
    .write(".PHONY: verilator main\n".as_bytes())
    .unwrap();
  make_fd.write("verilator:\n".as_bytes()).unwrap();
  make_fd
    .write(
      format!(
        "\t$(VERILATOR_ROOT)/bin/verilator --cc {}.sv --exe main.cpp --top tb --timing --trace\n",
        sys.get_name()
      )
      .as_bytes(),
    )
    .unwrap();
  make_fd.write("main: verilator\n".as_bytes()).unwrap();
  make_fd
    .write("\tmake -C obj_dir -f Vtb.mk\n".as_bytes())
    .unwrap();
}

pub fn elaborate(sys: &SysBuilder, config: &Config, simulator: Simulator) -> Result<(), Error> {
  create_and_clean_dir(config.dir_name(sys), config.override_dump);
  let dir_name = config.dir_name(sys);
  let fname = dir_name.join(format!("{}.sv", sys.get_name()));

  println!("Writing verilog rtl to {}", fname.to_str().unwrap());

  generate_cpp_testbench(&dir_name, sys, &simulator);

  let mut vd = VerilogDumper::new(sys, config, simulator);

  let mut fd = File::create(fname)?;

  for module in vd.sys.module_iter() {
    vd.current_module = namify(module.get_name()).to_string();
    fd.write(vd.visit_module(module).unwrap().as_bytes())?;
  }

  vd.dump_runtime(fd, config.sim_threshold)?;

  Ok(())
}
