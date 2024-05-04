use eda4eda::module_builder;
use eir::{builder::SysBuilder, test_utils::run_simulator};

#[test]
fn adder() {
  module_builder!(adder()(a:int<32>, b:int<32>) {
    c = a.add(b);
    log("adder: {} + {} = {}", a, b, c);
  });

  module_builder!(driver(adder)() {
    cnt = array(int<32>, 1);
    k = cnt[0.int<32>];
    v = k.add(1);
    cnt[0] = v;
    async_call adder { a: v, b: v };
  });

  let mut sys = SysBuilder::new("adder");
  let adder = adder_builder(&mut sys);
  driver_builder(&mut sys, adder);
  eir::builder::verify(&sys);

  println!("{}", sys);

  let config = eir::backend::common::Config::default();

  // TODO(@boyang): Should we also test the verilog backend?
  // eir::backend::verilog::elaborate(&sys, &config).unwrap();

  run_simulator(
    &sys,
    &config,
    Some((
      |x| {
        if x.contains("adder") {
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
