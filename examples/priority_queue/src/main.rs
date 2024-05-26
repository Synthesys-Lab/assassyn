use eir::{builder::SysBuilder, test_utils::run_simulator};

fn main() {
  module_builder!(
    deq_sink(deq_valid, deq_value)() {
      when deq_valid {
        log("deq = {}", deq_value);
      }
    }
  );

  module_builder!(
    // last level
    priority_queue_ll (
      cl_nodes
    ) (
      op: bits<2>,
      position: uint<3>,
      enq_value: uint<32>
    ) #no_arbiter {
      _a = cl_nodes[0];
      when op.eq(1.bits<2>) { // ENQ
        cl_nodes[position] = 1.bits<1>.concat(1.uint<3>).concat(enq_value);
      }

      when op.eq(2.bits<2>) { // DEQ
        cl_nodes[position] = 0.bits<36>;
      }
    }
  );

  module_builder!(
    // middle levels
    priority_queue_ml (
      cl_nodes,
      nl_nodes,
      nl
    ) (
      op: bits<2>,
      position: uint<3>,
      enq_value: uint<32>
    ) #no_arbiter {
      target_node_active = cl_nodes[position].slice(35, 35);
      target_node_occupied = cl_nodes[position].slice(32, 34).bitcast(uint<3>);
      target_node_value = cl_nodes[position].slice(0, 31).bitcast(uint<32>);

      nl_position_left = position.slice(0, 1).concat(0.uint<1>).bitcast(uint<3>);
      nl_position_right = position.slice(0, 1).concat(1.uint<1>).bitcast(uint<3>);

      when op.eq(0.bits<2>) {
        async_call nl { op: 0.bits<2>, position: 0.uint<3>, enq_value: 0.uint<32> };
      }

      when op.eq(1.bits<2>) { // ENQ
        when target_node_active {
          replace = enq_value.igt(target_node_value);
          left_child = nl_nodes[nl_position_left];
          left_child_full = left_child.slice(32, 34).eq(1.uint<3>);
          when replace {
            when left_child_full {
              async_call nl { op: 1.bits<2>, position: nl_position_right, enq_value: target_node_value };
            }
            when left_child_full.flip() {
              async_call nl { op: 1.bits<2>, position: nl_position_left, enq_value: target_node_value };
            }
            cl_nodes[position] = 1.bits<1>.concat(target_node_occupied.add(1.uint<3>)).concat(enq_value);
          }
          when replace.flip() {
            when left_child_full {
              async_call nl { op: 1.bits<2>, position: nl_position_right, enq_value: enq_value };
            }
            when left_child_full.flip() {
              async_call nl { op: 1.bits<2>, position: nl_position_left, enq_value: enq_value };
            }
            cl_nodes[position] = 1.bits<1>.concat(target_node_occupied.add(1.uint<3>)).concat(target_node_value);
          }
        }
        when target_node_active.flip() {
          cl_nodes[position] = 1.bits<1>.concat(target_node_occupied.add(1.uint<3>)).concat(enq_value);
          async_call nl { op: 0.bits<2>, position: 0.uint<3>, enq_value: 0.uint<32> };
        }
      }

      when op.eq(2.bits<2>) { // DEQ
        left_child = nl_nodes[nl_position_left];
        right_child = nl_nodes[nl_position_right];
        left_child_active = left_child.slice(35, 35);
        right_child_active = right_child.slice(35, 35);
        left_child_value = left_child.slice(0, 31);
        right_child_value = right_child.slice(0, 31);
        when left_child_active.flip().bitwise_and(right_child_active.flip()) {
          cl_nodes[position] = 0.bits<1>.concat(target_node_occupied.sub(1.uint<3>)).concat(0.uint<32>);
          async_call nl { op: 0.bits<2>, position: nl_position_left, enq_value: 0.uint<32> };
        }
        when left_child_active.flip().bitwise_and(right_child_active) {
          cl_nodes[position] = 1.bits<1>.concat(target_node_occupied.sub(1.uint<3>)).concat(right_child_value);
          async_call nl { op: 2.bits<2>, position: nl_position_right, enq_value: 0.uint<32> };
        }
        when left_child_active.bitwise_and(right_child_active.flip()) {
          cl_nodes[position] = 1.bits<1>.concat(target_node_occupied.sub(1.uint<3>)).concat(left_child_value);
          async_call nl { op: 2.bits<2>, position: nl_position_left, enq_value: 0.uint<32> };
        }
        when left_child_active.bitwise_and(right_child_active) {
          pop_left = left_child_value.ige(right_child_value);
          when pop_left {
            cl_nodes[position] = 1.bits<1>.concat(target_node_occupied.sub(1.uint<3>)).concat(left_child_value);
            async_call nl { op: 2.bits<2>, position: nl_position_left, enq_value: 0.uint<32> };
          }
          when pop_left.flip() {
            cl_nodes[position] = 1.bits<1>.concat(target_node_occupied.sub(1.uint<3>)).concat(right_child_value);
            async_call nl { op: 2.bits<2>, position: nl_position_right, enq_value: 0.uint<32> };
          }
        }
      }
    }
  );

  module_builder!(
    // first level (level 0)
    priority_queue_fl (
      cl_nodes,
      nl_nodes,
      nl
    ) (
      op: bits<2>,
      enq_value: uint<32>
    ) #no_arbiter {
      target_node_active = cl_nodes[0].slice(35, 35);
      target_node_occupied = cl_nodes[0].slice(32, 34).bitcast(uint<3>);
      target_node_value = cl_nodes[0].slice(0, 31).bitcast(uint<32>);
  
      when op.eq(0.bits<2>) {
        async_call nl { op: 0.bits<2>, position: 0.uint<3>, enq_value: 0.uint<32> };
      }

      when op.eq(1.bits<2>) { // ENQ
        when target_node_active {
          replace = enq_value.igt(target_node_value);
          left_child = nl_nodes[0];
          left_child_full = left_child.slice(32, 34).eq(3.uint<3>);
          when replace {
            when left_child_full {
              async_call nl { op: 1.bits<2>, position: 1.uint<3>, enq_value: target_node_value };
            }
            when left_child_full.flip() {
              async_call nl { op: 1.bits<2>, position: 0.uint<3>, enq_value: target_node_value };
            }
            cl_nodes[0] = 1.bits<1>.concat(target_node_occupied.add(1.uint<3>)).concat(enq_value);
          }
          when replace.flip() {
            when left_child_full {
              async_call nl { op: 1.bits<2>, position: 1.uint<3>, enq_value: enq_value };
            }
            when left_child_full.flip() {
              async_call nl { op: 1.bits<2>, position: 0.uint<3>, enq_value: enq_value };
            }
            cl_nodes[0] = 1.bits<1>.concat(target_node_occupied.add(1.uint<3>)).concat(target_node_value);
          }
        }
        when target_node_active.flip() {
          cl_nodes[0] = 1.bits<1>.concat(target_node_occupied.add(1.uint<3>)).concat(enq_value);
          async_call nl { op: 0.bits<2>, position: 0.uint<3>, enq_value: 0.uint<32> };
        }
      }

      when op.eq(2.bits<2>) { // DEQ
        left_child = nl_nodes[0];
        right_child = nl_nodes[1];
        left_child_active = left_child.slice(35, 35);
        right_child_active = right_child.slice(35, 35);
        left_child_value = left_child.slice(0, 31);
        right_child_value = right_child.slice(0, 31);
        when target_node_active {
          when left_child_active.flip().bitwise_and(right_child_active.flip()) {
            cl_nodes[0] = 0.bits<1>.concat(target_node_occupied.sub(1.uint<3>)).concat(0.uint<32>);
            async_call nl { op: 0.bits<2>, position: 0.uint<3>, enq_value: 0.uint<32> };
          }
          when left_child_active.flip().bitwise_and(right_child_active) {
            cl_nodes[0] = 1.bits<1>.concat(target_node_occupied.sub(1.uint<3>)).concat(right_child_value);
            async_call nl { op: 2.bits<2>, position: 1.uint<3>, enq_value: 0.uint<32> };
          }
          when left_child_active.bitwise_and(right_child_active.flip()) {
            cl_nodes[0] = 1.bits<1>.concat(target_node_occupied.sub(1.uint<3>)).concat(left_child_value);
            async_call nl { op: 2.bits<2>, position: 0.uint<3>, enq_value: 0.uint<32> };
          }
          when left_child_active.bitwise_and(right_child_active) {
            pop_left = left_child_value.ige(right_child_value);
            when pop_left {
              cl_nodes[0] = 1.bits<1>.concat(target_node_occupied.sub(1.uint<3>)).concat(left_child_value);
              async_call nl { op: 2.bits<2>, position: 0.uint<3>, enq_value: 0.uint<32> };
            }
            when pop_left.flip() {
              cl_nodes[0] = 1.bits<1>.concat(target_node_occupied.sub(1.uint<3>)).concat(right_child_value);
              async_call nl { op: 2.bits<2>, position: 1.uint<3>, enq_value: 0.uint<32> };
            }
          }
        }
      }
    }
  );

  module_builder!(
    driver(pq_level_0, pq_level_0_node)() {
      cnt    = array(uint<32>, 1);
      v      = cnt[0];
      new_v  = v.add(1.uint<32>);
      cnt[0] = new_v;
      fire = v.slice(0, 0);
      vd2 = v.slice(1, 31).zext(uint<32>);
      do_push = vd2.ilt(7.uint<32>);
      do_pop = vd2.ige(8.uint<32>);
      when fire {
        when do_push {
          // enq_value = 6.uint<32>.sub(vd2);
          enq_value = vd2;
          log("enq = {}", enq_value);
          async_call pq_level_0 { op: 1.bits<2>, enq_value: enq_value };
        }
        when do_pop {
          async_call pq_level_0 { op: 2.bits<2>, enq_value: 0.uint<32> };
        }
      }
      when fire.flip() {
        async_call pq_level_0 { op: 0.bits<2>, enq_value: 0.uint<32> };
      }

      // deq logic
      root = pq_level_0_node[0];
      inline deq_sink(root.slice(35, 35).bitwise_and(fire).bitwise_and(do_pop), root.slice(0, 31))();
    }
  );

  let mut sys = SysBuilder::new("priority_queue");

  let tree_level_0 = sys.create_array(eir::ir::DataType::Bits(36), "tree_level_0", 1, None, vec![]);
  let tree_level_1 = sys.create_array(eir::ir::DataType::Bits(36), "tree_level_1", 2, None, vec![]);
  let tree_level_2 = sys.create_array(eir::ir::DataType::Bits(36), "tree_level_2", 4, None, vec![]);

  let level_2 = priority_queue_ll_builder(&mut sys, tree_level_2);
  let level_1 = priority_queue_ml_builder(&mut sys, tree_level_1, tree_level_2, level_2);
  let level_0 = priority_queue_fl_builder(&mut sys, tree_level_0, tree_level_1, level_1);
  driver_builder(&mut sys, level_0, tree_level_0);

  eir::builder::verify(&sys);
  let o0 = eir::xform::Config {
    rewrite_wait_until: false,
  };
  eir::xform::basic(&mut sys, &o0);
  eir::builder::verify(&sys);
  println!("{}", sys);

  let mut config = eir::backend::common::Config::default();
  config.sim_threshold = 40;
  config.idle_threshold = 10;

  // eir::backend::verilog::elaborate(&sys, &config).unwrap();

  run_simulator(
    &sys,
    &config,
    None,
  );
}
