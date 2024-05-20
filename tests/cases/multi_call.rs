use assassyn::module_builder;
use eir::{builder::SysBuilder, test_utils::run_simulator};

pub fn multi_call() {
  module_builder!(adder()(a:int<32>, b:int<32>) {
    c = a.add(b);
    log("adder: {} + {} = {}", a, b, c);
  });

  module_builder!(driver(sqr)() {
    cnt    = array(int<32>, 1);
    k      = cnt[0.int<32>];
    v      = k.add(1);
    even   = v.mul(2).slice(0, 31).bitcast(int<32>);
    even2  = even.mul(2).slice(0, 31).bitcast(int<32>);
    odd    = even.add(1);
    odd2   = odd.mul(2).slice(0, 31).bitcast(int<32>);
    cnt[0] = v;
    is_odd = v.bitwise_and(1);
    when is_odd {
      // TODO(@were): Enforce the partial call.
      async_call sqr { a: even, b: even2 };
      async_call sqr { a: odd,  b: odd2 };
    }
  });

  let mut sys = SysBuilder::new("multi_call");
  let adder = adder_builder(&mut sys);
  driver_builder(&mut sys, adder);
  eir::builder::verify(&sys);
  let pass = eir::xform::Config {
    rewrite_wait_until: true,
  };
  eir::xform::basic(&mut sys, &pass);

  println!("{}", sys);

  let config = eir::backend::common::Config::default();

  // TODO(@boyang): Should we also test the verilog backend?
  // eir::backend::verilog::elaborate(&sys, &config).unwrap();

  let mut last_grant = None;
  run_simulator(&sys, &config, None).lines().for_each(|x| {
    if x.contains("adder") {
      let raw = x.split_whitespace().collect::<Vec<&str>>();
      let len = raw.len();
      let a = raw[len - 5].parse::<i32>().unwrap();
      let b = raw[len - 3].parse::<i32>().unwrap();
      let c = raw[len - 1].parse::<i32>().unwrap();
      assert_eq!(a + b, c);
      assert!(last_grant.map_or(true, |last| { last % 2 != a % 2 }));
      last_grant = Some(a);
    }
  });
}
