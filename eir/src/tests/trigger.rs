use super::utils;
use crate::builder::system::SysBuilder;
use crate::{module_builder, sim};

#[test]
fn trigger() {
  module_builder!(
    adder[a:int<32>, b:int<32>][] {
      a  = a.pop();
      b  = b.pop();
      _c = a.add(b);
    }
  );

  module_builder!(
    driver[/*in-ports*/] [/*external interf*/adder] {
      cnt    = array(int<32>, 1);
      read   = cnt[0];
      plus   = read.add(1);
      cnt[0] = plus;
      cond   = read.ilt(100);
      when cond {
        async adder(read, read);
      }
    }
  );

  let mut sys = SysBuilder::new("main");
  // Create a trivial module.
  let m1 = adder_builder(&mut sys);
  // Build the driver module.
  driver_builder(&mut sys, m1);

  println!("{}", sys);

  let src_name = utils::temp_dir(&"trigger.rs".to_string());

  println!("Writing simulator code to {}", src_name);

  let config = sim::Config {
    fname: src_name,
    sim_threshold: 200,
    idle_threshold: 200,
  };

  sim::elaborate(&sys, &config).unwrap();

  let exec_name = utils::temp_dir(&"trigger".to_string());
  utils::compile(&config.fname, &exec_name);

  let output = utils::run(&exec_name);
  let times_invoked = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .filter(|x| x.contains("Simulating module adder"))
    .count();
  assert_eq!(times_invoked, 100);
}
