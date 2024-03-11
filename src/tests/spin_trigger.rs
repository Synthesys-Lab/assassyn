use crate::{
  builder::system::{PortInfo, SysBuilder},
  node::IsElement,
  xform, BaseNode, DataType, Module,
};

#[test]
fn spin_trigger() {
  fn squarer(sys: &mut SysBuilder) -> BaseNode {
    let int32 = DataType::int(32);
    let port = PortInfo::new("i0", int32);
    let module = sys.create_module("square", vec![port]);
    sys.set_current_module(&module);
    let i0 = {
      let module = module.as_ref::<Module>(sys).unwrap();
      let i0_port = module.get_input(0).unwrap().clone();
      sys.create_fifo_pop(&i0_port, None, None)
    };
    sys.create_mul(None, &i0, &i0, None);
    module
  }

  fn driver(sys: &mut SysBuilder, dst: BaseNode) {
    let driver = sys.get_driver().upcast();
    sys.set_current_module(&driver);
    let int32 = DataType::int(32);
    let stamp = sys.create_array(&int32, "stamp", 1);
    let zero = sys.get_const_int(&int32, 0);
    let handle = sys.create_handle(&stamp, &zero);
    let a0 = sys.create_array_read(&handle, None);
    let one = sys.get_const_int(&int32, 1);
    let plused = sys.create_add(None, &a0, &one, None);
    sys.create_array_write(&handle, &plused, None);
    let lock = sys.create_array(&DataType::int(1), "lock", 1);
    let lock_handle = sys.create_handle(&lock, &zero);
    sys.create_spin_trigger(&lock_handle, &dst, vec![a0], None);
  }

  let mut sys = SysBuilder::new("main");
  let sqr_module = squarer(&mut sys);
  driver(&mut sys, sqr_module);
  println!("{}", sys);
  xform::rewrite_spin_triggers(&mut sys);
  println!("{}", sys);
}
