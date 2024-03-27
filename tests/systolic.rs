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
      feast = eager_bind east(west);
      async south(north);
    }.expose[feast, acc]
  );

  let mut sys = SysBuilder::new("systolic_array");
  let mut pe_array = [[(
    BaseNode::unknown(),
    BaseNode::unknown(),
    BaseNode::unknown(),
  ); 6]; 6];

  module_builder!(sink[v:int<32>][] { _v = v.pop(); });
  (1..=4).for_each(|i| {
    pe_array[i][5].0 = sink_builder(&mut sys);
    pe_array[i][5].1 = pe_array[i][5].0;
  });
  (1..=4).for_each(|i| {
    pe_array[5][i].0 = sink_builder(&mut sys);
    pe_array[5][i].1 = pe_array[5][i].0;
  });

  // pripheral module to initialize the first row.
  module_builder!(entry_controller[][pe, next_lock] {
    lock = array(int<1>, 1);
    lv = lock[0];
    when lv {
      async self {};
    }
    nlv = lv.flip();
    when nlv {
      init = eager_bind pe(0.int<32>);
      next_lock[0] = nlv;
    }
  }.expose[init, lock]);

  for i in (1..=4).rev() {
    for j in (1..=4).rev() {
      let (peeast, _feast, _array) = pe_array[i][j + 1];
      let (_pesouth, fsouth, _array) = pe_array[i + 1][j];
      let (pe, feast, acc) = pe_builder(&mut sys, peeast, fsouth);
      pe_array[i][j] = (pe, pe_array[i][j].1, acc);
      pe_array[i][j + 1].1 = feast;
    }
    let (init_pe, bound, lock) =
      entry_controller_builder(&mut sys, pe_array[i][1].0, pe_array[i + 1][1].2);
    pe_array[i][0].0 = init_pe;
    pe_array[i][1].1 = bound;
    pe_array[i][1].0 = init_pe;
  }

  for i in (1..=4).rev() {
    let (init_pe, bound, lock) =
      entry_controller_builder(&mut sys, pe_array[1][i].0, pe_array[1][i + 1].2);
    pe_array[0][i].0 = init_pe;
    pe_array[1][i].1 = bound;
  }

  // row [[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11], [12, 13, 14, 15]]
  // col [[0, 4, 8, 12], [1, 5, 9, 13], [2, 6, 10, 14], [3, 7, 11, 15]]
  module_builder!(driver[][row1, row2, row3, row4, col1, col2, col3, col4] {
    cnt = array(int<32>, 1);
    v = cnt[0];
    new_v = v.add(1);
    lt4 = new_v.ilt(4);
    nlt4 = lt4.flip();
    cnt[0] = new_v;
    // before 4, feed the data.
    when lt4 {
      async row1(v);
      v1 = v.add(1);
      async row2(v1);
      v2 = v1.add(1);
      async row3(v2);
      v3 = v2.add(1);
      async row4(v3);
      async col1(v);
      async col2(v1);
      async col3(v2);
      async col4(v3);
    }
    // after 4, feed zero paddings.
    when nlt4 {
      async row1(0.int<32>);
      async row2(0.int<32>);
      async row3(0.int<32>);
      async row4(0.int<32>);
      async col1(0.int<32>);
      async col2(0.int<32>);
      async col3(0.int<32>);
      async col4(0.int<32>);
    }
  });

  eprintln!("{}", sys);
}
