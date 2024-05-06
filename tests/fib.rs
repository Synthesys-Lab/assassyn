use eda4eda::module_builder;
use eir::{builder::SysBuilder, test_utils::run_simulator};
use num_bigint::BigInt;

#[test]
fn fib() {
  module_builder!(driver()() {
    a = array(int<256>, 1, [0.int<256>]);
    b = array(int<256>, 1, [1.int<256>]);
    aa = a[0];
    bb = b[0];
    cc = aa.add(bb);
    log("fib: {} + {} = {}", aa, bb, cc);
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
  run_simulator(
    &sys,
    &config,
    Some((
      |x| {
        if x.contains("fib") {
          let raw = x.split_whitespace().collect::<Vec<&str>>();
          let len = raw.len();
          let a = BigInt::parse_bytes(raw[len - 5].as_bytes(), 10).unwrap();
          let b = BigInt::parse_bytes(raw[len - 3].as_bytes(), 10).unwrap();
          let c = BigInt::parse_bytes(raw[len - 1].as_bytes(), 10).unwrap();
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
