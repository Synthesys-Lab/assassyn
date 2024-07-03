use assassyn::module_builder;
use eir::{builder::SysBuilder, test_utils::run_simulator};

pub fn bind() {
  module_builder!(sub()(a:int<32>, b:int<32>) {
    c = a.sub(b);
    log("sub: {} - {} = {}", a, b, c);
  });

  module_builder!(driver(lhs, rhs)() {
    cnt = array(int<32>, 1);
    v = cnt[0].add(1);
    cnt[0] = v;
    async_call lhs { a: v.mul(v).slice(0, 31).bitcast(int<32>) };
    async_call rhs { a: v };
  });

  module_builder!(
    lhs(sub)(a:int<32>) {
      aa = bind sub { a: a };
    }.expose(aa)
  );

  module_builder!(
    rhs(sub)(a:int<32>) {
      async_call sub { b: a };
    }
  );

  let mut sys = SysBuilder::new("bind");
  let adder = sub_builder(&mut sys);
  let (lhs, aa) = lhs_builder(&mut sys, adder);
  let rhs = rhs_builder(&mut sys, aa);
  driver_builder(&mut sys, lhs, rhs);
  eir::builder::verify(&sys);
  println!("{}", sys);

  let config = eir::backend::common::Config::default();
  eir::backend::verilog::elaborate(&sys, &config, eir::backend::verilog::Simulator::VCS).unwrap();

  run_simulator(
    &sys,
    &config,
    Some((
      |x| {
        if x.contains("sub") {
          let raw = x.split(" ").collect::<Vec<&str>>();
          let len = raw.len();
          let a = raw[len - 5].parse::<i32>().unwrap();
          let b = raw[len - 3].parse::<i32>().unwrap();
          let c = raw[len - 1].parse::<i32>().unwrap();
          assert_eq!(c, a - b);
          true
        } else {
          false
        }
      },
      Some(99),
    )),
  );
}
