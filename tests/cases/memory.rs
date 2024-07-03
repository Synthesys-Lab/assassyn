use std::path::PathBuf;

use assassyn::module_builder;
use eir::{backend, builder::SysBuilder, ir::node::BaseNode, xform};

module_builder!(
  reader(k, write, rdata)() {
    is_read = write.flip();
    when is_read {
      rdata = rdata.bitcast(int<32>);
      delta = rdata.add(k);
      log("{} + {} = {}", rdata, k, delta);
    }
  }
);

fn sram_sys() -> SysBuilder {
  module_builder!(
    driver(memory)() {
      cnt = array(int<32>, 1);
      v = cnt[0];
      write = v.slice(0, 0);
      plused = v.add(1);
      waddr = plused.slice(0, 9);
      waddr = waddr.bitcast(uint<10>);
      raddr = v.slice(0, 9);
      raddr = raddr.bitcast(uint<10>);
      addr = default raddr.case(write, waddr);
      async_call memory { addr: addr, write: write, wdata: v.bitcast(bits<32>) };
      cnt[0] = plused;
    }
  );

  let mut sys = SysBuilder::new("sram");

  let const128 = sys.get_const_int(eir::ir::DataType::Int(32), 128);

  let memory = sys.create_memory(
    "memory",
    32,
    1024,
    1..=1,
    None,
    |x: &mut SysBuilder, module: BaseNode, write: BaseNode, rdata: BaseNode| {
      reader_impl(x, module, const128, write, rdata);
    },
  );
  let _driver = driver_builder(&mut sys, memory);
  sys
}

pub fn sram() {
  let mut sys = sram_sys();

  println!("{}", sys);

  eir::builder::verify(&sys);
  let o0 = xform::Config {
    rewrite_wait_until: false,
  };
  eir::xform::basic(&mut sys, &o0);
  eir::builder::verify(&sys);

  println!("{}", sys);

  let config = backend::common::Config {
    sim_threshold: 200,
    idle_threshold: 200,
    ..Default::default()
  };

  eir::backend::verilog::elaborate(&sys, &config, backend::verilog::Simulator::VCS).unwrap();

  eir::test_utils::run_simulator(&sys, &config, None);
}

fn sram_init_sys() -> SysBuilder {
  module_builder!(
    driver(memory)() {
      cnt = array(int<32>, 1);
      v = cnt[0];
      plused = v.add(1);
      raddr = v.slice(0, 9);
      raddr = raddr.bitcast(uint<10>);
      write = 0.bits<1>;
      async_call memory { addr: raddr, write: write, wdata: v.bitcast(bits<32>) };
      cnt[0] = plused;
    }
  );

  let mut sys = SysBuilder::new("sram_init");

  let const128 = sys.get_const_int(eir::ir::DataType::Int(32), 128);

  let memory = sys.create_memory(
    "memory",
    32,
    1024,
    1..=1,
    Some("./init.hex".to_string()),
    |x: &mut SysBuilder, module: BaseNode, write: BaseNode, rdata: BaseNode| {
      reader_impl(x, module, const128, write, rdata);
    },
  );
  let _driver = driver_builder(&mut sys, memory);
  sys
}

pub fn sram_init() {
  let mut sys = sram_init_sys();

  println!("{}", sys);

  eir::builder::verify(&sys);
  let o0 = xform::Config {
    rewrite_wait_until: false,
  };
  eir::xform::basic(&mut sys, &o0);
  eir::builder::verify(&sys);

  println!("{}", sys);

  eprintln!(
    "{:?}",
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/resources")
  );

  let config = backend::common::Config {
    sim_threshold: 200,
    idle_threshold: 200,
    resource_base: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("tests/resources")
      .into(),
    ..Default::default()
  };

  eir::backend::verilog::elaborate(&sys, &config, backend::verilog::Simulator::VCS).unwrap();

  eir::test_utils::run_simulator(&sys, &config, None);
}
