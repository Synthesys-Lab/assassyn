use eda4eda::module_builder;
use eir::{builder::SysBuilder, test_utils};

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
    async lhs { a: mul };
    async rhs { a: v };
  });

  module_builder!(
    lhs(sub)(a:int<32>) {
      aa = eager_bind sub { a: a };
    }.expose(aa)
  );

  module_builder!(
    rhs(sub)(a:int<32>) {
      async sub { b: a };
    }
  );

  let mut sys = SysBuilder::new("main");
  let adder = sub_builder(&mut sys);
  let (lhs, aa) = lhs_builder(&mut sys, adder);
  let rhs = rhs_builder(&mut sys, aa);
  driver_builder(&mut sys, lhs, rhs);
  eir::builder::verify(&sys);
  println!("{}", sys);

  let verilog_name = test_utils::temp_dir(&"bind.sv".to_string());
  let verilog_config = eir::verilog::Config {
    fname: verilog_name,
    sim_threshold: 100,
  };
  eir::verilog::elaborate(&sys, &verilog_config).unwrap();

  let src_name = test_utils::temp_dir(&"bind.rs".to_string());
  let config = eir::sim::Config {
    fname: src_name,
    sim_threshold: 100,
    idle_threshold: 100,
  };

  eir::sim::elaborate(&sys, &config).unwrap();

  let exec_name = test_utils::temp_dir(&"bind".to_string());
  test_utils::compile(&config.fname, &exec_name);

  let output = test_utils::run(&exec_name);
  let times_invoked = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .filter(|x| {
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
    })
    .count();
  assert_eq!(times_invoked, 99);
}
