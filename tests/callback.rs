use eda4eda::module_builder;
use eir::frontend::SysBuilder;

#[test]
fn callback() {
  module_builder!(
    driver[][sqr, memory_read] {
      cnt = array(int<32>, 1);
      v = cnt[0];
      async memory_read { addr: v, callback: sqr };
      plused = v.add(1);
      cnt[0] = plused;
    }
  );

  module_builder!(
    sqr[a:int<32>][] {
      a = a.pop();
      _b = a.mul(a);
      // TODO(@were): Implement a logger?
      // log("{}", b);
    }
  );

  module_builder!(
    memory_read[addr:int<32>, callback: module(int<32>)][] {
      addr = addr.pop();
      callback = callback.pop();
      // TODO(@were): How to make this wait for 200 cycles?
      dram = array(int<32>, 16384);
      value = dram[addr];
      // async callback(value);
    }
  );

  let mut sys = SysBuilder::new("callback");
  let memory_read = memory_read_builder(&mut sys);
  let sqr = sqr_builder(&mut sys);
  let _ = driver_builder(&mut sys, sqr, memory_read);

  println!("{}", sys);
}
