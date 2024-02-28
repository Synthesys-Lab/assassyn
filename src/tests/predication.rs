use crate::{builder::system::SysBuilder, DataType};

#[test]
fn predication() {
  let mut sys = SysBuilder::new("main");
  let int32 = DataType::int(32);
  let a = sys.create_array(&int32, "a", 1);
  let zero = sys.get_const_int(&int32, 0);
  sys.create_array_read(a, zero, None);
}
