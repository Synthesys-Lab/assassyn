use crate::{
  frontend::*,
  module_builder,
  sim::{self, elaborate},
  tests::utils,
};

fn manual() -> SysBuilder {
  module_builder!(
    squarer[a:int<32>][] {
      a = a.pop();
      _b = a.mul(a);
    }
  );

  module_builder!(
    spin_agent[a:int<32>][sqr] {
      lock = array(int<1>, 1);
      v = lock[0];
      when v {
        a = a.pop();
        async sqr(a);
      }
      nv = v.flip();
      when nv {
        async self();
      }
    }.expose(lock)
  );

  module_builder!(
    driver[][spin_agent, lock] {
      cnt = array(int<32>, 1);
      v = cnt[0];
      is_odd = v.bitwise_and(1);
      is_even = is_odd.flip();
      v = v.add(1);
      cnt[0] = v;
      when is_odd {
        async spin_agent(v);
      }
      when is_even {
        lv = lock[0];
        flipped = lv.flip();
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

#[test]
fn spin_trigger() {
  let raw_sys = manual();

  println!("{}", raw_sys);

  let config = sim::Config {
    fname: utils::temp_dir(&String::from("spin_trigger.rs")),
    sim_threshold: 200,
    idle_threshold: 200,
  };

  elaborate(&raw_sys, &config).unwrap();
  let exec_name = utils::temp_dir(&"spin_trigger".to_string());
  utils::compile(&config.fname, &exec_name);

  // TODO(@were): Make a time timeout here.
  utils::run(&exec_name);
}
