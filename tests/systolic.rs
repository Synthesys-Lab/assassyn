use eda4eda::module_builder;
use eir::{
  frontend::{BaseNode, SysBuilder},
  test_utils,
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

#[derive(Debug, Clone, Copy)]
struct EntryController {
  lock_reg: BaseNode,
  controller: BaseNode,
}

impl EntryController {
  fn new(lock_reg: BaseNode, controller: BaseNode) -> Self {
    Self {
      lock_reg,
      controller,
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
      acc[0] = mac;
      feast = eager_bind east(west);
      async south(north);
    }.expose[feast, acc]
  );

  let mut sys = SysBuilder::new("systolic_array");
  let mut pe_array = [[ProcElem::new(
    BaseNode::unknown(),
    BaseNode::unknown(),
    BaseNode::unknown(),
  ); 6]; 6];

  // # PE Array (1 + 4 + 1) x (1 + 4 + 1)
  //                [Data Pusher] [Data Pusher] [Data Pusher] [Data Pusher]
  // [Data Pusher]  [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
  // [Data Pusher]  [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
  // [Data Pusher]  [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
  // [Data Pusher]  [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
  //                [Sink]        [Sink]        [Sink]        [Sink]

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
    bound = eager_bind dest(data);
  }.expose[bound]);

  // pripheral module to initialize the first row.
  module_builder!(entry_controller[data: int<32>][pusher, next_lock] {
    lock = array(int<1>, 1);
    lv = lock[0];
    nlv = lv.flip();
    when nlv {
      async self {};
    }
    when lv {
      data = data.pop();
      async pusher(data);
      next_lock[0] = nlv;
    }
  }.expose[lock]);

  for i in (1..=4).rev() {
    for j in (1..=4).rev() {
      let peeast = pe_array[i][j + 1].pe;
      let fsouth = pe_array[i + 1][j].bound;
      let (pe, feast, acc) = pe_builder(&mut sys, peeast, fsouth);
      pe_array[i][j].pe = pe;
      pe_array[i][j + 1].bound = feast;
      pe_array[i][j].accumulator = acc;
    }
    let (pusher_pe, bound) = data_pusher_builder(&mut sys, pe_array[i][1].pe);
    pe_array[i][0].pe = pusher_pe;
    pe_array[i][1].bound = bound;
  }

  for i in 1..=4 {
    let (pusher_pe, bound) = data_pusher_builder(&mut sys, pe_array[1][i].pe);
    pe_array[0][i].pe = pusher_pe;
    pe_array[1][i].bound = bound;
  }

  let mut row_ctrls = [EntryController::new(BaseNode::unknown(), BaseNode::unknown()); 6];
  let mut col_ctrls = [EntryController::new(BaseNode::unknown(), BaseNode::unknown()); 6];

  row_ctrls[5].lock_reg = sys.create_array(eir::frontend::DataType::Int(1), "dummy.sentinel", 1);
  col_ctrls[5].lock_reg = row_ctrls[5].lock_reg;

  for i in (1..=4).rev() {
    let (controller, lock) =
      entry_controller_builder(&mut sys, pe_array[i][0].pe, row_ctrls[i + 1].lock_reg);
    row_ctrls[i].controller = controller;
    row_ctrls[i].lock_reg = lock;

    let (controller, lock) =
      entry_controller_builder(&mut sys, pe_array[0][i].pe, col_ctrls[i + 1].lock_reg);
    col_ctrls[i].controller = controller;
    col_ctrls[i].lock_reg = lock;
  }

  // row [[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11], [12, 13, 14, 15]]
  // col [[0, 4, 8, 12], [1, 5, 9, 13], [2, 6, 10, 14], [3, 7, 11, 15]]
  module_builder!(driver[][row1, row2, row3, row4, row_lock, col1, col2, col3, col4, col_lock] {
    cnt = array(int<32>, 1);
    v = cnt[0];
    new_v = v.add(1);
    lt4 = new_v.ilt(4);
    nlt4 = lt4.flip();
    cnt[0] = new_v;
    // before 4, feed the data.
    when lt4 {
      async row1(v);
      row_lock[0] = lt4;
      col_lock[0] = lt4;
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

  driver_builder(
    &mut sys,
    row_ctrls[1].controller,
    row_ctrls[2].controller,
    row_ctrls[3].controller,
    row_ctrls[4].controller,
    row_ctrls[1].lock_reg,
    col_ctrls[1].controller,
    col_ctrls[2].controller,
    col_ctrls[3].controller,
    col_ctrls[4].controller,
    col_ctrls[1].lock_reg,
  );

  eprintln!("{}", sys);

  let src_name = test_utils::temp_dir(&"systolic.rs".to_string());
  let config = eir::sim::Config {
    fname: src_name,
    sim_threshold: 200,
    idle_threshold: 200,
  };

  eir::sim::elaborate(&sys, &config).unwrap();

  let exec_name = test_utils::temp_dir(&"systolic".to_string());
  test_utils::compile(&config.fname, &exec_name);
}
