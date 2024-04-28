use eda4eda::module_builder;
use eir::builder::SysBuilder;
use eir::test_utils::{self, parse_cycle};

module_builder!(
  squarer[a:int<32>][] {
    a = a.pop();
    b = a.mul(a);
    log("squarer: {}", b);
  }
);

fn syntactical_sugar() -> SysBuilder {
  module_builder!(
    driver[][sqr] {
      cnt = array(int<32>, 1);
      lock = array(int<1>, 1);
      v = cnt[0];
      and_1 = v.bitwise_and(1);
      is_odd = and_1.eq(1);
      is_even = is_odd.flip();
      zero = is_odd.bitwise_and(is_even);
      v = v.add(1);
      cnt[0] = v;
      when is_odd {
        spin lock[zero] sqr{ a: v };
      }
      when is_even {
        lv = lock[0];
        flipped = lv.flip();
        log("flip to {}", flipped);
        lock[0] = flipped;
      }
    }
  );

  let mut res = SysBuilder::new("raw");
  let sqr = squarer_builder(&mut res);
  let _driver = driver_builder(&mut res, sqr);
  res
}

fn testit(fname: &str, mut sys: SysBuilder) {
  let config = eir::sim::Config {
    fname: test_utils::temp_dir(&format!("{}.rs", fname)),
    sim_threshold: 200,
    idle_threshold: 200,
  };
  eir::builder::verify(&sys);

  let o0 = eir::xform::Config {
    rewrite_wait_until: false,
  };
  eir::xform::basic(&mut sys, &o0);
  println!("{}", sys);
  eir::builder::verify(&sys);
  let verilog_name = test_utils::temp_dir(&format!("{}.sv", fname));
  let verilog_config = eir::verilog::Config {
    fname: verilog_name,
    sim_threshold: 100,
  };
  eir::verilog::elaborate(&sys, &verilog_config).unwrap();
  eir::sim::elaborate(&sys, &config).unwrap();
  let exec_name = test_utils::temp_dir(&fname.to_string());
  test_utils::compile(&config.fname, &exec_name);
  // TODO(@were): Make a time timeout here.
  let raw = test_utils::run(&exec_name);
  String::from_utf8(raw.stdout)
    .unwrap()
    .lines()
    .for_each(|l| {
      if l.contains("agent move on") {
        let (cycle, _) = parse_cycle(l);
        assert!(
          cycle % 4 == 1 || cycle % 4 == 2,
          "agent move on {} % 4 = {}",
          cycle,
          cycle % 4
        );
      }
      if l.contains("squarer") {
        let (cycle, _) = parse_cycle(l);
        assert!(cycle % 4 == 2 || cycle % 4 == 3, "{}", l);
      }
    });
}

#[test]
fn reg_handle() {
  let sugar_sys = syntactical_sugar();
  // println!("{}", sugar_sys);
  testit("reg_handle", sugar_sys);
}
