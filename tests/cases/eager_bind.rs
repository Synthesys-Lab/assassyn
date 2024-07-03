use assassyn::module_builder;
use eir::builder::SysBuilder;

pub fn eager_bind() {
  module_builder!(sub()(a:int<32>, b:int<32>) #eager_callee {
    c = a.sub(b);
    log("sub: {} - {} = {}", a, b, c);
  });

  module_builder!(driver(lhs, rhs)() {
    cnt = array(int<32>, 1);
    k = cnt[0.int<32>];
    v = k.add(1);
    cnt[0] = v;
    vv = v.add(v);
    async_call lhs { v: vv };
    async_call rhs { v: v };
  });

  module_builder!(
    lhs(sub)(v:int<32>) {
      bound = bind sub { a: v };
    }.expose(bound)
  );

  module_builder!(
    rhs(bound)(v:int<32>) {
      _bound = bind bound { b: v };
    }
  );

  let mut sys = SysBuilder::new("eager_bind");
  let sub = sub_builder(&mut sys);
  let (lhs, aa) = lhs_builder(&mut sys, sub);
  let rhs = rhs_builder(&mut sys, aa);
  driver_builder(&mut sys, lhs, rhs);
  println!("{}", sys);
  eir::builder::verify(&sys);

  let config = eir::backend::common::Config::default();
  eir::backend::verilog::elaborate(&sys, &config, eir::backend::verilog::Simulator::VCS).unwrap();

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
