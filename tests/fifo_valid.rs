use eda4eda::module_builder;
use eir::builder::SysBuilder;

#[test]
fn fifo_valid() {
  module_builder!(sub()(a:int<32>, b:int<32>) {
    wait_until {
      a_valid = a.valid();
      b_valid = b.valid();
      both_valid = a_valid.bitwise_and(b_valid);
      both_valid
    } {
      c = a.sub(b);
      log("sub: {} - {} = {}", a, b, c);
    }
  });

  module_builder!(driver(lhs, rhs)() {
    cnt = array(int<32>, 1);
    k = cnt[0.int<32>];
    v = k.add(1);
    cnt[0] = v;
    add = v.add(v);
    async_call lhs { v: add };
    async_call rhs { b: v };
  });

  module_builder!(
    lhs(sub)(v:int<32>) {
      rhs = bind sub { a: v };
    }.expose(rhs)
  );

  let mut sys = SysBuilder::new("main");
  let suber = sub_builder(&mut sys);
  let (lhs, rhs) = lhs_builder(&mut sys, suber);
  driver_builder(&mut sys, lhs, rhs);
  eir::builder::verify(&sys);
  println!("{}", sys);

  let config = eir::backend::common::Config {
    temp_dir: true,
    sim_threshold: 100,
    idle_threshold: 100,
  };
  eir::backend::verilog::elaborate(&sys, &config).unwrap();

  eir::test_utils::run_simulator(
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
