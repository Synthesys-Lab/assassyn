use eda4eda::module_builder;
use eir::{builder::SysBuilder, test_utils};

#[test]
fn inline0() {
  module_builder!(adder(a, b)() {
    c = a.add(b);
    log("adder: {} + {} = {}", a, b, c);
  });

  module_builder!(driver()() {
    cnt = array(int<32>, 1);
    k = cnt[0.int<32>];
    v = k.add(1);
    cnt[0] = v;
    inline adder(v, v)();
  });

  let mut sys = SysBuilder::new("main");
  driver_builder(&mut sys);
  eir::builder::verify(&sys);

  eprintln!("{}", sys);

  let verilog_name = test_utils::temp_dir(&"inline0.sv".to_string());
  let verilog_config = eir::verilog::Config {
    fname: verilog_name,
    sim_threshold: 101,
  };
  eir::verilog::elaborate(&sys, &verilog_config).unwrap();

  let src_name = test_utils::temp_dir(&"inline0.rs".to_string());
  let config = eir::sim::Config {
    fname: src_name,
    sim_threshold: 100,
    idle_threshold: 100,
  };

  eir::sim::elaborate(&sys, &config).unwrap();

  let exec_name = test_utils::temp_dir(&"inline0".to_string());
  test_utils::compile(&config.fname, &exec_name);

  let output = test_utils::run(&exec_name);
  let times_invoked = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .filter(|x| {
      if x.contains("add") {
        let raw = x.split(" ").collect::<Vec<&str>>();
        let len = raw.len();
        let a = raw[len - 5].parse::<i32>().unwrap();
        let b = raw[len - 3].parse::<i32>().unwrap();
        let c = raw[len - 1].parse::<i32>().unwrap();
        assert_eq!(c, a + b);
        true
      } else {
        false
      }
    })
    .count();
  assert_eq!(times_invoked, 100);
}

#[test]
fn inline1() {
  module_builder!(ae(a, b)() {
    c = a.add(b);
    eq = a.eq(b);
  }.expose(c, eq));

  module_builder!(driver()() {
    cnt = array(int<32>, 1);
    k = cnt[0.int<32>];
    v = k.add(1);
    cnt[0] = v;
    a, e = inline ae(v, v)();
    log("add: {} + {} = {}", v, v, a);
    log("eq: {} == {} ? {}", v, v, e);
  });

  let mut sys = SysBuilder::new("main");
  driver_builder(&mut sys);
  eir::builder::verify(&sys);

  eprintln!("{}", sys);

  let verilog_name = test_utils::temp_dir(&"inline1.sv".to_string());
  let verilog_config = eir::verilog::Config {
    fname: verilog_name,
    sim_threshold: 101,
  };
  eir::verilog::elaborate(&sys, &verilog_config).unwrap();

  let src_name = test_utils::temp_dir(&"inline1.rs".to_string());
  let config = eir::sim::Config {
    fname: src_name,
    sim_threshold: 100,
    idle_threshold: 100,
  };

  eir::sim::elaborate(&sys, &config).unwrap();

  let exec_name = test_utils::temp_dir(&"inline1".to_string());
  test_utils::compile(&config.fname, &exec_name);

  let output = test_utils::run(&exec_name);
  let raw = String::from_utf8(output.stdout).unwrap();
  let times_invoked = raw
    .lines()
    .filter(|x| {
      if x.contains("add") {
        let raw = x.split(" ").collect::<Vec<&str>>();
        let len = raw.len();
        let a = raw[len - 5].parse::<i32>().unwrap();
        let b = raw[len - 3].parse::<i32>().unwrap();
        let c = raw[len - 1].parse::<i32>().unwrap();
        assert_eq!(c, a + b);
        true
      } else if x.contains("eq") {
        let raw = x.split(" ").collect::<Vec<&str>>();
        let len = raw.len();
        let a = raw[len - 5].parse::<i32>().unwrap();
        let b = raw[len - 3].parse::<i32>().unwrap();
        let c = raw[len - 1].parse::<bool>().unwrap();
        assert_eq!(c, a == b);
        true
      } else {
        false
      }
    })
    .count();
  assert_eq!(times_invoked, 200);
}
