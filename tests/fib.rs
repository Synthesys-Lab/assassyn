use eda4eda::module_builder;
use eir::{builder::SysBuilder, test_utils::run_simulator};

#[test]
fn adder() {
  module_builder!(driver()() {
    a = array(int<256>, 1, [0]);
    b = array(int<256>, 1, [1.int<256>]);
    aa = a[0];
    bb = b[0];
    cc = aa.add(bb);
    a[0] = bb;
    b[0] = cc;
  });

  let mut sys = SysBuilder::new("adder");
  driver_builder(&mut sys);
  eir::builder::verify(&sys);

  println!("{}", sys);

  let config = eir::backend::common::Config::default();

  // TODO(@boyang): Should we also test the verilog backend?
  eir::backend::verilog::elaborate(&sys, &config).unwrap();

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
