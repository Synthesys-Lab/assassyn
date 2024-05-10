use eda4eda::module_builder;
use eir::{builder::SysBuilder, test_utils::run_simulator};

pub fn multi_call() {
  module_builder!(sqr()(a:int<32>) #no_arbiter {
    b = a.mul(a);
    log("adder: {} * {} = {}", a, a, b);
  });

  module_builder!(
    arbiter(sqr)(a0:int<32>, a1:int<32>)
      #explicit_pop, #allow_partial_call, #no_arbiter {
      wait_until {
        a0_valid = a0.valid();
        a1_valid = a1.valid();
        valid = a0_valid.bitwise_or(a1_valid);
        valid
      } {
        hot_valid = a1_valid.concat(a0_valid);
        // grant is a one-hot vector
        grant_1h = array(int<2>, 1, [1.int<2>]);
        gv = grant_1h[0];
        gv_flip = gv.flip();
        hi = gv_flip.bitwise_and(hot_valid);
        lo = gv.bitwise_and(hot_valid);
        hi_nez = hi.neq(0.bits<2>);
        new_grant = default lo.case(hi_nez, hi);
        grant0 = new_grant.eq(1.bits<2>);
        grant1 = new_grant.eq(2.bits<2>);
        when grant0 {
          log("grants even");
          a0 = a0.pop();
          async_call sqr { a: a0 };
          grant_1h[0] = 1.int<2>;
        }
        when grant1 {
          log("grants odd");
          a1 = a1.pop();
          async_call sqr { a: a1 };
          grant_1h[0] = 2.int<2>;
        }
      }
    }
  );

  module_builder!(driver(arbiter)() {
    cnt = array(int<32>, 1);
    k = cnt[0.int<32>];
    v = k.add(1);
    even = v.mul(2);
    even = even.slice(0, 31);
    even = even.cast(int<32>);
    odd = even.add(1);
    cnt[0] = v;
    is_odd = v.bitwise_and(1);
    when is_odd {
      async_call arbiter { a0: even };
      async_call arbiter { a1: odd };
    }
  });

  let mut sys = SysBuilder::new("multi_call");
  let adder = sqr_builder(&mut sys);
  let arbiter = arbiter_builder(&mut sys, adder);
  driver_builder(&mut sys, arbiter);
  eir::builder::verify(&sys);

  println!("{}", sys);

  let config = eir::backend::common::Config::default();

  // TODO(@boyang): Should we also test the verilog backend?
  // eir::backend::verilog::elaborate(&sys, &config).unwrap();

  let mut last_grant: Option<i32> = None;
  run_simulator(&sys, &config, None).lines().for_each(|x| {
    if x.contains("grants odd") {
      assert!(last_grant.map_or(true, |x| x == 0));
      last_grant = Some(1);
    }
    if x.contains("grants even") {
      assert!(last_grant.map_or(true, |x| x == 1));
      last_grant = Some(0);
    }
  });
}
