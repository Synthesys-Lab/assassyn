use eda4eda::module_builder;
use eir::{builder::SysBuilder, test_utils::run_simulator};

pub fn multi_call() {
  module_builder!(sqr()(a:int<32>) {
    b = a.mul(a);
    log("adder: {} * {} = {}", a, a, b);
  });

  module_builder!(driver(sqr)() {
    cnt    = array(int<32>, 1);
    k      = cnt[0.int<32>];
    v      = k.add(1);
    even   = v.mul(2).slice(0, 31).cast(int<32>);
    odd    = even.add(1);
    cnt[0] = v;
    is_odd = v.bitwise_and(1);
    when is_odd {
      async_call sqr { a: even };
      async_call sqr { a: odd };
    }
  });

  let mut sys = SysBuilder::new("multi_call");
  let adder = sqr_builder(&mut sys);
  driver_builder(&mut sys, adder);
  eir::builder::verify(&sys);
  let pass = eir::xform::Config {
    rewrite_wait_until: true,
  };
  eir::xform::basic(&mut sys, &pass);

  // println!("{}", sys);

  let config = eir::backend::common::Config::default();

  // TODO(@boyang): Should we also test the verilog backend?
  // eir::backend::verilog::elaborate(&sys, &config).unwrap();

  let mut last_grant: Option<i32> = None;
  run_simulator(&sys, &config, None).lines().for_each(|x| {
    if x.contains("grants odd") {
      assert!(last_grant.map_or(true, |x| x == 0));
      last_grant = Some(1);
    }
    if x.contains("grants even") {
      assert!(last_grant.map_or(true, |x| x == 1));
      last_grant = Some(0);
    }
  });
}
