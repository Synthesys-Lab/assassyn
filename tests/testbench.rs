use eda4eda::module_builder;
use eir::{
  builder::SysBuilder,
  test_utils::{parse_cycle, run_simulator},
};

#[test]
fn testbench() {
  module_builder!(testbench()() {
    cycle 0 { log("cycle 0"); }
    cycle 2 { log("cycle 2"); }
    cycle 80 { log("cycle 80"); }
  });

  let mut sys = SysBuilder::new("testbench");
  testbench_builder(&mut sys);
  eir::builder::verify(&sys);

  eprintln!("{}", sys);

  let config = eir::backend::common::Config {
    temp_dir: true,
    sim_threshold: 101,
    idle_threshold: 100,
  };
  eir::backend::verilog::elaborate(&sys, &config).unwrap();

  run_simulator(
    &sys,
    &config,
    Some((
      |l| {
        if l.contains("testbench") {
          let (cycle, _) = parse_cycle(l);
          assert!(
            cycle == 0 || cycle == 2 || cycle == 80,
            "testbench triggered on {}",
            cycle
          );
        }
        false
      },
      None,
    )),
  );
}
