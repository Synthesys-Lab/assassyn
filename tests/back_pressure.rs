use eda4eda::module_builder;
use eir::{builder::SysBuilder, test_utils};

#[test]
fn back_pressure() {
  module_builder!(sub[a:int<32>, b:int<32>][] {
    a = a.pop();
    b = b.pop();
    c = a.sub(b);
    log("sub: {} - {} = {}", a, b, c);
  });

  module_builder!(driver[][lhs, rhs] {
    cnt = array(int<32>, 1);
    k = cnt[0.int<32>];
    v = k.add(1);
    cnt[0] = v;
    add = v.add(v);
    async lhs { a: add };
    async rhs { b: v };
  });

  module_builder!(
    lhs[a:int<32>][suber] {
      v = a.pop();
      rhs = bind suber { a: v };
    }.expose[rhs]
  );

  let mut sys = SysBuilder::new("main");
  let suber = sub_builder(&mut sys);
  let (lhs, rhs) = lhs_builder(&mut sys, suber);
  driver_builder(&mut sys, lhs, rhs);
  eir::builder::verify(&sys);
  println!("{}", sys);
  let o1 = eir::xform::Config {
    rewrite_wait_until: true,
  };
  eir::xform::basic(&mut sys, &o1);
  println!("{}", sys);

  let verilog_name = test_utils::temp_dir(&"back_pressure.sv".to_string());
  let verilog_config = eir::verilog::Config {
    fname: verilog_name,
    sim_threshold: 100,
  };
  eir::verilog::elaborate(&sys, &verilog_config).unwrap();

  let src_name = test_utils::temp_dir(&"back_pressure.rs".to_string());
  let config = eir::sim::Config {
    fname: src_name,
    sim_threshold: 100,
    idle_threshold: 100,
  };

  eir::sim::elaborate(&sys, &config).unwrap();

  let exec_name = test_utils::temp_dir(&"fifo_valid".to_string());
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
