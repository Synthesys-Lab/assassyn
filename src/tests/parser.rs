use crate::{
  builder::system::{InsertPoint, PortInfo, SysBuilder},
  emit_ports,
  ir::block::Block,
  module_builder, parse_idx, parse_port, parse_stmts, parse_type, BaseNode, DataType, Module,
};

use paste::paste;

#[test]
fn parser() {
  let mut sys = SysBuilder::new("main");

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

  let adder = adder_builder(&mut sys);
  driver_builder(&mut sys, adder);
  println!("{}", sys)
}
