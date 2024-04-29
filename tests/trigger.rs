use eda4eda::module_builder;
use eir::{builder::SysBuilder, test_utils};

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

  println!("{}", sys);

  let verilog_name = test_utils::temp_dir(&"trigger.sv".to_string());
  let verilog_config = eir::verilog::Config {
    fname: verilog_name,
    sim_threshold: 200,
  };
  eir::verilog::elaborate(&sys, &verilog_config).unwrap();

  let src_name = test_utils::temp_dir(&"trigger.rs".to_string());
  let config = eir::sim::Config {
    fname: src_name,
    sim_threshold: 200,
    idle_threshold: 200,
  };

  eir::builder::verify(&sys);
  eir::sim::elaborate(&sys, &config).unwrap();

  let exec_name = test_utils::temp_dir(&"trigger".to_string());
  test_utils::compile(&config.fname, &exec_name);

  let output = test_utils::run(&exec_name);
  let times_invoked = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .filter(|x| x.contains("Simulating module adder"))
    .count();
  assert_eq!(times_invoked, 100);
}
