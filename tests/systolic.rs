use eda4eda::{module_builder, testbench_builder};
use eir::{builder::SysBuilder, ir::node::BaseNode, ir::Module, test_utils};

#[derive(Debug, Clone, Copy)]
struct ProcElem {
  pe: BaseNode,
  bound: BaseNode,
  accumulator: BaseNode,
}

impl ProcElem {
  fn new(pe: BaseNode, bound: BaseNode, accumulator: BaseNode) -> Self {
    Self {
      pe,
      bound,
      accumulator,
    }
  }
}

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
      log("MAC value: {}", mac);
      acc[0] = mac;
      feast = eager_bind east(west);
      fsouth = eager_bind south(north);
    }.expose[feast, fsouth, acc]
  );

  let mut sys = SysBuilder::new("systolic_array");
  let mut pe_array = [[ProcElem::new(
    BaseNode::unknown(),
    BaseNode::unknown(),
    BaseNode::unknown(),
  ); 6]; 6];

  // # PE Array (4 + 1) x (4 + 1)
  //          [Pusher]      [Pusher]      [Pusher]      [Pusher]
  // [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
  // [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
  // [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
  // [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
  //          [Sink]        [Sink]        [Sink]        [Sink]

  // Sink Sentinels
  module_builder!(sink[v:int<32>][] { _v = v.pop(); });
  (1..=4).for_each(|i| {
    pe_array[i][5].pe = sink_builder(&mut sys);
    pe_array[i][5].bound = pe_array[i][5].pe;
  });
  (1..=4).for_each(|i| {
    pe_array[5][i].pe = sink_builder(&mut sys);
    pe_array[5][i].bound = pe_array[5][i].pe;
  });

  module_builder!(data_pusher[data: int<32>][dest] {
    data = data.pop();
    log("pusher pushes {}", data);
    bound = eager_bind dest(data);
  }.expose[bound]);

  for i in (1..=4).rev() {
    for j in (1..=4).rev() {
      let peeast = pe_array[i][j + 1].pe;
      let fsouth = pe_array[i + 1][j].bound;
      let (pe, feast, fsouth, acc) = pe_builder(&mut sys, peeast, fsouth);
      pe.as_mut::<Module>(&mut sys)
        .unwrap()
        .set_name(format!("pe_{}_{}", i, j));
      pe_array[i][j].pe = pe;
      pe_array[i][j].bound = pe;
      pe_array[i][j + 1].bound = feast;
      pe_array[i][j].accumulator = acc;
      pe_array[i + 1][j].bound = fsouth;
    }
    let (pusher_pe, bound) = data_pusher_builder(&mut sys, pe_array[i][1].bound);
    pusher_pe
      .as_mut::<Module>(&mut sys)
      .unwrap()
      .set_name(format!("row_pusher_{}", i));
    pe_array[i][0].pe = pusher_pe;
    pe_array[i][1].bound = bound;
  }

  for i in 1..=4 {
    let (pusher_pe, bound) = data_pusher_builder(&mut sys, pe_array[1][i].bound);
    pusher_pe
      .as_mut::<Module>(&mut sys)
      .unwrap()
      .set_name(format!("col_pusher_{}", i));
    pe_array[0][i].pe = pusher_pe;
    pe_array[1][i].bound = bound;
  }

  // what if i do this?
  // Cycle:
  //    6                    15
  //      5               11 14
  //        4           7 10 13
  //          3       3 6 9  12
  //            2     2 5 8
  //              1   1 4
  //                0 0
  //          3 2 1 0 P P P  P
  //        7 6 5 4   P P P  P
  //    11 10 9 8     P P P  P
  // 15 14 13 12      P P P  P

  // row [[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11], [12, 13, 14, 15]]
  // col [[0, 4, 8, 12], [1, 5, 9, 13], [2, 6, 10, 14], [3, 7, 11, 15]]
  testbench_builder!(
    testbench[row1, row2, row3, row4, col1, col2, col3, col4] {
      // 0 0
      // 0 P P P  P
      //   P P P  P
      //   P P P  P
      //   P P P  P
      0: col1(0);
      0: row1(0);
      // 1 1 4
      // 1 P P P  P
      // 4 P P P  P
      //   P P P  P
      //   P P P  P
      1: row1(1);
      1: col1(1);
      1: col2(4);
      1: row2(4);
      // 2 2 5 8
      // 2 P P P  P
      // 5 P P P  P
      // 8 P P P  P
      //   P P P  P
      2: row1(2);
      2: col1(2);
      2: col2(5);
      2: row2(5);
      2: row3(8);
      2: col3(8);
      // 3  3 6 9  12
      // 3  P P P  P
      // 6  P P P  P
      // 9  P P P  P
      // 12 P P P  P
      3: row1(3);
      3: col1(3);
      3: col2(6);
      3: row2(6);
      3: row3(9);
      3: col3(9);
      3: row4(12);
      3: col4(12);
      // 4    7 10 13
      //    P P P  P
      // 7  P P P  P
      // 10 P P P  P
      // 13 P P P  P
      4: row2(7);
      4: col2(7);
      4: row3(10);
      4: col3(10);
      4: row4(13);
      4: col4(13);
      //  5    11 14
      //    P P P  P
      //    P P P  P
      // 11 P P P  P
      // 14 P P P  P
      5: row3(11);
      5: col3(11);
      5: row4(14);
      5: col4(14);
      //   6      15
      //    P P P  P
      //    P P P  P
      //    P P P  P
      // 15 P P P  P
      6: row4(15);
      6: col4(15);
    }
  );

  let src_name = test_utils::temp_dir(&"systolic.rs".to_string());
  let config = eir::sim::Config {
    fname: src_name,
    sim_threshold: 100,
    idle_threshold: 100,
  };

  eir::sim::elaborate(&sys, &config, vec![]).unwrap();

  let exec_name = test_utils::temp_dir(&"systolic".to_string());
  test_utils::compile(&config.fname, &exec_name);
  let output = test_utils::run(&exec_name);
  let output = String::from_utf8(output.stdout).unwrap();

  let mut a = [[0; 4]; 4];
  let mut b = [[0; 4]; 4];
  let mut c = [[0; 4]; 4];
  for i in 0..4 {
    for j in 0..4 {
      a[i][j] = i * 4 + j;
      b[j][i] = i * 4 + j;
    }
  }

  for i in 0..4 {
    for j in 0..4 {
      for k in 0..4 {
        c[i][j] += a[i][k] * b[k][j];
      }
    }
  }

  for i in 0..4 {
    for j in 0..4 {
      let expected = c[i][j];
      let actual = output
        .lines()
        .rfind(|line| {
          if line.contains(format!("pe_{}_{}", i + 1, j + 1).as_str()) {
            println!("{}", line);
            true
          } else {
            false
          }
        })
        .unwrap();
      eprintln!("{}", actual);
      let actual = actual
        .split_whitespace()
        .last()
        .unwrap()
        .parse::<i32>()
        .unwrap();
      assert_eq!(expected as i32, actual);
    }
  }
}
