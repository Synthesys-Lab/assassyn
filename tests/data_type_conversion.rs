use eda4eda::module_builder;
use eir::{backend, builder::SysBuilder, xform};

fn i2b_sys() -> SysBuilder {
  module_builder!(
    driver()() {
      i32 = 0.int<32>;
      _b32 = i32.cast(bits<32>);
    }
  );

  let mut sys = SysBuilder::new("i2b");
  let _driver = driver_builder(&mut sys);
  sys
}

#[test]
fn i2b() {
  let mut sys = i2b_sys();

  println!("{}", sys);

  eir::builder::verify(&sys);
  let o0 = xform::Config {
    rewrite_wait_until: false,
  };
  eir::xform::basic(&mut sys, &o0);
  eir::builder::verify(&sys);

  println!("{}", sys);

  let config = backend::common::Config {
    temp_dir: true,
    sim_threshold: 200,
    idle_threshold: 200,
  };

  eir::backend::verilog::elaborate(&sys, &config).unwrap();

  eir::test_utils::run_simulator(&sys, &config, None);
}
