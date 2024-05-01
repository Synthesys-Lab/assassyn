use eda4eda::module_builder;
use eir::{builder::SysBuilder, test_utils::run_simulator};

#[test]
fn bind() {
  module_builder!(sub()(a:int<32>, b:int<32>) {
    c = a.sub(b);
    log("sub: {} - {} = {}", a, b, c);
  });

  module_builder!(driver(lhs, rhs)() {
    cnt = array(int<32>, 1);
    k = cnt[0.int<32>];
    v = k.add(1);
    cnt[0] = v;
    mul = v.add(v);
    async_call lhs { a: mul };
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

  let config = eir::backend::common::Config {
    temp_dir: true,
    sim_threshold: 100,
    idle_threshold: 100,
  };
  // eir::backend::verilog::elaborate(&sys, &config).unwrap();

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
