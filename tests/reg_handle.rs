use eda4eda::module_builder;
use eir::frontend::SysBuilder;
use eir::test_utils;

module_builder!(
  squarer[a:int<32>][] {
    a  = a.pop();
    _b = a.mul(a);
  }
);

fn syntactical_sugar() -> SysBuilder {
  module_builder!(
    driver[][sqr] {
      cnt = array(int<32>, 1);
      lock = array(int<1>, 2);
      v = cnt[0];
      is_odd = v.bitwise_and(1);
      v = v.add(1);
      cnt[0] = v;
      spin lock[is_odd] sqr{ a: v };
      lv = lock[is_odd];
      flipped = lv.flip();
      lock[is_odd] = flipped;
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
  eir::xform::basic(&mut sys);
  println!("{}", sys);
  eir::sim::elaborate(&sys, &config).unwrap();
  let exec_name = test_utils::temp_dir(&fname.to_string());
  test_utils::compile(&config.fname, &exec_name);
  // TODO(@were): Make a time timeout here.
  test_utils::run(&exec_name);
}

#[test]
fn reg_handle() {
  let sugar_sys = syntactical_sugar();
  println!("{}", sugar_sys);
  testit("spin_sugar", sugar_sys);
}
