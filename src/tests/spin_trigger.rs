use crate::{
  builder::system::{PortInfo, SysBuilder},
  node::IsElement,
  sim::{self, elaborate},
  tests::utils,
  xform, BaseNode, DataType, Module,
};

#[test]
fn spin_trigger() {
  fn squarer(sys: &mut SysBuilder) -> BaseNode {
    let int32 = DataType::int(32);
    let port = PortInfo::new("i0", int32.clone());
    let module = sys.create_module("square", vec![port]);
    sys.set_current_module(&module);
    let i0 = {
      let module = module.as_ref::<Module>(sys).unwrap();
      let i0_port = module.get_input(0).unwrap().clone();
      sys.create_fifo_pop(&i0_port, None)
    };
    sys.create_mul(Some(int32), &i0, &i0);
    module
  }

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

  let mut sys = SysBuilder::new("main");
  let sqr_module = squarer(&mut sys);
  driver(&mut sys, sqr_module);
  println!("{}", sys);
  xform::basic(&mut sys);
  println!("{}", sys);

  let config = sim::Config {
    fname: utils::temp_dir(&String::from("spin_trigger.rs")),
    sim_threshold: 200,
    idle_threshold: 200,
  };

  elaborate(&sys, &config).unwrap();
  let exec_name = utils::temp_dir(&"spin_trigger".to_string());
  utils::compile(&config.fname, &exec_name);

  // TODO(@were): Make a time timeout here.
  utils::run(&exec_name);
}
