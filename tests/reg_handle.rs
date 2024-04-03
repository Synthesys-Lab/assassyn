use eda4eda::module_builder;
use eir::builder::SysBuilder;
use eir::test_utils;

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
      lock = array(int<1>, 2);
      v = cnt[0];
      is_odd = v.bitwise_and(1);
      v = v.add(1);
      cnt[0] = v;
      spin lock[is_odd] sqr{ a: v };
      lv = lock[is_odd];
      flipped = lv.flip();
      log("lock[{}] = {}", is_odd, flipped);
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
  let output = test_utils::run(&exec_name);
  let mut idx = false;
  String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .for_each(|line| {
      if line.contains("squarer") {
        // Cycle:   0 1 2 3 4 5 6 7 8 9 10
        // lock[0]: 0 1 1 0 0 1 1 0 0 1 1
        // lock[1]: 0 0 1 1 0 0 1 1 0 0 1
        // trigger: x 0 1 0 1 0 1 0 1 0 1
        // if lock[trigger[cycle - 2]] = 1 then we should have a squarer
        // For each cycle.
        let (cycle, _) = test_utils::parse_cycle(line);
        // then trigger is triggered the cycle before last cycle, (cycle - 2), since last cycle
        // we were on the agent module. the lock is checked last cycle,
        // so we are calculating the lock value of the last cycle.
        // lock[0] is (cycle - 1 + 1) / 2 % 2
        // lock[1] is (cycle - 1) / 2 % 2
        let lock = [cycle / 2 % 2, (cycle - 1) / 2 % 2];
        // the lock should be true.
        assert!(
          lock[idx as usize] != 0,
          "cycle: {}, idx: {}, lock: {:?}",
          cycle,
          idx,
          lock
        );
        idx = !idx;
      }
    });
}

#[test]
fn reg_handle() {
  let sugar_sys = syntactical_sugar();
  println!("{}", sugar_sys);
  testit("reg_handle", sugar_sys);
}
