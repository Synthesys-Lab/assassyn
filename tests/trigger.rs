use eda4eda::module_builder;
use eir::{builder::SysBuilder, test_utils::run_simulator};

#[test]
fn trigger() {
  module_builder!(
    adder()(a:int<32>, b:int<32>) {
      log("Simulating module adder");
      _c = a.add(b);
    }
  );

  module_builder!(
    driver(/*external interf*/adder)(/*in-ports*/) {
      cnt    = array(int<32>, 1);
      read   = cnt[0];
      plus   = read.add(1);
      cnt[0] = plus;
      cond   = read.ilt(100);
      when cond {
        async_call adder { a: read, b: read };
      }
    }
  );

  let mut sys = SysBuilder::new("main");
  // Create a trivial module.
  let adder = adder_builder(&mut sys);
  // Build the driver module.
  driver_builder(&mut sys, adder);

  eir::builder::verify(&sys);
  println!("{}", sys);

  let config = eir::backend::common::Config {
    temp_dir: true,
    sim_threshold: 200,
    idle_threshold: 200,
  };
  // eir::backend::verilog::elaborate(&sys, &config).unwrap();

  eir::backend::simulator::elaborate(&sys, &config).unwrap();

  run_simulator(
    &sys,
    &config,
    Some((|x| x.contains("Simulating module adder"), Some(100))),
  );
}
