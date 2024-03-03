use crate::{builder::system::SysBuilder, xform, DataType};

#[test]
fn predication() {
  let mut sys = SysBuilder::new("main");
  let int32 = DataType::int(32);
  let a = sys.create_array(&int32, "a", 1);
  let odd = sys.create_array(&int32, "odd", 1);
  /*let even = */sys.create_array(&int32, "even", 1);
  let zero = sys.get_const_int(&int32, 0);
  let a0 = sys.create_array_read(&a, &zero, None);
  let one = sys.get_const_int(&int32, 1);
  let plused = sys.create_add(None, &a0, &one, None);
  sys.create_array_write(&a, &zero, &plused, None);
  let is_odd = sys.create_bitwise_and(None, &a0, &one, None);
  let odd0 = sys.create_array_read(&odd, &zero, None);
  let acc_odd = sys.create_add(None, &odd0, &one, Some(&is_odd));
  sys.create_array_write(&odd, &zero, &acc_odd, None);
  println!("{}", sys);
  xform::propagate_predications(&mut sys);
  println!("{}", sys);
}
