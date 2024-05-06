use eda4eda::module_builder;
use eir::{builder::SysBuilder, test_utils::run_simulator};

#[test]
fn fib() {
  module_builder!(driver()() {
    a = array(int<256>, 1, [0.int<256>]);
    b = array(int<256>, 1, [1.int<256>]);
    aa = a[0];
    bb = b[0];
    cc = aa.add(bb);
    a[0] = bb;
    b[0] = cc;
  });

  let mut sys = SysBuilder::new("fib");
  driver_builder(&mut sys);
  eir::builder::verify(&sys);

  println!("{}", sys);

  let config = eir::backend::common::Config::default();

  // TODO(@boyang): Should we also test the verilog backend?
  // eir::backend::verilog::elaborate(&sys, &config).unwrap();

  // TODO(@were): Check the results.
  run_simulator(&sys, &config, None);
}
