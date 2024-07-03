use assassyn::module_builder;
use eir::{builder::SysBuilder, test_utils::run_simulator};

pub fn inline0() {
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

  let mut sys = SysBuilder::new("inline0");
  driver_builder(&mut sys);
  eir::builder::verify(&sys);

  eprintln!("{}", sys);

  let config = eir::backend::common::Config::default();
  eir::backend::verilog::elaborate(&sys, &config, eir::backend::verilog::Simulator::VCS).unwrap();

  run_simulator(
    &sys,
    &config,
    Some((
      |x| {
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
      },
      Some(100),
    )),
  );
}

pub fn inline1() {
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

  let mut sys = SysBuilder::new("inline1");
  driver_builder(&mut sys);
  eir::builder::verify(&sys);

  eprintln!("{}", sys);

  let config = eir::backend::common::Config::default();
  eir::backend::verilog::elaborate(&sys, &config, eir::backend::verilog::Simulator::VCS).unwrap();

  run_simulator(
    &sys,
    &config,
    Some((
      |x| {
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
      },
      Some(200),
    )),
  );
}
