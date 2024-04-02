use eda4eda::module_builder;
use eir::{builder::SysBuilder, test_utils};

module_builder!(
  squarer[a:int<32>][] {
    a  = a.pop();
    b = a.mul(a);
    log("squarer: {}", b);
  }
);

fn manual() -> SysBuilder {
  module_builder!(
    spin_agent[a:int<32>][sqr] {
      lock = array(int<1>, 1);
      v = lock[0];
      when v {
        a = a.pop();
        async sqr {a: a};
        log("agent move on {}", a);
      }
      nv = v.flip();
      when nv {
        log("agent backpressure");
        async self {};
      }
    }.expose[lock]
  );

  module_builder!(
    driver[][spin_agent, lock] {
      cnt = array(int<32>, 1);
      v = cnt[0];
      and_1 = v.bitwise_and(1);
      is_odd = and_1.eq(1);
      is_even = is_odd.flip();
      v = v.add(1);
      cnt[0] = v;
      when is_odd {
        async spin_agent { a: v };
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
  let (spin_agent, lock) = spin_agent_builder(&mut res, sqr);
  let _driver = driver_builder(&mut res, spin_agent, lock);
  res
}

fn syntactical_sugar() -> SysBuilder {
  module_builder!(
    driver[][sqr] {
      cnt = array(int<32>, 1);
      lock = array(int<1>, 1);
      v = cnt[0];
      and_1 = v.bitwise_and(1);
      is_odd = and_1.eq(1);
      is_even = is_odd.flip();
      v = v.add(1);
      cnt[0] = v;
      when is_odd {
        spin lock[0] sqr{ a: v };
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
  eir::xform::basic(&mut sys);
  // println!("{}", sys);
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
        let toks = l.split_whitespace().collect::<Vec<_>>();
        let len = toks[3].len();
        let cycle = toks[3][1..len - 4].parse::<i32>().unwrap();
        assert!(cycle % 4 == 1 || cycle % 4 == 2, "agent move on");
      }
      if l.contains("squarer") {
        let toks = l.split_whitespace().collect::<Vec<_>>();
        let len = toks[3].len();
        let cycle = toks[3][1..len - 4].parse::<i32>().unwrap();
        assert!(cycle % 4 == 2 || cycle % 4 == 3, "{}", l);
      }
    });
}

#[test]
fn spin_trigger() {
  let raw_sys = manual();
  testit("spin_trigger", raw_sys);

  let sugar_sys = syntactical_sugar();
  // println!("{}", sugar_sys);
  testit("spin_sugar", sugar_sys);
}
