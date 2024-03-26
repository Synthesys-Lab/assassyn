use eda4eda::module_builder;
use eir::frontend::{BaseNode, SysBuilder};

#[test]
fn systolic_array() {
  module_builder!(
    pe[west:int<32>, north:int<32>][east, south] {
      west = west.pop();
      north = north.pop();
      c = west.mul(north);
      acc = array(int<32>, 1);
      val = acc[0];
      mac = val.add(c);
      acc[0] = mac;
      async east { west : west };
      fsouth = bind south { north : north };
    }.expose[fsouth, acc]
  );

  module_builder!(sink[_v:int<32>][] {});

  let mut sys = SysBuilder::new("systolic_array");
  let mut pe_array = [[(
    BaseNode::unknown(),
    BaseNode::unknown(),
    BaseNode::unknown(),
  ); 6]; 6];
  (1..=4).for_each(|i| {
    pe_array[i][5].0 = sink_builder(&mut sys);
  });
  (1..=4).for_each(|i| {
    pe_array[5][i].0 = sink_builder(&mut sys);
  });
  for i in (1..=4).rev() {
    for j in (1..=4).rev() {
      let east = pe_array[i][j + 1].0;
      let south = pe_array[i + 1][j].0;
      pe_array[i][j] = pe_builder(&mut sys, east, south);
    }
  }
  eprintln!("{}", sys);
}
