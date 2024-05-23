pub fn array() {
  use assassyn::module_builder;
  use eir::{builder::SysBuilder, test_utils::run_simulator};

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

  let mut sys = SysBuilder::new("array");
  let arr = sys.create_array(eir::ir::DataType::Int(32), "arr", 1, None);
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
