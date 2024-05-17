use std::{
  collections::{HashMap, HashSet},
  fs::File,
  io::{Error, Write},
};

use crate::{
  backend::common::Config,
  builder::system::SysBuilder,
  ir::{module::memory::parse_memory_module_name, node::*, visitor::Visitor, *},
};

use self::{
  expr::subcode,
  instructions::{Bind, GetElementPtr},
};

fn namify(name: &str) -> String {
  name.replace(".", "_")
}

macro_rules! fifo_name {
  ($fifo:expr) => {{
    format!("{}", namify($fifo.get_name()))
  }};
}

struct VerilogDumper<'a> {
  sys: &'a SysBuilder,
  indent: usize,
  pred: Option<String>,
  fifo_pushes: HashMap<String, Vec<(String, String)>>, // fifo_name -> [(pred, value)]
  triggers: HashMap<String, Vec<String>>,              // module_name -> [pred]
  current_module: String,
  has_testbench: bool,
  has_driver: bool,
  trigger_drivers: HashMap<String, HashSet<String>>, // module_name -> {driver module}
  array_drivers: HashMap<String, HashSet<String>>,   // array -> {driver module}
  fifo_drivers: HashMap<String, HashSet<String>>,    // fifo -> {driver module}
}

impl<'a> VerilogDumper<'a> {
  fn new(sys: &'a SysBuilder) -> Self {
    Self {
      sys,
      indent: 0,
      pred: None,
      fifo_pushes: HashMap::new(),
      triggers: HashMap::new(),
      current_module: String::new(),
      has_testbench: false,
      has_driver: false,
      trigger_drivers: HashMap::new(),
      array_drivers: HashMap::new(),
      fifo_drivers: HashMap::new(),
    }
  }

  fn dump_array(&self, array: &ArrayRef) -> String {
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
    res.push_str("always_ff @(posedge clk or negedge rst_n)");
    res.push_str(
      format!(
        "if (!rst_n) array_{}_q <= '{{default : {}'d0}};\n",
        array_name,
        array.scalar_ty().get_bits()
      )
      .as_str(),
    );
    res.push_str(&format!(
      " else if (array_{}_w) array_{}_q[array_{}_widx] <= array_{}_d;\n",
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
    self: VerilogDumper<'a>,
    mut fd: File,
    sim_threshold: usize,
  ) -> Result<(), Error> {
    // runtime
    let mut res = String::new();

    res.push_str("module top (\n");
    res.push_str("  input logic clk,\n");
    res.push_str("  input logic rst_n\n");
    res.push_str(");\n\n");

    // array storage element definitions
    for array in self.sys.array_iter() {
      res.push_str(self.dump_array(&array).as_str());
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
  $fsdbDumpfile(\"wave.fsdb\");
  $fsdbDumpvars();
end

initial begin
  clk = 1'b1;
  rst_n = 1'b0;
  #1;
  rst_n = 1'b1;
  #{}00;
  $finish();
end

always #50 clk <= !clk;

top top_i (
  .clk(clk),
  .rst_n(rst_n)
);

endmodule
",
        sim_threshold
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
      for elem in module.get_body().iter() {
        if elem.get_kind() == NodeKind::Expr || elem.get_kind() == NodeKind::Block {
          triggered_modules.append(get_triggered_modules(elem, sys).as_mut());
        }
      }
    }
    NodeKind::Block => {
      let block = node.as_ref::<Block>(sys).unwrap();
      for elem in block.iter() {
        if elem.get_kind() == NodeKind::Expr || elem.get_kind() == NodeKind::Block {
          triggered_modules.append(get_triggered_modules(elem, sys).as_mut());
        }
      }
    }
    NodeKind::Expr => {
      let expr = node.as_ref::<Expr>(sys).unwrap();
      if expr.get_opcode() == Opcode::AsyncCall {
        let triggered_module = {
          let bind = expr
            .get_operand(0)
            .unwrap()
            .get_value()
            .as_expr::<Bind>(sys)
            .unwrap();
          bind.get_callee()
        };
        let triggered_module = triggered_module.as_ref::<Module>(sys).unwrap();
        triggered_modules.push(namify(triggered_module.get_name()));
      }
    }
    _ => {}
  }
  triggered_modules
}

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
        Some(format!("{}", int_imm.get_value()))
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
}

macro_rules! dump_ref {
  ($sys:expr, $value:expr) => {
    NodeRefDumper.dispatch($sys, $value, vec![]).unwrap()
  };
}

impl<'a> Visitor<String> for VerilogDumper<'a> {
  fn visit_module(&mut self, module: &ModuleRef<'_>) -> Option<String> {
    if module.get_name() == "testbench" {
      self.has_testbench = true;
    }

    if module.get_name() == "driver" {
      self.has_driver = true;
    }

    let mut res = String::new();

    res.push_str(format!("module {} (\n", namify(module.get_name())).as_str());

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

    if let Some(params) = parse_memory_module_name(&self.current_module) {
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
        self.visit_expr(&expr);
        match expr.get_opcode() {
          Opcode::FIFOPush => {
            let fifo = expr
              .get_operand(0)
              .unwrap()
              .get_value()
              .as_ref::<FIFO>(self.sys)
              .unwrap();
            let fifo_module = namify(
              fifo
                .get_parent()
                .as_ref::<Module>(self.sys)
                .unwrap()
                .get_name(),
            );
            let fifo_name = namify(fifo.get_name());
            rdata_fifo = Some(format!("{}_{}", fifo_module, fifo_name));
          }
          Opcode::AsyncCall => {
            let module = {
              let operand = expr.get_operand(0).unwrap();
              let bind = operand.get_value();
              let module = {
                let bind = bind.as_expr::<Bind>(expr.sys).unwrap();
                bind.get_callee()
              };
              module.as_ref::<Module>(expr.sys).unwrap()
            };
            rdata_module = Some(namify(module.get_name()));
          }
          Opcode::Bind => { /* don't care, currently processed in corresponding AsyncCall` */ }
          _ => panic!("Unexpected expr of {:?} in memory body", expr.get_opcode()),
        }
      }

      res.push_str(format!("logic trigger;\nassign trigger = trigger_pop_valid;\nassign trigger_pop_ready = trigger;\n\n").as_str());

      res.push_str(
        format!(
          "logic [{}:0] mem [0:{}];\n\n",
          params.width - 1,
          params.depth - 1
        )
        .as_str(),
      );

      res.push_str("logic write;\n");
      res.push_str("assign write = fifo_write_pop_data;\n\n");

      // TODO: truely randomize latency
      res.push_str(format!("localparam read_latency = {};\n\n", params.lat_min).as_str());

      res.push_str("// read logic\n");
      let rdata_fifo = rdata_fifo.unwrap();
      res.push_str(
        format!(
          "assign fifo_{}_push_valid = trigger && !write;\n",
          rdata_fifo
        )
        .as_str(),
      );
      res.push_str(
        format!(
          "assign fifo_{}_push_data = mem[fifo_addr_pop_data];\n\n",
          rdata_fifo
        )
        .as_str(),
      );

      res.push_str("// write logic\n");
      res.push_str(
        "always @(posedge clk) if (write) mem[fifo_addr_pop_data] <= fifo_wdata_pop_data;\n\n",
      );

      res.push_str(format!("assign fifo_addr_pop_ready = trigger;\n").as_str());
      res.push_str(format!("assign fifo_write_pop_ready = trigger;\n").as_str());
      res.push_str(format!("assign fifo_wdata_pop_ready = trigger;\n\n").as_str());

      let rdata_module = rdata_module.unwrap();
      res.push_str(
        format!(
          "assign {}_trigger_push_valid = trigger && !write;\n\n",
          rdata_module
        )
        .as_str(),
      );
    } else {
      let mut wait_until: Option<String> = None;

      if let BlockKind::WaitUntil(cond) = module.get_body().get_kind() {
        let cond_block = cond.as_ref::<Block>(self.sys).unwrap();
        match cond_block.get_kind() {
          BlockKind::Valued(value_node) => {
            for elem in cond_block.iter() {
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
            let value = value_node.as_ref::<Expr>(self.sys).unwrap();
            wait_until = Some(format!(
              " && ({}{})",
              namify(value_node.to_string(self.sys).as_str()),
              if value.dtype().get_bits() == 1 {
                "".into()
              } else {
                format!(" != 0")
              }
            ));
          }
          _ => panic!("Expect valued block for wait_until condition"),
        }
      }

      res.push_str(format!("logic trigger;\n").as_str());
      res.push_str(format!("assign trigger_pop_ready = trigger;\n\n").as_str());

      if module.get_name() == "testbench" {
        res.push_str("int cycle_cnt;\n");
        res.push_str("always_ff @(posedge clk or negedge rst_n) if (!rst_n) cycle_cnt <= 0; ");
        res.push_str("else if (trigger) cycle_cnt <= cycle_cnt + 1;\n\n");
      }

      self.fifo_pushes.clear();
      self.triggers.clear();
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

      res.push_str(
        format!(
          "assign trigger = trigger_pop_valid{};\n\n",
          wait_until.unwrap_or("".to_string())
        )
        .as_str(),
      );
    }

    res.push_str(format!("endmodule // {}\n\n\n", namify(module.get_name())).as_str());

    Some(res)
  }

  fn visit_block(&mut self, block: &BlockRef<'_>) -> Option<String> {
    let mut res = String::new();
    match block.get_kind() {
      BlockKind::Condition(cond) => {
        self.pred = Some(format!(
          "({}{})",
          dump_ref!(self.sys, &cond),
          if cond.get_dtype(block.sys).unwrap().get_bits() == 1 {
            "".into()
          } else {
            format!(" != 0")
          }
        ));
      }
      BlockKind::Cycle(cycle) => {
        self.pred = Some(format!("(cycle_cnt == {})", cycle));
      }
      BlockKind::WaitUntil(_) | BlockKind::Valued(_) | BlockKind::None => (),
    }
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
    self.pred = None;
    res.into()
  }

  fn visit_expr(&mut self, expr: &ExprRef<'_>) -> Option<String> {
    if expr.get_opcode().is_binary() || expr.get_opcode().is_cmp() {
      Some(format!(
        "logic [{}:0] {};\nassign {} = {} {} {};\n\n",
        expr.dtype().get_bits() - 1,
        namify(expr.upcast().to_string(self.sys).as_str()),
        namify(expr.upcast().to_string(self.sys).as_str()),
        dump_ref!(self.sys, expr.get_operand(0).unwrap().get_value()),
        expr.get_opcode().to_string(),
        dump_ref!(self.sys, expr.get_operand(1).unwrap().get_value())
      ))
    } else if expr.get_opcode().is_unary() {
      Some(format!(
        "logic [{}:0] {};\nassign {} = {}{};\n\n",
        expr.dtype().get_bits() - 1,
        namify(expr.upcast().to_string(self.sys).as_str()),
        namify(expr.upcast().to_string(self.sys).as_str()),
        expr.get_opcode().to_string(),
        dump_ref!(self.sys, expr.get_operand(0).unwrap().get_value())
      ))
    } else {
      match expr.get_opcode() {
        Opcode::FIFOPop => {
          let fifo = expr
            .get_operand(0)
            .unwrap()
            .get_value()
            .as_ref::<FIFO>(self.sys)
            .unwrap();
          Some(format!(
            "logic [{}:0] {};\nassign {} = fifo_{}_pop_data;\nassign fifo_{}_pop_ready = trigger{};\n\n",
            fifo.scalar_ty().get_bits() - 1,
            namify(expr.upcast().to_string(self.sys).as_str()),
            namify(expr.upcast().to_string(self.sys).as_str()),
            fifo_name!(fifo),
            fifo_name!(fifo),
            (self.pred.clone().and_then(|p| Some(format!(" && {}", p)))).unwrap_or("".to_string())
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
              (self.pred.clone().and_then(|p| Some(format!(" && {}", p))))
                .unwrap_or("".to_string())
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
          let (array_ref, array_idx) = {
            let gep = expr
              .get_operand(0)
              .unwrap()
              .get_value()
              .as_expr::<GetElementPtr>(self.sys)
              .unwrap();
            (gep.get_array(), gep.get_index())
          };
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
          Some(format!(
            "logic [{}:0] {};\nassign {} = array_{}_q[{}];\n\n",
            expr.dtype().get_bits() - 1,
            namify(expr.upcast().to_string(self.sys).as_str()),
            namify(expr.upcast().to_string(self.sys).as_str()),
            namify(array_ref.get_name()),
            dump_ref!(self.sys, &array_idx)
          ))
        }

        Opcode::GetElementPtr => Some("".into()),

        Opcode::Store => {
          let (array_ref, array_idx) = {
            let gep = expr
              .get_operand(0)
              .unwrap()
              .get_value()
              .as_expr::<GetElementPtr>(self.sys)
              .unwrap();
            (gep.get_array(), gep.get_index())
          };
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
          Some(format!(
            "assign array_{}_w = trigger{};\nassign array_{}_d = {};\nassign array_{}_widx = {};\n\n",
            array_name,
            (self.pred.clone().and_then(|p| Some(format!(" && {}", p)))).unwrap_or("".to_string()),
            array_name,
            dump_ref!(self.sys, expr.get_operand(1).unwrap().get_value()),
            array_name,
            dump_ref!(self.sys, &array_idx)
          ))
        }

        Opcode::FIFOPush => {
          let fifo = expr
            .get_operand(0)
            .unwrap()
            .get_value()
            .as_ref::<FIFO>(self.sys)
            .unwrap();
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
          match self.fifo_pushes.get_mut(&fifo_name) {
            Some(fps) => fps.push((
              self.pred.clone().unwrap_or("".to_string()),
              dump_ref!(self.sys, expr.get_operand(1).unwrap().get_value()),
            )),
            None => {
              self.fifo_pushes.insert(
                fifo_name.clone(),
                vec![(
                  self.pred.clone().unwrap_or("".to_string()),
                  dump_ref!(self.sys, expr.get_operand(1).unwrap().get_value()),
                )],
              );
            }
          }
          Some("".to_string())
        }

        Opcode::FIFOField { field } => {
          let fifo = expr
            .get_operand(0)
            .unwrap()
            .get_value()
            .as_ref::<FIFO>(self.sys)
            .unwrap();
          let fifo_name = fifo_name!(fifo);
          match field {
            subcode::FIFO::Valid => Some(format!(
              "logic {};\nassign {} = fifo_{}_pop_valid;\n\n",
              namify(expr.upcast().to_string(self.sys).as_str()),
              namify(expr.upcast().to_string(self.sys).as_str()),
              fifo_name
            )),
            subcode::FIFO::Peek => Some(format!(
              "logic [{}:0] {};\nassign {} = fifo_{}_pop_data;\n\n",
              fifo.scalar_ty().get_bits() - 1,
              namify(expr.upcast().to_string(self.sys).as_str()),
              namify(expr.upcast().to_string(self.sys).as_str()),
              fifo_name
            )),
            subcode::FIFO::AlmostFull => todo!(),
          }
        }

        Opcode::AsyncCall => {
          let module = {
            let operand = expr
              .get_operand(0)
              .unwrap()
              .get_value()
              .as_expr::<Bind>(self.sys)
              .unwrap();
            operand.get_callee()
          };
          let module = module.as_ref::<Module>(self.sys).unwrap();
          let module_name = namify(module.get_name());
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
          match self.triggers.get_mut(&module_name) {
            Some(trgs) => trgs.push(self.pred.clone().unwrap_or("".to_string())),
            None => {
              self.triggers.insert(
                module_name.clone(),
                vec![self.pred.clone().unwrap_or("".to_string())],
              );
            }
          }
          Some("".to_string())
        }

        Opcode::Slice => {
          let a = dump_ref!(self.sys, &expr.get_operand(0).unwrap().get_value());
          let l = dump_ref!(self.sys, &expr.get_operand(1).unwrap().get_value());
          let r = dump_ref!(self.sys, &expr.get_operand(2).unwrap().get_value());
          Some(format!(
            "logic [{}:0] {};\nassign {} = {}[{}:{}];\n\n",
            expr.dtype().get_bits() - 1,
            namify(expr.upcast().to_string(self.sys).as_str()),
            namify(expr.upcast().to_string(self.sys).as_str()),
            a,
            r,
            l
          ))
        }

        Opcode::Cast { cast } => {
          let a = dump_ref!(self.sys, &expr.get_operand(0).unwrap().get_value());
          match cast {
            subcode::Cast::Cast | subcode::Cast::ZExt => Some(format!(
              "logic [{}:0] {};\nassign {} = {};\n\n",
              expr.dtype().get_bits() - 1,
              namify(expr.upcast().to_string(self.sys).as_str()),
              namify(expr.upcast().to_string(self.sys).as_str()),
              a
            )),
            subcode::Cast::SExt => {
              let src_ref = expr.get_operand(0).unwrap();
              let src = src_ref.get_value();
              let src_dtype = src.get_dtype(expr.sys).unwrap();
              let dest_dtype = expr.dtype();
              let a = dump_ref!(self.sys, src);
              if src_dtype.is_int()
                && src_dtype.is_signed()
                && dest_dtype.is_int()
                && dest_dtype.is_signed()
                && dest_dtype.get_bits() > src_dtype.get_bits()
              {
                // perform sext
                Some(format!(
                  "logic [{}:0] {};\nassign {} = {{{}'{{{}[{}]}}, {}}};\n\n",
                  expr.dtype().get_bits() - 1,
                  namify(expr.upcast().to_string(self.sys).as_str()),
                  namify(expr.upcast().to_string(self.sys).as_str()),
                  dest_dtype.get_bits() - src_dtype.get_bits(),
                  a,
                  src_dtype.get_bits() - 1,
                  a
                ))
              } else {
                Some(format!(
                  "logic [{}:0] {};\nassign {} = {};\n\n",
                  expr.dtype().get_bits() - 1,
                  namify(expr.upcast().to_string(self.sys).as_str()),
                  namify(expr.upcast().to_string(self.sys).as_str()),
                  a
                ))
              }
            }
          }
        }

        Opcode::Select => {
          let cond = dump_ref!(self.sys, &expr.get_operand(0).unwrap().get_value());
          let true_value = dump_ref!(self.sys, &expr.get_operand(1).unwrap().get_value());
          let false_value = dump_ref!(self.sys, &expr.get_operand(2).unwrap().get_value());
          Some(format!(
            "logic [{}:0] {};\nassign {} = {} ? {} : {};\n\n",
            expr.dtype().get_bits() - 1,
            namify(expr.upcast().to_string(self.sys).as_str()),
            namify(expr.upcast().to_string(self.sys).as_str()),
            cond,
            true_value,
            false_value
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
}

pub fn elaborate(sys: &SysBuilder, config: &Config) -> Result<(), Error> {
  let fname = config.fname(sys, "sv");
  println!("Writing verilog rtl to {}", fname);

  let mut vd = VerilogDumper::new(sys);

  let mut fd = File::create(fname)?;

  for module in vd.sys.module_iter() {
    vd.current_module = namify(module.get_name()).to_string();
    fd.write(vd.visit_module(&module).unwrap().as_bytes())?;
  }

  vd.dump_runtime(fd, config.sim_threshold)?;

  Ok(())
}
