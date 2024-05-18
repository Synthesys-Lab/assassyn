use assassyn::module_builder;
use eir::{builder::SysBuilder, test_utils::run_simulator};

pub fn common_read() {
  module_builder!(
    adder()(a:int<32>, b:int<32>) {
      c = a.add(b);
      log("Simulating module adder {} = {} + {}", c, a, b);
    }
  );

  module_builder!(
    driver(/*external interf*/adder)(/*in-ports*/) {
      cnt     = array(int<32>, 1);
      new_cnt = cnt[0].add(1);
      cnt[0]  = new_cnt;
      cond    = cnt[0].ilt(100);
      when cond {
        async_call adder { a: cnt[0], b: cnt[0] };
      }
    }
  );

  let mut sys = SysBuilder::new("common_read");

  // Create a trivial module.
  let adder = adder_builder(&mut sys);
  // Build the driver module.
  driver_builder(&mut sys, adder);

  eir::builder::verify(&sys);
  println!("{}", sys);

  let mut config = eir::backend::common::Config::default();
  config.sim_threshold = 200;
  config.idle_threshold = 200;

  eir::backend::verilog::elaborate(&sys, &config).unwrap();

  run_simulator(
    &sys,
    &config,
    Some((|x| x.contains("Simulating module adder"), Some(100))),
  );
}
