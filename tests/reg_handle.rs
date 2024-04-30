use eda4eda::module_builder;
use eir::builder::SysBuilder;
use eir::test_utils::{parse_cycle, run_simulator};

module_builder!(
  squarer()(a:int<32>) {
    b = a.mul(a);
    log("squarer: {}", b);
  }
);

fn sys_impl() -> SysBuilder {
  module_builder!(
    driver(sqr)() {
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

  let mut res = SysBuilder::new("reg_handle");
  let sqr = squarer_builder(&mut res);
  let _driver = driver_builder(&mut res, sqr);
  res
}

fn testit(mut sys: SysBuilder) {
  let config = eir::backend::common::Config {
    temp_dir: true,
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
  eir::backend::verilog::elaborate(&sys, &config).unwrap();

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

#[test]
fn reg_handle() {
  let sugar_sys = sys_impl();
  // println!("{}", sugar_sys);
  testit(sugar_sys);
}
