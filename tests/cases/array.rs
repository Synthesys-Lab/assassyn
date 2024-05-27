use assassyn::module_builder;
use eir::{
  builder::SysBuilder,
  test_utils::{parse_cycle, run_simulator},
  xform,
};

pub fn array_multi_read() {
  module_builder!(
    mod_a(arr)(a:int<32>) {
      when a.slice(0, 0) {
        arr[0] = a;
      }
    }
  );

  module_builder!(
    mod_b(arr)(a:int<32>) {
      when a.slice(0, 0).flip() {
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

pub fn array_multi_write_in_same_module() {
  module_builder!(
    mod_a(arr)(a:int<32>) {
      when a.slice(0, 0) {
        when a.slice(1, 1) { }
        arr[0] = a;
      }
      when a.slice(0, 0).flip() {
        arr[0] = a.add(1);
      }
    }
  );

  module_builder!(
    driver(a)() {
      cnt    = array(int<32>, 1);
      v      = cnt[0];
      new_v  = v.add(1);
      cnt[0] = new_v;
      async_call a { a : v };
    }
  );

  let mut sys = SysBuilder::new("array_multi_write_in_same_module");
  let arr = sys.create_array(eir::ir::DataType::Int(32), "arr", 1, None, vec![]);
  let mod_a = mod_a_builder(&mut sys, arr);
  // Build the driver module.
  driver_builder(&mut sys, mod_a);

  eir::builder::verify(&sys);
  println!("{}", sys);

  let config = eir::backend::common::Config::default();

  eir::backend::verilog::elaborate(&sys, &config).unwrap();

  run_simulator(&sys, &config, None);
}

pub fn array_partition0() {
  module_builder!(
    driver()() {
      a = array(int<32>, 4, #fully_partitioned);
      cnt = array(int<32>, 1);
      v = cnt[0];
      cnt[0] = v.add(1.int<32>);
      a[0] = v;
      a[1] = v;
      a[2] = v;
      a[3] = v;
      all = add(a[0], a[1], a[2], a[3]);
      log("sum(a[:]) = {}", all);
    }
  );

  let mut sys = SysBuilder::new("array_partition0");
  driver_builder(&mut sys);

  println!("{}", sys);

  let o1 = eir::xform::Config {
    rewrite_wait_until: true,
  };
  let config = eir::backend::common::Config::default();
  xform::basic(&mut sys, &o1);

  println!("{}", sys);

  run_simulator(
    &sys,
    &config,
    Some((
      |x| {
        if x.contains("sum(a[:])") {
          let raw = x.split_whitespace().collect::<Vec<_>>();
          let (cycle, _) = parse_cycle(x);
          let sum = raw[raw.len() - 1].parse::<i32>().unwrap();
          assert_eq!(sum % 4, 0);
          assert_eq!(((cycle as i32) - 1).max(0) * 4, sum);
        }
        false
      },
      None,
    )),
  );
}

pub fn array_partition1() {
  module_builder!(
    driver()() {
      a = array(int<32>, 4, #fully_partitioned);
      cnt = array(int<32>, 1);
      v = cnt[0];
      new_v = v.add(1.int<32>);
      cnt[0] = new_v;
      idx0 = v.slice(0, 1);
      idx1 = new_v.slice(0, 1);
      a[idx0] = v.mul(v).slice(0, 31).bitcast(int<32>);
      a[idx1] = new_v.add(new_v);
      sum = a[idx0].add(a[idx1]);
      log("a[idx0] + a[idx1] = {}", sum);
    }
  );

  let mut sys = SysBuilder::new("array_partition1");
  driver_builder(&mut sys);

  println!("{}", sys);

  let o1 = eir::xform::Config {
    rewrite_wait_until: true,
  };
  let config = eir::backend::common::Config::default();
  xform::basic(&mut sys, &o1);

  println!("{}", sys);

  let mut a = [0, 0, 0, 0];

  run_simulator(&sys, &config, None).lines().for_each(|x| {
    if x.contains("a[idx0] + a[idx1]") {
      let raw = x.split_whitespace().collect::<Vec<_>>();
      let (cycle, _) = parse_cycle(x);
      let idx0 = cycle % 4;
      let idx1 = (cycle + 1) % 4;
      let cycle = cycle as i32;
      let sum = raw[raw.len() - 1].parse::<i32>().unwrap();
      let expect = a[idx0] + a[idx1];
      assert_eq!(expect, sum, "@cycle: {}", cycle);
      a[idx0] = cycle * cycle;
      a[idx1] = (cycle + 1) * 2;
    }
  });
}
