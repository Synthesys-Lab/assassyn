use crate::{builder::system::SysBuilder, node::IsElement, DataType, BaseNode};

#[test]
fn spin_trigger() {

  fn empty(sys: &mut SysBuilder) -> BaseNode {
    sys.create_module("empty", vec![])
  }

  fn driver(sys: &mut SysBuilder, dst: BaseNode) {
    let driver = sys.get_driver().upcast();
    sys.set_current_module(driver);
    let int32 = DataType::int(32);
    let a = sys.create_array(&int32, "a", 1);
    let zero = sys.get_const_int(&int32, 0);
    let a0 = sys.create_array_read(&a, &zero, None);
    let one = sys.get_const_int(&int32, 1);
    let plused = sys.create_add(None, &a0, &one, None);
    sys.create_array_write(&a, &zero, &plused, None);
    let lock = sys.create_array(&DataType::int(1), "lock", 1);
    sys.create_spin_trigger(&lock, &zero, &dst, vec![], None);
  }

  let mut sys = SysBuilder::new("main");
  let empty_module = empty(&mut sys);
  driver(&mut sys, empty_module);

  println!("{}", sys);
}
