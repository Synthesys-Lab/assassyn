use assassyn::module_builder;
use eir::{builder::SysBuilder, test_utils::run_simulator};

pub fn concat() {
  module_builder!(adder()(a:int<32>, b:int<32>) {
    c = a.concat(b);
    // TODO: Change this 32 to 33 later
    log("add with pred: {} << 32 + {} = {}", a, b, c);
  });

  module_builder!(driver(adder)() {
    cnt = array(int<32>, 1);
    k = cnt[0.int<32>];
    v = k.add(1);
    cnt[0] = v;
    async_call adder { a: v, b: v };
  });

  let mut sys = SysBuilder::new("concat");
  let adder = adder_builder(&mut sys);
  driver_builder(&mut sys, adder);
  eir::builder::verify(&sys);

  println!("{}", sys);

  let config = eir::backend::common::Config::default();

  eir::backend::verilog::elaborate(&sys, &config, eir::backend::verilog::Simulator::VCS).unwrap();

  run_simulator(
    &sys,
    &config,
    Some((
      |x| {
        if x.contains("add with pred") {
          eprintln!("{}", x);
          let raw = x.split(" ").collect::<Vec<&str>>();
          let len = raw.len();
          let ref_a = raw[len - 7].parse::<i32>().unwrap();
          let ref_b = raw[len - 3].parse::<i32>().unwrap();
          let c = raw[len - 1].parse::<u64>().unwrap();
          let a = c >> 32 & 0xFFFFFFFF;
          let b = c & 0xFFFFFFFF;
          assert_eq!(a, ref_a as u64);
          assert_eq!(b, ref_b as u64);
          true
        } else {
          false
        }
      },
      Some(100),
    )),
  );
}
