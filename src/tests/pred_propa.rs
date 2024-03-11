use crate::{
  builder::system::SysBuilder,
  expr::{Expr, Opcode},
  ir::block::Block,
  node::IsElement,
  xform, DataType,
};

fn check(sys: &mut SysBuilder, before: bool) {
  let int32 = DataType::int(32);
  let zero = sys.get_const_int(&int32, 0);
  let one = sys.get_const_int(&int32, 1);
  let (a, odd) = {
    assert_eq!(sys.array_iter().count(), 2);
    let odd = sys.get_array("odd").unwrap().upcast();
    let a = sys.get_array("a").unwrap().upcast();
    (a, odd)
  };
  let ptr_a0 = sys.create_handle(&a, &zero);
  let ptr_odd0 = sys.create_handle(&odd, &zero);

  let body = sys.get_driver().get_body();
  let mut iter = body.iter();
  let read = iter.next().unwrap();
  {
    let read = read.as_ref::<Expr>(&sys).unwrap();
    assert_eq!(read.get_opcode(), Opcode::Load);
    assert_eq!(read.get_operand(0).unwrap(), &ptr_a0);
  }
  let add = iter.next().unwrap();
  {
    let add = add.as_ref::<Expr>(&sys).unwrap();
    assert_eq!(add.get_opcode(), Opcode::Add);
    assert_eq!(add.get_operand(0).unwrap(), read);
    assert_eq!(add.get_operand(1).unwrap(), &one);
  }
  let write = iter.next().unwrap();
  {
    let write = write.as_ref::<Expr>(&sys).unwrap();
    assert_eq!(write.get_opcode(), Opcode::Store);
    assert_eq!(write.get_operand(0).unwrap(), &ptr_a0);
    assert_eq!(write.get_operand(1).unwrap(), add);
  }
  let and = iter.next().unwrap();
  {
    let and = and.as_ref::<Expr>(&sys).unwrap();
    assert_eq!(and.get_opcode(), Opcode::BitwiseAnd);
    assert_eq!(and.get_operand(0).unwrap(), read);
    assert_eq!(and.get_operand(1).unwrap(), &one);
  }
  let block = iter.next().unwrap();
  {
    let block = block.as_ref::<Block>(&sys).unwrap();
    let mut expr_iter = block.iter();
    let read = expr_iter.next().unwrap();
    {
      let read = read.as_ref::<Expr>(&sys).unwrap();
      assert_eq!(read.get_opcode(), Opcode::Load);
      assert_eq!(read.get_operand(0).unwrap(), &ptr_odd0);
    }
    let (mut iter_to_check, mut iter_to_end) = if before {
      (iter, expr_iter)
    } else {
      (expr_iter, iter)
    };
    assert!(iter_to_end.next().is_none());
    let acc_odd = iter_to_check.next().unwrap();
    {
      let acc_odd = acc_odd.as_ref::<Expr>(&sys).unwrap();
      assert_eq!(acc_odd.get_opcode(), Opcode::Add);
      assert_eq!(acc_odd.get_operand(0).unwrap(), read);
      assert_eq!(acc_odd.get_operand(1).unwrap(), &one);
    }
    let write = iter_to_check.next().unwrap();
    {
      let write = write.as_ref::<Expr>(&sys).unwrap();
      assert_eq!(write.get_opcode(), Opcode::Store);
      assert_eq!(write.get_operand(0).unwrap(), &ptr_odd0);
      assert_eq!(write.get_operand(1).unwrap(), acc_odd);
    }
    assert!(iter_to_check.next().is_none());
  }
}

#[test]
fn predication_propagation() {
  let mut sys = SysBuilder::new("main");
  let int32 = DataType::int(32);
  let a = sys.create_array(&int32, "a", 1);
  let odd = sys.create_array(&int32, "odd", 1);
  let zero = sys.get_const_int(&int32, 0);
  let ptr_a = sys.create_handle(&a, &zero);
  let a0 = sys.create_array_read(&ptr_a, None);
  let one = sys.get_const_int(&int32, 1);
  let plused = sys.create_add(None, &a0, &one, None);
  sys.create_array_write(&ptr_a, &plused, None);
  let is_odd = sys.create_bitwise_and(None, &a0, &one, None);
  let ptr_odd = sys.create_handle(&odd, &zero);
  let odd0 = sys.create_array_read(&ptr_odd, Some(is_odd.clone()));
  let acc_odd = sys.create_add(None, &odd0, &one, None);
  sys.create_array_write(&ptr_odd, &acc_odd, None);
  println!("{}", sys);
  check(&mut sys, true);
  xform::propagate_predications(&mut sys);
  println!("{}", sys);
  check(&mut sys, false);
}
