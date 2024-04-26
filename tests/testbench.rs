use eda4eda::module_builder;
use eir::{
  builder::SysBuilder,
  test_utils::{self, parse_cycle},
};

#[test]
fn testbench() {
  module_builder!(testbench[][] {
    cycle 0 { log("cycle 0"); }
    cycle 2 { log("cycle 2"); }
    cycle 80 { log("cycle 80"); }
  });

  let mut sys = SysBuilder::new("main");
  testbench_builder(&mut sys);
  eir::builder::verify(&sys);

  eprintln!("{}", sys);

  let verilog_name = test_utils::temp_dir(&"testbench.sv".to_string());
  let verilog_config = eir::verilog::Config {
    fname: verilog_name,
    sim_threshold: 101,
  };
  eir::verilog::elaborate(&sys, &verilog_config).unwrap();

  let src_name = test_utils::temp_dir(&"testbench.rs".to_string());
  let config = eir::sim::Config {
    fname: src_name,
    sim_threshold: 101,
    idle_threshold: 100,
  };

  eir::sim::elaborate(&sys, &config).unwrap();

  let exec_name = test_utils::temp_dir(&"testbench".to_string());
  test_utils::compile(&config.fname, &exec_name);

  let raw = test_utils::run(&exec_name);
  String::from_utf8(raw.stdout)
    .unwrap()
    .lines()
    .for_each(|l| {
      if l.contains("testbench") {
        let (cycle, _) = parse_cycle(l);
        assert!(
          cycle == 0 || cycle == 2 || cycle == 80,
          "testbench triggered on {}",
          cycle
        );
      }
    });
}
