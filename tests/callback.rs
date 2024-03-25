use eda4eda::module_builder;
use eir::{frontend::SysBuilder, sim, test_utils};

#[test]
fn callback() {
  module_builder!(
    driver[][sqr, memory_read] {
      cnt = array(int<32>, 1);
      v = cnt[0];
      async memory_read { addr: v, func: sqr };
      plused = v.add(1);
      cnt[0] = plused;
    }
  );

  module_builder!(
    sqr[a:int<32>][] {
      a = a.pop();
      _b = a.mul(a);
      // TODO(@were): Implement a logger!
      // log("{}", b);
    }
  );

  module_builder!(
    memory_read[addr:int<32>, func: module(int<32>)][] {
      addr = addr.pop();
      func = func.pop();
      // TODO(@were): How to make this wait for 200 cycles?
      dram = array(int<32>, 16384);
      value = dram[addr];
      async func(value);
    }
  );

  let mut sys = SysBuilder::new("callback");
  let memory_read = memory_read_builder(&mut sys);
  let sqr = sqr_builder(&mut sys);
  let _ = driver_builder(&mut sys, sqr, memory_read);

  println!("{}", sys);

  let src_name = test_utils::temp_dir(&"callback.rs".to_string());
  let exec_name = test_utils::temp_dir(&"callback".to_string());
  let config = sim::Config {
    fname: src_name,
    idle_threshold: 1000,
    sim_threshold: 1000,
  };

  sim::elaborate(&sys, &config).unwrap();
  test_utils::compile(&config.fname, &exec_name);
  let _ = test_utils::run(&exec_name);
}
