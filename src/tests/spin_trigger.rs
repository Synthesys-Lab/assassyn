use crate::{
  module_builder,
  frontend::*,
  sim::{self, elaborate},
  tests::utils,
  xform,
};

fn raw() -> SysBuilder {

  module_builder!(
    squarer[a:int<32>][] {
      a = a.pop();
      _b = a.mul(a);
    }
  );

  module_builder!(
    driver[][sqr] {
      lock = array(int<1>, 1);
      cnt = array(int<32>, 1);
      v = cnt[0];
      is_odd = v.bitwise_and(1);
      is_even = is_odd.flip();
      when is_odd {
        spin lock[0] sqr(v);
      }
      when is_even {
        flipped = lock.flip();
        lock[0] = flipped;
      }
    }.expose(lock)
  );

  let mut res = SysBuilder::new("raw");

  let sqr = squarer_builder(&mut res);
  let _driver = driver_builder(&mut res, sqr);

  res
}

#[test]
fn spin_trigger() {



  fn driver(sys: &mut SysBuilder, dst: BaseNode) {
    let driver = sys.create_module("driver", vec![]);
    sys.set_current_module(&driver);
    let int32 = DataType::int(32);
    let stamp = sys.create_array(&int32, "cnt", 1);
    let zero = sys.get_const_int(&int32, 0);
    let a0ptr = sys.create_array_ptr(&stamp, &zero);
    let a0 = sys.create_array_read(&a0ptr);
    let one = sys.get_const_int(&int32, 1);
    let is_odd = sys.create_bitwise_and(None, &a0, &one);
    let is_even = sys.create_flip(&is_odd);
    let plused = sys.create_add(None, &a0, &one);
    sys.create_array_write(&a0ptr, &plused);
    let lock = sys.create_array(&DataType::int(1), "lock", 1);
    let lock_ptr = sys.create_array_ptr(&lock, &zero);
    let orig_block = sys.get_current_block().unwrap().upcast();
    let block = sys.create_block(Some(is_odd));
    sys.set_current_block(block.clone());
    sys.create_spin_trigger(&lock_ptr, &dst, vec![a0]);
    sys.set_current_block(orig_block);
    let block = sys.create_block(Some(is_even));
    sys.set_current_block(block.clone());
    let lock_val = sys.create_array_read(&lock_ptr);
    let flipped = sys.create_flip(&lock_val);
    sys.create_array_write(&lock_ptr, &flipped);
  }

  let raw_sys = raw();

  // driver(&mut sys, sqr_module);
  // println!("{}", sys);
  // xform::basic(&mut sys);
  // println!("{}", sys);

  let config = sim::Config {
    fname: utils::temp_dir(&String::from("spin_trigger.rs")),
    sim_threshold: 200,
    idle_threshold: 200,
  };

  // elaborate(&sys, &config).unwrap();
  let exec_name = utils::temp_dir(&"spin_trigger".to_string());
  utils::compile(&config.fname, &exec_name);

  // TODO(@were): Make a time timeout here.
  utils::run(&exec_name);
}
