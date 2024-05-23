use assassyn::module_builder;
use eir::{builder::SysBuilder, test_utils::run_simulator};

pub fn array_multi_read() {
  use assassyn::module_builder;

  module_builder!(
    mod_a(arr)(a:int<32>) {
      is_odd = a.slice(0, 0);
      when is_odd {
        arr[0] = a;
      }
    }
  );

  module_builder!(
    mod_b(arr)(a:int<32>) {
      is_odd = a.slice(0, 0);
      is_even = is_odd.flip();
      when is_even {
        arr[0] = a;
      }
    }
  );

  module_builder!(
    mod_c(arr)(a:int<32>) {
      v = arr[0];
      log("a = {} arr = {}", a, v);
    }
  );

  module_builder!(
    driver(a, b, c)() {
      cnt    = array(int<32>, 1);
      v      = cnt[0];
      new_v  = v.add(1);
      cnt[0] = new_v;
      async_call a { a : v };
      async_call b { a : v };
      async_call c { a : v };
    }
  );

  let mut sys = SysBuilder::new("array_multi_read");
  let arr = sys.create_array(eir::ir::DataType::Int(32), "arr", 1, None, vec![]);
  let mod_a = mod_a_builder(&mut sys, arr);
  let mod_b = mod_b_builder(&mut sys, arr);
  let mod_c = mod_c_builder(&mut sys, arr);
  // Build the driver module.
  driver_builder(&mut sys, mod_a, mod_b, mod_c);

  eir::builder::verify(&sys);
  println!("{}", sys);

  let config = eir::backend::common::Config::default();

  eir::backend::verilog::elaborate(&sys, &config).unwrap();

  run_simulator(
    &sys,
    &config,
    Some((
      /*Condition Assertion*/
      |x| {
        if x.contains("arr") {
          let raw = x.split(" ").collect::<Vec<&str>>();
          let len = raw.len();
          let a = raw[len - 4].parse::<i32>().unwrap();
          let arr = raw[len - 1].parse::<i32>().unwrap();
          assert!(a == 0 || arr == a - 1);
          true
        } else {
          false
        }
      },
      /*Expected Lines*/ Some(100),
    )),
  );
}

pub fn array_partition() {
  module_builder!(
    driver()() {
      a = array(int<32>, 4, #fully_partition);
      cnt = array(int<32>, 1);
      v = cnt[0];
      cnt[0] = v.add(1.int<32>);
      a[0] = v;
      a[1] = v;
      a[2] = v;
      a[3] = v;
      all = a[0].add(a[1]).add(a[2]).add(a[3]);
      log("sum(a[:]) = {}", all);
    }
  );

  let mut sys = SysBuilder::new("array_partition");
  driver_builder(&mut sys);

  println!("{}", sys);

  let config = eir::backend::common::Config::default();

  run_simulator(&sys, &config, None);
}
