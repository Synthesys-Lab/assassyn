use eda4eda::module_builder;
use eir::{builder::SysBuilder, test_utils::run_simulator};

// #[test]
fn multi_call() {
  module_builder!(adder()(a:int<32>, b:int<32>) {
    c = a.add(b);
    log("adder: {} + {} = {}", a, b, c);
  });

  module_builder!(
    arbiter(adder)(a0:int<32>, b0:int<32>, a1:int<32>, b1:int<32>)
      #explicit_pop, #allow_partial_call, #no_arbiter {
      wait_until {
        a0_valid = a0.valid();
        b0_valid = b0.valid();
        valid0 = a0_valid.bitwise_and(b0_valid);
        a1_valid = a1.valid();
        b1_valid = b1.valid();
        valid1 = a1_valid.bitwise_and(b1_valid);
        valid = valid0.bitwise_or(valid1);
        valid
      } {
        hot_valid = valid0.concat(valid1);
        // grant is a one-hot vector
        last_grant = array(int<1>, 1);
        gv = last_grant[0];
        // gv_1h = lut[0b10, 0b01][gv];
        gv_zero = gv.eq(0);
        gv_1h = default 2.int<32>.case(gv_zero, 1);
        gv_1h_flip = gv.flip();
        hi = gv_1h_flip.bitwise_and(hot_valid);
        lo = gv_1h.bitwise_and(hot_valid);
        hi_flip = hi.flip();
        new_grant = default hi.case(hi_flip, lo);
        grant0 = new_grant.eq(1);
        grant1 = new_grant.eq(2);
        when grant0 {
          a0 = a0.pop();
          b0 = b0.pop();
          async_call adder { a: a0, b: b0 };
          last_grant[0] = 0;
        }
        when grant1 {
          a1 = a1.pop();
          b1 = b1.pop();
          async_call adder { a: a1, b: b1 };
          last_grant[0] = 1;
        }
      }
    }
  );

  module_builder!(driver(arbiter)() {
    cnt = array(int<32>, 1);
    k = cnt[0.int<32>];
    v = k.add(1);
    cnt[0] = v;
    is_odd = v.bitwise_and(1);
    when is_odd {
      async_call arbiter { a0: v, b0: v };
      async_call arbiter { a1: v, b1: v };
    }
  });

  let mut sys = SysBuilder::new("multi_call");
  let adder = adder_builder(&mut sys);
  driver_builder(&mut sys, adder);
  eir::builder::verify(&sys);

  println!("{}", sys);

  let config = eir::backend::common::Config::default();

  // TODO(@boyang): Should we also test the verilog backend?
  // eir::backend::verilog::elaborate(&sys, &config).unwrap();

  run_simulator(
    &sys,
    &config,
    Some((
      |x| {
        if x.contains("adder") {
          let raw = x.split(" ").collect::<Vec<&str>>();
          let len = raw.len();
          let a = raw[len - 5].parse::<i32>().unwrap();
          let b = raw[len - 3].parse::<i32>().unwrap();
          let c = raw[len - 1].parse::<i32>().unwrap();
          assert_eq!(c, a + b);
          true
        } else {
          false
        }
      },
      Some(100),
    )),
  );
}
