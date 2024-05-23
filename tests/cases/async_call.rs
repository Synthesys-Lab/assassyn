pub fn async_call() {
  use assassyn::module_builder;
  use eir::{builder::SysBuilder, test_utils::run_simulator};

  module_builder!(
    adder()(a:int<32>, b:int<32>) {
      c = a.add(b);
      log("Simulating module adder {} = {} + {}", c, a, b);
    }
  );

  module_builder!(
    driver(/*external interf*/adder)(/*in-ports*/) {
      cnt    = array(int<32>, 1);
      v      = cnt[0];
      new_v  = v.add(1);
      cnt[0] = new_v;
      when v.ilt(100.int<32>) {
        async_call adder { a: v, b: v };
      }
    }
  );

  let mut sys = SysBuilder::new("async_call");
  // Create a trivial module.
  let adder = adder_builder(&mut sys);
  // Build the driver module.
  driver_builder(&mut sys, adder);

  eir::builder::verify(&sys);
  println!("{}", sys);

  let config = eir::backend::common::Config {
    sim_threshold: 200,
    idle_threshold: 200,
    ..Default::default()
  };

  eir::backend::verilog::elaborate(&sys, &config).unwrap();

  run_simulator(
    &sys,
    &config,
    Some((|x| x.contains("Simulating module adder"), Some(100))),
  );
}
