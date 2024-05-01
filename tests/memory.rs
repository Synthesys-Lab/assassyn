use eda4eda::module_builder;
use eir::{backend, builder::SysBuilder, xform};

module_builder!(
  mem_sink()(rdata:int<32>) {
    log("rdata: {}", rdata);
  }
);

fn sram_sys() -> SysBuilder {
  module_builder!(
    driver(sink, mem)() {
      cnt = array(int<32>, 1);
      v = cnt[0];
      v = v.slice(0, 9);
      async_call mem { raddr: v, r: sink };
      plused = v.add(1);
      cnt[0] = plused;
    }
  );

  let mut sys = SysBuilder::new("sram");
  let sink = mem_sink_builder(&mut sys);
  // TODO: data is a Bits, not Int
  let memory = sys.create_memory(
    "sram",
    eir::ir::DataType::Int(32),
    1024,
    /* latency: [min, max] */ (1, 1),
    None,
  );
  let _driver = driver_builder(&mut sys, sink, memory);
  sys
}

#[test]
fn sram() {
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
    temp_dir: true,
    sim_threshold: 200,
    idle_threshold: 200,
  };

  eir::backend::verilog::elaborate(&sys, &config).unwrap();

  eir::test_utils::run_simulator(&sys, &config, None);
}
