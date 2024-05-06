use eda4eda::module_builder;
use eir::{builder::SysBuilder, test_utils::run_simulator};

#[test]
fn concat() {
  module_builder!(adder()(a:int<32>, b:int<32>) {
    a_valid = a.valid();
    b_valid = b.valid();
    valids = a_valid.concat(b_valid);
    c = a.add(b);
    cc = valids.concat(c);
    // TODO: Change this 32 to 33 later
    log("add with pred: (0b11 << 32) | {} + {} = {}", a, b, cc);
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

  // FIXME(@boyang): Implement FIFOValid!
  // eir::backend::verilog::elaborate(&sys, &config).unwrap();

  run_simulator(
    &sys,
    &config,
    Some((
      |x| {
        if x.contains("add with pred") {
          eprintln!("{}", x);
          let raw = x.split(" ").collect::<Vec<&str>>();
          let len = raw.len();
          let a = raw[len - 5].parse::<i32>().unwrap();
          let b = raw[len - 3].parse::<i32>().unwrap();
          let c = raw[len - 1].parse::<u64>().unwrap();
          let sum = c & 0xFFFFFFFF;
          assert_eq!(sum, (a + b) as u64);
          assert_eq!(c >> 32, 0b11);
          true
        } else {
          false
        }
      },
      Some(100),
    )),
  );
}
