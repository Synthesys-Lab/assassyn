use assassyn::module_builder;
use eir::{
  builder::SysBuilder,
  ir::{node::BaseNode, Module},
  test_utils::run_simulator,
};

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

pub fn systolic_array() {
  module_builder!(
    pe(east, south)(west:int<32>, north:int<32>) #eager_bind {
      c = west.mul(north);
      acc = array(int<64>, 1);
      val = acc[0];
      mac = val.add(c);
      log("MAC value: {} * {} + {} = {}", west, north, val, mac);
      acc[0] = mac;
      feast = bind east(west);
      async_call south(north);
    }.expose(feast, acc)
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
  module_builder!(sink()(_v:int<32>) { });
  (1..=4).for_each(|i| {
    pe_array[i][5].pe = sink_builder(&mut sys);
    pe_array[i][5].bound = pe_array[i][5].pe;
  });
  (1..=4).for_each(|i| {
    pe_array[5][i].pe = sink_builder(&mut sys);
    pe_array[5][i].bound = pe_array[5][i].pe;
  });

  module_builder!(row_pusher(dest)(data: int<32>) #eager_bind {
    log("pushes {}", data);
    bound = bind dest(data);
  }.expose(bound));

  for i in (1..=4).rev() {
    for j in (1..=4).rev() {
      let peeast = pe_array[i][j + 1].pe;
      let fsouth = pe_array[i + 1][j].bound;
      let (pe, feast, acc) = pe_builder(&mut sys, peeast, fsouth);
      pe.as_mut::<Module>(&mut sys)
        .unwrap()
        .set_name(format!("pe_{}_{}", i, j));
      pe_array[i][j].pe = pe;
      pe_array[i][j].bound = pe;
      pe_array[i][j + 1].bound = feast;
      pe_array[i][j].accumulator = acc;
      pe_array[i + 1][j].bound = fsouth;
    }
    let (pusher_pe, bound) = row_pusher_builder(&mut sys, pe_array[i][1].bound);
    pusher_pe
      .as_mut::<Module>(&mut sys)
      .unwrap()
      .set_name(format!("row_pusher_{}", i));
    pe_array[i][0].pe = pusher_pe;
    pe_array[i][1].bound = bound;
  }

  module_builder!(col_pusher(dest)(data: int<32>) #eager_bind {
    log("pushes {}", data);
    async_call dest(data);
  });
  for i in 1..=4 {
    let pusher_pe = col_pusher_builder(&mut sys, pe_array[1][i].bound);
    pusher_pe
      .as_mut::<Module>(&mut sys)
      .unwrap()
      .set_name(format!("col_pusher_{}", i));
    pe_array[0][i].pe = pusher_pe;
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
  module_builder!(testbench(col1, col2, col3, col4, row1, row2, row3, row4)() {
    cycle 0 {
      // 0 0
      // 0 P P P  P
      //   P P P  P
      //   P P P  P
      //   P P P  P
      async_call col1(0);
      async_call row1(0);
    }
    cycle 1 {
      // 1 1 4
      // 1 P P P  P
      // 4 P P P  P
      //   P P P  P
      //   P P P  P
      async_call row1(1);
      async_call col1(1);
      async_call col2(4);
      async_call row2(4);
    }
    cycle 2 {
      // 2 2 5 8
      // 2 P P P  P
      // 5 P P P  P
      // 8 P P P  P
      //   P P P  P
      async_call row1(2);
      async_call col1(2);
      async_call col2(5);
      async_call row2(5);
      async_call row3(8);
      async_call col3(8);
    }
    cycle 3 {
      // 3  3 6 9  12
      // 3  P P P  P
      // 6  P P P  P
      // 9  P P P  P
      // 12 P P P  P
      async_call row1(3);
      async_call col1(3);
      async_call col2(6);
      async_call row2(6);
      async_call row3(9);
      async_call col3(9);
      async_call row4(12);
      async_call col4(12);
    }
    cycle 4 {
      // 4    7 10 13
      //    P P P  P
      // 7  P P P  P
      // 10 P P P  P
      // 13 P P P  P
      async_call row2(7);
      async_call col2(7);
      async_call row3(10);
      async_call col3(10);
      async_call row4(13);
      async_call col4(13);
    }
    cycle 5 {
      //  5    11 14
      //    P P P  P
      //    P P P  P
      // 11 P P P  P
      // 14 P P P  P
      async_call row3(11);
      async_call col3(11);
      async_call row4(14);
      async_call col4(14);
    }
    cycle 6 {
      //   6      15
      //    P P P  P
      //    P P P  P
      //    P P P  P
      // 15 P P P  P
      async_call row4(15);
      async_call col4(15);
    }
  });

  testbench_builder(
    &mut sys,
    pe_array[0][1].pe,
    pe_array[0][2].pe,
    pe_array[0][3].pe,
    pe_array[0][4].pe,
    pe_array[1][0].pe,
    pe_array[2][0].pe,
    pe_array[3][0].pe,
    pe_array[4][0].pe,
  );

  println!("{}", sys);
  eir::builder::verify(&sys);

  let config = eir::backend::common::Config::default();
  eir::backend::verilog::elaborate(&sys, &config).unwrap();

  let output = run_simulator(&sys, &config, None);

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
