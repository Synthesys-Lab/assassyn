use crate::{
  builder::system::{InsertPoint, PortInfo, SysBuilder},
  Module,
  data::Array,
  ir::block::Block,
  module_builder, parse_idx, parse_port, parse_stmts, parse_type, BaseNode, DataType, emit_ports,
};

use paste::paste;

#[test]
fn parser() {
  let mut sys = SysBuilder::new("main");

  // module_builder!(
  //   adder[a:int<32>, b:int<32>] {
  //     externals: [];
  //     a = a.pop();
  //     b = b.pop();
  //     // c = a + b;
  //   }
  // );

  module_builder!(
    driver[] {
      externals: [];
      cnt    = array(int<32>, 1);
      read   = cnt[0];
      plus   = read.add(1);
      cnt[0] = plus;
      cond   = read.ilt(100);
      when cond {
        cnt[0] = 0;
      }
    }
  );

  // let adder = adder_builder(&mut sys);
  driver_builder(&mut sys);
  println!("{}", sys)
}
