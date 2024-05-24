use eir::xform;

pub fn cond_cse() {
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
      when v.ilt(100.int<32>) {
        log("aaa");
      }
    }
  );

  let mut sys = SysBuilder::new("cond_cse");
  // Create a trivial module.
  let adder = adder_builder(&mut sys);
  // Build the driver module.
  driver_builder(&mut sys, adder);

  eir::builder::verify(&sys);
  let o1 = xform::Config {
    rewrite_wait_until: true,
  };
  eir::xform::basic(&mut sys, &o1);
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
