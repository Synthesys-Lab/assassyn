use eda4eda::module_builder;
use eir::{backend, builder::SysBuilder, xform};

#[test]
fn callback() {
  module_builder!(
    driver(sqr, memory_read)() {
      cnt = array(int<32>, 1);
      v = cnt[0];
      async_call memory_read { v: cnt[0], func: sqr };
      cnt[0] = v.add(1);
    }
  );

  module_builder!(
    sqr()(a:int<32>) {
      b = a.mul(a);
      log("sqr: {}^2 = {}", a, b);
    }
  );

  module_builder!(
    agent()(v:int<32>, func: module(int<32>)) {
      async_call func(v);
    }
  );

  let mut sys = SysBuilder::new("callback");
  let agent = agent_builder(&mut sys);
  let sqr = sqr_builder(&mut sys);
  let _ = driver_builder(&mut sys, sqr, agent);
  println!("Before:\n{}", sys);
  let o0 = xform::Config {
    rewrite_wait_until: false,
  };
  eir::xform::basic(&mut sys, &o0);
  println!("After:\n{}", sys);

  let config = backend::common::Config::default();

  eir::backend::verilog::elaborate(&sys, &config).unwrap();

  eir::test_utils::run_simulator(
    &sys,
    &config,
    Some((
      |x| {
        if x.contains("sqr: ") {
          let raw = x.split(" ").collect::<Vec<&str>>();
          let len = raw.len();
          let raw_len = raw[len - 3].len();
          let a = raw[len - 3][0..(raw_len - 2)].parse::<i32>().unwrap();
          let b = raw[len - 1].parse::<i32>().unwrap();
          assert_eq!(b, a * a);
          true
        } else {
          false
        }
      },
      Some(99),
    )),
  );
}
