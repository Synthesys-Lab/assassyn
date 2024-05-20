use assassyn::module_builder;
use eir::{backend, builder::SysBuilder, ir::node::BaseNode, xform};

module_builder!(
  mem_sink(memory_buffer, k)(addr: uint<10>, write: bits<1>, wdata: bits<32>) {
    when write {
      memory_buffer[addr] = wdata;
    }
    read = write.flip();
    when read {
      rdata = memory_buffer[addr];
      delta = rdata.add(k);
      log("{} was read", delta);
    }
  }
);

fn sram_sys() -> SysBuilder {
  module_builder!(
    driver(mem)() {
      cnt = array(int<32>, 1);
      v = cnt[0];
      write = v.slice(0, 0);
      write = write.cast(bits<1>);
      wdata = v.cast(bits<32>);
      plused = v.add(1);
      waddr = plused.slice(0, 9);
      waddr = waddr.cast(uint<10>);
      raddr = v.slice(0, 9);
      raddr = raddr.cast(uint<10>);
      addr = default raddr.case(write, waddr);
      async_call mem { addr: addr, write: write, wdata: wdata };
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
    |x: &mut SysBuilder, memory: BaseNode| mem_sink_builder(x, memory, const128),
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
    override_dump: true,
    temp_dir: true,
    sim_threshold: 200,
    idle_threshold: 200,
  };

  eir::backend::verilog::elaborate(&sys, &config).unwrap();

  eir::test_utils::run_simulator(&sys, &config, None);
}
