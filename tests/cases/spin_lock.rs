use assassyn::module_builder;
use eir::{
  builder::SysBuilder,
  test_utils::{parse_cycle, run_simulator},
  xform,
};

pub fn spin_lock() {
  module_builder!(
    squarer()(a:int<32>) {
      b = a.mul(a);
      log("squarer: {}", b);
    }
  );

  module_builder!(
    spin_agent(sqr, lock)(a:int<32>) {
      wait_until { v = lock[0]; v } {
        async_call sqr { a: a };
        log("agent move on, {}", a);
      }
    }
  );

  module_builder!(
    driver(spin_agent, lock)() {
      cnt = array(int<32>, 1);
      v = cnt[0];
      is_odd = v.slice(0, 0);
      is_even = is_odd.flip();
      v = v.add(1);
      cnt[0] = v;
      when is_odd {
        async_call spin_agent { a: v };
      }
      when is_even {
        lv = lock[0];
        flipped = lv.flip();
        log("flip to {}", flipped);
        lock[0] = flipped;
      }
    }
  );

  let mut sys = SysBuilder::new("spin_trigger");
  let sqr = squarer_builder(&mut sys);
  let lock = sys.create_array(eir::ir::DataType::Int(1), "lock", 1, None, vec![]);
  let spin_agent = spin_agent_builder(&mut sys, sqr, lock);
  let _driver = driver_builder(&mut sys, spin_agent, lock);

  let config = eir::backend::common::Config {
    sim_threshold: 200,
    idle_threshold: 200,
    ..Default::default()
  };

  eir::builder::verify(&sys);

  let o0 = xform::Config {
    rewrite_wait_until: false,
  };
  eir::xform::basic(&mut sys, &o0);
  eir::builder::verify(&sys);

  // println!("{}", sys);
  eir::backend::verilog::elaborate(&sys, &config, eir::backend::verilog::Simulator::VCS).unwrap();

  run_simulator(
    &sys,
    &config,
    Some((
      |l| {
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
        false
      },
      None,
    )),
  );
}
