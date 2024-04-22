use std::{collections::{HashMap, HashSet}, fs::File, io::{Error, Write}};

use crate::{builder::system::SysBuilder, ir::node::*, ir::visitor::Visitor, ir::*};

use super::Config;

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
  triggers: HashMap<String, Vec<String>>, // module_name -> [pred]
  current_module: String,
  trigger_drivers: HashMap<String, HashSet<String>>, // module_name -> {driver module}
  array_drivers: HashMap<String, HashSet<String>>, // array -> {driver module}
  fifo_drivers: HashMap<String, HashSet<String>>, // fifo -> {driver module}
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
      trigger_drivers: HashMap::new(),
      array_drivers: HashMap::new(),
      fifo_drivers: HashMap::new(),
    }
  }
  
  fn dump_runtime(self: VerilogDumper<'a>, mut fd: File, sim_threshold: usize) -> Result<(), Error> {
    // runtime
    let mut res = String::new();
    res.push_str("module top (\n");
    res.push_str("  input logic clk,\n");
    res.push_str("  input logic rst_n\n");
    res.push_str(");\n\n");
    for array in self.sys.array_iter() {
      let array_name = namify(array.get_name());
      res.push_str(format!(
        "// array: {}[{}]\n",
        array_name,
        array.get_size()
      ).as_str());
      res.push_str(format!(
        "logic [{}:0] array_{}_q[0:{}];\n",
        array.scalar_ty().bits() - 1,
        array_name,
        array.get_size() - 1
      ).as_str());

      for driver in self.array_drivers.get(&array_name).unwrap().into_iter() {
        res.push_str(format!(
          "logic [{}:0] array_{}_driver_{}_d;\n",
          array.scalar_ty().bits() - 1,
          array_name,
          driver
        ).as_str());
        res.push_str(format!(
          "logic [{}:0] array_{}_driver_{}_widx;\n",
          (array.get_size() + 1).ilog2() - 1,
          array_name,
          driver
        ).as_str());
        res.push_str(format!(
          "logic array_{}_driver_{}_w;\n",
          array_name,
          driver
        ).as_str());
      }

      res.push_str(format!(
        "logic [{}:0] array_{}_d;\n",
        array.scalar_ty().bits() - 1,
        array_name
      ).as_str());
      res.push_str(format!(
        "assign array_{}_d = \n{};\n",
        array_name,
        self.array_drivers.get(&array_name).unwrap().into_iter().map(|driver|
          format!(
            "  ({{{}{{array_{}_driver_{}_w}}}} & array_{}_driver_{}_d)",
            array.scalar_ty().bits(),
            array_name,
            driver,
            array_name,
            driver
          )
        ).collect::<Vec<String>>().join(" |\n")).as_str()
      );
      res.push_str(format!(
        "logic [{}:0] array_{}_widx;\n",
        (array.get_size() + 1).ilog2() - 1,
        array_name
      ).as_str());
      res.push_str(format!(
        "assign array_{}_widx = \n{};\n",
        array_name,
        self.array_drivers.get(&array_name).unwrap().into_iter().map(|driver|
          format!(
            "  ({{{}{{array_{}_driver_{}_w}}}} & array_{}_driver_{}_widx)",
            (array.get_size() + 1).ilog2(),
            array_name,
            driver,
            array_name,
            driver
          )
        ).collect::<Vec<String>>().join(" |\n")).as_str()
      );
      res.push_str(format!(
        "logic array_{}_w;\n",
        array_name
      ).as_str());
      res.push_str(format!(
        "assign array_{}_w = {};\n",
        array_name,
        self.array_drivers.get(&array_name).unwrap().into_iter().map(|driver|
          format!("array_{}_driver_{}_w", array_name, driver)
        ).collect::<Vec<String>>().join(" | ")).as_str()
      );
      res.push_str(format!(
        "always_ff @(posedge clk or negedge rst_n) if (!rst_n) array_{}_q <= '{{default : 4'h0}}; else if (array_{}_w) array_{}_q[array_{}_widx] <= array_{}_d;\n\n",
        array_name,
        array_name,
        array_name,
        array_name,
        array_name
      ).as_str());
    }
    for module in self.sys.module_iter() {
      for port in module.port_iter() {
        let fifo_name = namify(format!(
          "{}_{}",
          port.get_parent().as_ref::<Module>(self.sys).unwrap().get_name(),
          fifo_name!(port)
        ).as_str());
        let fifo_width = port.scalar_ty().bits();
        res.push_str(format!("// fifo: {}\n", fifo_name).as_str());
        for driver in self.fifo_drivers.get(&fifo_name).unwrap().into_iter() {
          res.push_str(format!("logic fifo_{}_driver_{}_push_valid;\n", fifo_name, driver).as_str());
          res.push_str(format!("logic [{}:0] fifo_{}_driver_{}_push_data;\n", fifo_width - 1, fifo_name, driver).as_str());
          res.push_str(format!("logic fifo_{}_driver_{}_push_ready;\n", fifo_name, driver).as_str());
        }
        res.push_str(format!("logic fifo_{}_push_valid;\n", fifo_name).as_str());
        res.push_str(format!(
          "assign fifo_{}_push_valid = {};\n",
          fifo_name,
          self.fifo_drivers.get(&fifo_name).unwrap().into_iter().map(|driver|
            format!("fifo_{}_driver_{}_push_valid", fifo_name, driver)
          ).collect::<Vec<String>>().join(" | ")).as_str()
        );
        res.push_str(format!("logic [{}:0] fifo_{}_push_data;\n", fifo_width - 1, fifo_name).as_str());
        res.push_str(format!(
          "assign fifo_{}_push_data = \n{};\n",
          fifo_name,
          self.fifo_drivers.get(&fifo_name).unwrap().into_iter().map(|driver|
            format!(
              "  ({{{}{{fifo_{}_driver_{}_push_valid}}}} & fifo_{}_driver_{}_push_data)",
              fifo_width,
              fifo_name,
              driver,
              fifo_name,
              driver
            )
          ).collect::<Vec<String>>().join(" |\n")).as_str()
        );
        res.push_str(format!("logic fifo_{}_push_ready;\n", fifo_name).as_str());
        for driver in self.fifo_drivers.get(&fifo_name).unwrap().into_iter() {
          res.push_str(format!("assign fifo_{}_driver_{}_push_ready = fifo_{}_push_ready;\n", fifo_name, driver, fifo_name).as_str());
        }
        res.push_str(format!("logic fifo_{}_pop_valid;\n", fifo_name).as_str());
        res.push_str(format!("logic [{}:0] fifo_{}_pop_data;\n", fifo_width - 1, fifo_name).as_str());
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
      }
    }
    // triggers
    for module in self.sys.module_iter() {
      let module_name = namify(module.get_name());
      res.push_str(format!("// {} trigger\n", module_name).as_str());
      if module_name != "driver" {
        for driver in self.trigger_drivers.get(&module_name).unwrap().into_iter() {
          res.push_str(format!("logic {}_driver_{}_trigger_push_valid;\n", module_name, driver).as_str());
          res.push_str(format!("logic {}_driver_{}_trigger_push_ready;\n", module_name, driver).as_str());
        }
      }
      res.push_str(format!("logic {}_trigger_push_valid;\n", module_name).as_str());
      if module_name != "driver" {
        res.push_str(format!(
          "assign {}_trigger_push_valid = \n{};\n",
          module_name,
          self.trigger_drivers.get(&module_name).unwrap().into_iter().map(|driver|
            format!(
              "  {}_driver_{}_trigger_push_valid",
              module_name,
              driver
            )
          ).collect::<Vec<String>>().join(" |\n").as_str()
        ).as_str());
      }
      res.push_str(format!("logic {}_trigger_push_ready;\n", module_name).as_str());
      if module_name != "driver" {
        for driver in self.trigger_drivers.get(&module_name).unwrap().into_iter() {
          res.push_str(format!("assign {}_driver_{}_trigger_push_ready = {}_trigger_push_ready;\n", module_name, driver, module_name).as_str());
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
    }
    res.push_str("assign driver_trigger_push_valid = 1'b1;\n\n");
    // module insts
    for module in self.sys.module_iter() {
      let module_name = namify(module.get_name());
      res.push_str(format!("// {}\n", module_name).as_str());
      res.push_str(format!("{} {}_i (\n", module_name, module_name).as_str());
      res.push_str(format!("  .clk(clk),\n").as_str());
      res.push_str(format!("  .rst_n(rst_n),\n").as_str());
      for port in module.port_iter() {
        let fifo_name = namify(format!(
          "{}_{}",
          port.get_parent().as_ref::<Module>(self.sys).unwrap().get_name(),
          fifo_name!(port)
        ).as_str());
        res.push_str(format!("  .fifo_{}_pop_valid(fifo_{}_pop_valid),\n", namify(port.get_name().as_str()), fifo_name).as_str());
        res.push_str(format!("  .fifo_{}_pop_data(fifo_{}_pop_data),\n", namify(port.get_name().as_str()), fifo_name).as_str());
        res.push_str(format!("  .fifo_{}_pop_ready(fifo_{}_pop_ready),\n", namify(port.get_name().as_str()), fifo_name).as_str());
      }
      for (interf, _) in module.ext_interf_iter() {
        if interf.get_kind() == NodeKind::FIFO {
          let fifo = interf.as_ref::<FIFO>(self.sys).unwrap();
          let fifo_name = namify(format!(
            "{}_{}",
            fifo.get_parent().as_ref::<Module>(self.sys).unwrap().get_name(),
            fifo_name!(fifo)
          ).as_str());
          res.push_str(format!("  .fifo_{}_push_valid(fifo_{}_driver_{}_push_valid),\n", fifo_name, fifo_name, module_name).as_str());
          res.push_str(format!("  .fifo_{}_push_data(fifo_{}_driver_{}_push_data),\n", fifo_name, fifo_name, module_name).as_str());
          res.push_str(format!("  .fifo_{}_push_ready(fifo_{}_driver_{}_push_ready),\n", fifo_name, fifo_name, module_name).as_str());
        } else if interf.get_kind() == NodeKind::Array {
          let array_ref = interf.as_ref::<Array>(self.sys).unwrap();
            res.push_str(format!("  .array_{}_q(array_{}_q),\n", namify(array_ref.get_name()), namify(array_ref.get_name())).as_str());
            res.push_str(format!("  .array_{}_w(array_{}_driver_{}_w),\n", namify(array_ref.get_name()), namify(array_ref.get_name()), module_name).as_str());
            res.push_str(format!("  .array_{}_widx(array_{}_driver_{}_widx),\n", namify(array_ref.get_name()), namify(array_ref.get_name()), module_name).as_str());
            res.push_str(format!("  .array_{}_d(array_{}_driver_{}_d),\n", namify(array_ref.get_name()), namify(array_ref.get_name()), module_name).as_str());
        } else {
          panic!("Unknown interf kind {:?}", interf.get_kind());
        }
      }

      let mut trigger_modules = get_triggered_modules(&module.upcast(), self.sys);
      trigger_modules.sort_unstable();
      trigger_modules.dedup();
      for trigger_module in trigger_modules {
        res.push_str(format!(
          "  .{}_trigger_push_valid({}_driver_{}_trigger_push_valid),\n",
          trigger_module,
          trigger_module,
          module_name
        ).as_str());
        res.push_str(format!(
          "  .{}_trigger_push_ready({}_driver_{}_trigger_push_ready),\n",
          trigger_module,
          trigger_module,
          module_name
        ).as_str());
      }
      res.push_str(format!("  .trigger_pop_valid({}_trigger_pop_valid),\n", module_name).as_str());
      res.push_str(format!("  .trigger_pop_ready({}_trigger_pop_ready)\n", module_name).as_str());
      res.push_str(format!(");\n\n").as_str());
    }
    res.push_str("endmodule // top\n\n");

    fd.write(res.as_bytes()).unwrap();

    fd.write(format!("
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
", sim_threshold).as_bytes())?;

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
      if expr.get_opcode() == Opcode::Trigger {
        let triggered_module = expr.get_operand(0).unwrap().get_value().as_ref::<Module>(sys).unwrap();
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
        Some(format!(
          "({})",
          int_imm.get_value()
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

macro_rules! dump_ref {
  ($sys:expr, $value:expr) => {
    NodeRefDumper.dispatch($sys, $value, vec![]).unwrap()
  };
}

impl<'a> Visitor<String> for VerilogDumper<'a> {
  fn visit_module(&mut self, module: &ModuleRef<'_>) -> Option<String> {
    let mut res = String::new();

    res.push_str(format!("module {} (\n", namify(module.get_name())).as_str());

    self.indent += 2;
    res.push_str(format!(
      "{}input logic clk,\n",
      " ".repeat(self.indent)
    ).as_str());
    res.push_str(format!(
      "{}input logic rst_n,\n",
      " ".repeat(self.indent)
    ).as_str());
    res.push_str("\n");
    for port in module.port_iter() {
      res.push_str(format!(
        "{}// port {}\n",
        " ".repeat(self.indent),
        fifo_name!(port)
      ).as_str());
      res.push_str(format!(
        "{}input logic fifo_{}_pop_valid,\n",
        " ".repeat(self.indent),
        fifo_name!(port)
      ).as_str());
      res.push_str(format!(
        "{}input logic [{}:0] fifo_{}_pop_data,\n",
        " ".repeat(self.indent),
        port.scalar_ty().bits() - 1,
        fifo_name!(port)
      ).as_str());
      res.push_str(format!(
        "{}output logic fifo_{}_pop_ready,\n\n",
        " ".repeat(self.indent),
        fifo_name!(port)
      ).as_str());
    }

    for (interf, _ops) in module.ext_interf_iter() {
      if interf.get_kind() == NodeKind::FIFO {
        let fifo = interf.as_ref::<FIFO>(self.sys).unwrap();
        let fifo_name = namify(format!(
          "{}_{}",
          fifo.get_parent().as_ref::<Module>(self.sys).unwrap().get_name(),
          fifo_name!(fifo)
        ).as_str());
        res.push_str(format!(
          "{}// port {}\n",
          " ".repeat(self.indent),
          fifo_name
        ).as_str());
        res.push_str(format!(
          "{}output logic fifo_{}_push_valid,\n",
          " ".repeat(self.indent),
          fifo_name
        ).as_str());
        res.push_str(format!(
          "{}output logic [{}:0] fifo_{}_push_data,\n",
          " ".repeat(self.indent),
          fifo.scalar_ty().bits() - 1,
          fifo_name
        ).as_str());
        res.push_str(format!(
          "{}input logic fifo_{}_push_ready,\n",
          " ".repeat(self.indent),
          fifo_name
        ).as_str());
      } else if interf.get_kind() == NodeKind::Array {
        let array_ref = interf.as_ref::<Array>(self.sys).unwrap();
        res.push_str(format!(
          "{}// array {}\n",
          " ".repeat(self.indent),
          namify(array_ref.get_name())
        ).as_str());
        res.push_str(format!(
          "{}input logic [{}:0] array_{}_q[0:{}],\n",
          " ".repeat(self.indent),
          array_ref.scalar_ty().bits() - 1,
          namify(array_ref.get_name()),
          array_ref.get_size() - 1
        ).as_str());
        res.push_str(format!(
          "{}output logic array_{}_w,\n",
          " ".repeat(self.indent),
          namify(array_ref.get_name())
        ).as_str());
        res.push_str(format!(
          "{}output logic [{}:0] array_{}_widx,\n",
          " ".repeat(self.indent),
          (array_ref.get_size() + 1).ilog2() - 1,
          namify(array_ref.get_name())
        ).as_str());
        res.push_str(format!(
          "{}output logic [{}:0] array_{}_d,\n",
          " ".repeat(self.indent),
          array_ref.scalar_ty().bits() - 1,
          namify(array_ref.get_name())
        ).as_str());
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
      res.push_str(format!(
        "{}output logic {}_trigger_push_valid,\n",
        " ".repeat(self.indent),
        trigger_module
      ).as_str());
      res.push_str(format!(
        "{}input logic {}_trigger_push_ready,\n",
        " ".repeat(self.indent),
        trigger_module
      ).as_str());
    }

    if has_trigger_modules {
      res.push_str("\n");
    }

    res.push_str(format!(
      "{}// trigger\n",
      " ".repeat(self.indent)
    ).as_str());
    res.push_str(format!(
      "{}input logic trigger_pop_valid,\n",
      " ".repeat(self.indent)
    ).as_str());
    res.push_str(format!(
      "{}output logic trigger_pop_ready\n",
      " ".repeat(self.indent)
    ).as_str());
    self.indent -= 2;
    res.push_str(");\n\n");

    let mut body_waituntil_cnt = 0;
    let mut wait_until: Option<String> = None;

    for elem in module.get_body().iter().skip(1) {
      match elem.get_kind() {
        NodeKind::Expr => break,
        NodeKind::Block => {
          let block = elem.as_ref::<Block>(self.sys).unwrap();
          if let BlockPred::WaitUntil(cond) = block.get_pred() {
            body_waituntil_cnt += 1;
            let cond = cond
              .as_ref::<Expr>(self.sys)
              .unwrap();
            wait_until =
              Some(format!(
                " && (_{}{})",
                cond.get_key(),
                if cond.dtype().bits() == 1 {
                  "".into()
                } else {
                  format!(" != 0")
                }
              ));
          }
        }
        _ => {
          panic!("Unexpected reference type: {:?}", elem);
        }
      }
    }

    if body_waituntil_cnt > 1 {
      panic!("multiple wait_until blocks in {}", module.get_name());
    }

    res.push_str(format!("logic trigger;\n").as_str());
    res.push_str(format!("assign trigger_pop_ready = trigger;\n\n").as_str());

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
            panic!("mixed conditional and unconditional branches for trigger {}", m);
          }
          has_unconditional_branch = true;
        } else {
          if has_unconditional_branch {
            panic!("mixed conditional and unconditional branches for trigger {}", m);
          }
          has_conditional_branch = true;
          valid_conds.push(p.clone());
        }
      }
      if has_conditional_branch {
        res.push_str(format!("assign {}_trigger_push_valid = trigger && ({});\n\n", m, valid_conds.join(" || ")).as_str());
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
            panic!("mixed conditional and unconditional branches for fifo {}", f);
          }
          has_unconditional_branch = true;
          data_str.push_str(format!("{}", v).as_str());
        } else {
          if has_unconditional_branch {
            panic!("mixed conditional and unconditional branches for fifo {}", f);
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
        res.push_str(format!("assign fifo_{}_push_valid = trigger && ({});\n", f, valid_conds.join(" || ")).as_str());
      } else {
        res.push_str(format!("assign fifo_{}_push_valid = trigger;\n", f).as_str());
      }
      res.push_str(format!("assign fifo_{}_push_data = {};\n\n", f, data_str).as_str());
    }

    res.push_str(format!("assign trigger = trigger_pop_valid{};\n", wait_until.unwrap_or("".to_string())).as_str());

    res.push_str(format!("endmodule // {}\n\n\n", namify(module.get_name())).as_str());

    Some(res)
  }


  fn visit_block(&mut self, block: &BlockRef<'_>) -> Option<String> {
    let mut res = String::new();
    if let BlockPred::Condition(cond) = block.get_pred() {
      self.pred =
        Some(format!(
          "({}{})",
          dump_ref!(self.sys, &cond),
          if cond.get_dtype(block.sys).unwrap().bits() == 1 {
            "".into()
          } else {
            format!(" != 0")
          }
        ));
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
    if expr.get_opcode().is_binary() {
      Some(format!(
        "logic [{}:0] _{};\nassign _{} = {} {} {};\n\n",
        expr.dtype().bits() - 1,
        expr.get_key(),
        expr.get_key(),
        dump_ref!(self.sys, expr.get_operand(0).unwrap().get_value()),
        expr.get_opcode().to_string(),
        dump_ref!(self.sys, expr.get_operand(1).unwrap().get_value())
      ))
    } else if expr.get_opcode().is_unary() {
      Some(format!(
        "logic [{}:0] _{};\nassign _{} = {}{};\n\n",
        expr.dtype().bits() - 1,
        expr.get_key(),
        expr.get_key(),
        expr.get_opcode().to_string(),
        dump_ref!(self.sys, expr.get_operand(0).unwrap().get_value())
      ))
     }  else {
      match expr.get_opcode() {

        Opcode::FIFOPop => {
          let fifo = expr
            .get_operand(0)
            .unwrap()
            .get_value()
            .as_ref::<FIFO>(self.sys)
            .unwrap();
          Some(format!(
            "logic [{}:0] _{};\nassign _{} = fifo_{}_pop_data;\nassign fifo_{}_pop_ready = trigger{};\n\n",
            fifo.scalar_ty().bits() - 1,
            expr.get_key(),
            expr.get_key(),
            fifo_name!(fifo),
            fifo_name!(fifo),
            (self.pred.clone().and_then(|p| Some(format!(" && {}", p)))).unwrap_or("".to_string())
          ))
        }

        Opcode::Log => {
          let mut format_str = dump_ref!(self.sys, expr.operand_iter().collect::<Vec<OperandRef>>().first().unwrap().get_value());
          for elem in expr.operand_iter().skip(1) {
            format_str = format_str.replacen("{}", match elem.get_value().get_dtype(self.sys).unwrap() {
              DataType::Int(_) => "%d",
              DataType::Str => "%s",
              _ => "?",
            }, 1);
          }
          format_str = format_str.replace("\"", "");
          let mut res = String::new();
          res.push_str(format!("always_ff @(posedge clk iff trigger{}) ", (self.pred.clone().and_then(|p| Some(format!(" && {}", p)))).unwrap_or("".to_string())).as_str());
          res.push_str("$display(\"%t\\t");
          res.push_str(format_str.as_str());
          res.push_str("\", $time, ");
          for elem in expr.operand_iter().skip(1) {
            res.push_str(format!("{}, ", dump_ref!(self.sys, elem.get_value())).as_str());
          }
          res.pop(); res.pop();
          res.push_str(");\n");
          res.push_str("\n");
          Some(res)
        }

        Opcode::Load => {
          let array_ptr = expr
            .get_operand(0)
            .unwrap()
            .get_value()
            .as_ref::<ArrayPtr>(self.sys)
            .unwrap();
          let array_ref = &array_ptr
            .get_array()
            .as_ref::<Array>(self.sys)
            .unwrap();
          Some(format!(
            "logic [{}:0] _{};\nassign _{} = array_{}_q[{}];\n\n",
            expr.dtype().bits() - 1,
            expr.get_key(),
            expr.get_key(),
            namify(array_ref.get_name()),
            dump_ref!(self.sys, array_ptr.get_idx())
          ))
        }

        Opcode::Store => {
          let array_ptr = expr
            .get_operand(0)
            .unwrap()
            .get_value()
            .as_ref::<ArrayPtr>(self.sys)
            .unwrap();
          let array_ref = &array_ptr
            .get_array()
            .as_ref::<Array>(self.sys)
            .unwrap();
          let array_name = namify(array_ref.get_name());
          match self.array_drivers.get_mut(&array_name) {
            Some(ads) => {
              ads.insert(self.current_module.clone());
            }
            None => {
              self.array_drivers.insert(array_name.clone(), HashSet::from([self.current_module.clone()]));
            }
          }
          Some(format!(
            "assign array_{}_w = trigger{};\nassign array_{}_d = {};\nassign array_{}_widx = {};\n\n",
            array_name,
            (self.pred.clone().and_then(|p| Some(format!(" && {}", p)))).unwrap_or("".to_string()),
            array_name,
            dump_ref!(self.sys, expr.get_operand(1).unwrap().get_value()),
            array_name,
            dump_ref!(self.sys, array_ptr.get_idx())
          ))
        }

        Opcode::FIFOPush => {
          let fifo = expr.get_operand(0).unwrap().get_value().as_ref::<FIFO>(self.sys).unwrap();
          let fifo_name = namify(format!(
            "{}_{}",
            fifo.get_parent().as_ref::<Module>(self.sys).unwrap().get_name(),
            fifo_name!(fifo)
          ).as_str());
          match self.fifo_drivers.get_mut(&fifo_name) {
            Some(fds) => {
              fds.insert(self.current_module.clone());
            }
            None => {
              self.fifo_drivers.insert(fifo_name.clone(), HashSet::from([self.current_module.clone()]));
            }
          }
          match self.fifo_pushes.get_mut(&fifo_name) {
            Some(fps) => {
              fps.push((
                self.pred.clone().unwrap_or("".to_string()),
                dump_ref!(self.sys, expr.get_operand(1).unwrap().get_value())
              ))
            }
            None => {
              self.fifo_pushes.insert(
                fifo_name.clone(),
                vec![(
                  self.pred.clone().unwrap_or("".to_string()),
                  dump_ref!(self.sys, expr.get_operand(1).unwrap().get_value())
                )]
              );
            }
          }
          Some("".to_string())
        }

        Opcode::FIFOPeek => {
          let fifo = expr.get_operand(0).unwrap().get_value().as_ref::<FIFO>(self.sys).unwrap();
          let fifo_name = fifo_name!(fifo);
          Some(format!(
            "logic [{}:0] _{};\nassign _{} = fifo_{}_pop_data;\n\n",
            fifo.scalar_ty().bits() - 1,
            expr.get_key(),
            expr.get_key(),
            fifo_name
          ))
        }

        Opcode::Trigger => {
          let module = expr
            .get_operand(0)
            .unwrap()
            .get_value()
            .as_ref::<Module>(self.sys)
            .unwrap();
          let module_name = namify(module.get_name());
          match self.trigger_drivers.get_mut(&module_name) {
            Some(tds) => {
              tds.insert(self.current_module.clone());
            }
            None => {
              self.trigger_drivers.insert(module_name.clone(), HashSet::from([self.current_module.clone()]));
            }
          }
          match self.triggers.get_mut(&module_name) {
            Some(trgs) => {
              trgs.push(self.pred.clone().unwrap_or("".to_string()))
            }
            None => {
              self.triggers.insert(
                module_name.clone(),
                vec![self.pred.clone().unwrap_or("".to_string())]
              );
            }
          }
          Some("".to_string())
        }

        _ => {
          Some(format!("{:?}\n\n", expr.get_opcode()))
        }
      }
    }
  }
}

pub fn elaborate(sys: &SysBuilder, config: &Config) -> Result<(), Error> {
  println!("Writing verilog rtl to {}", config.fname);

  let mut vd = VerilogDumper::new(sys);

  let mut fd = File::create(config.fname.clone())?;

  for module in vd.sys.module_iter() {
    vd.current_module = namify(module.get_name()).to_string();
    fd.write(vd.visit_module(&module).unwrap().as_bytes())?;
  }

  vd.dump_runtime(fd, config.sim_threshold)?;

  Ok(())
}
