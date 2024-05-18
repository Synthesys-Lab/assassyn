use assassyn::module_builder;
use eir::{builder::SysBuilder, xform};

fn dt_conv_sys() -> SysBuilder {
  module_builder!(
    driver()() {
      i32 = 0.int<32>;
      b32 = i32.cast(bits<32>);
      u32 = i32.cast(uint<32>);
      i64 = i32.sext(int<64>);
      log("{} {} {}", b32, u32, i64);
    }
  );

  let mut sys = SysBuilder::new("dt_conv");
  let _driver = driver_builder(&mut sys);
  sys
}

pub fn dt_conv() {
  let mut sys = dt_conv_sys();

  println!("{}", sys);

  eir::builder::verify(&sys);
  let o0 = xform::Config {
    rewrite_wait_until: false,
  };
  eir::xform::basic(&mut sys, &o0);
  eir::builder::verify(&sys);

  println!("{}", sys);

  let config = eir::backend::common::Config::default();

  eir::backend::verilog::elaborate(&sys, &config).unwrap();

  eir::test_utils::run_simulator(&sys, &config, None);
}
