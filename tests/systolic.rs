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
      feast = eager_bind east { west : west };
      async south { north : north };
    }.expose[feast, acc]
  );

  let mut sys = SysBuilder::new("systolic_array");
  let mut pe_array = [[(
    BaseNode::unknown(),
    BaseNode::unknown(),
    BaseNode::unknown(),
  ); 6]; 6];

  module_builder!(sink_row[west:int<32>][] { _v = west.pop(); });
  (1..=4).for_each(|i| {
    pe_array[i][5].0 = sink_row_builder(&mut sys);
    pe_array[i][5].1 = pe_array[i][5].0;
  });
  module_builder!(sink_col[north:int<32>][] { _v = north.pop(); });
  (1..=4).for_each(|i| {
    pe_array[5][i].0 = sink_col_builder(&mut sys);
    pe_array[5][i].1 = pe_array[5][i].0;
  });

  // pripheral module to initialize the first row.
  module_builder!(row_init[][pe] {
    init = bind pe { west : 0 };
  }.expose[init]);

  for i in (1..=4).rev() {
    for j in (1..=4).rev() {
      println!("building {}, {}", i, j);
      let (peeast, _feast, _array) = pe_array[i][j + 1];
      let (_pesouth, fsouth, _array) = pe_array[i + 1][j];
      let (pe, feast, acc) = pe_builder(&mut sys, peeast, fsouth);
      pe_array[i][j] = (pe, pe_array[i][j].1, acc);
      pe_array[i][j + 1].1 = feast;
      println!("done w/ {}, {}", i, j);
    }
    let (init_pe, bound) = row_init_builder(&mut sys, pe_array[i][1].0);
    pe_array[i][1].1 = bound;
    pe_array[i][1].0 = init_pe;
  }

  eprintln!("{}", sys);
}
