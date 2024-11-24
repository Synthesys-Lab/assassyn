use assassyn::builder::SysBuilder;
use assassyn::ir::node::IsElement;
use std::collections::HashMap;
use std::path::PathBuf;
fn main() {
    let mut sys = SysBuilder::new("minor_cpu");
    let mut block_stack: Vec<assassyn::ir::node::BaseNode> = Vec::new();

    // Declare modules
    let _b9a3d = sys.create_module("f", vec![]);
    let _ba599 = sys.create_module(
        "w",
        vec![
            assassyn::builder::PortInfo::new("is_memory_read", assassyn::ir::DataType::bits_ty(1)),
            assassyn::builder::PortInfo::new("result", assassyn::ir::DataType::bits_ty(32)),
            assassyn::builder::PortInfo::new("rd", assassyn::ir::DataType::bits_ty(5)),
            assassyn::builder::PortInfo::new("mdata", assassyn::ir::DataType::bits_ty(32)),
            assassyn::builder::PortInfo::new("is_csr", assassyn::ir::DataType::bits_ty(1)),
            assassyn::builder::PortInfo::new("csr_id", assassyn::ir::DataType::bits_ty(4)),
            assassyn::builder::PortInfo::new("csr_new", assassyn::ir::DataType::bits_ty(32)),
            assassyn::builder::PortInfo::new("mem_ext", assassyn::ir::DataType::bits_ty(2)),
        ],
    );
    _ba599
        .as_mut::<assassyn::ir::Module>(&mut sys)
        .unwrap()
        .add_attr(assassyn::ir::module::Attribute::NoArbiter);
    let _ba5e5 = sys.create_module(
        "m",
        vec![
            assassyn::builder::PortInfo::new("rdata", assassyn::ir::DataType::bits_ty(32)),
            assassyn::builder::PortInfo::new("rd", assassyn::ir::DataType::bits_ty(5)),
        ],
    );
    _ba5e5
        .as_mut::<assassyn::ir::Module>(&mut sys)
        .unwrap()
        .add_attr(assassyn::ir::module::Attribute::Systolic);
    _ba5e5
        .as_mut::<assassyn::ir::Module>(&mut sys)
        .unwrap()
        .add_attr(assassyn::ir::module::Attribute::NoArbiter);
    let _ba7f9 = sys.create_module(
        "e",
        vec![
            assassyn::builder::PortInfo::new("signals", assassyn::ir::DataType::bits_ty(97)),
            assassyn::builder::PortInfo::new("fetch_addr", assassyn::ir::DataType::bits_ty(32)),
        ],
    );
    let _bedf5 = sys.create_module("p", vec![]);
    let _ba7b9 = sys.create_module(
        "d",
        vec![
            assassyn::builder::PortInfo::new("rdata", assassyn::ir::DataType::bits_ty(32)),
            assassyn::builder::PortInfo::new("fetch_addr", assassyn::ir::DataType::bits_ty(32)),
        ],
    );
    _ba7b9
        .as_mut::<assassyn::ir::Module>(&mut sys)
        .unwrap()
        .add_attr(assassyn::ir::module::Attribute::Systolic);
    let _c9a29 = sys.create_module("driver", vec![]);
    // Declare downstream modules
    let _25d8d = sys.create_downstream("F1");
    let _c0ba1 = sys.create_downstream("dcache");
    let _d16e5 = sys.create_downstream("icache");
    // declare arrays
    // array array_b9a51[b32; 1] = None
    let array_b9a51 = sys.create_array(
        assassyn::ir::DataType::bits_ty(32),
        "array_b9a51",
        1,
        None,
        vec![],
    );
    // array array_b9a49[b32; 32] = None
    let array_b9a49 = sys.create_array(
        assassyn::ir::DataType::bits_ty(32),
        "array_b9a49",
        32,
        None,
        vec![],
    );
    let init_0 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_2 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_3 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_4 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_6 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_7 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_8 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_10 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_11 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_12 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_13 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_14 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_15 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_16 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_17 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_18 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_19 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_20 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_21 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_22 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_23 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_24 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_25 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_26 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_27 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_28 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_29 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_30 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init_31 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64);
    let init = vec![
        init_0, init_1, init_2, init_3, init_4, init_5, init_6, init_7, init_8, init_9, init_10,
        init_11, init_12, init_13, init_14, init_15, init_16, init_17, init_18, init_19, init_20,
        init_21, init_22, init_23, init_24, init_25, init_26, init_27, init_28, init_29, init_30,
        init_31,
    ];
    // array array_ba589[b5; 32] = [16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16]
    let array_ba589 = sys.create_array(
        assassyn::ir::DataType::bits_ty(5),
        "array_ba589",
        32,
        Some(init),
        vec![],
    );
    // array array_ba53d[record { fetch_addr: b32, signals: record { mem_ext: b2, link_pc: b1, is_offset_br: b1, is_branch: b1, flip: b1, cond: b16, alu: b16, memory: b2, imm_valid: b1, imm: b32, is_pc_calc: b1, is_mepc: b1, is_zimm: b1, csr_calculate: b1, csr_write: b1, csr_read: b1, rd_valid: b1, rd: b5, rs2_valid: b1, rs2: b5, rs1_valid: b1, rs1: b5 }, status: b2, rs2_dep: b5, rs1_dep: b5, rs2_value: b32, rs1_value: b32, rs2_ready: b1, rs1_ready: b1, rs2: b5, rs1: b5, rd: b5, valid: b1 }; 16] = None
    let array_ba53d = sys.create_array(
        assassyn::ir::DataType::bits_ty(223),
        "array_ba53d",
        16,
        None,
        vec![assassyn::ir::array::ArrayAttr::FullyPartitioned],
    );
    let init_0 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64);
    let init_1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64);
    let init_2 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64);
    let init_3 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64);
    let init_4 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64);
    let init_5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64);
    let init_6 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64);
    let init_7 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64);
    let init_8 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64);
    let init_9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64);
    let init_10 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64);
    let init_11 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64);
    let init_12 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64);
    let init_13 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64);
    let init_14 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64);
    let init_15 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64);
    let init = vec![
        init_0, init_1, init_2, init_3, init_4, init_5, init_6, init_7, init_8, init_9, init_10,
        init_11, init_12, init_13, init_14, init_15,
    ];
    // array array_ba595[b32; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    let array_ba595 = sys.create_array(
        assassyn::ir::DataType::bits_ty(32),
        "array_ba595",
        16,
        Some(init),
        vec![],
    );
    // array array_c24b1[b1; 1] = None
    let array_c24b1 = sys.create_array(
        assassyn::ir::DataType::bits_ty(1),
        "array_c24b1",
        1,
        None,
        vec![],
    );
    // array array_c2731[b32; 65536] = None
    let array_c2731 = sys.create_array(
        assassyn::ir::DataType::bits_ty(32),
        "array_c2731",
        65536,
        None,
        vec![],
    );
    let init_0 = sys.get_const_int(assassyn::ir::DataType::int_ty(8), 0x0 as u64);
    let init = vec![init_0];
    // array array_d0e35[i8; 1] = [0]
    let array_d0e35 = sys.create_array(
        assassyn::ir::DataType::int_ty(8),
        "array_d0e35",
        1,
        Some(init),
        vec![],
    );
    // array array_d17c9[b32; 65536] = None
    let array_d17c9 = sys.create_array(
        assassyn::ir::DataType::bits_ty(32),
        "array_d17c9",
        65536,
        None,
        vec![],
    );
    // Gathered binds
    // Set the current module redundantly to emit related binds
    sys.set_current_module(_b9a3d);
    // Set the current module redundantly to emit related binds
    sys.set_current_module(_ba599);
    // Set the current module redundantly to emit related binds
    sys.set_current_module(_ba5e5);
    // Set the current module redundantly to emit related binds
    sys.set_current_module(_ba7f9);
    let _9c0d5 = sys.get_init_bind(_ba599);
    // Set the current module redundantly to emit related binds
    sys.set_current_module(_bedf5);
    let _c68a1 = sys.get_init_bind(_ba7f9);
    let _c6a1d = sys.get_init_bind(_ba7f9);
    let _c6b85 = sys.get_init_bind(_ba7f9);
    let _c6cf1 = sys.get_init_bind(_ba7f9);
    let _c6e59 = sys.get_init_bind(_ba7f9);
    let _c6fc1 = sys.get_init_bind(_ba7f9);
    let _c792d = sys.get_init_bind(_ba7f9);
    let _c7a9d = sys.get_init_bind(_ba7f9);
    let _c8011 = sys.get_init_bind(_ba7f9);
    let _c8185 = sys.get_init_bind(_ba7f9);
    let _c82f9 = sys.get_init_bind(_ba7f9);
    let _c8c71 = sys.get_init_bind(_ba7f9);
    let _c8de5 = sys.get_init_bind(_ba7f9);
    let _c8f59 = sys.get_init_bind(_ba7f9);
    let _c98d1 = sys.get_init_bind(_ba7f9);
    let _c9a45 = sys.get_init_bind(_ba7f9);
    // Set the current module redundantly to emit related binds
    sys.set_current_module(_ba7b9);
    let _d1749 = sys.get_init_bind(_bedf5);
    // Set the current module redundantly to emit related binds
    sys.set_current_module(_c9a29);
    let _d24d1 = sys.get_init_bind(_b9a3d);
    // Set the current module redundantly to emit related binds
    sys.set_current_module(_25d8d);
    // Set the current module redundantly to emit related binds
    sys.set_current_module(_c0ba1);
    let _c27a5 = sys.get_init_bind(_ba5e5);
    // Set the current module redundantly to emit related binds
    sys.set_current_module(_d16e5);
    let _d2495 = sys.get_init_bind(_ba7b9);
    // Fill in the body of _b9a3d
    sys.set_current_module(_b9a3d);
    // module root block
    // _b9a61 = array_b9a51[(0:u1)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:289
    let imm_b9a65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let _b9a61 = sys.create_array_read(array_b9a51, imm_b9a65);
    // Fill in the body of _ba599
    sys.set_current_module(_ba599);
    // module root block
    // _ba5e9 = _ba599.is_memory_read.valid(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    // Get port is_memory_read
    let _ba599_is_memory_read = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("is_memory_read").unwrap().upcast()
    };
    // Get port is_memory_read
    let _ba599_is_memory_read = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("is_memory_read").unwrap().upcast()
    };
    let _ba5e9 = sys.create_fifo_valid(_ba599_is_memory_read);
    // _ba5fd = _ba599.result.valid(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    // Get port result
    let _ba599_result = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("result").unwrap().upcast()
    };
    // Get port result
    let _ba599_result = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("result").unwrap().upcast()
    };
    let _ba5fd = sys.create_fifo_valid(_ba599_result);
    // _ba609 = _ba5e9 & _ba5fd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    let _ba609 = sys.create_bitwise_and(_ba5e9, _ba5fd);
    // _ba60d = _ba599.rd.valid(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    // Get port rd
    let _ba599_rd = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("rd").unwrap().upcast()
    };
    // Get port rd
    let _ba599_rd = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("rd").unwrap().upcast()
    };
    let _ba60d = sys.create_fifo_valid(_ba599_rd);
    // _ba61d = _ba609 & _ba60d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    let _ba61d = sys.create_bitwise_and(_ba609, _ba60d);
    // _ba629 = _ba599.mdata.valid(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    // Get port mdata
    let _ba599_mdata = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("mdata").unwrap().upcast()
    };
    // Get port mdata
    let _ba599_mdata = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("mdata").unwrap().upcast()
    };
    let _ba629 = sys.create_fifo_valid(_ba599_mdata);
    // _ba635 = _ba61d & _ba629, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    let _ba635 = sys.create_bitwise_and(_ba61d, _ba629);
    // _ba639 = _ba599.is_csr.valid(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    // Get port is_csr
    let _ba599_is_csr = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("is_csr").unwrap().upcast()
    };
    // Get port is_csr
    let _ba599_is_csr = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("is_csr").unwrap().upcast()
    };
    let _ba639 = sys.create_fifo_valid(_ba599_is_csr);
    // _ba649 = _ba635 & _ba639, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    let _ba649 = sys.create_bitwise_and(_ba635, _ba639);
    // _ba651 = _ba599.csr_id.valid(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    // Get port csr_id
    let _ba599_csr_id = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("csr_id").unwrap().upcast()
    };
    // Get port csr_id
    let _ba599_csr_id = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("csr_id").unwrap().upcast()
    };
    let _ba651 = sys.create_fifo_valid(_ba599_csr_id);
    // _ba659 = _ba649 & _ba651, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    let _ba659 = sys.create_bitwise_and(_ba649, _ba651);
    // _ba65d = _ba599.csr_new.valid(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    // Get port csr_new
    let _ba599_csr_new = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("csr_new").unwrap().upcast()
    };
    // Get port csr_new
    let _ba599_csr_new = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("csr_new").unwrap().upcast()
    };
    let _ba65d = sys.create_fifo_valid(_ba599_csr_new);
    // _ba66d = _ba659 & _ba65d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    let _ba66d = sys.create_bitwise_and(_ba659, _ba65d);
    // _ba671 = _ba599.mem_ext.valid(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    // Get port mem_ext
    let _ba599_mem_ext = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("mem_ext").unwrap().upcast()
    };
    // Get port mem_ext
    let _ba599_mem_ext = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("mem_ext").unwrap().upcast()
    };
    let _ba671 = sys.create_fifo_valid(_ba599_mem_ext);
    // _ba681 = _ba66d & _ba671, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    let _ba681 = sys.create_bitwise_and(_ba66d, _ba671);
    // side effect intrinsic.wait_until({'_ba681'}), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    sys.create_wait_until(_ba681);
    // _ba685 = _ba599.is_memory_read.pop(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    // Get port is_memory_read
    let _ba599_is_memory_read = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("is_memory_read").unwrap().upcast()
    };
    let _ba685 = sys.create_fifo_pop(_ba599_is_memory_read);
    // _ba695 = _ba599.result.pop(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    // Get port result
    let _ba599_result = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("result").unwrap().upcast()
    };
    let _ba695 = sys.create_fifo_pop(_ba599_result);
    // _ba691 = _ba599.rd.pop(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    // Get port rd
    let _ba599_rd = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("rd").unwrap().upcast()
    };
    let _ba691 = sys.create_fifo_pop(_ba599_rd);
    // _ba69d = _ba599.mdata.pop(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    // Get port mdata
    let _ba599_mdata = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("mdata").unwrap().upcast()
    };
    let _ba69d = sys.create_fifo_pop(_ba599_mdata);
    // _ba6a5 = _ba599.is_csr.pop(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    // Get port is_csr
    let _ba599_is_csr = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("is_csr").unwrap().upcast()
    };
    let _ba6a5 = sys.create_fifo_pop(_ba599_is_csr);
    // _ba6b5 = _ba599.csr_id.pop(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    // Get port csr_id
    let _ba599_csr_id = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("csr_id").unwrap().upcast()
    };
    let _ba6b5 = sys.create_fifo_pop(_ba599_csr_id);
    // _ba6b9 = _ba599.csr_new.pop(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    // Get port csr_new
    let _ba599_csr_new = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("csr_new").unwrap().upcast()
    };
    let _ba6b9 = sys.create_fifo_pop(_ba599_csr_new);
    // _ba6c1 = _ba599.mem_ext.pop(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:24
    // Get port mem_ext
    let _ba599_mem_ext = {
        let module = _ba599.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("mem_ext").unwrap().upcast()
    };
    let _ba6c1 = sys.create_fifo_pop(_ba599_mem_ext);
    // _ba6d1 = _ba69d[(7:u3):(7:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:26
    let imm_ba6dd = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_ba6e5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let _ba6d1 = sys.create_slice(_ba69d, imm_ba6dd, imm_ba6e5);
    // _ba6fd = _ba6d1 ? (16777215:b24) : (0:b24), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:27
    let imm_ba6f1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(24), 0xffffff as u64); // (16777215:b24)
    let imm_ba6f9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(24), 0x0 as u64); // (0:b24)
    let _ba6fd = sys.create_select(_ba6d1, imm_ba6f1, imm_ba6f9);
    // _ba701 = _ba6c1[(1:u1):(1:u1)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:28
    let imm_ba711 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x1 as u64); // (1:u1)
    let imm_ba719 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x1 as u64); // (1:u1)
    let _ba701 = sys.create_slice(_ba6c1, imm_ba711, imm_ba719);
    // _ba729 = _ba69d[(0:u1):(7:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:28
    let imm_ba735 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_ba73d = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let _ba729 = sys.create_slice(_ba69d, imm_ba735, imm_ba73d);
    // _ba721 = { (0:b24) _ba729 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:28
    let imm_ba725 = sys.get_const_int(assassyn::ir::DataType::bits_ty(24), 0x0 as u64); // (0:b24)
    let _ba721 = sys.create_concat(imm_ba725, _ba729);
    // _ba755 = _ba69d[(0:u1):(7:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:28
    let imm_ba759 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_ba761 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let _ba755 = sys.create_slice(_ba69d, imm_ba759, imm_ba761);
    // _ba75d = { _ba6fd _ba755 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:28
    let _ba75d = sys.create_concat(_ba6fd, _ba755);
    // _ba76d = _ba701 ? _ba721 : _ba75d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:28
    let _ba76d = sys.create_select(_ba701, _ba721, _ba75d);
    // _ba6b1 = _ba685 ? _ba69d : _ba695, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:30
    let _ba6b1 = sys.create_select(_ba685, _ba69d, _ba695);
    // _ba779 = _ba6c1[(0:u1):(0:u1)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:31
    let imm_ba785 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_ba78d = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let _ba779 = sys.create_slice(_ba6c1, imm_ba785, imm_ba78d);
    // _ba791 = _ba779 ? _ba76d : _ba6b1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:31
    let _ba791 = sys.create_select(_ba779, _ba76d, _ba6b1);
    // _ba7a9 = _ba691 != (0:b5), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:33
    let imm_ba7a5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x0 as u64); // (0:b5)
    let _ba7a9 = sys.create_neq(_ba691, imm_ba7a5);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _ba7ad = sys.create_conditional_block(_ba7a9);
    sys.set_current_block(_ba7ad);
    // log('writeback        | x{:02}          | 0x{:08x}', _ba691, _ba791), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:34
    let fmt = sys.get_str_literal("writeback        | x{:02}          | 0x{:08x}".into());
    sys.create_log(fmt, vec![_ba691, _ba791]);
    // array_b9a49[_ba691] = _ba791, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:35
    sys.create_array_write(array_b9a49, _ba691, _ba791);
    // _ba7d1 = array_ba589[_ba691], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:37
    let _ba7d1 = sys.create_array_read(array_ba589, _ba691);
    // _ba7d5 = array_ba53d[_ba7d1], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:37
    let _ba7d5 = sys.create_array_read(array_ba53d, _ba7d1);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _ba7e1 = sys.create_conditional_block(_ba6a5);
    sys.set_current_block(_ba7e1);
    // log('writeback        | csr[{:02}]       | 0x{:08x}', _ba6b5, _ba6b9), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:41
    let fmt = sys.get_str_literal("writeback        | csr[{:02}]       | 0x{:08x}".into());
    sys.create_log(fmt, vec![_ba6b5, _ba6b9]);
    // array_ba595[_ba6b5] = _ba6b9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/writeback.py:42
    sys.create_array_write(array_ba595, _ba6b5, _ba6b9);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // Fill in the body of _ba5e5
    sys.set_current_module(_ba5e5);
    // module root block
    // _b9a41 = _ba5e5.rdata.valid(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:23
    // Get port rdata
    let _ba5e5_rdata = {
        let module = _ba5e5.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("rdata").unwrap().upcast()
    };
    // Get port rdata
    let _ba5e5_rdata = {
        let module = _ba5e5.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("rdata").unwrap().upcast()
    };
    let _b9a41 = sys.create_fifo_valid(_ba5e5_rdata);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c2851 = sys.create_conditional_block(_b9a41);
    sys.set_current_block(_c2851);
    // _c280d = _ba5e5.rdata.pop(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:24
    // Get port rdata
    let _ba5e5_rdata = {
        let module = _ba5e5.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("rdata").unwrap().upcast()
    };
    let _c280d = sys.create_fifo_pop(_ba5e5_rdata);
    // _c2869 = _ba5e5.rd.pop(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:25
    // Get port rd
    let _ba5e5_rd = {
        let module = _ba5e5.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("rd").unwrap().upcast()
    };
    let _c2869 = sys.create_fifo_pop(_ba5e5_rd);
    // log('mem.rdata        | 0x{:x}', _c280d), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:26
    let fmt = sys.get_str_literal("mem.rdata        | 0x{:x}".into());
    sys.create_log(fmt, vec![_c280d]);
    // _c287d = _c2869 != (0:b5), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:27
    let imm_c2875 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x0 as u64); // (0:b5)
    let _c287d = sys.create_neq(_c2869, imm_c2875);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c2885 = sys.create_conditional_block(_c287d);
    sys.set_current_block(_c2885);
    // log('mem.bypass       | x{:02} = 0x{:x}', _c2869, _c280d), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:28
    let fmt = sys.get_str_literal("mem.bypass       | x{:02} = 0x{:x}".into());
    sys.create_log(fmt, vec![_c2869, _c280d]);
    // _c289d = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:30
    let _c289d = sys.create_array_read(array_ba589, _c2869);
    // _c28a5 = array_ba53d[_c289d], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:30
    let _c28a5 = sys.create_array_read(array_ba53d, _c289d);
    // _c28a1 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c28ad = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c28a1 = sys.create_array_read(array_ba53d, imm_c28ad);
    // _c28b1 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c28b9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c28b1 = sys.create_array_read(array_ba53d, imm_c28b9);
    // _c28c5 = _c28b1[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c28d1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c28d9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c28c5 = sys.create_slice(_c28b1, imm_c28d1, imm_c28d9);
    // _c28d5 = _c28c5 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c28e1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c28d5 = sys.create_eq(_c28c5, imm_c28e1);
    // _c28e5 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c28ed = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c28e5 = sys.create_array_read(array_ba53d, imm_c28ed);
    // _c28f1 = _c28e5[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c2901 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c2909 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c28f1 = sys.create_slice(_c28e5, imm_c2901, imm_c2909);
    // _b9aa1 = ~_c28f1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _b9aa1 = sys.create_flip(_c28f1);
    // _c28c1 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c2905 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c28c1 = sys.create_array_read(array_ba53d, imm_c2905);
    // _c2915 = _c28c1[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c2925 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c292d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c2915 = sys.create_slice(_c28c1, imm_c2925, imm_c292d);
    // _bcdb9 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _bcdb9 = sys.create_array_read(array_ba589, _c2869);
    // _c2929 = _c2915 == _bcdb9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c2929 = sys.create_eq(_c2915, _bcdb9);
    // _c2941 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:35
    let imm_c2945 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c2941 = sys.create_array_read(array_ba53d, imm_c2945);
    // _c2949 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:36
    let imm_c2951 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c2949 = sys.create_array_read(array_ba53d, imm_c2951);
    // _c2955 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c295d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c2955 = sys.create_array_read(array_ba53d, imm_c295d);
    // _c2961 = _c2955[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c2971 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c2979 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c2961 = sys.create_slice(_c2955, imm_c2971, imm_c2979);
    // _c2839 = ~_c2961, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c2839 = sys.create_flip(_c2961);
    // _c2981 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c2985 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c2981 = sys.create_array_read(array_ba53d, imm_c2985);
    // _c2989 = _c2981[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c2999 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c29a1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c2989 = sys.create_slice(_c2981, imm_c2999, imm_c29a1);
    // _c2919 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c2919 = sys.create_array_read(array_ba589, _c2869);
    // _c299d = _c2989 == _c2919, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c299d = sys.create_eq(_c2989, _c2919);
    // _c29b5 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:38
    let imm_c29b9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c29b5 = sys.create_array_read(array_ba53d, imm_c29b9);
    // _c29bd = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:39
    let imm_c29c5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c29bd = sys.create_array_read(array_ba53d, imm_c29c5);
    // _c29c9 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c29d1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c29c9 = sys.create_array_read(array_ba53d, imm_c29d1);
    // _c29d5 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c29dd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c29d5 = sys.create_array_read(array_ba53d, imm_c29dd);
    // _c29e1 = _c29d5[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c29f1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c29f9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c29e1 = sys.create_slice(_c29d5, imm_c29f1, imm_c29f9);
    // _c29f5 = _c29e1 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c2a01 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c29f5 = sys.create_eq(_c29e1, imm_c2a01);
    // _c2a05 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c2a0d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c2a05 = sys.create_array_read(array_ba53d, imm_c2a0d);
    // _c2a11 = _c2a05[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c2a21 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c2a29 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c2a11 = sys.create_slice(_c2a05, imm_c2a21, imm_c2a29);
    // _bcc25 = ~_c2a11, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _bcc25 = sys.create_flip(_c2a11);
    // _c2a31 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c2a35 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c2a31 = sys.create_array_read(array_ba53d, imm_c2a35);
    // _c2a39 = _c2a31[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c2a49 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c2a51 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c2a39 = sys.create_slice(_c2a31, imm_c2a49, imm_c2a51);
    // _c2879 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c2879 = sys.create_array_read(array_ba589, _c2869);
    // _c2a4d = _c2a39 == _c2879, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c2a4d = sys.create_eq(_c2a39, _c2879);
    // _c2a65 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:35
    let imm_c2a69 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c2a65 = sys.create_array_read(array_ba53d, imm_c2a69);
    // _c2a6d = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:36
    let imm_c2a75 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c2a6d = sys.create_array_read(array_ba53d, imm_c2a75);
    // _c2a79 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c2a81 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c2a79 = sys.create_array_read(array_ba53d, imm_c2a81);
    // _c2a85 = _c2a79[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c2a95 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c2a9d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c2a85 = sys.create_slice(_c2a79, imm_c2a95, imm_c2a9d);
    // _c2849 = ~_c2a85, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c2849 = sys.create_flip(_c2a85);
    // _c2aa5 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c2aa9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c2aa5 = sys.create_array_read(array_ba53d, imm_c2aa9);
    // _c2aad = _c2aa5[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c2abd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c2ac5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c2aad = sys.create_slice(_c2aa5, imm_c2abd, imm_c2ac5);
    // _c2a3d = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c2a3d = sys.create_array_read(array_ba589, _c2869);
    // _c2ac1 = _c2aad == _c2a3d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c2ac1 = sys.create_eq(_c2aad, _c2a3d);
    // _c2ad9 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:38
    let imm_c2add = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c2ad9 = sys.create_array_read(array_ba53d, imm_c2add);
    // _c2ae1 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:39
    let imm_c2ae9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c2ae1 = sys.create_array_read(array_ba53d, imm_c2ae9);
    // _c2aed = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c2af5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c2aed = sys.create_array_read(array_ba53d, imm_c2af5);
    // _c2af9 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c2b01 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c2af9 = sys.create_array_read(array_ba53d, imm_c2b01);
    // _c2b05 = _c2af9[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c2b15 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c2b1d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c2b05 = sys.create_slice(_c2af9, imm_c2b15, imm_c2b1d);
    // _c2b19 = _c2b05 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c2b25 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c2b19 = sys.create_eq(_c2b05, imm_c2b25);
    // _c2b29 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c2b31 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c2b29 = sys.create_array_read(array_ba53d, imm_c2b31);
    // _c2b35 = _c2b29[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c2b45 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c2b4d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c2b35 = sys.create_slice(_c2b29, imm_c2b45, imm_c2b4d);
    // _c2899 = ~_c2b35, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c2899 = sys.create_flip(_c2b35);
    // _c2b55 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c2b59 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c2b55 = sys.create_array_read(array_ba53d, imm_c2b59);
    // _c2b5d = _c2b55[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c2b6d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c2b75 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c2b5d = sys.create_slice(_c2b55, imm_c2b6d, imm_c2b75);
    // _c298d = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c298d = sys.create_array_read(array_ba589, _c2869);
    // _c2b71 = _c2b5d == _c298d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c2b71 = sys.create_eq(_c2b5d, _c298d);
    // _c2b89 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:35
    let imm_c2b8d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c2b89 = sys.create_array_read(array_ba53d, imm_c2b8d);
    // _c2b91 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:36
    let imm_c2b99 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c2b91 = sys.create_array_read(array_ba53d, imm_c2b99);
    // _c2b9d = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c2ba5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c2b9d = sys.create_array_read(array_ba53d, imm_c2ba5);
    // _c2ba9 = _c2b9d[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c2bb9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c2bc1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c2ba9 = sys.create_slice(_c2b9d, imm_c2bb9, imm_c2bc1);
    // _c2965 = ~_c2ba9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c2965 = sys.create_flip(_c2ba9);
    // _c2bc9 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c2bcd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c2bc9 = sys.create_array_read(array_ba53d, imm_c2bcd);
    // _c2bd1 = _c2bc9[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c2be1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c2be9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c2bd1 = sys.create_slice(_c2bc9, imm_c2be1, imm_c2be9);
    // _c2b61 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c2b61 = sys.create_array_read(array_ba589, _c2869);
    // _c2be5 = _c2bd1 == _c2b61, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c2be5 = sys.create_eq(_c2bd1, _c2b61);
    // _c4005 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:38
    let imm_c2bfd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c4005 = sys.create_array_read(array_ba53d, imm_c2bfd);
    // _c4009 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:39
    let imm_c4011 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c4009 = sys.create_array_read(array_ba53d, imm_c4011);
    // _c4015 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c401d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c4015 = sys.create_array_read(array_ba53d, imm_c401d);
    // _c4021 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4029 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c4021 = sys.create_array_read(array_ba53d, imm_c4029);
    // _c402d = _c4021[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c403d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c4045 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c402d = sys.create_slice(_c4021, imm_c403d, imm_c4045);
    // _c4041 = _c402d == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c2a89 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c4041 = sys.create_eq(_c402d, imm_c2a89);
    // _c4051 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c4059 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c4051 = sys.create_array_read(array_ba53d, imm_c4059);
    // _c405d = _c4051[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c406d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c4075 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c405d = sys.create_slice(_c4051, imm_c406d, imm_c4075);
    // _c29ad = ~_c405d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c29ad = sys.create_flip(_c405d);
    // _c407d = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c4081 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c407d = sys.create_array_read(array_ba53d, imm_c4081);
    // _c4085 = _c407d[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c4095 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c409d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c4085 = sys.create_slice(_c407d, imm_c4095, imm_c409d);
    // _c2ab1 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c2ab1 = sys.create_array_read(array_ba589, _c2869);
    // _c4099 = _c4085 == _c2ab1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c4099 = sys.create_eq(_c4085, _c2ab1);
    // _c40b1 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:35
    let imm_c40b5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c40b1 = sys.create_array_read(array_ba53d, imm_c40b5);
    // _c40b9 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:36
    let imm_c40c1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c40b9 = sys.create_array_read(array_ba53d, imm_c40c1);
    // _c40c5 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c40cd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c40c5 = sys.create_array_read(array_ba53d, imm_c40cd);
    // _c40d1 = _c40c5[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c40e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c40e9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c40d1 = sys.create_slice(_c40c5, imm_c40e1, imm_c40e9);
    // _c404d = ~_c40d1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c404d = sys.create_flip(_c40d1);
    // _c40f1 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c40f5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c40f1 = sys.create_array_read(array_ba53d, imm_c40f5);
    // _c40f9 = _c40f1[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4109 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c4111 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c40f9 = sys.create_slice(_c40f1, imm_c4109, imm_c4111);
    // _c4089 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c4089 = sys.create_array_read(array_ba589, _c2869);
    // _c410d = _c40f9 == _c4089, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c410d = sys.create_eq(_c40f9, _c4089);
    // _c4125 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:38
    let imm_c4129 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c4125 = sys.create_array_read(array_ba53d, imm_c4129);
    // _c412d = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:39
    let imm_c4135 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c412d = sys.create_array_read(array_ba53d, imm_c4135);
    // _c4139 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4141 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c4139 = sys.create_array_read(array_ba53d, imm_c4141);
    // _c4145 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c414d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c4145 = sys.create_array_read(array_ba53d, imm_c414d);
    // _c4151 = _c4145[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4161 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c4169 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c4151 = sys.create_slice(_c4145, imm_c4161, imm_c4169);
    // _c4165 = _c4151 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c2bad = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c4165 = sys.create_eq(_c4151, imm_c2bad);
    // _c4175 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c417d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c4175 = sys.create_array_read(array_ba53d, imm_c417d);
    // _c4181 = _c4175[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c4191 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c4199 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c4181 = sys.create_slice(_c4175, imm_c4191, imm_c4199);
    // _c2ad1 = ~_c4181, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c2ad1 = sys.create_flip(_c4181);
    // _c41a1 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c41a5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c41a1 = sys.create_array_read(array_ba53d, imm_c41a5);
    // _c41a9 = _c41a1[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c41b9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c41c1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c41a9 = sys.create_slice(_c41a1, imm_c41b9, imm_c41c1);
    // _c2bd5 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c2bd5 = sys.create_array_read(array_ba589, _c2869);
    // _c41bd = _c41a9 == _c2bd5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c41bd = sys.create_eq(_c41a9, _c2bd5);
    // _c41d5 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:35
    let imm_c41d9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c41d5 = sys.create_array_read(array_ba53d, imm_c41d9);
    // _c41dd = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:36
    let imm_c41e5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c41dd = sys.create_array_read(array_ba53d, imm_c41e5);
    // _c41e9 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c41f1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c41e9 = sys.create_array_read(array_ba53d, imm_c41f1);
    // _c41f5 = _c41e9[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4205 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c420d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c41f5 = sys.create_slice(_c41e9, imm_c4205, imm_c420d);
    // _c4171 = ~_c41f5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c4171 = sys.create_flip(_c41f5);
    // _c4215 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4219 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c4215 = sys.create_array_read(array_ba53d, imm_c4219);
    // _c421d = _c4215[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c422d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c4235 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c421d = sys.create_slice(_c4215, imm_c422d, imm_c4235);
    // _c41ad = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c41ad = sys.create_array_read(array_ba589, _c2869);
    // _c4231 = _c421d == _c41ad, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c4231 = sys.create_eq(_c421d, _c41ad);
    // _c4249 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:38
    let imm_c424d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c4249 = sys.create_array_read(array_ba53d, imm_c424d);
    // _c4251 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:39
    let imm_c4259 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c4251 = sys.create_array_read(array_ba53d, imm_c4259);
    // _c425d = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4265 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c425d = sys.create_array_read(array_ba53d, imm_c4265);
    // _c4269 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4271 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c4269 = sys.create_array_read(array_ba53d, imm_c4271);
    // _c4275 = _c4269[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4285 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c428d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c4275 = sys.create_slice(_c4269, imm_c4285, imm_c428d);
    // _c4289 = _c4275 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4295 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c4289 = sys.create_eq(_c4275, imm_c4295);
    // _c4299 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c42a1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c4299 = sys.create_array_read(array_ba53d, imm_c42a1);
    // _c42a5 = _c4299[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c42b5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c42bd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c42a5 = sys.create_slice(_c4299, imm_c42b5, imm_c42bd);
    // _c2bf5 = ~_c42a5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c2bf5 = sys.create_flip(_c42a5);
    // _c42c5 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c42c9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c42c5 = sys.create_array_read(array_ba53d, imm_c42c9);
    // _c42cd = _c42c5[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c42dd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c42e5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c42cd = sys.create_slice(_c42c5, imm_c42dd, imm_c42e5);
    // _c40fd = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c40fd = sys.create_array_read(array_ba589, _c2869);
    // _c42e1 = _c42cd == _c40fd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c42e1 = sys.create_eq(_c42cd, _c40fd);
    // _c42f9 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:35
    let imm_c42fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c42f9 = sys.create_array_read(array_ba53d, imm_c42fd);
    // _c4301 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:36
    let imm_c4309 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c4301 = sys.create_array_read(array_ba53d, imm_c4309);
    // _c430d = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4315 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c430d = sys.create_array_read(array_ba53d, imm_c4315);
    // _c4319 = _c430d[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4329 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c4331 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c4319 = sys.create_slice(_c430d, imm_c4329, imm_c4331);
    // _c40d5 = ~_c4319, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c40d5 = sys.create_flip(_c4319);
    // _c4339 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c433d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c4339 = sys.create_array_read(array_ba53d, imm_c433d);
    // _c4341 = _c4339[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4351 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c4359 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c4341 = sys.create_slice(_c4339, imm_c4351, imm_c4359);
    // _c42d1 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c42d1 = sys.create_array_read(array_ba589, _c2869);
    // _c4355 = _c4341 == _c42d1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c4355 = sys.create_eq(_c4341, _c42d1);
    // _c436d = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:38
    let imm_c4371 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c436d = sys.create_array_read(array_ba53d, imm_c4371);
    // _c4375 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:39
    let imm_c437d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c4375 = sys.create_array_read(array_ba53d, imm_c437d);
    // _c4381 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4389 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c4381 = sys.create_array_read(array_ba53d, imm_c4389);
    // _c438d = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4395 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c438d = sys.create_array_read(array_ba53d, imm_c4395);
    // _c4399 = _c438d[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c43a9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c43b1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c4399 = sys.create_slice(_c438d, imm_c43a9, imm_c43b1);
    // _c43ad = _c4399 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c43b9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c43ad = sys.create_eq(_c4399, imm_c43b9);
    // _c43bd = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c43c5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c43bd = sys.create_array_read(array_ba53d, imm_c43c5);
    // _c43c9 = _c43bd[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c43d9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c43e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c43c9 = sys.create_slice(_c43bd, imm_c43d9, imm_c43e1);
    // _c411d = ~_c43c9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c411d = sys.create_flip(_c43c9);
    // _c43e9 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c43ed = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c43e9 = sys.create_array_read(array_ba53d, imm_c43ed);
    // _c43f1 = _c43e9[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c43fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c4c0d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c43f1 = sys.create_slice(_c43e9, imm_c43fd, imm_c4c0d);
    // _c4221 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c4221 = sys.create_array_read(array_ba589, _c2869);
    // _c4c09 = _c43f1 == _c4221, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c4c09 = sys.create_eq(_c43f1, _c4221);
    // _c4c21 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:35
    let imm_c4c25 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c4c21 = sys.create_array_read(array_ba53d, imm_c4c25);
    // _c4c29 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:36
    let imm_c4c31 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c4c29 = sys.create_array_read(array_ba53d, imm_c4c31);
    // _c4c35 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4c3d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c4c35 = sys.create_array_read(array_ba53d, imm_c4c3d);
    // _c4c41 = _c4c35[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4c51 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c4c59 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c4c41 = sys.create_slice(_c4c35, imm_c4c51, imm_c4c59);
    // _c41f9 = ~_c4c41, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c41f9 = sys.create_flip(_c4c41);
    // _c4c61 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4c65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c4c61 = sys.create_array_read(array_ba53d, imm_c4c65);
    // _c4c69 = _c4c61[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4c79 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c4c81 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c4c69 = sys.create_slice(_c4c61, imm_c4c79, imm_c4c81);
    // _c27d5 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c27d5 = sys.create_array_read(array_ba589, _c2869);
    // _c4c7d = _c4c69 == _c27d5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c4c7d = sys.create_eq(_c4c69, _c27d5);
    // _c4c95 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:38
    let imm_c4c99 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c4c95 = sys.create_array_read(array_ba53d, imm_c4c99);
    // _c4c9d = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:39
    let imm_c4ca5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c4c9d = sys.create_array_read(array_ba53d, imm_c4ca5);
    // _c4ca9 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4cb1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c4ca9 = sys.create_array_read(array_ba53d, imm_c4cb1);
    // _c4cb5 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4cbd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c4cb5 = sys.create_array_read(array_ba53d, imm_c4cbd);
    // _c4cc1 = _c4cb5[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4cd1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c4cd9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c4cc1 = sys.create_slice(_c4cb5, imm_c4cd1, imm_c4cd9);
    // _c4cd5 = _c4cc1 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c431d = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c4cd5 = sys.create_eq(_c4cc1, imm_c431d);
    // _c4ce5 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c4ced = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c4ce5 = sys.create_array_read(array_ba53d, imm_c4ced);
    // _c4cf1 = _c4ce5[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c4d01 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c4d09 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c4cf1 = sys.create_slice(_c4ce5, imm_c4d01, imm_c4d09);
    // _c4241 = ~_c4cf1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c4241 = sys.create_flip(_c4cf1);
    // _c4d11 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c4d15 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c4d11 = sys.create_array_read(array_ba53d, imm_c4d15);
    // _c4d19 = _c4d11[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c4d29 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c4d31 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c4d19 = sys.create_slice(_c4d11, imm_c4d29, imm_c4d31);
    // _c4345 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c4345 = sys.create_array_read(array_ba589, _c2869);
    // _c4d2d = _c4d19 == _c4345, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c4d2d = sys.create_eq(_c4d19, _c4345);
    // _c4d45 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:35
    let imm_c4d49 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c4d45 = sys.create_array_read(array_ba53d, imm_c4d49);
    // _c4d4d = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:36
    let imm_c4d55 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c4d4d = sys.create_array_read(array_ba53d, imm_c4d55);
    // _c4d59 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4d61 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c4d59 = sys.create_array_read(array_ba53d, imm_c4d61);
    // _c4d65 = _c4d59[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4d75 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c4d7d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c4d65 = sys.create_slice(_c4d59, imm_c4d75, imm_c4d7d);
    // _c4ce1 = ~_c4d65, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c4ce1 = sys.create_flip(_c4d65);
    // _c4d85 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4d89 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c4d85 = sys.create_array_read(array_ba53d, imm_c4d89);
    // _c4d8d = _c4d85[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4d9d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c4da5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c4d8d = sys.create_slice(_c4d85, imm_c4d9d, imm_c4da5);
    // _c4d1d = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c4d1d = sys.create_array_read(array_ba589, _c2869);
    // _c4da1 = _c4d8d == _c4d1d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c4da1 = sys.create_eq(_c4d8d, _c4d1d);
    // _c4db9 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:38
    let imm_c4dbd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c4db9 = sys.create_array_read(array_ba53d, imm_c4dbd);
    // _c4dc1 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:39
    let imm_c4dc9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c4dc1 = sys.create_array_read(array_ba53d, imm_c4dc9);
    // _c4dcd = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4dd5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c4dcd = sys.create_array_read(array_ba53d, imm_c4dd5);
    // _c4dd9 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4de1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c4dd9 = sys.create_array_read(array_ba53d, imm_c4de1);
    // _c4de5 = _c4dd9[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4df5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c4dfd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c4de5 = sys.create_slice(_c4dd9, imm_c4df5, imm_c4dfd);
    // _c4df9 = _c4de5 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4e05 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c4df9 = sys.create_eq(_c4de5, imm_c4e05);
    // _c4e09 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c4e11 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c4e09 = sys.create_array_read(array_ba53d, imm_c4e11);
    // _c4e15 = _c4e09[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c4e25 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c4e2d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c4e15 = sys.create_slice(_c4e09, imm_c4e25, imm_c4e2d);
    // _c4365 = ~_c4e15, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c4365 = sys.create_flip(_c4e15);
    // _c4e35 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c4e39 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c4e35 = sys.create_array_read(array_ba53d, imm_c4e39);
    // _c4e3d = _c4e35[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c4e4d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c4e55 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c4e3d = sys.create_slice(_c4e35, imm_c4e4d, imm_c4e55);
    // _c4c6d = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c4c6d = sys.create_array_read(array_ba589, _c2869);
    // _c4e51 = _c4e3d == _c4c6d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c4e51 = sys.create_eq(_c4e3d, _c4c6d);
    // _c4e69 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:35
    let imm_c4e6d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c4e69 = sys.create_array_read(array_ba53d, imm_c4e6d);
    // _c4e71 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:36
    let imm_c4e79 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c4e71 = sys.create_array_read(array_ba53d, imm_c4e79);
    // _c4e7d = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4e85 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c4e7d = sys.create_array_read(array_ba53d, imm_c4e85);
    // _c4e89 = _c4e7d[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4e99 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c4ea1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c4e89 = sys.create_slice(_c4e7d, imm_c4e99, imm_c4ea1);
    // _c4c45 = ~_c4e89, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c4c45 = sys.create_flip(_c4e89);
    // _c4ea9 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4ead = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c4ea9 = sys.create_array_read(array_ba53d, imm_c4ead);
    // _c4eb1 = _c4ea9[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4ec1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c4ec9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c4eb1 = sys.create_slice(_c4ea9, imm_c4ec1, imm_c4ec9);
    // _c4e41 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c4e41 = sys.create_array_read(array_ba589, _c2869);
    // _c4ec5 = _c4eb1 == _c4e41, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c4ec5 = sys.create_eq(_c4eb1, _c4e41);
    // _c4edd = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:38
    let imm_c4ee1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c4edd = sys.create_array_read(array_ba53d, imm_c4ee1);
    // _c4ee5 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:39
    let imm_c4eed = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c4ee5 = sys.create_array_read(array_ba53d, imm_c4eed);
    // _c4ef1 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4ef9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c4ef1 = sys.create_array_read(array_ba53d, imm_c4ef9);
    // _c4efd = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4f05 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c4efd = sys.create_array_read(array_ba53d, imm_c4f05);
    // _c4f09 = _c4efd[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4f19 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c4f21 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c4f09 = sys.create_slice(_c4efd, imm_c4f19, imm_c4f21);
    // _c4f1d = _c4f09 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4f29 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c4f1d = sys.create_eq(_c4f09, imm_c4f29);
    // _c4f2d = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c4f35 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c4f2d = sys.create_array_read(array_ba53d, imm_c4f35);
    // _c4f39 = _c4f2d[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c4f49 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c4f51 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c4f39 = sys.create_slice(_c4f2d, imm_c4f49, imm_c4f51);
    // _c4c8d = ~_c4f39, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c4c8d = sys.create_flip(_c4f39);
    // _c4f59 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c4f5d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c4f59 = sys.create_array_read(array_ba53d, imm_c4f5d);
    // _c4f61 = _c4f59[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c4f71 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c4f79 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c4f61 = sys.create_slice(_c4f59, imm_c4f71, imm_c4f79);
    // _c4d91 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c4d91 = sys.create_array_read(array_ba589, _c2869);
    // _c4f75 = _c4f61 == _c4d91, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c4f75 = sys.create_eq(_c4f61, _c4d91);
    // _c4f8d = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:35
    let imm_c4f91 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c4f8d = sys.create_array_read(array_ba53d, imm_c4f91);
    // _c4f95 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:36
    let imm_c4f9d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c4f95 = sys.create_array_read(array_ba53d, imm_c4f9d);
    // _c4fa1 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4fa9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c4fa1 = sys.create_array_read(array_ba53d, imm_c4fa9);
    // _c4fad = _c4fa1[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4fbd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c4fc5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c4fad = sys.create_slice(_c4fa1, imm_c4fbd, imm_c4fc5);
    // _c4d69 = ~_c4fad, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c4d69 = sys.create_flip(_c4fad);
    // _c4fcd = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4fd1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c4fcd = sys.create_array_read(array_ba53d, imm_c4fd1);
    // _c4fd5 = _c4fcd[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c4fe5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c4fed = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c4fd5 = sys.create_slice(_c4fcd, imm_c4fe5, imm_c4fed);
    // _c4f65 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c4f65 = sys.create_array_read(array_ba589, _c2869);
    // _c4fe9 = _c4fd5 == _c4f65, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c4fe9 = sys.create_eq(_c4fd5, _c4f65);
    // _c5005 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:38
    let imm_c5009 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c5005 = sys.create_array_read(array_ba53d, imm_c5009);
    // _c500d = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:39
    let imm_c5015 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c500d = sys.create_array_read(array_ba53d, imm_c5015);
    // _c5019 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c5021 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c5019 = sys.create_array_read(array_ba53d, imm_c5021);
    // _c5025 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c502d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c5025 = sys.create_array_read(array_ba53d, imm_c502d);
    // _c5031 = _c5025[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c5041 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c5049 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c5031 = sys.create_slice(_c5025, imm_c5041, imm_c5049);
    // _c5045 = _c5031 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4e8d = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c5045 = sys.create_eq(_c5031, imm_c4e8d);
    // _c5055 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c505d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c5055 = sys.create_array_read(array_ba53d, imm_c505d);
    // _c5061 = _c5055[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c5071 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c5079 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c5061 = sys.create_slice(_c5055, imm_c5071, imm_c5079);
    // _c4db1 = ~_c5061, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c4db1 = sys.create_flip(_c5061);
    // _c5081 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c5085 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c5081 = sys.create_array_read(array_ba53d, imm_c5085);
    // _c5089 = _c5081[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c5099 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c50a1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c5089 = sys.create_slice(_c5081, imm_c5099, imm_c50a1);
    // _c4eb5 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c4eb5 = sys.create_array_read(array_ba589, _c2869);
    // _c509d = _c5089 == _c4eb5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c509d = sys.create_eq(_c5089, _c4eb5);
    // _c50b5 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:35
    let imm_c50b9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c50b5 = sys.create_array_read(array_ba53d, imm_c50b9);
    // _c50bd = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:36
    let imm_c50c5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c50bd = sys.create_array_read(array_ba53d, imm_c50c5);
    // _c50c9 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c50d1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c50c9 = sys.create_array_read(array_ba53d, imm_c50d1);
    // _c50d5 = _c50c9[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c50e5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c50ed = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c50d5 = sys.create_slice(_c50c9, imm_c50e5, imm_c50ed);
    // _c5051 = ~_c50d5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c5051 = sys.create_flip(_c50d5);
    // _c50f5 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c50f9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c50f5 = sys.create_array_read(array_ba53d, imm_c50f9);
    // _c50fd = _c50f5[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c510d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c5115 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c50fd = sys.create_slice(_c50f5, imm_c510d, imm_c5115);
    // _c508d = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c508d = sys.create_array_read(array_ba589, _c2869);
    // _c5111 = _c50fd == _c508d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c5111 = sys.create_eq(_c50fd, _c508d);
    // _c5129 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:38
    let imm_c512d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c5129 = sys.create_array_read(array_ba53d, imm_c512d);
    // _c5131 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:39
    let imm_c5139 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c5131 = sys.create_array_read(array_ba53d, imm_c5139);
    // _c513d = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c5145 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c513d = sys.create_array_read(array_ba53d, imm_c5145);
    // _c5149 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c5151 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c5149 = sys.create_array_read(array_ba53d, imm_c5151);
    // _c5155 = _c5149[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c5165 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c516d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c5155 = sys.create_slice(_c5149, imm_c5165, imm_c516d);
    // _c5169 = _c5155 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c4fb1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c5169 = sys.create_eq(_c5155, imm_c4fb1);
    // _c5179 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c5181 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c5179 = sys.create_array_read(array_ba53d, imm_c5181);
    // _c5185 = _c5179[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c5195 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c519d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c5185 = sys.create_slice(_c5179, imm_c5195, imm_c519d);
    // _c4ed5 = ~_c5185, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c4ed5 = sys.create_flip(_c5185);
    // _c51a5 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c51a9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c51a5 = sys.create_array_read(array_ba53d, imm_c51a9);
    // _c51ad = _c51a5[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c51bd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c51c5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c51ad = sys.create_slice(_c51a5, imm_c51bd, imm_c51c5);
    // _c4fd9 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c4fd9 = sys.create_array_read(array_ba589, _c2869);
    // _c51c1 = _c51ad == _c4fd9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c51c1 = sys.create_eq(_c51ad, _c4fd9);
    // _c51d9 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:35
    let imm_c51dd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c51d9 = sys.create_array_read(array_ba53d, imm_c51dd);
    // _c51e1 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:36
    let imm_c51e9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c51e1 = sys.create_array_read(array_ba53d, imm_c51e9);
    // _c51ed = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c51f5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c51ed = sys.create_array_read(array_ba53d, imm_c51f5);
    // _c51f9 = _c51ed[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c5209 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c5211 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c51f9 = sys.create_slice(_c51ed, imm_c5209, imm_c5211);
    // _c5175 = ~_c51f9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c5175 = sys.create_flip(_c51f9);
    // _c5219 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c521d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c5219 = sys.create_array_read(array_ba53d, imm_c521d);
    // _c5221 = _c5219[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c5231 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c5239 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c5221 = sys.create_slice(_c5219, imm_c5231, imm_c5239);
    // _c51b1 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c51b1 = sys.create_array_read(array_ba589, _c2869);
    // _c5235 = _c5221 == _c51b1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c5235 = sys.create_eq(_c5221, _c51b1);
    // _c524d = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:38
    let imm_c5251 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c524d = sys.create_array_read(array_ba53d, imm_c5251);
    // _c5255 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:39
    let imm_c525d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c5255 = sys.create_array_read(array_ba53d, imm_c525d);
    // _c5261 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c5269 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c5261 = sys.create_array_read(array_ba53d, imm_c5269);
    // _c526d = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c5275 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c526d = sys.create_array_read(array_ba53d, imm_c5275);
    // _c5279 = _c526d[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c5289 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c5291 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c5279 = sys.create_slice(_c526d, imm_c5289, imm_c5291);
    // _c528d = _c5279 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c5299 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c528d = sys.create_eq(_c5279, imm_c5299);
    // _c529d = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c52a5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c529d = sys.create_array_read(array_ba53d, imm_c52a5);
    // _c52a9 = _c529d[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c52b9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c52c1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c52a9 = sys.create_slice(_c529d, imm_c52b9, imm_c52c1);
    // _c4ff9 = ~_c52a9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c4ff9 = sys.create_flip(_c52a9);
    // _c52c9 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c52cd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c52c9 = sys.create_array_read(array_ba53d, imm_c52cd);
    // _c52d1 = _c52c9[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c52e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c52e9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c52d1 = sys.create_slice(_c52c9, imm_c52e1, imm_c52e9);
    // _c5101 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c5101 = sys.create_array_read(array_ba589, _c2869);
    // _c52e5 = _c52d1 == _c5101, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c52e5 = sys.create_eq(_c52d1, _c5101);
    // _c52fd = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:35
    let imm_c5301 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c52fd = sys.create_array_read(array_ba53d, imm_c5301);
    // _c5305 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:36
    let imm_c530d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c5305 = sys.create_array_read(array_ba53d, imm_c530d);
    // _c5311 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c5319 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c5311 = sys.create_array_read(array_ba53d, imm_c5319);
    // _c531d = _c5311[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c532d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c5335 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c531d = sys.create_slice(_c5311, imm_c532d, imm_c5335);
    // _c50d9 = ~_c531d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c50d9 = sys.create_flip(_c531d);
    // _c533d = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c5341 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c533d = sys.create_array_read(array_ba53d, imm_c5341);
    // _c5345 = _c533d[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c5355 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c535d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c5345 = sys.create_slice(_c533d, imm_c5355, imm_c535d);
    // _c52d5 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c52d5 = sys.create_array_read(array_ba589, _c2869);
    // _c5359 = _c5345 == _c52d5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c5359 = sys.create_eq(_c5345, _c52d5);
    // _c5371 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:38
    let imm_c5375 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c5371 = sys.create_array_read(array_ba53d, imm_c5375);
    // _c5379 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:39
    let imm_c5381 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c5379 = sys.create_array_read(array_ba53d, imm_c5381);
    // _c5385 = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c538d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c5385 = sys.create_array_read(array_ba53d, imm_c538d);
    // _c5391 = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c5399 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c5391 = sys.create_array_read(array_ba53d, imm_c5399);
    // _c539d = _c5391[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c53ad = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c53b5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c539d = sys.create_slice(_c5391, imm_c53ad, imm_c53b5);
    // _c53b1 = _c539d == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c53bd = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c53b1 = sys.create_eq(_c539d, imm_c53bd);
    // _c53c1 = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c53c9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c53c1 = sys.create_array_read(array_ba53d, imm_c53c9);
    // _c53cd = _c53c1[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c53dd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c53e5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c53cd = sys.create_slice(_c53c1, imm_c53dd, imm_c53e5);
    // _c5121 = ~_c53cd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c5121 = sys.create_flip(_c53cd);
    // _c53ed = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c53f1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c53ed = sys.create_array_read(array_ba53d, imm_c53f1);
    // _c53f5 = _c53ed[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c5c09 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c5c11 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c53f5 = sys.create_slice(_c53ed, imm_c5c09, imm_c5c11);
    // _c5225 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c5225 = sys.create_array_read(array_ba589, _c2869);
    // _c5c0d = _c53f5 == _c5225, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c5c0d = sys.create_eq(_c53f5, _c5225);
    // _c5c25 = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:35
    let imm_c5c29 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c5c25 = sys.create_array_read(array_ba53d, imm_c5c29);
    // _c5c2d = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:36
    let imm_c5c35 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c5c2d = sys.create_array_read(array_ba53d, imm_c5c35);
    // _c5c39 = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c5c41 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c5c39 = sys.create_array_read(array_ba53d, imm_c5c41);
    // _c5c45 = _c5c39[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c5c55 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c5c5d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c5c45 = sys.create_slice(_c5c39, imm_c5c55, imm_c5c5d);
    // _c51fd = ~_c5c45, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c51fd = sys.create_flip(_c5c45);
    // _c5c65 = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c5c69 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c5c65 = sys.create_array_read(array_ba53d, imm_c5c69);
    // _c5c6d = _c5c65[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c5c7d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c5c85 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c5c6d = sys.create_slice(_c5c65, imm_c5c7d, imm_c5c85);
    // _c43f5 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c43f5 = sys.create_array_read(array_ba589, _c2869);
    // _c5c81 = _c5c6d == _c43f5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c5c81 = sys.create_eq(_c5c6d, _c43f5);
    // _c5c99 = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:38
    let imm_c5c9d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c5c99 = sys.create_array_read(array_ba53d, imm_c5c9d);
    // _c5ca1 = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:39
    let imm_c5ca9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c5ca1 = sys.create_array_read(array_ba53d, imm_c5ca9);
    // _c5cad = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c5cb5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c5cad = sys.create_array_read(array_ba53d, imm_c5cb5);
    // _c5cb9 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c5cc1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c5cb9 = sys.create_array_read(array_ba53d, imm_c5cc1);
    // _c5cc5 = _c5cb9[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c5cd5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c5cdd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c5cc5 = sys.create_slice(_c5cb9, imm_c5cd5, imm_c5cdd);
    // _c5cd9 = _c5cc5 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c5321 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c5cd9 = sys.create_eq(_c5cc5, imm_c5321);
    // _c5ce9 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c5cf1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c5ce9 = sys.create_array_read(array_ba53d, imm_c5cf1);
    // _c5cf5 = _c5ce9[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c5d05 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c5d0d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c5cf5 = sys.create_slice(_c5ce9, imm_c5d05, imm_c5d0d);
    // _c5245 = ~_c5cf5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c5245 = sys.create_flip(_c5cf5);
    // _c5d15 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c5d19 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c5d15 = sys.create_array_read(array_ba53d, imm_c5d19);
    // _c5d1d = _c5d15[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c5d2d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c5d35 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c5d1d = sys.create_slice(_c5d15, imm_c5d2d, imm_c5d35);
    // _c5349 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c5349 = sys.create_array_read(array_ba589, _c2869);
    // _c5d31 = _c5d1d == _c5349, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c5d31 = sys.create_eq(_c5d1d, _c5349);
    // _c5d49 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:35
    let imm_c5d4d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c5d49 = sys.create_array_read(array_ba53d, imm_c5d4d);
    // _c5d51 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:36
    let imm_c5d59 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c5d51 = sys.create_array_read(array_ba53d, imm_c5d59);
    // _c5d5d = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c5d65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c5d5d = sys.create_array_read(array_ba53d, imm_c5d65);
    // _c5d69 = _c5d5d[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c5d79 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c5d81 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c5d69 = sys.create_slice(_c5d5d, imm_c5d79, imm_c5d81);
    // _c5ce5 = ~_c5d69, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c5ce5 = sys.create_flip(_c5d69);
    // _c5d89 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c5d8d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c5d89 = sys.create_array_read(array_ba53d, imm_c5d8d);
    // _c5d91 = _c5d89[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c5da1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c5da9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c5d91 = sys.create_slice(_c5d89, imm_c5da1, imm_c5da9);
    // _c5d21 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c5d21 = sys.create_array_read(array_ba589, _c2869);
    // _c5da5 = _c5d91 == _c5d21, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c5da5 = sys.create_eq(_c5d91, _c5d21);
    // _c5dbd = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:38
    let imm_c5dc1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c5dbd = sys.create_array_read(array_ba53d, imm_c5dc1);
    // _c5dc5 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:39
    let imm_c5dcd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c5dc5 = sys.create_array_read(array_ba53d, imm_c5dcd);
    // _c5dd1 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c5dd9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c5dd1 = sys.create_array_read(array_ba53d, imm_c5dd9);
    // _c5ddd = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c5de5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c5ddd = sys.create_array_read(array_ba53d, imm_c5de5);
    // _c5de9 = _c5ddd[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c5df9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c5e01 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c5de9 = sys.create_slice(_c5ddd, imm_c5df9, imm_c5e01);
    // _c5dfd = _c5de9 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:33
    let imm_c5e09 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c5dfd = sys.create_eq(_c5de9, imm_c5e09);
    // _c5e0d = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c5e15 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c5e0d = sys.create_array_read(array_ba53d, imm_c5e15);
    // _c5e19 = _c5e0d[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c5e29 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c5e31 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c5e19 = sys.create_slice(_c5e0d, imm_c5e29, imm_c5e31);
    // _c5369 = ~_c5e19, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c5369 = sys.create_flip(_c5e19);
    // _c5e39 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c5e3d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c5e39 = sys.create_array_read(array_ba53d, imm_c5e3d);
    // _c5e41 = _c5e39[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let imm_c5e51 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c5e59 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c5e41 = sys.create_slice(_c5e39, imm_c5e51, imm_c5e59);
    // _c5c71 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c5c71 = sys.create_array_read(array_ba589, _c2869);
    // _c5e55 = _c5e41 == _c5c71, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:34
    let _c5e55 = sys.create_eq(_c5e41, _c5c71);
    // _c5e6d = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:35
    let imm_c5e71 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c5e6d = sys.create_array_read(array_ba53d, imm_c5e71);
    // _c5e75 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:36
    let imm_c5e7d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c5e75 = sys.create_array_read(array_ba53d, imm_c5e7d);
    // _c5e81 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c5e89 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c5e81 = sys.create_array_read(array_ba53d, imm_c5e89);
    // _c5e8d = _c5e81[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c5e9d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c5ea5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c5e8d = sys.create_slice(_c5e81, imm_c5e9d, imm_c5ea5);
    // _c5c49 = ~_c5e8d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c5c49 = sys.create_flip(_c5e8d);
    // _c5ead = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c5eb1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c5ead = sys.create_array_read(array_ba53d, imm_c5eb1);
    // _c5eb5 = _c5ead[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let imm_c5ec5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c5ecd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c5eb5 = sys.create_slice(_c5ead, imm_c5ec5, imm_c5ecd);
    // _c5e45 = array_ba589[_c2869], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c5e45 = sys.create_array_read(array_ba589, _c2869);
    // _c5ec9 = _c5eb5 == _c5e45, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:37
    let _c5ec9 = sys.create_eq(_c5eb5, _c5e45);
    // _c5ee1 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:38
    let imm_c5ee5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c5ee1 = sys.create_array_read(array_ba53d, imm_c5ee5);
    // _c5ee9 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:39
    let imm_c5ef1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c5ee9 = sys.create_array_read(array_ba53d, imm_c5ef1);
    // array_ba589[_c2869] = (16:b5), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:40
    let imm_c5ef5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64); // (16:b5)
    sys.create_array_write(array_ba589, _c2869, imm_c5ef5);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _c2881 = _ba5e5.rdata.valid(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:42
    // Get port rdata
    let _ba5e5_rdata = {
        let module = _ba5e5.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("rdata").unwrap().upcast()
    };
    // Get port rdata
    let _ba5e5_rdata = {
        let module = _ba5e5.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("rdata").unwrap().upcast()
    };
    let _c2881 = sys.create_fifo_valid(_ba5e5_rdata);
    // _c5f01 = _ba5e5.rdata.peek(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:42
    // Get port rdata
    let _ba5e5_rdata = {
        let module = _ba5e5.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("rdata").unwrap().upcast()
    };
    // Get port rdata
    let _ba5e5_rdata = {
        let module = _ba5e5.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("rdata").unwrap().upcast()
    };
    let _c5f01 = sys.create_fifo_peek(_ba5e5_rdata);
    // _c5f09 = _c2881 ? _c5f01 : (0:b32), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:42
    let imm_c5f11 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let _c5f09 = sys.create_select(_c2881, _c5f01, imm_c5f11);
    // _ba599.mdata.push(_c5f09) // handle = _c5f25, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:43
    let push_9c0d5 = sys.bind_arg(_9c0d5, "mdata".into(), _c5f09);
    // async_call _9c0d5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/memory_access.py:43
    sys.create_async_call(_9c0d5);
    // Fill in the body of _ba7f9
    sys.set_current_module(_ba7f9);
    // module root block
    // _bcc31 = _ba7f9.signals.peek(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:48
    // Get port signals
    let _ba7f9_signals = {
        let module = _ba7f9.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("signals").unwrap().upcast()
    };
    // Get port signals
    let _ba7f9_signals = {
        let module = _ba7f9.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("signals").unwrap().upcast()
    };
    let _bcc31 = sys.create_fifo_peek(_ba7f9_signals);
    // _bcc45 = _ba7f9.signals.valid(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:50
    // Get port signals
    let _ba7f9_signals = {
        let module = _ba7f9.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("signals").unwrap().upcast()
    };
    // Get port signals
    let _ba7f9_signals = {
        let module = _ba7f9.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("signals").unwrap().upcast()
    };
    let _bcc45 = sys.create_fifo_valid(_ba7f9_signals);
    // _bcc59 = _ba7f9.fetch_addr.valid(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:50
    // Get port fetch_addr
    let _ba7f9_fetch_addr = {
        let module = _ba7f9.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("fetch_addr").unwrap().upcast()
    };
    // Get port fetch_addr
    let _ba7f9_fetch_addr = {
        let module = _ba7f9.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("fetch_addr").unwrap().upcast()
    };
    let _bcc59 = sys.create_fifo_valid(_ba7f9_fetch_addr);
    // _bcc65 = _bcc45 & _bcc59, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:50
    let _bcc65 = sys.create_bitwise_and(_bcc45, _bcc59);
    // side effect intrinsic.wait_until({'_bcc65'}), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:50
    sys.create_wait_until(_bcc65);
    // _bcc79 = _ba7f9.signals.pop(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:50
    // Get port signals
    let _ba7f9_signals = {
        let module = _ba7f9.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("signals").unwrap().upcast()
    };
    let _bcc79 = sys.create_fifo_pop(_ba7f9_signals);
    // _bcc85 = _ba7f9.fetch_addr.pop(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:50
    // Get port fetch_addr
    let _ba7f9_fetch_addr = {
        let module = _ba7f9.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("fetch_addr").unwrap().upcast()
    };
    let _bcc85 = sys.create_fifo_pop(_ba7f9_fetch_addr);
    // _bcc9d = _bcc79[(92:u7):(96:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:53
    let imm_bcca9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x5c as u64); // (92:u7)
    let imm_bccb1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x60 as u64); // (96:u7)
    let _bcc9d = sys.create_slice(_bcc79, imm_bcca9, imm_bccb1);
    // _bccbd = _bcc79[(86:u7):(90:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:54
    let imm_bcd35 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x56 as u64); // (86:u7)
    let imm_bcd3d = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x5a as u64); // (90:u7)
    let _bccbd = sys.create_slice(_bcc79, imm_bcd35, imm_bcd3d);
    // _bcd45 = _bcc79[(80:u7):(84:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:55
    let imm_bcd4d = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x50 as u64); // (80:u7)
    let imm_bcd55 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x54 as u64); // (84:u7)
    let _bcd45 = sys.create_slice(_bcc79, imm_bcd4d, imm_bcd55);
    // _bcd69 = (1:b1) & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:57
    let imm_bcd51 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let imm_bcd65 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _bcd69 = sys.create_bitwise_and(imm_bcd51, imm_bcd65);
    // _bcd6d = _bcc79[(41:u6):(72:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bcd81 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x29 as u64); // (41:u6)
    let imm_bcd89 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x48 as u64); // (72:u7)
    let _bcd6d = sys.create_slice(_bcc79, imm_bcd81, imm_bcd89);
    // _bcd91 = _bcd6d[(0:u1):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bcd9d = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_bcda5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _bcd91 = sys.create_slice(_bcd6d, imm_bcd9d, imm_bcda5);
    // _bcdb5 = _bcd91 == (3860:b12), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bcdb1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(12), 0xf14 as u64); // (3860:b12)
    let _bcdb5 = sys.create_eq(_bcd91, imm_bcdb1);
    // _bcdc5 = _bcdb5 ? (9:b4) : (0:b4), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bcdc1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0x9 as u64); // (9:b4)
    let imm_bcd75 = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0x0 as u64); // (0:b4)
    let _bcdc5 = sys.create_select(_bcdb5, imm_bcdc1, imm_bcd75);
    // _bcdd1 = _bcc79[(74:u7):(74:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_bcde1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let imm_bcde9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let _bcdd1 = sys.create_slice(_bcc79, imm_bcde1, imm_bcde9);
    // _bcdf5 = _bcdd1 ? (2:b4) : _bcdc5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_bcde5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0x2 as u64); // (2:b4)
    let _bcdf5 = sys.create_select(_bcdd1, imm_bcde5, _bcdc5);
    // _bce01 = _bcc79[(41:u6):(72:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bce0d = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x29 as u64); // (41:u6)
    let imm_bce15 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x48 as u64); // (72:u7)
    let _bce01 = sys.create_slice(_bcc79, imm_bce0d, imm_bce15);
    // _bce1d = _bce01[(0:u1):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bce25 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_bce2d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _bce1d = sys.create_slice(_bce01, imm_bce25, imm_bce2d);
    // _bce3d = _bce1d == (773:b12), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bce39 = sys.get_const_int(assassyn::ir::DataType::bits_ty(12), 0x305 as u64); // (773:b12)
    let _bce3d = sys.create_eq(_bce1d, imm_bce39);
    // _bce4d = _bce3d ? (1:b4) : _bcdf5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bce49 = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0x1 as u64); // (1:b4)
    let _bce4d = sys.create_select(_bce3d, imm_bce49, _bcdf5);
    // _bce59 = _bcc79[(74:u7):(74:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_bce65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let imm_bce6d = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let _bce59 = sys.create_slice(_bcc79, imm_bce65, imm_bce6d);
    // _bce79 = _bce59 ? (2:b4) : _bce4d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_bce69 = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0x2 as u64); // (2:b4)
    let _bce79 = sys.create_select(_bce59, imm_bce69, _bce4d);
    // _bce85 = _bcc79[(41:u6):(72:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bce91 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x29 as u64); // (41:u6)
    let imm_bce99 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x48 as u64); // (72:u7)
    let _bce85 = sys.create_slice(_bcc79, imm_bce91, imm_bce99);
    // _bcea1 = _bce85[(0:u1):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bcea9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_bceb1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _bcea1 = sys.create_slice(_bce85, imm_bcea9, imm_bceb1);
    // _bcebd = _bcea1 == (1860:b12), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bcead = sys.get_const_int(assassyn::ir::DataType::bits_ty(12), 0x744 as u64); // (1860:b12)
    let _bcebd = sys.create_eq(_bcea1, imm_bcead);
    // _bcec9 = _bcebd ? (15:b4) : _bce79, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bcec1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0xf as u64); // (15:b4)
    let _bcec9 = sys.create_select(_bcebd, imm_bcec1, _bce79);
    // _bcecd = _bcc79[(74:u7):(74:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_bcedd = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let imm_bcee5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let _bcecd = sys.create_slice(_bcc79, imm_bcedd, imm_bcee5);
    // _bcee1 = _bcecd ? (2:b4) : _bcec9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_ba7d9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0x2 as u64); // (2:b4)
    let _bcee1 = sys.create_select(_bcecd, imm_ba7d9, _bcec9);
    // _bcef1 = _bcc79[(41:u6):(72:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bcf01 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x29 as u64); // (41:u6)
    let imm_bcf09 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x48 as u64); // (72:u7)
    let _bcef1 = sys.create_slice(_bcc79, imm_bcf01, imm_bcf09);
    // _bce29 = _bcef1[(0:u1):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bcf15 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_bcf1d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _bce29 = sys.create_slice(_bcef1, imm_bcf15, imm_bcf1d);
    // _bcf29 = _bce29 == (384:b12), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bcf19 = sys.get_const_int(assassyn::ir::DataType::bits_ty(12), 0x180 as u64); // (384:b12)
    let _bcf29 = sys.create_eq(_bce29, imm_bcf19);
    // _bcf35 = _bcf29 ? (10:b4) : _bcee1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bcf2d = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0xa as u64); // (10:b4)
    let _bcf35 = sys.create_select(_bcf29, imm_bcf2d, _bcee1);
    // _bcf39 = _bcc79[(74:u7):(74:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_bcf49 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let imm_bcf51 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let _bcf39 = sys.create_slice(_bcc79, imm_bcf49, imm_bcf51);
    // _bcf4d = _bcf39 ? (2:b4) : _bcf35, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_bcf59 = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0x2 as u64); // (2:b4)
    let _bcf4d = sys.create_select(_bcf39, imm_bcf59, _bcf35);
    // _bcf5d = _bcc79[(41:u6):(72:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bcf6d = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x29 as u64); // (41:u6)
    let imm_bcf75 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x48 as u64); // (72:u7)
    let _bcf5d = sys.create_slice(_bcc79, imm_bcf6d, imm_bcf75);
    // _bce51 = _bcf5d[(0:u1):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bcf81 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_bcf89 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _bce51 = sys.create_slice(_bcf5d, imm_bcf81, imm_bcf89);
    // _bcf95 = _bce51 == (944:b12), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bcf85 = sys.get_const_int(assassyn::ir::DataType::bits_ty(12), 0x3b0 as u64); // (944:b12)
    let _bcf95 = sys.create_eq(_bce51, imm_bcf85);
    // _bcfa1 = _bcf95 ? (11:b4) : _bcf4d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bcf99 = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0xb as u64); // (11:b4)
    let _bcfa1 = sys.create_select(_bcf95, imm_bcf99, _bcf4d);
    // _bcfa5 = _bcc79[(74:u7):(74:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_bcfb5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let imm_bcfbd = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let _bcfa5 = sys.create_slice(_bcc79, imm_bcfb5, imm_bcfbd);
    // _bcfb9 = _bcfa5 ? (2:b4) : _bcfa1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_bcfc5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0x2 as u64); // (2:b4)
    let _bcfb9 = sys.create_select(_bcfa5, imm_bcfc5, _bcfa1);
    // _bcfc9 = _bcc79[(41:u6):(72:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bcfd9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x29 as u64); // (41:u6)
    let imm_bcfe1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x48 as u64); // (72:u7)
    let _bcfc9 = sys.create_slice(_bcc79, imm_bcfd9, imm_bcfe1);
    // _bcf9d = _bcfc9[(0:u1):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bcfed = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_bcff5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _bcf9d = sys.create_slice(_bcfc9, imm_bcfed, imm_bcff5);
    // _be405 = _bcf9d == (928:b12), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_bcff1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(12), 0x3a0 as u64); // (928:b12)
    let _be405 = sys.create_eq(_bcf9d, imm_bcff1);
    // _be411 = _be405 ? (12:b4) : _bcfb9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be409 = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0xc as u64); // (12:b4)
    let _be411 = sys.create_select(_be405, imm_be409, _bcfb9);
    // _be415 = _bcc79[(74:u7):(74:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_be425 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let imm_be42d = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let _be415 = sys.create_slice(_bcc79, imm_be425, imm_be42d);
    // _be429 = _be415 ? (2:b4) : _be411, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_bcf3d = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0x2 as u64); // (2:b4)
    let _be429 = sys.create_select(_be415, imm_bcf3d, _be411);
    // _be439 = _bcc79[(41:u6):(72:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be449 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x29 as u64); // (41:u6)
    let imm_be451 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x48 as u64); // (72:u7)
    let _be439 = sys.create_slice(_bcc79, imm_be449, imm_be451);
    // _be40d = _be439[(0:u1):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be45d = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_be465 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _be40d = sys.create_slice(_be439, imm_be45d, imm_be465);
    // _be471 = _be40d == (772:b12), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be461 = sys.get_const_int(assassyn::ir::DataType::bits_ty(12), 0x304 as u64); // (772:b12)
    let _be471 = sys.create_eq(_be40d, imm_be461);
    // _be47d = _be471 ? (4:b4) : _be429, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be475 = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0x4 as u64); // (4:b4)
    let _be47d = sys.create_select(_be471, imm_be475, _be429);
    // _be481 = _bcc79[(74:u7):(74:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_be491 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let imm_be499 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let _be481 = sys.create_slice(_bcc79, imm_be491, imm_be499);
    // _be495 = _be481 ? (2:b4) : _be47d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_bcfa9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0x2 as u64); // (2:b4)
    let _be495 = sys.create_select(_be481, imm_bcfa9, _be47d);
    // _be4a5 = _bcc79[(41:u6):(72:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be4b5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x29 as u64); // (41:u6)
    let imm_be4bd = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x48 as u64); // (72:u7)
    let _be4a5 = sys.create_slice(_bcc79, imm_be4b5, imm_be4bd);
    // _be479 = _be4a5[(0:u1):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be4c9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_be4d1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _be479 = sys.create_slice(_be4a5, imm_be4c9, imm_be4d1);
    // _be4dd = _be479 == (770:b12), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be4cd = sys.get_const_int(assassyn::ir::DataType::bits_ty(12), 0x302 as u64); // (770:b12)
    let _be4dd = sys.create_eq(_be479, imm_be4cd);
    // _be4e9 = _be4dd ? (13:b4) : _be495, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be4e1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0xd as u64); // (13:b4)
    let _be4e9 = sys.create_select(_be4dd, imm_be4e1, _be495);
    // _be4ed = _bcc79[(74:u7):(74:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_be4fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let imm_be505 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let _be4ed = sys.create_slice(_bcc79, imm_be4fd, imm_be505);
    // _be501 = _be4ed ? (2:b4) : _be4e9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_be50d = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0x2 as u64); // (2:b4)
    let _be501 = sys.create_select(_be4ed, imm_be50d, _be4e9);
    // _be511 = _bcc79[(41:u6):(72:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be521 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x29 as u64); // (41:u6)
    let imm_be529 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x48 as u64); // (72:u7)
    let _be511 = sys.create_slice(_bcc79, imm_be521, imm_be529);
    // _be4e5 = _be511[(0:u1):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be535 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_be53d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _be4e5 = sys.create_slice(_be511, imm_be535, imm_be53d);
    // _be549 = _be4e5 == (771:b12), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be539 = sys.get_const_int(assassyn::ir::DataType::bits_ty(12), 0x303 as u64); // (771:b12)
    let _be549 = sys.create_eq(_be4e5, imm_be539);
    // _be555 = _be549 ? (14:b4) : _be501, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be54d = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0xe as u64); // (14:b4)
    let _be555 = sys.create_select(_be549, imm_be54d, _be501);
    // _be559 = _bcc79[(74:u7):(74:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_be569 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let imm_be571 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let _be559 = sys.create_slice(_bcc79, imm_be569, imm_be571);
    // _be56d = _be559 ? (2:b4) : _be555, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_be579 = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0x2 as u64); // (2:b4)
    let _be56d = sys.create_select(_be559, imm_be579, _be555);
    // _be57d = _bcc79[(41:u6):(72:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be58d = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x29 as u64); // (41:u6)
    let imm_be595 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x48 as u64); // (72:u7)
    let _be57d = sys.create_slice(_bcc79, imm_be58d, imm_be595);
    // _be551 = _be57d[(0:u1):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be5a1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_be5a9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _be551 = sys.create_slice(_be57d, imm_be5a1, imm_be5a9);
    // _be5b5 = _be551 == (768:b12), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be5a5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(12), 0x300 as u64); // (768:b12)
    let _be5b5 = sys.create_eq(_be551, imm_be5a5);
    // _be5c1 = _be5b5 ? (8:b4) : _be56d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be5b9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0x8 as u64); // (8:b4)
    let _be5c1 = sys.create_select(_be5b5, imm_be5b9, _be56d);
    // _be5c5 = _bcc79[(74:u7):(74:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_be5d5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let imm_be5dd = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let _be5c5 = sys.create_slice(_bcc79, imm_be5d5, imm_be5dd);
    // _be5d9 = _be5c5 ? (2:b4) : _be5c1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_be5e5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0x2 as u64); // (2:b4)
    let _be5d9 = sys.create_select(_be5c5, imm_be5e5, _be5c1);
    // _be5ed = _bcc79[(41:u6):(72:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be5fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x29 as u64); // (41:u6)
    let imm_be605 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x48 as u64); // (72:u7)
    let _be5ed = sys.create_slice(_bcc79, imm_be5fd, imm_be605);
    // _be5bd = _be5ed[(0:u1):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be611 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_be619 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _be5bd = sys.create_slice(_be5ed, imm_be611, imm_be619);
    // _be625 = _be5bd == (833:b12), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be615 = sys.get_const_int(assassyn::ir::DataType::bits_ty(12), 0x341 as u64); // (833:b12)
    let _be625 = sys.create_eq(_be5bd, imm_be615);
    // _be631 = _be625 ? (2:b4) : _be5d9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:66
    let imm_be629 = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0x2 as u64); // (2:b4)
    let _be631 = sys.create_select(_be625, imm_be629, _be5d9);
    // _be639 = _bcc79[(74:u7):(74:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_be649 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let imm_be651 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4a as u64); // (74:u7)
    let _be639 = sys.create_slice(_bcc79, imm_be649, imm_be651);
    // _be64d = _be639 ? (2:b4) : _be631, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:67
    let imm_be659 = sys.get_const_int(assassyn::ir::DataType::bits_ty(4), 0x2 as u64); // (2:b4)
    let _be64d = sys.create_select(_be639, imm_be659, _be631);
    // _be66d = _bcc79[(78:u7):(78:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:70
    let imm_be679 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4e as u64); // (78:u7)
    let imm_be681 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4e as u64); // (78:u7)
    let _be66d = sys.create_slice(_bcc79, imm_be679, imm_be681);
    // _be689 = _bcc79[(77:u7):(77:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:70
    let imm_be691 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4d as u64); // (77:u7)
    let imm_be699 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4d as u64); // (77:u7)
    let _be689 = sys.create_slice(_bcc79, imm_be691, imm_be699);
    // _be55d = _be66d | _be689, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:70
    let _be55d = sys.create_bitwise_or(_be66d, _be689);
    // _be6a1 = _bcc79[(77:u7):(77:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:72
    let imm_be6ad = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4d as u64); // (77:u7)
    let imm_be6b5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4d as u64); // (77:u7)
    let _be6a1 = sys.create_slice(_bcc79, imm_be6ad, imm_be6b5);
    // _be4f1 = array_b9a49[_bcc9d], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:72
    let _be4f1 = sys.create_array_read(array_b9a49, _bcc9d);
    // _be6b1 = _be6a1 ? _be4f1 : (0:b32), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:72
    let imm_be669 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let _be6b1 = sys.create_select(_be6a1, _be4f1, imm_be669);
    // _be6c1 = _bcc79[(75:u7):(75:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:73
    let imm_be6d1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4b as u64); // (75:u7)
    let imm_be6d9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4b as u64); // (75:u7)
    let _be6c1 = sys.create_slice(_bcc79, imm_be6d1, imm_be6d9);
    // _be581 = { (0:b27) _bcc9d }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:73
    let imm_be6e1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(27), 0x0 as u64); // (0:b27)
    let _be581 = sys.create_concat(imm_be6e1, _bcc9d);
    // _be6e5 = _be6c1 ? _be581 : _be6b1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:73
    let _be6e5 = sys.create_select(_be6c1, _be581, _be6b1);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _be6f5 = sys.create_conditional_block(_be55d);
    sys.set_current_block(_be6f5);
    // log('csr_id: {} | new: {:08x} |', _be64d, _be6e5), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:76
    let fmt = sys.get_str_literal("csr_id: {} | new: {:08x} |".into());
    sys.create_log(fmt, vec![_be64d, _be6e5]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _be701 = _bcc79[(80:u7):(84:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:81
    let imm_be709 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x50 as u64); // (80:u7)
    let imm_be711 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x54 as u64); // (84:u7)
    let _be701 = sys.create_slice(_bcc79, imm_be709, imm_be711);
    // _be719 = _bcc79[(91:u7):(91:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:83
    let imm_be721 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x5b as u64); // (91:u7)
    let imm_be729 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x5b as u64); // (91:u7)
    let _be719 = sys.create_slice(_bcc79, imm_be721, imm_be729);
    // _be731 = _bcc79[(40:u6):(40:u6)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:83
    let imm_be739 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x28 as u64); // (40:u6)
    let imm_be741 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x28 as u64); // (40:u6)
    let _be731 = sys.create_slice(_bcc79, imm_be739, imm_be741);
    // _be5c9 = _be719 & _be731, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:83
    let _be5c9 = sys.create_bitwise_and(_be719, _be731);
    // _be749 = _bcc79[(41:u6):(72:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:83
    let imm_be755 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x29 as u64); // (41:u6)
    let imm_be75d = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x48 as u64); // (72:u7)
    let _be749 = sys.create_slice(_bcc79, imm_be755, imm_be75d);
    // _be759 = _be749 == (1:b32), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:83
    let imm_be765 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x1 as u64); // (1:b32)
    let _be759 = sys.create_eq(_be749, imm_be765);
    // _be76d = _bcc79[(41:u6):(72:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:83
    let imm_be77d = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x29 as u64); // (41:u6)
    let imm_be785 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x48 as u64); // (72:u7)
    let _be76d = sys.create_slice(_bcc79, imm_be77d, imm_be785);
    // _be781 = _be76d == (0:b32), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:83
    let imm_be78d = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let _be781 = sys.create_eq(_be76d, imm_be78d);
    // _be795 = _be759 | _be781, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:83
    let _be795 = sys.create_bitwise_or(_be759, _be781);
    // _be799 = _be5c9 & _be795, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:83
    let _be799 = sys.create_bitwise_and(_be5c9, _be795);
    // _be791 = _bcc79[(22:u5):(37:u6)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:83
    let imm_be7a9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x16 as u64); // (22:u5)
    let imm_be7b1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x25 as u64); // (37:u6)
    let _be791 = sys.create_slice(_bcc79, imm_be7a9, imm_be7b1);
    // _be7ad = _be791 == (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:83
    let imm_be7b9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _be7ad = sys.create_eq(_be791, imm_be7b9);
    // _be7c1 = _be799 & _be7ad, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:83
    let _be7c1 = sys.create_bitwise_and(_be799, _be7ad);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _be7c5 = sys.create_conditional_block(_be7c1);
    sys.set_current_block(_be7c5);
    // log('ebreak | halt | ecall', ), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:85
    let fmt = sys.get_str_literal("ebreak | halt | ecall".into());
    sys.create_log(fmt, vec![]);
    // side effect intrinsic.finish({''}), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:86
    sys.create_finish();
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _be7d5 = _bcc79[(4:u3):(4:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:88
    let imm_be7dd = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x4 as u64); // (4:u3)
    let imm_be7e5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x4 as u64); // (4:u3)
    let _be7d5 = sys.create_slice(_bcc79, imm_be7dd, imm_be7e5);
    // _be7ed = _bcc79[(3:u2):(3:u2)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:89
    let imm_be7f9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(2), 0x3 as u64); // (3:u2)
    let imm_be7fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(2), 0x3 as u64); // (3:u2)
    let _be7ed = sys.create_slice(_bcc79, imm_be7f9, imm_be7fd);
    // _be6d5 = _be7d5 & _be7ed, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:88
    let _be6d5 = sys.create_bitwise_and(_be7d5, _be7ed);
    // _bec0d = _bcc79[(40:u6):(40:u6)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:90
    let imm_bec19 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x28 as u64); // (40:u6)
    let imm_bec21 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x28 as u64); // (40:u6)
    let _bec0d = sys.create_slice(_bcc79, imm_bec19, imm_bec21);
    // _be5f1 = _be6d5 & _bec0d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:88
    let _be5f1 = sys.create_bitwise_and(_be6d5, _bec0d);
    // _bec29 = _bcc79[(41:u6):(72:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:91
    let imm_bec35 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x29 as u64); // (41:u6)
    let imm_bec3d = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x48 as u64); // (72:u7)
    let _bec29 = sys.create_slice(_bcc79, imm_bec35, imm_bec3d);
    // _bec39 = _bec29 == (0:b32), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:91
    let imm_be79d = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let _bec39 = sys.create_eq(_bec29, imm_be79d);
    // _bec4d = _be5f1 & _bec39, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:88
    let _bec4d = sys.create_bitwise_and(_be5f1, _bec39);
    // _bec49 = _bcc79[(6:u3):(21:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:92
    let imm_bec5d = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let imm_bec65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x15 as u64); // (21:u5)
    let _bec49 = sys.create_slice(_bcc79, imm_bec5d, imm_bec65);
    // _bec61 = _bec49 == (2048:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:92
    let imm_be7c9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x800 as u64); // (2048:b16)
    let _bec61 = sys.create_eq(_bec49, imm_be7c9);
    // _bec75 = _bec4d & _bec61, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:88
    let _bec75 = sys.create_bitwise_and(_bec4d, _bec61);
    // _bec71 = _bcc79[(22:u5):(37:u6)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:93
    let imm_bec85 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x16 as u64); // (22:u5)
    let imm_bec8d = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x25 as u64); // (37:u6)
    let _bec71 = sys.create_slice(_bcc79, imm_bec85, imm_bec8d);
    // _bec89 = _bec71 == (1:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:93
    let imm_be6c5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let _bec89 = sys.create_eq(_bec71, imm_be6c5);
    // _bec9d = _bec75 & _bec89, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:88
    let _bec9d = sys.create_bitwise_and(_bec75, _bec89);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _beca1 = sys.create_conditional_block(_bec9d);
    sys.set_current_block(_beca1);
    // log('trap', ), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:95
    let fmt = sys.get_str_literal("trap".into());
    sys.create_log(fmt, vec![]);
    // side effect intrinsic.finish({''}), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:96
    sys.create_finish();
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _becb1 = _bcc79[(77:u7):(77:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:104
    let imm_becb9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4d as u64); // (77:u7)
    let imm_becc1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4d as u64); // (77:u7)
    let _becb1 = sys.create_slice(_bcc79, imm_becb9, imm_becc1);
    // _beccd = _becb1 ? (0:b32) : (0:b32), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:104
    let imm_becc9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let imm_bcc91 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let _beccd = sys.create_select(_becb1, imm_becc9, imm_bcc91);
    // _becd9 = array_ba595[_be64d], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:108
    let _becd9 = sys.create_array_read(array_ba595, _be64d);
    // _becdd = _be55d ? _becd9 : (0:b32), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:108
    let imm_bcc99 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let _becdd = sys.create_select(_be55d, _becd9, imm_bcc99);
    // _bece1 = _bcc79[(3:u2):(3:u2)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:113
    let imm_becf1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(2), 0x3 as u64); // (3:u2)
    let imm_becf9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(2), 0x3 as u64); // (3:u2)
    let _bece1 = sys.create_slice(_bcc79, imm_becf1, imm_becf9);
    // _bed01 = _bcc79[(73:u7):(73:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:113
    let imm_bed09 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x49 as u64); // (73:u7)
    let imm_bed11 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x49 as u64); // (73:u7)
    let _bed01 = sys.create_slice(_bcc79, imm_bed09, imm_bed11);
    // _bec79 = _bece1 | _bed01, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:113
    let _bec79 = sys.create_bitwise_or(_bece1, _bed01);
    // _bed0d = _bec79 ? _bcc85 : _beccd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:113
    let _bed0d = sys.create_select(_bec79, _bcc85, _beccd);
    // _bed1d = _bcc79[(40:u6):(40:u6)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:114
    let imm_bed2d = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x28 as u64); // (40:u6)
    let imm_bed35 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x28 as u64); // (40:u6)
    let _bed1d = sys.create_slice(_bcc79, imm_bed2d, imm_bed35);
    // _bed3d = _bcc79[(41:u6):(72:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:114
    let imm_bed45 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x29 as u64); // (41:u6)
    let imm_bed4d = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x48 as u64); // (72:u7)
    let _bed3d = sys.create_slice(_bcc79, imm_bed45, imm_bed4d);
    // _bece5 = _bed1d ? _bed3d : _becdd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:114
    let _bece5 = sys.create_select(_bed1d, _bed3d, _becdd);
    // _bed5d = bitcast _bed0d to i32, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:118
    let _bed5d = sys.create_bitcast(_bed0d, assassyn::ir::DataType::int_ty(32));
    // _bed69 = bitcast _bece5 to i32, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:118
    let _bed69 = sys.create_bitcast(_bece5, assassyn::ir::DataType::int_ty(32));
    // _bed71 = _bed5d + _bed69, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:118
    let _bed71 = sys.create_add(_bed5d, _bed69);
    // _bed75 = bitcast _bed71 to b32, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:118
    let _bed75 = sys.create_bitcast(_bed71, assassyn::ir::DataType::bits_ty(32));
    // _bed7d = bitcast _beccd to i32, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:119
    let _bed7d = sys.create_bitcast(_beccd, assassyn::ir::DataType::int_ty(32));
    // _bed85 = bitcast _becdd to i32, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:119
    let _bed85 = sys.create_bitcast(_becdd, assassyn::ir::DataType::int_ty(32));
    // _bed8d = _bed7d < _bed85, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:119
    let _bed8d = sys.create_ilt(_bed7d, _bed85);
    // _bed9d = _bed8d ? (1:b32) : (0:b32), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:119
    let imm_bed89 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x1 as u64); // (1:b32)
    let imm_bed99 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let _bed9d = sys.create_select(_bed8d, imm_bed89, imm_bed99);
    // _beda9 = _beccd == _becdd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:120
    let _beda9 = sys.create_eq(_beccd, _becdd);
    // _bedb9 = _beda9 ? (1:b32) : (0:b32), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:120
    let imm_beda5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x1 as u64); // (1:b32)
    let imm_bedb5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let _bedb9 = sys.create_select(_beda9, imm_beda5, imm_bedb5);
    // _bedc5 = _beccd < _becdd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:121
    let _bedc5 = sys.create_ilt(_beccd, _becdd);
    // _bedd5 = _bedc5 ? (1:b32) : (0:b32), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:121
    let imm_bedc1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x1 as u64); // (1:b32)
    let imm_bedd1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let _bedd5 = sys.create_select(_bedc5, imm_bedc1, imm_bedd1);
    // _bede1 = bitcast _beccd to i32, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:122
    let _bede1 = sys.create_bitcast(_beccd, assassyn::ir::DataType::int_ty(32));
    // _bede9 = _bece5[(0:u1):(4:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:122
    let imm_bedf1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_bedf9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x4 as u64); // (4:u3)
    let _bede9 = sys.create_slice(_bece5, imm_bedf1, imm_bedf9);
    // _bed81 = bitcast _bede9 to i5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:122
    let _bed81 = sys.create_bitcast(_bede9, assassyn::ir::DataType::int_ty(5));
    // _bee09 = _bede1 >> _bed81, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:122
    let _bee09 = sys.create_shr(_bede1, _bed81);
    // _bee0d = bitcast _bee09 to b32, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:122
    let _bee0d = sys.create_bitcast(_bee09, assassyn::ir::DataType::bits_ty(32));
    // _bee15 = bitcast _beccd to i32, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:123
    let _bee15 = sys.create_bitcast(_beccd, assassyn::ir::DataType::int_ty(32));
    // _bee1d = bitcast _becdd to i32, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:123
    let _bee1d = sys.create_bitcast(_becdd, assassyn::ir::DataType::int_ty(32));
    // _bee25 = _bee15 - _bee1d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:123
    let _bee25 = sys.create_sub(_bee15, _bee1d);
    // _bee29 = bitcast _bee25 to b32, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:123
    let _bee29 = sys.create_bitcast(_bee25, assassyn::ir::DataType::bits_ty(32));
    // _bee31 = _beccd ^ _bece5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:130
    let _bee31 = sys.create_bitwise_xor(_beccd, _bece5);
    // _bee35 = _beccd | _becdd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:131
    let _bee35 = sys.create_bitwise_or(_beccd, _becdd);
    // _bee39 = _beccd | _bece5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:132
    let _bee39 = sys.create_bitwise_or(_beccd, _bece5);
    // _bee3d = _beccd & _bece5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:133
    let _bee3d = sys.create_bitwise_and(_beccd, _bece5);
    // _bee45 = _bece5[(0:u1):(4:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:135
    let imm_bee51 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_bee59 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x4 as u64); // (4:u3)
    let _bee45 = sys.create_slice(_bece5, imm_bee51, imm_bee59);
    // _bee11 = _beccd << _bee45, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:135
    let _bee11 = sys.create_shl(_beccd, _bee45);
    // _bee65 = _bece5[(0:u1):(4:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:137
    let imm_bee6d = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_bee75 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x4 as u64); // (4:u3)
    let _bee65 = sys.create_slice(_bece5, imm_bee6d, imm_bee75);
    // _bed65 = _beccd >> _bee65, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:137
    let _bed65 = sys.create_shr(_beccd, _bee65);
    // _bee71 = _bcc79[(22:u5):(37:u6)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:140
    let imm_bee8d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x16 as u64); // (22:u5)
    let imm_bee95 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x25 as u64); // (37:u6)
    let _bee71 = sys.create_slice(_bcc79, imm_bee8d, imm_bee95);
    // _bee9d = select_1hot _bee71 (_bed75, _bee29, _bee31, _bee35, _bee3d, _bee11, (0:b32), _bee0d, _bedb9, _bed9d, _bedd5, (1:b32), _bed65, _bee39, (0:b32), (0:b32)), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:141
    let imm_bed49 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let imm_bee2d = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x1 as u64); // (1:b32)
    let imm_bed49 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let imm_bed49 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let _bee9d = sys.create_select_1hot(
        _bee71,
        vec![
            _bed75, _bee29, _bee31, _bee35, _bee3d, _bee11, imm_bed49, _bee0d, _bedb9, _bed9d,
            _bedd5, imm_bee2d, _bed65, _bee39, imm_bed49, imm_bed49,
        ],
    );
    // _beea1 = _bcc79[(3:u2):(3:u2)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:143
    let imm_beeb1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(2), 0x3 as u64); // (3:u2)
    let imm_beeb9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(2), 0x3 as u64); // (3:u2)
    let _beea1 = sys.create_slice(_bcc79, imm_beeb1, imm_beeb9);
    // _beec1 = _bcc79[(73:u7):(73:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:143
    let imm_beec9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x49 as u64); // (73:u7)
    let imm_beed1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x49 as u64); // (73:u7)
    let _beec1 = sys.create_slice(_bcc79, imm_beec9, imm_beed1);
    // log('pc: 0x{:08x}   |is_offset_br: {}| is_pc_calc: {}|', _bcc85, _beea1, _beec1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:143
    let fmt = sys.get_str_literal("pc: 0x{:08x}   |is_offset_br: {}| is_pc_calc: {}|".into());
    sys.create_log(fmt, vec![_bcc85, _beea1, _beec1]);
    // _beddd = _bcc79[(41:u6):(72:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:144
    let imm_beee5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x29 as u64); // (41:u6)
    let imm_beeed = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x48 as u64); // (72:u7)
    let _beddd = sys.create_slice(_bcc79, imm_beee5, imm_beeed);
    // log('0x{:08x}       | a: {:08x}  | b: {:08x}   | imm: {:08x} | result: {:08x}', _bee71, _beccd, _becdd, _beddd, _bee9d), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:144
    let fmt = sys.get_str_literal(
        "0x{:08x}       | a: {:08x}  | b: {:08x}   | imm: {:08x} | result: {:08x}".into(),
    );
    sys.create_log(fmt, vec![_bee71, _beccd, _becdd, _beddd, _bee9d]);
    // log('0x{:08x}       |a.a:{:08x}  |a.b:{:08x}   | res: {:08x} |', _bee71, _bed0d, _bece5, _bee9d), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:145
    let fmt =
        sys.get_str_literal("0x{:08x}       |a.a:{:08x}  |a.b:{:08x}   | res: {:08x} |".into());
    sys.create_log(fmt, vec![_bee71, _bed0d, _bece5, _bee9d]);
    // _bee19 = _bcc79[(6:u3):(21:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:147
    let imm_bef05 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let imm_bef0d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x15 as u64); // (21:u5)
    let _bee19 = sys.create_slice(_bcc79, imm_bef05, imm_bef0d);
    // _bef15 = select_1hot _bee19 (_bed75, _bee29, _bee31, _bee35, _bee3d, _bee11, (0:b32), _bee0d, _bedb9, _bed9d, _bedd5, (1:b32), _bed65, _bee39, (0:b32), (0:b32)), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:147
    let imm_bed49 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let imm_bee2d = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x1 as u64); // (1:b32)
    let imm_bed49 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let imm_bed49 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let _bef15 = sys.create_select_1hot(
        _bee19,
        vec![
            _bed75, _bee29, _bee31, _bee35, _bee3d, _bee11, imm_bed49, _bee0d, _bedb9, _bed9d,
            _bedd5, imm_bee2d, _bed65, _bee39, imm_bed49, imm_bed49,
        ],
    );
    // _bef09 = _bcc79[(5:u3):(5:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:148
    let imm_bef25 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x5 as u64); // (5:u3)
    let imm_bef2d = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x5 as u64); // (5:u3)
    let _bef09 = sys.create_slice(_bcc79, imm_bef25, imm_bef2d);
    // _bee41 = ~_bef15, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:148
    let _bee41 = sys.create_flip(_bef15);
    // _bef29 = _bef09 ? _bee41 : _bef15, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:148
    let _bef29 = sys.create_select(_bef09, _bee41, _bef15);
    // _bef39 = _bcc79[(38:u6):(39:u6)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:150
    let imm_bef49 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x26 as u64); // (38:u6)
    let imm_bef51 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x27 as u64); // (39:u6)
    let _bef39 = sys.create_slice(_bcc79, imm_bef49, imm_bef51);
    // _bee61 = _bef39[(0:u1):(0:u1)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:150
    let imm_bef5d = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_bef65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let _bee61 = sys.create_slice(_bef39, imm_bef5d, imm_bef65);
    // _bef61 = _bcc79[(38:u6):(39:u6)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:151
    let imm_bef79 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x26 as u64); // (38:u6)
    let imm_bef81 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x27 as u64); // (39:u6)
    let _bef61 = sys.create_slice(_bcc79, imm_bef79, imm_bef81);
    // _bee81 = _bef61[(1:u1):(1:u1)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:151
    let imm_bef8d = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x1 as u64); // (1:u1)
    let imm_bef95 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x1 as u64); // (1:u1)
    let _bee81 = sys.create_slice(_bef61, imm_bef8d, imm_bef95);
    // _bee01 = ~_bee61, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:154
    let _bee01 = sys.create_flip(_bee61);
    // _befa1 = _be701 != (0:b5), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:154
    let imm_bef91 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x0 as u64); // (0:b5)
    let _befa1 = sys.create_neq(_be701, imm_bef91);
    // _befa9 = _bee01 & _befa1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:154
    let _befa9 = sys.create_bitwise_and(_bee01, _befa1);
    // _befb1 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:158
    let _befb1 = sys.create_array_read(array_ba589, _be701);
    // _befb9 = array_ba53d[_befb1], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:158
    let _befb9 = sys.create_array_read(array_ba53d, _befb1);
    // _befb5 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_befc1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _befb5 = sys.create_array_read(array_ba53d, imm_befc1);
    // _befc5 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_befcd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _befc5 = sys.create_array_read(array_ba53d, imm_befcd);
    // _befd1 = _befc5[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_befe1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_befe9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _befd1 = sys.create_slice(_befc5, imm_befe1, imm_befe9);
    // _befe5 = _befd1 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_beff1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _befe5 = sys.create_eq(_befd1, imm_beff1);
    // _beff5 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_beffd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _beff5 = sys.create_array_read(array_ba53d, imm_beffd);
    // _bf805 = _beff5[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_bf815 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_bf81d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _bf805 = sys.create_slice(_beff5, imm_bf815, imm_bf81d);
    // _bee21 = ~_bf805, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _bee21 = sys.create_flip(_bf805);
    // _bf825 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_bf829 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _bf825 = sys.create_array_read(array_ba53d, imm_bf829);
    // _bf82d = _bf825[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_bf83d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_bf845 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _bf82d = sys.create_slice(_bf825, imm_bf83d, imm_bf845);
    // _bed79 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _bed79 = sys.create_array_read(array_ba589, _be701);
    // _bf841 = _bf82d == _bed79, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _bf841 = sys.create_eq(_bf82d, _bed79);
    // _bf859 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:162
    let imm_bf85d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _bf859 = sys.create_array_read(array_ba53d, imm_bf85d);
    // _bf861 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:163
    let imm_bf869 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _bf861 = sys.create_array_read(array_ba53d, imm_bf869);
    // _bf86d = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_bf875 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _bf86d = sys.create_array_read(array_ba53d, imm_bf875);
    // _bf879 = _bf86d[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_bf889 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_bf891 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _bf879 = sys.create_slice(_bf86d, imm_bf889, imm_bf891);
    // _bf809 = ~_bf879, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _bf809 = sys.create_flip(_bf879);
    // _bf899 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_bf89d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _bf899 = sys.create_array_read(array_ba53d, imm_bf89d);
    // _bf8a1 = _bf899[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_bf8b1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_bf8b9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _bf8a1 = sys.create_slice(_bf899, imm_bf8b1, imm_bf8b9);
    // _bf831 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _bf831 = sys.create_array_read(array_ba589, _be701);
    // _bf8b5 = _bf8a1 == _bf831, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _bf8b5 = sys.create_eq(_bf8a1, _bf831);
    // _bf8cd = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:165
    let imm_bf8d1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _bf8cd = sys.create_array_read(array_ba53d, imm_bf8d1);
    // _bf8d5 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:166
    let imm_bf8dd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _bf8d5 = sys.create_array_read(array_ba53d, imm_bf8dd);
    // _bf8e1 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_bf8e9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _bf8e1 = sys.create_array_read(array_ba53d, imm_bf8e9);
    // _bf8ed = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_bf8f5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _bf8ed = sys.create_array_read(array_ba53d, imm_bf8f5);
    // _bf8f9 = _bf8ed[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_bf909 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_bf911 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _bf8f9 = sys.create_slice(_bf8ed, imm_bf909, imm_bf911);
    // _bf90d = _bf8f9 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_beee9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _bf90d = sys.create_eq(_bf8f9, imm_beee9);
    // _bf91d = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_bf925 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _bf91d = sys.create_array_read(array_ba53d, imm_bf925);
    // _bf929 = _bf91d[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_bf939 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_bf941 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _bf929 = sys.create_slice(_bf91d, imm_bf939, imm_bf941);
    // _bec05 = ~_bf929, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _bec05 = sys.create_flip(_bf929);
    // _bf949 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_bf94d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _bf949 = sys.create_array_read(array_ba53d, imm_bf94d);
    // _bf951 = _bf949[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_bf961 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_bf969 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _bf951 = sys.create_slice(_bf949, imm_bf961, imm_bf969);
    // _bef6d = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _bef6d = sys.create_array_read(array_ba589, _be701);
    // _bf965 = _bf951 == _bef6d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _bf965 = sys.create_eq(_bf951, _bef6d);
    // _bf97d = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:162
    let imm_bf981 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _bf97d = sys.create_array_read(array_ba53d, imm_bf981);
    // _bf985 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:163
    let imm_bf98d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _bf985 = sys.create_array_read(array_ba53d, imm_bf98d);
    // _bf991 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_bf999 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _bf991 = sys.create_array_read(array_ba53d, imm_bf999);
    // _bf99d = _bf991[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_bf9ad = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_bf9b5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _bf99d = sys.create_slice(_bf991, imm_bf9ad, imm_bf9b5);
    // _bf919 = ~_bf99d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _bf919 = sys.create_flip(_bf99d);
    // _bf9bd = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_bf9c1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _bf9bd = sys.create_array_read(array_ba53d, imm_bf9c1);
    // _bf9c5 = _bf9bd[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_bf9d5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_bf9dd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _bf9c5 = sys.create_slice(_bf9bd, imm_bf9d5, imm_bf9dd);
    // _bf955 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _bf955 = sys.create_array_read(array_ba589, _be701);
    // _bf9d9 = _bf9c5 == _bf955, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _bf9d9 = sys.create_eq(_bf9c5, _bf955);
    // _bf9f1 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:165
    let imm_bf9f5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _bf9f1 = sys.create_array_read(array_ba53d, imm_bf9f5);
    // _bf9f9 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:166
    let imm_bfa01 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _bf9f9 = sys.create_array_read(array_ba53d, imm_bfa01);
    // _bfa05 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_bfa0d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _bfa05 = sys.create_array_read(array_ba53d, imm_bfa0d);
    // _bfa11 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_bfa19 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _bfa11 = sys.create_array_read(array_ba53d, imm_bfa19);
    // _bfa1d = _bfa11[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_bfa2d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_bfa35 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _bfa1d = sys.create_slice(_bfa11, imm_bfa2d, imm_bfa35);
    // _bfa31 = _bfa1d == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_bfa3d = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _bfa31 = sys.create_eq(_bfa1d, imm_bfa3d);
    // _bfa41 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_bfa49 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _bfa41 = sys.create_array_read(array_ba53d, imm_bfa49);
    // _bfa4d = _bfa41[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_bfa5d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_bfa65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _bfa4d = sys.create_slice(_bfa41, imm_bfa5d, imm_bfa65);
    // _befad = ~_bfa4d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _befad = sys.create_flip(_bfa4d);
    // _bfa6d = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_bfa71 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _bfa6d = sys.create_array_read(array_ba53d, imm_bfa71);
    // _bfa75 = _bfa6d[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_bfa85 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_bfa8d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _bfa75 = sys.create_slice(_bfa6d, imm_bfa85, imm_bfa8d);
    // _bf8a5 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _bf8a5 = sys.create_array_read(array_ba589, _be701);
    // _bfa89 = _bfa75 == _bf8a5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _bfa89 = sys.create_eq(_bfa75, _bf8a5);
    // _bfaa1 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:162
    let imm_bfaa5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _bfaa1 = sys.create_array_read(array_ba53d, imm_bfaa5);
    // _bfaa9 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:163
    let imm_bfab1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _bfaa9 = sys.create_array_read(array_ba53d, imm_bfab1);
    // _bfab5 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_bfabd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _bfab5 = sys.create_array_read(array_ba53d, imm_bfabd);
    // _bfac1 = _bfab5[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_bfad1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_bfad9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _bfac1 = sys.create_slice(_bfab5, imm_bfad1, imm_bfad9);
    // _bf87d = ~_bfac1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _bf87d = sys.create_flip(_bfac1);
    // _bfae1 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_bfae5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _bfae1 = sys.create_array_read(array_ba53d, imm_bfae5);
    // _bfae9 = _bfae1[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_bfaf9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_bfb01 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _bfae9 = sys.create_slice(_bfae1, imm_bfaf9, imm_bfb01);
    // _bfa79 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _bfa79 = sys.create_array_read(array_ba589, _be701);
    // _bfafd = _bfae9 == _bfa79, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _bfafd = sys.create_eq(_bfae9, _bfa79);
    // _bfb15 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:165
    let imm_bfb19 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _bfb15 = sys.create_array_read(array_ba53d, imm_bfb19);
    // _bfb1d = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:166
    let imm_bfb25 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _bfb1d = sys.create_array_read(array_ba53d, imm_bfb25);
    // _bfb29 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_bfb31 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _bfb29 = sys.create_array_read(array_ba53d, imm_bfb31);
    // _bfb35 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_bfb3d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _bfb35 = sys.create_array_read(array_ba53d, imm_bfb3d);
    // _bfb41 = _bfb35[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_bfb51 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_bfb59 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _bfb41 = sys.create_slice(_bfb35, imm_bfb51, imm_bfb59);
    // _bfb55 = _bfb41 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_bfb61 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _bfb55 = sys.create_eq(_bfb41, imm_bfb61);
    // _bfb65 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_bfb6d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _bfb65 = sys.create_array_read(array_ba53d, imm_bfb6d);
    // _bfb71 = _bfb65[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_bfb81 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_bfb89 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _bfb71 = sys.create_slice(_bfb65, imm_bfb81, imm_bfb89);
    // _bf8c5 = ~_bfb71, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _bf8c5 = sys.create_flip(_bfb71);
    // _bfb91 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_bfb95 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _bfb91 = sys.create_array_read(array_ba53d, imm_bfb95);
    // _bfb99 = _bfb91[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_bfba9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_bfbb1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _bfb99 = sys.create_slice(_bfb91, imm_bfba9, imm_bfbb1);
    // _bf9c9 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _bf9c9 = sys.create_array_read(array_ba589, _be701);
    // _bfbad = _bfb99 == _bf9c9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _bfbad = sys.create_eq(_bfb99, _bf9c9);
    // _bfbc5 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:162
    let imm_bfbc9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _bfbc5 = sys.create_array_read(array_ba53d, imm_bfbc9);
    // _bfbcd = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:163
    let imm_bfbd5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _bfbcd = sys.create_array_read(array_ba53d, imm_bfbd5);
    // _bfbd9 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_bfbe1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _bfbd9 = sys.create_array_read(array_ba53d, imm_bfbe1);
    // _bfbe5 = _bfbd9[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_bfbf5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_bfbfd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _bfbe5 = sys.create_slice(_bfbd9, imm_bfbf5, imm_bfbfd);
    // _bf9a1 = ~_bfbe5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _bf9a1 = sys.create_flip(_bfbe5);
    // _c0409 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c040d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c0409 = sys.create_array_read(array_ba53d, imm_c040d);
    // _c0411 = _c0409[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c0421 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c0429 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c0411 = sys.create_slice(_c0409, imm_c0421, imm_c0429);
    // _56835 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _56835 = sys.create_array_read(array_ba589, _be701);
    // _c0425 = _c0411 == _56835, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c0425 = sys.create_eq(_c0411, _56835);
    // _c043d = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:165
    let imm_c0441 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c043d = sys.create_array_read(array_ba53d, imm_c0441);
    // _c0445 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:166
    let imm_c044d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c0445 = sys.create_array_read(array_ba53d, imm_c044d);
    // _c0451 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c0459 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c0451 = sys.create_array_read(array_ba53d, imm_c0459);
    // _c045d = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c0465 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c045d = sys.create_array_read(array_ba53d, imm_c0465);
    // _c0469 = _c045d[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c0479 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c0481 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c0469 = sys.create_slice(_c045d, imm_c0479, imm_c0481);
    // _c047d = _c0469 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_bfac5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c047d = sys.create_eq(_c0469, imm_bfac5);
    // _c048d = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c0495 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c048d = sys.create_array_read(array_ba53d, imm_c0495);
    // _c0499 = _c048d[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c04a9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c04b1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c0499 = sys.create_slice(_c048d, imm_c04a9, imm_c04b1);
    // _bf9e9 = ~_c0499, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _bf9e9 = sys.create_flip(_c0499);
    // _c04b9 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c04bd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c04b9 = sys.create_array_read(array_ba53d, imm_c04bd);
    // _c04c1 = _c04b9[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c04d1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c04d9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c04c1 = sys.create_slice(_c04b9, imm_c04d1, imm_c04d9);
    // _bfaed = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _bfaed = sys.create_array_read(array_ba589, _be701);
    // _c04d5 = _c04c1 == _bfaed, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c04d5 = sys.create_eq(_c04c1, _bfaed);
    // _c04ed = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:162
    let imm_c04f1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c04ed = sys.create_array_read(array_ba53d, imm_c04f1);
    // _c04f5 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:163
    let imm_c04fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c04f5 = sys.create_array_read(array_ba53d, imm_c04fd);
    // _c0501 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c0509 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c0501 = sys.create_array_read(array_ba53d, imm_c0509);
    // _c050d = _c0501[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c051d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c0525 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c050d = sys.create_slice(_c0501, imm_c051d, imm_c0525);
    // _c0489 = ~_c050d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c0489 = sys.create_flip(_c050d);
    // _c052d = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c0531 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c052d = sys.create_array_read(array_ba53d, imm_c0531);
    // _c0535 = _c052d[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c0545 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c054d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c0535 = sys.create_slice(_c052d, imm_c0545, imm_c054d);
    // _c04c5 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c04c5 = sys.create_array_read(array_ba589, _be701);
    // _c0549 = _c0535 == _c04c5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c0549 = sys.create_eq(_c0535, _c04c5);
    // _c0561 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:165
    let imm_c0565 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c0561 = sys.create_array_read(array_ba53d, imm_c0565);
    // _c0569 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:166
    let imm_c0571 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c0569 = sys.create_array_read(array_ba53d, imm_c0571);
    // _c0575 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c057d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c0575 = sys.create_array_read(array_ba53d, imm_c057d);
    // _c0581 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c0589 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c0581 = sys.create_array_read(array_ba53d, imm_c0589);
    // _c058d = _c0581[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c059d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c05a5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c058d = sys.create_slice(_c0581, imm_c059d, imm_c05a5);
    // _c05a1 = _c058d == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_bfbe9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c05a1 = sys.create_eq(_c058d, imm_bfbe9);
    // _c05b1 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c05b9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c05b1 = sys.create_array_read(array_ba53d, imm_c05b9);
    // _c05bd = _c05b1[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c05cd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c05d5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c05bd = sys.create_slice(_c05b1, imm_c05cd, imm_c05d5);
    // _bfb0d = ~_c05bd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _bfb0d = sys.create_flip(_c05bd);
    // _c05dd = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c05e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c05dd = sys.create_array_read(array_ba53d, imm_c05e1);
    // _c05e5 = _c05dd[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c05f5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c05fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c05e5 = sys.create_slice(_c05dd, imm_c05f5, imm_c05fd);
    // _c0415 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c0415 = sys.create_array_read(array_ba589, _be701);
    // _c05f9 = _c05e5 == _c0415, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c05f9 = sys.create_eq(_c05e5, _c0415);
    // _c0611 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:162
    let imm_c0615 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c0611 = sys.create_array_read(array_ba53d, imm_c0615);
    // _c0619 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:163
    let imm_c0621 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c0619 = sys.create_array_read(array_ba53d, imm_c0621);
    // _c0625 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c062d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c0625 = sys.create_array_read(array_ba53d, imm_c062d);
    // _c0631 = _c0625[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c0641 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c0649 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c0631 = sys.create_slice(_c0625, imm_c0641, imm_c0649);
    // _c05ad = ~_c0631, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c05ad = sys.create_flip(_c0631);
    // _c0651 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c0655 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c0651 = sys.create_array_read(array_ba53d, imm_c0655);
    // _c0659 = _c0651[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c0669 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c0671 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c0659 = sys.create_slice(_c0651, imm_c0669, imm_c0671);
    // _c05e9 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c05e9 = sys.create_array_read(array_ba589, _be701);
    // _c066d = _c0659 == _c05e9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c066d = sys.create_eq(_c0659, _c05e9);
    // _c0685 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:165
    let imm_c0689 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c0685 = sys.create_array_read(array_ba53d, imm_c0689);
    // _c068d = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:166
    let imm_c0695 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c068d = sys.create_array_read(array_ba53d, imm_c0695);
    // _c0699 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c06a1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c0699 = sys.create_array_read(array_ba53d, imm_c06a1);
    // _c06a5 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c06ad = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c06a5 = sys.create_array_read(array_ba53d, imm_c06ad);
    // _c06b1 = _c06a5[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c06c1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c06c9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c06b1 = sys.create_slice(_c06a5, imm_c06c1, imm_c06c9);
    // _c06c5 = _c06b1 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c06d1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c06c5 = sys.create_eq(_c06b1, imm_c06d1);
    // _c06d5 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c06dd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c06d5 = sys.create_array_read(array_ba53d, imm_c06dd);
    // _c06e1 = _c06d5[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c06f1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c06f9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c06e1 = sys.create_slice(_c06d5, imm_c06f1, imm_c06f9);
    // _c0435 = ~_c06e1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c0435 = sys.create_flip(_c06e1);
    // _c0701 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c0705 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c0701 = sys.create_array_read(array_ba53d, imm_c0705);
    // _c0709 = _c0701[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c0719 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c0721 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c0709 = sys.create_slice(_c0701, imm_c0719, imm_c0721);
    // _c0539 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c0539 = sys.create_array_read(array_ba589, _be701);
    // _c071d = _c0709 == _c0539, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c071d = sys.create_eq(_c0709, _c0539);
    // _c0735 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:162
    let imm_c0739 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c0735 = sys.create_array_read(array_ba53d, imm_c0739);
    // _c073d = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:163
    let imm_c0745 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c073d = sys.create_array_read(array_ba53d, imm_c0745);
    // _c0749 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c0751 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c0749 = sys.create_array_read(array_ba53d, imm_c0751);
    // _c0755 = _c0749[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c0765 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c076d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c0755 = sys.create_slice(_c0749, imm_c0765, imm_c076d);
    // _c0511 = ~_c0755, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c0511 = sys.create_flip(_c0755);
    // _c0775 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c0779 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c0775 = sys.create_array_read(array_ba53d, imm_c0779);
    // _c077d = _c0775[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c078d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c0795 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c077d = sys.create_slice(_c0775, imm_c078d, imm_c0795);
    // _c070d = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c070d = sys.create_array_read(array_ba589, _be701);
    // _c0791 = _c077d == _c070d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c0791 = sys.create_eq(_c077d, _c070d);
    // _c07a9 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:165
    let imm_c07ad = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c07a9 = sys.create_array_read(array_ba53d, imm_c07ad);
    // _c07b1 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:166
    let imm_c07b9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c07b1 = sys.create_array_read(array_ba53d, imm_c07b9);
    // _c07bd = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c07c5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c07bd = sys.create_array_read(array_ba53d, imm_c07c5);
    // _c07c9 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c07d1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c07c9 = sys.create_array_read(array_ba53d, imm_c07d1);
    // _c07d5 = _c07c9[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c07e5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c07ed = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c07d5 = sys.create_slice(_c07c9, imm_c07e5, imm_c07ed);
    // _c07e9 = _c07d5 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c07f5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c07e9 = sys.create_eq(_c07d5, imm_c07f5);
    // _c0805 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c07f9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c0805 = sys.create_array_read(array_ba53d, imm_c07f9);
    // _c0809 = _c0805[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c0819 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c0821 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c0809 = sys.create_slice(_c0805, imm_c0819, imm_c0821);
    // _c0559 = ~_c0809, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c0559 = sys.create_flip(_c0809);
    // _c0829 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c082d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c0829 = sys.create_array_read(array_ba53d, imm_c082d);
    // _c0831 = _c0829[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c0841 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c0849 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c0831 = sys.create_slice(_c0829, imm_c0841, imm_c0849);
    // _c065d = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c065d = sys.create_array_read(array_ba589, _be701);
    // _c0845 = _c0831 == _c065d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c0845 = sys.create_eq(_c0831, _c065d);
    // _c085d = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:162
    let imm_c0861 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c085d = sys.create_array_read(array_ba53d, imm_c0861);
    // _c0865 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:163
    let imm_c086d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c0865 = sys.create_array_read(array_ba53d, imm_c086d);
    // _c0871 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c0879 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c0871 = sys.create_array_read(array_ba53d, imm_c0879);
    // _c087d = _c0871[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c088d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c0895 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c087d = sys.create_slice(_c0871, imm_c088d, imm_c0895);
    // _c080d = ~_c087d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c080d = sys.create_flip(_c087d);
    // _c089d = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c08a1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c089d = sys.create_array_read(array_ba53d, imm_c08a1);
    // _c08a5 = _c089d[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c08b5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c08bd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c08a5 = sys.create_slice(_c089d, imm_c08b5, imm_c08bd);
    // _c0835 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c0835 = sys.create_array_read(array_ba589, _be701);
    // _c08b9 = _c08a5 == _c0835, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c08b9 = sys.create_eq(_c08a5, _c0835);
    // _c08d1 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:165
    let imm_c08d5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c08d1 = sys.create_array_read(array_ba53d, imm_c08d5);
    // _c08d9 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:166
    let imm_c08e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c08d9 = sys.create_array_read(array_ba53d, imm_c08e1);
    // _c08e5 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c08ed = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c08e5 = sys.create_array_read(array_ba53d, imm_c08ed);
    // _c08f1 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c08f9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c08f1 = sys.create_array_read(array_ba53d, imm_c08f9);
    // _c08fd = _c08f1[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c090d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c0915 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c08fd = sys.create_slice(_c08f1, imm_c090d, imm_c0915);
    // _c0911 = _c08fd == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c0759 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c0911 = sys.create_eq(_c08fd, imm_c0759);
    // _c0921 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c0929 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c0921 = sys.create_array_read(array_ba53d, imm_c0929);
    // _c092d = _c0921[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c093d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c0945 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c092d = sys.create_slice(_c0921, imm_c093d, imm_c0945);
    // _c067d = ~_c092d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c067d = sys.create_flip(_c092d);
    // _c094d = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c0951 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c094d = sys.create_array_read(array_ba53d, imm_c0951);
    // _c0955 = _c094d[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c0965 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c096d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c0955 = sys.create_slice(_c094d, imm_c0965, imm_c096d);
    // _c0781 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c0781 = sys.create_array_read(array_ba589, _be701);
    // _c0969 = _c0955 == _c0781, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c0969 = sys.create_eq(_c0955, _c0781);
    // _c0981 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:162
    let imm_c0985 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c0981 = sys.create_array_read(array_ba53d, imm_c0985);
    // _c0989 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:163
    let imm_c0991 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c0989 = sys.create_array_read(array_ba53d, imm_c0991);
    // _c0995 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c099d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c0995 = sys.create_array_read(array_ba53d, imm_c099d);
    // _c09a1 = _c0995[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c09b1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c09b9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c09a1 = sys.create_slice(_c0995, imm_c09b1, imm_c09b9);
    // _c091d = ~_c09a1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c091d = sys.create_flip(_c09a1);
    // _c09c1 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c09c5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c09c1 = sys.create_array_read(array_ba53d, imm_c09c5);
    // _c09c9 = _c09c1[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c09d9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c09e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c09c9 = sys.create_slice(_c09c1, imm_c09d9, imm_c09e1);
    // _c0959 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c0959 = sys.create_array_read(array_ba589, _be701);
    // _c09dd = _c09c9 == _c0959, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c09dd = sys.create_eq(_c09c9, _c0959);
    // _c09f5 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:165
    let imm_c09f9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c09f5 = sys.create_array_read(array_ba53d, imm_c09f9);
    // _c09fd = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:166
    let imm_c0a05 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c09fd = sys.create_array_read(array_ba53d, imm_c0a05);
    // _c0a09 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c0a11 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c0a09 = sys.create_array_read(array_ba53d, imm_c0a11);
    // _c0a15 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c0a1d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c0a15 = sys.create_array_read(array_ba53d, imm_c0a1d);
    // _c0a21 = _c0a15[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c0a31 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c0a39 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c0a21 = sys.create_slice(_c0a15, imm_c0a31, imm_c0a39);
    // _c0a35 = _c0a21 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c0a41 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c0a35 = sys.create_eq(_c0a21, imm_c0a41);
    // _c0a45 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c0a4d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c0a45 = sys.create_array_read(array_ba53d, imm_c0a4d);
    // _c0a51 = _c0a45[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c0a61 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c0a69 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c0a51 = sys.create_slice(_c0a45, imm_c0a61, imm_c0a69);
    // _c07a1 = ~_c0a51, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c07a1 = sys.create_flip(_c0a51);
    // _c0a71 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c0a75 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c0a71 = sys.create_array_read(array_ba53d, imm_c0a75);
    // _c0a79 = _c0a71[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c0a89 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c0a91 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c0a79 = sys.create_slice(_c0a71, imm_c0a89, imm_c0a91);
    // _c08a9 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c08a9 = sys.create_array_read(array_ba589, _be701);
    // _c0a8d = _c0a79 == _c08a9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c0a8d = sys.create_eq(_c0a79, _c08a9);
    // _c0aa5 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:162
    let imm_c0aa9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c0aa5 = sys.create_array_read(array_ba53d, imm_c0aa9);
    // _c0aad = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:163
    let imm_c0ab5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c0aad = sys.create_array_read(array_ba53d, imm_c0ab5);
    // _c0ab9 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c0ac1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c0ab9 = sys.create_array_read(array_ba53d, imm_c0ac1);
    // _c0ac5 = _c0ab9[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c0ad5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c0add = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c0ac5 = sys.create_slice(_c0ab9, imm_c0ad5, imm_c0add);
    // _c0881 = ~_c0ac5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c0881 = sys.create_flip(_c0ac5);
    // _c0ae5 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c0ae9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c0ae5 = sys.create_array_read(array_ba53d, imm_c0ae9);
    // _c0aed = _c0ae5[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c0afd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c0b05 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c0aed = sys.create_slice(_c0ae5, imm_c0afd, imm_c0b05);
    // _c0a7d = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c0a7d = sys.create_array_read(array_ba589, _be701);
    // _c0b01 = _c0aed == _c0a7d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c0b01 = sys.create_eq(_c0aed, _c0a7d);
    // _c0b19 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:165
    let imm_c0b1d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c0b19 = sys.create_array_read(array_ba53d, imm_c0b1d);
    // _c0b21 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:166
    let imm_c0b29 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c0b21 = sys.create_array_read(array_ba53d, imm_c0b29);
    // _c0b2d = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c0b35 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c0b2d = sys.create_array_read(array_ba53d, imm_c0b35);
    // _c0b39 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c0b41 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c0b39 = sys.create_array_read(array_ba53d, imm_c0b41);
    // _c0b45 = _c0b39[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c0b55 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c0b5d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c0b45 = sys.create_slice(_c0b39, imm_c0b55, imm_c0b5d);
    // _c0b59 = _c0b45 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c0b65 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c0b59 = sys.create_eq(_c0b45, imm_c0b65);
    // _c0b69 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c0b71 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c0b69 = sys.create_array_read(array_ba53d, imm_c0b71);
    // _c0b75 = _c0b69[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c0b85 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c0b8d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c0b75 = sys.create_slice(_c0b69, imm_c0b85, imm_c0b8d);
    // _c08c9 = ~_c0b75, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c08c9 = sys.create_flip(_c0b75);
    // _c0b95 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c0b99 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c0b95 = sys.create_array_read(array_ba53d, imm_c0b99);
    // _c0b9d = _c0b95[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c0bad = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c0bb5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c0b9d = sys.create_slice(_c0b95, imm_c0bad, imm_c0bb5);
    // _c09cd = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c09cd = sys.create_array_read(array_ba589, _be701);
    // _c0bb1 = _c0b9d == _c09cd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c0bb1 = sys.create_eq(_c0b9d, _c09cd);
    // _c0bc9 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:162
    let imm_c0bcd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c0bc9 = sys.create_array_read(array_ba53d, imm_c0bcd);
    // _c0bd1 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:163
    let imm_c0bd9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c0bd1 = sys.create_array_read(array_ba53d, imm_c0bd9);
    // _c0bdd = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c0be5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c0bdd = sys.create_array_read(array_ba53d, imm_c0be5);
    // _c0be9 = _c0bdd[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c0bf9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c0bfd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c0be9 = sys.create_slice(_c0bdd, imm_c0bf9, imm_c0bfd);
    // _c09a5 = ~_c0be9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c09a5 = sys.create_flip(_c0be9);
    // _c1805 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c1815 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c1805 = sys.create_array_read(array_ba53d, imm_c1815);
    // _c1819 = _c1805[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c1829 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c1831 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c1819 = sys.create_slice(_c1805, imm_c1829, imm_c1831);
    // _bfb9d = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _bfb9d = sys.create_array_read(array_ba589, _be701);
    // _c182d = _c1819 == _bfb9d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c182d = sys.create_eq(_c1819, _bfb9d);
    // _c1845 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:165
    let imm_c1849 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c1845 = sys.create_array_read(array_ba53d, imm_c1849);
    // _c184d = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:166
    let imm_c1855 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c184d = sys.create_array_read(array_ba53d, imm_c1855);
    // _c1859 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c1861 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c1859 = sys.create_array_read(array_ba53d, imm_c1861);
    // _c1865 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c186d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c1865 = sys.create_array_read(array_ba53d, imm_c186d);
    // _c1871 = _c1865[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c1881 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c1889 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c1871 = sys.create_slice(_c1865, imm_c1881, imm_c1889);
    // _c1885 = _c1871 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c0ac9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c1885 = sys.create_eq(_c1871, imm_c0ac9);
    // _c1895 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c189d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c1895 = sys.create_array_read(array_ba53d, imm_c189d);
    // _c18a1 = _c1895[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c18b1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c18b9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c18a1 = sys.create_slice(_c1895, imm_c18b1, imm_c18b9);
    // _c09ed = ~_c18a1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c09ed = sys.create_flip(_c18a1);
    // _c18c1 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c18c5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c18c1 = sys.create_array_read(array_ba53d, imm_c18c5);
    // _c18c9 = _c18c1[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c18d9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c18e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c18c9 = sys.create_slice(_c18c1, imm_c18d9, imm_c18e1);
    // _c0af1 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c0af1 = sys.create_array_read(array_ba589, _be701);
    // _c18dd = _c18c9 == _c0af1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c18dd = sys.create_eq(_c18c9, _c0af1);
    // _c18f5 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:162
    let imm_c18f9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c18f5 = sys.create_array_read(array_ba53d, imm_c18f9);
    // _c18fd = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:163
    let imm_c1905 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c18fd = sys.create_array_read(array_ba53d, imm_c1905);
    // _c1909 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c1911 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c1909 = sys.create_array_read(array_ba53d, imm_c1911);
    // _c1915 = _c1909[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c1925 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c192d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c1915 = sys.create_slice(_c1909, imm_c1925, imm_c192d);
    // _c1891 = ~_c1915, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c1891 = sys.create_flip(_c1915);
    // _c1935 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c1939 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c1935 = sys.create_array_read(array_ba53d, imm_c1939);
    // _c193d = _c1935[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c194d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c1955 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c193d = sys.create_slice(_c1935, imm_c194d, imm_c1955);
    // _c18cd = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c18cd = sys.create_array_read(array_ba589, _be701);
    // _c1951 = _c193d == _c18cd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c1951 = sys.create_eq(_c193d, _c18cd);
    // _c1969 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:165
    let imm_c196d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c1969 = sys.create_array_read(array_ba53d, imm_c196d);
    // _c1971 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:166
    let imm_c1979 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c1971 = sys.create_array_read(array_ba53d, imm_c1979);
    // _c197d = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c1985 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c197d = sys.create_array_read(array_ba53d, imm_c1985);
    // _c1989 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c1991 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c1989 = sys.create_array_read(array_ba53d, imm_c1991);
    // _c1995 = _c1989[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c19a5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c19ad = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c1995 = sys.create_slice(_c1989, imm_c19a5, imm_c19ad);
    // _c19a9 = _c1995 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c0bed = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c19a9 = sys.create_eq(_c1995, imm_c0bed);
    // _c19b9 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c19c1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c19b9 = sys.create_array_read(array_ba53d, imm_c19c1);
    // _c19c5 = _c19b9[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c19d5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c19dd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c19c5 = sys.create_slice(_c19b9, imm_c19d5, imm_c19dd);
    // _c0b11 = ~_c19c5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c0b11 = sys.create_flip(_c19c5);
    // _c19e5 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c19e9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c19e5 = sys.create_array_read(array_ba53d, imm_c19e9);
    // _c19ed = _c19e5[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c19fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c1a05 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c19ed = sys.create_slice(_c19e5, imm_c19fd, imm_c1a05);
    // _c181d = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c181d = sys.create_array_read(array_ba589, _be701);
    // _c1a01 = _c19ed == _c181d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c1a01 = sys.create_eq(_c19ed, _c181d);
    // _c1a19 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:162
    let imm_c1a1d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c1a19 = sys.create_array_read(array_ba53d, imm_c1a1d);
    // _c1a21 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:163
    let imm_c1a29 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c1a21 = sys.create_array_read(array_ba53d, imm_c1a29);
    // _c1a2d = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c1a35 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c1a2d = sys.create_array_read(array_ba53d, imm_c1a35);
    // _c1a39 = _c1a2d[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c1a49 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c1a51 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c1a39 = sys.create_slice(_c1a2d, imm_c1a49, imm_c1a51);
    // _c19b5 = ~_c1a39, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c19b5 = sys.create_flip(_c1a39);
    // _c1a59 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c1a5d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c1a59 = sys.create_array_read(array_ba53d, imm_c1a5d);
    // _c1a61 = _c1a59[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c1a71 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c1a79 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c1a61 = sys.create_slice(_c1a59, imm_c1a71, imm_c1a79);
    // _c19f1 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c19f1 = sys.create_array_read(array_ba589, _be701);
    // _c1a75 = _c1a61 == _c19f1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c1a75 = sys.create_eq(_c1a61, _c19f1);
    // _c1a8d = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:165
    let imm_c1a91 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c1a8d = sys.create_array_read(array_ba53d, imm_c1a91);
    // _c1a95 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:166
    let imm_c1a9d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c1a95 = sys.create_array_read(array_ba53d, imm_c1a9d);
    // _c1aa1 = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c1aa9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c1aa1 = sys.create_array_read(array_ba53d, imm_c1aa9);
    // _c1aad = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c1ab5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c1aad = sys.create_array_read(array_ba53d, imm_c1ab5);
    // _c1ab9 = _c1aad[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c1ac9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c1ad1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c1ab9 = sys.create_slice(_c1aad, imm_c1ac9, imm_c1ad1);
    // _c1acd = _c1ab9 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c1ad9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c1acd = sys.create_eq(_c1ab9, imm_c1ad9);
    // _c1add = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c1ae5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c1add = sys.create_array_read(array_ba53d, imm_c1ae5);
    // _c1ae9 = _c1add[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c1af9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c1b01 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c1ae9 = sys.create_slice(_c1add, imm_c1af9, imm_c1b01);
    // _c183d = ~_c1ae9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c183d = sys.create_flip(_c1ae9);
    // _c1b09 = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c1b0d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c1b09 = sys.create_array_read(array_ba53d, imm_c1b0d);
    // _c1b11 = _c1b09[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c1b21 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c1b29 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c1b11 = sys.create_slice(_c1b09, imm_c1b21, imm_c1b29);
    // _c1941 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c1941 = sys.create_array_read(array_ba589, _be701);
    // _c1b25 = _c1b11 == _c1941, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c1b25 = sys.create_eq(_c1b11, _c1941);
    // _c1b3d = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:162
    let imm_c1b41 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c1b3d = sys.create_array_read(array_ba53d, imm_c1b41);
    // _c1b45 = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:163
    let imm_c1b4d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c1b45 = sys.create_array_read(array_ba53d, imm_c1b4d);
    // _c1b51 = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c1b59 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c1b51 = sys.create_array_read(array_ba53d, imm_c1b59);
    // _c1b5d = _c1b51[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c1b6d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c1b75 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c1b5d = sys.create_slice(_c1b51, imm_c1b6d, imm_c1b75);
    // _c1919 = ~_c1b5d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c1919 = sys.create_flip(_c1b5d);
    // _c1b7d = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c1b81 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c1b7d = sys.create_array_read(array_ba53d, imm_c1b81);
    // _c1b85 = _c1b7d[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c1b95 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c1b9d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c1b85 = sys.create_slice(_c1b7d, imm_c1b95, imm_c1b9d);
    // _c1b15 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c1b15 = sys.create_array_read(array_ba589, _be701);
    // _c1b99 = _c1b85 == _c1b15, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c1b99 = sys.create_eq(_c1b85, _c1b15);
    // _c1bb1 = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:165
    let imm_c1bb5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c1bb1 = sys.create_array_read(array_ba53d, imm_c1bb5);
    // _c1bb9 = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:166
    let imm_c1bc1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c1bb9 = sys.create_array_read(array_ba53d, imm_c1bc1);
    // _c1bc5 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c1bcd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c1bc5 = sys.create_array_read(array_ba53d, imm_c1bcd);
    // _c1bd1 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c1bd9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c1bd1 = sys.create_array_read(array_ba53d, imm_c1bd9);
    // _c1bdd = _c1bd1[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c1bed = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c1bf5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c1bdd = sys.create_slice(_c1bd1, imm_c1bed, imm_c1bf5);
    // _c1bf1 = _c1bdd == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c1bfd = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c1bf1 = sys.create_eq(_c1bdd, imm_c1bfd);
    // _c2405 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c240d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c2405 = sys.create_array_read(array_ba53d, imm_c240d);
    // _c2411 = _c2405[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c2421 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c2429 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c2411 = sys.create_slice(_c2405, imm_c2421, imm_c2429);
    // _c1961 = ~_c2411, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c1961 = sys.create_flip(_c2411);
    // _c2431 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c2435 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c2431 = sys.create_array_read(array_ba53d, imm_c2435);
    // _c2439 = _c2431[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c2449 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c2451 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c2439 = sys.create_slice(_c2431, imm_c2449, imm_c2451);
    // _c1a65 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c1a65 = sys.create_array_read(array_ba589, _be701);
    // _c244d = _c2439 == _c1a65, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c244d = sys.create_eq(_c2439, _c1a65);
    // _c2465 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:162
    let imm_c2469 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c2465 = sys.create_array_read(array_ba53d, imm_c2469);
    // _c246d = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:163
    let imm_c2475 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c246d = sys.create_array_read(array_ba53d, imm_c2475);
    // _c2479 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c2481 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c2479 = sys.create_array_read(array_ba53d, imm_c2481);
    // _c2485 = _c2479[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c2495 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c249d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c2485 = sys.create_slice(_c2479, imm_c2495, imm_c249d);
    // _c2415 = ~_c2485, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c2415 = sys.create_flip(_c2485);
    // _c24a5 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c24a9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c24a5 = sys.create_array_read(array_ba53d, imm_c24a9);
    // _c24ad = _c24a5[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c24bd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c24c5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c24ad = sys.create_slice(_c24a5, imm_c24bd, imm_c24c5);
    // _c243d = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c243d = sys.create_array_read(array_ba589, _be701);
    // _c24c1 = _c24ad == _c243d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c24c1 = sys.create_eq(_c24ad, _c243d);
    // _c24d9 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:165
    let imm_c24dd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c24d9 = sys.create_array_read(array_ba53d, imm_c24dd);
    // _c24e1 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:166
    let imm_c24e9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c24e1 = sys.create_array_read(array_ba53d, imm_c24e9);
    // _c24ed = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c24f5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c24ed = sys.create_array_read(array_ba53d, imm_c24f5);
    // _c24f9 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c2501 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c24f9 = sys.create_array_read(array_ba53d, imm_c2501);
    // _c2505 = _c24f9[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c2515 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c251d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c2505 = sys.create_slice(_c24f9, imm_c2515, imm_c251d);
    // _c2519 = _c2505 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:160
    let imm_c1b61 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c2519 = sys.create_eq(_c2505, imm_c1b61);
    // _c2529 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c2531 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c2529 = sys.create_array_read(array_ba53d, imm_c2531);
    // _c2535 = _c2529[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c2545 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c254d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c2535 = sys.create_slice(_c2529, imm_c2545, imm_c254d);
    // _c1a85 = ~_c2535, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c1a85 = sys.create_flip(_c2535);
    // _c2555 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c2559 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c2555 = sys.create_array_read(array_ba53d, imm_c2559);
    // _c255d = _c2555[(136:u8):(140:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let imm_c256d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x88 as u64); // (136:u8)
    let imm_c2575 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8c as u64); // (140:u8)
    let _c255d = sys.create_slice(_c2555, imm_c256d, imm_c2575);
    // _c1b89 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c1b89 = sys.create_array_read(array_ba589, _be701);
    // _c2571 = _c255d == _c1b89, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:161
    let _c2571 = sys.create_eq(_c255d, _c1b89);
    // _c2589 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:162
    let imm_c258d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c2589 = sys.create_array_read(array_ba53d, imm_c258d);
    // _c2591 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:163
    let imm_c2599 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c2591 = sys.create_array_read(array_ba53d, imm_c2599);
    // _c259d = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c25a5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c259d = sys.create_array_read(array_ba53d, imm_c25a5);
    // _c25a9 = _c259d[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c25b9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c25c1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c25a9 = sys.create_slice(_c259d, imm_c25b9, imm_c25c1);
    // _c2525 = ~_c25a9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c2525 = sys.create_flip(_c25a9);
    // _c25c9 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c25cd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c25c9 = sys.create_array_read(array_ba53d, imm_c25cd);
    // _c25d1 = _c25c9[(131:u8):(135:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let imm_c25e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x83 as u64); // (131:u8)
    let imm_c25e9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x87 as u64); // (135:u8)
    let _c25d1 = sys.create_slice(_c25c9, imm_c25e1, imm_c25e9);
    // _c2561 = array_ba589[_be701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c2561 = sys.create_array_read(array_ba589, _be701);
    // _c25e5 = _c25d1 == _c2561, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:164
    let _c25e5 = sys.create_eq(_c25d1, _c2561);
    // _c25fd = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:165
    let imm_c2601 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c25fd = sys.create_array_read(array_ba53d, imm_c2601);
    // _c2605 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:166
    let imm_c260d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c2605 = sys.create_array_read(array_ba53d, imm_c260d);
    // array_ba589[_be701] = (16:b5), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:168
    let imm_c2611 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64); // (16:b5)
    sys.create_array_write(array_ba589, _be701, imm_c2611);
    // _c2625 = bitcast _bcc85 to i32, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:170
    let _c2625 = sys.create_bitcast(_bcc85, assassyn::ir::DataType::int_ty(32));
    // _c262d = _c2625 + (4:i32), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:170
    let imm_c261d = sys.get_const_int(assassyn::ir::DataType::int_ty(32), 0x4 as u64); // (4:i32)
    let _c262d = sys.create_add(_c2625, imm_c261d);
    // _c2635 = bitcast _c262d to b32, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:170
    let _c2635 = sys.create_bitcast(_c262d, assassyn::ir::DataType::bits_ty(32));
    // _c2639 = _bcc79[(4:u3):(4:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:171
    let imm_c2649 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x4 as u64); // (4:u3)
    let imm_c2651 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x4 as u64); // (4:u3)
    let _c2639 = sys.create_slice(_bcc79, imm_c2649, imm_c2651);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c25d5 = sys.create_conditional_block(_c2639);
    sys.set_current_block(_c25d5);
    // _c2659 = _bef29[(0:u1):(0:u1)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:172
    let imm_c2665 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c266d = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let _c2659 = sys.create_slice(_bef29, imm_c2665, imm_c266d);
    // _c25f5 = _c2659 ? _bee9d : _c2635, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:172
    let _c25f5 = sys.create_select(_c2659, _bee9d, _c2635);
    // _c267d = _bef29[(0:u1):(0:u1)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:173
    let imm_c2685 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c268d = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let _c267d = sys.create_slice(_bef29, imm_c2685, imm_c268d);
    // log('condition: {}.a.b | a: {:08x}  | b: {:08x}   |', _c267d, _bee9d, _c2635), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:173
    let fmt = sys.get_str_literal("condition: {}.a.b | a: {:08x}  | b: {:08x}   |".into());
    sys.create_log(fmt, vec![_c267d, _bee9d, _c2635]);
    // array_c24b1[(0:u1)] = (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:175
    let imm_c26ad = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c26a1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    sys.create_array_write(array_c24b1, imm_c26ad, imm_c26a1);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _c2539 = _bee61 | _bee81, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:177
    let _c2539 = sys.create_bitwise_or(_bee61, _bee81);
    // _c26b9 = bitcast _bee9d to u32, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:180
    let _c26b9 = sys.create_bitcast(_bee9d, assassyn::ir::DataType::uint_ty(32));
    // _c26c5 = _c2539 ? (184:u32) : (0:u32), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:180
    let imm_b9a2d = sys.get_const_int(assassyn::ir::DataType::uint_ty(32), 0xb8 as u64); // (184:u32)
    let imm_c26bd = sys.get_const_int(assassyn::ir::DataType::uint_ty(32), 0x0 as u64); // (0:u32)
    let _c26c5 = sys.create_select(_c2539, imm_b9a2d, imm_c26bd);
    // _c26d1 = _c26b9 - _c26c5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:180
    let _c26d1 = sys.create_sub(_c26b9, _c26c5);
    // _c26d5 = bitcast _c26d1 to b32, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:180
    let _c26d5 = sys.create_bitcast(_c26d1, assassyn::ir::DataType::bits_ty(32));
    // _c26dd = _c26d5[(2:u2):(17:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:181
    let imm_c26e5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(2), 0x2 as u64); // (2:u2)
    let imm_c26ed = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x11 as u64); // (17:u5)
    let _c26dd = sys.create_slice(_c26d5, imm_c26e5, imm_c26ed);
    // _c2695 = bitcast _c26dd to u16, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:181
    let _c2695 = sys.create_bitcast(_c26dd, assassyn::ir::DataType::uint_ty(16));
    // _c2701 = _c2539 ? _c2695 : (0:u16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:181
    let imm_c26f9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(16), 0x0 as u64); // (0:u16)
    let _c2701 = sys.create_select(_c2539, _c2695, imm_c26f9);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c270d = sys.create_conditional_block(_bee61);
    sys.set_current_block(_c270d);
    // log('mem-read         | addr: 0x{:05x}| line: 0x{:05x} |', _bee9d, _c2701), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:184
    let fmt = sys.get_str_literal("mem-read         | addr: 0x{:05x}| line: 0x{:05x} |".into());
    sys.create_log(fmt, vec![_bee9d, _c2701]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c26b1 = sys.create_conditional_block(_bee81);
    sys.set_current_block(_c26b1);
    // log('mem-write        | addr: 0x{:05x}| line: 0x{:05x} | value: 0x{:08x} | wdada: 0x{:08x}', _bee9d, _c2701, _beccd, _becdd), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:187
    let fmt = sys.get_str_literal(
        "mem-write        | addr: 0x{:05x}| line: 0x{:05x} | value: 0x{:08x} | wdada: 0x{:08x}"
            .into(),
    );
    sys.create_log(fmt, vec![_bee9d, _c2701, _beccd, _becdd]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _9c155 = sys.create_conditional_block(_bee61);
    sys.set_current_block(_9c155);
    // _ba5e5.rd.push(_be701) // handle = _c26cd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:193
    let push_c27a5 = sys.bind_arg(_c27a5, "rd".into(), _be701);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // async_call _c27a5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:194
    sys.create_async_call(_c27a5);
    // _c2781 = _bcc79[(2:u2):(2:u2)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:196
    let imm_c2799 = sys.get_const_int(assassyn::ir::DataType::uint_ty(2), 0x2 as u64); // (2:u2)
    let imm_c27c9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(2), 0x2 as u64); // (2:u2)
    let _c2781 = sys.create_slice(_bcc79, imm_c2799, imm_c27c9);
    // _c26e9 = _c2781 ? _c2635 : _bee9d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:196
    let _c26e9 = sys.create_select(_c2781, _c2635, _bee9d);
    // _c27c5 = _bcc79[(77:u7):(77:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:198
    let imm_c27e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4d as u64); // (77:u7)
    let imm_c27e9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4d as u64); // (77:u7)
    let _c27c5 = sys.create_slice(_bcc79, imm_c27e1, imm_c27e9);
    // _c27f1 = _bcc79[(0:u1):(1:u1)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:201
    let imm_c27f9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c27fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x1 as u64); // (1:u1)
    let _c27f1 = sys.create_slice(_bcc79, imm_c27f9, imm_c27fd);
    // _ba599.is_memory_read.push(_bee61) // handle = _c2815, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:195
    let push_9c0d5 = sys.bind_arg(_9c0d5, "is_memory_read".into(), _bee61);
    // _ba599.result.push(_c26e9) // handle = _c2819, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:195
    let push_9c0d5 = sys.bind_arg(_9c0d5, "result".into(), _c26e9);
    // _ba599.rd.push(_be701) // handle = _c2821, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:195
    let push_9c0d5 = sys.bind_arg(_9c0d5, "rd".into(), _be701);
    // _ba599.is_csr.push(_c27c5) // handle = _c2829, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:195
    let push_9c0d5 = sys.bind_arg(_9c0d5, "is_csr".into(), _c27c5);
    // _ba599.csr_id.push(_be64d) // handle = _c2831, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:195
    let push_9c0d5 = sys.bind_arg(_9c0d5, "csr_id".into(), _be64d);
    // _ba599.csr_new.push(_be6e5) // handle = _c2841, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:195
    let push_9c0d5 = sys.bind_arg(_9c0d5, "csr_new".into(), _be6e5);
    // _ba599.mem_ext.push(_c27f1) // handle = _c2845, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:195
    let push_9c0d5 = sys.bind_arg(_9c0d5, "mem_ext".into(), _c27f1);
    // _9c0d5 = _ba599.bind([_c2815 /* _ba599.is_memory_read=_bee61 */, _c2819 /* _ba599.result=_c26e9 */, _c2821 /* _ba599.rd=_be701 */, _c2829 /* _ba599.is_csr=_c27c5 */, _c2831 /* _ba599.csr_id=_be64d */, _c2841 /* _ba599.csr_new=_be6e5 */, _c2845 /* _ba599.mem_ext=_c27f1 */, _c5f25 /* _ba599.mdata=_c5f09 */]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:195
    // Already handled by `EmitBinds` {}
    // _c2629 = _be701 != (0:b5), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:203
    let imm_c2785 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x0 as u64); // (0:b5)
    let _c2629 = sys.create_neq(_be701, imm_c2785);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c269d = sys.create_conditional_block(_c2629);
    sys.set_current_block(_c269d);
    // log('own x{:02}          |', _be701), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:204
    let fmt = sys.get_str_literal("own x{:02}          |".into());
    sys.create_log(fmt, vec![_be701]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // Fill in the body of _bedf5
    sys.set_current_module(_bedf5);
    // module root block
    // _c5f3d = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c5f35 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c5f3d = sys.create_array_read(array_ba53d, imm_c5f35);
    // _c5f49 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c5f4d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c5f49 = sys.create_array_read(array_ba53d, imm_c5f4d);
    // _c5f59 = _c5f49[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c5f69 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c5f71 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c5f59 = sys.create_slice(_c5f49, imm_c5f69, imm_c5f71);
    // _c5f7d = _c5f59 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c5f6d = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c5f7d = sys.create_eq(_c5f59, imm_c5f6d);
    // _c5f89 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c5f8d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c5f89 = sys.create_array_read(array_ba53d, imm_c5f8d);
    // _c5f99 = _c5f89[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c5fa5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c5fad = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c5f99 = sys.create_slice(_c5f89, imm_c5fa5, imm_c5fad);
    // _c5fb5 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c5fa9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c5fb5 = sys.create_array_read(array_ba53d, imm_c5fa9);
    // _c5fb9 = _c5fb5[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c5fc9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c5fd1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c5fb9 = sys.create_slice(_c5fb5, imm_c5fc9, imm_c5fd1);
    // _c5f85 = ~(0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:224
    let imm_c2835 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _c5f85 = sys.create_flip(imm_c2835);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c5f95 = sys.create_conditional_block(_c5f85);
    sys.set_current_block(_c5f95);
    // _c5fe5 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:225
    let imm_c5fe9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c5fe5 = sys.create_array_read(array_ba53d, imm_c5fe9);
    // _c5fed = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c5ff5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c5fed = sys.create_array_read(array_ba53d, imm_c5ff5);
    // _c5ff9 = _c5fed[(32:u6):(128:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c680d = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x20 as u64); // (32:u6)
    let imm_c6815 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x80 as u64); // (128:u8)
    let _c5ff9 = sys.create_slice(_c5fed, imm_c680d, imm_c6815);
    // _c681d = bitcast _c5ff9 to record { mem_ext: b2, link_pc: b1, is_offset_br: b1, is_branch: b1, flip: b1, cond: b16, alu: b16, memory: b2, imm_valid: b1, imm: b32, is_pc_calc: b1, is_mepc: b1, is_zimm: b1, csr_calculate: b1, csr_write: b1, csr_read: b1, rd_valid: b1, rd: b5, rs2_valid: b1, rs2: b5, rs1_valid: b1, rs1: b5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let _c681d = sys.create_bitcast(_c5ff9, assassyn::ir::DataType::bits_ty(97));
    // _c6825 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c6821 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c6825 = sys.create_array_read(array_ba53d, imm_c6821);
    // _c6831 = _c6825[(0:u1):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c683d = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c6845 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _c6831 = sys.create_slice(_c6825, imm_c683d, imm_c6845);
    // _c684d = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c6841 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c684d = sys.create_array_read(array_ba53d, imm_c6841);
    // _c6851 = _c684d[(173:u8):(204:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c6861 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xad as u64); // (173:u8)
    let imm_c6869 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcc as u64); // (204:u8)
    let _c6851 = sys.create_slice(_c684d, imm_c6861, imm_c6869);
    // _c6871 = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c6865 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c6871 = sys.create_array_read(array_ba53d, imm_c6865);
    // _c6879 = _c6871[(141:u8):(172:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c6881 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8d as u64); // (141:u8)
    let imm_c6889 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xac as u64); // (172:u8)
    let _c6879 = sys.create_slice(_c6871, imm_c6881, imm_c6889);
    // _ba7f9.signals.push((0:b97)) // handle = _c68ad, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c5fbd = sys.get_const_int(assassyn::ir::DataType::bits_ty(97), 0x0 as u64); // (0:b97)
    let push_c68a1 = sys.bind_arg(_c68a1, "signals".into(), imm_c5fbd);
    // _ba7f9.fetch_addr.push((0:b32)) // handle = _c68d1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c6895 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let push_c68a1 = sys.bind_arg(_c68a1, "fetch_addr".into(), imm_c6895);
    // _c68a1 = _ba7f9.bind([_c68ad /* _ba7f9.signals=(0:b97) */, _c68d1 /* _ba7f9.fetch_addr=(0:b32) */]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    // Already handled by `EmitBinds` {}
    // async_call _c68a1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    sys.create_async_call(_c68a1);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _c68d9 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c68a9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c68d9 = sys.create_array_read(array_ba53d, imm_c68a9);
    // _c68d5 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c68e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c68d5 = sys.create_array_read(array_ba53d, imm_c68e1);
    // _c68e5 = _c68d5[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c68f5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c68fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c68e5 = sys.create_slice(_c68d5, imm_c68f5, imm_c68fd);
    // _c68f9 = _c68e5 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6905 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c68f9 = sys.create_eq(_c68e5, imm_c6905);
    // _c6909 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6911 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c6909 = sys.create_array_read(array_ba53d, imm_c6911);
    // _c6915 = _c6909[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6925 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c692d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c6915 = sys.create_slice(_c6909, imm_c6925, imm_c692d);
    // _c6935 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6929 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c6935 = sys.create_array_read(array_ba53d, imm_c6929);
    // _c6939 = _c6935[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6949 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c6951 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c6939 = sys.create_slice(_c6935, imm_c6949, imm_c6951);
    // _c68a5 = ~(1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:224
    let imm_c5f91 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _c68a5 = sys.create_flip(imm_c5f91);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c694d = sys.create_conditional_block(_c68a5);
    sys.set_current_block(_c694d);
    // _c6969 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:225
    let imm_c696d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c6969 = sys.create_array_read(array_ba53d, imm_c696d);
    // _c6971 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c6979 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c6971 = sys.create_array_read(array_ba53d, imm_c6979);
    // _c697d = _c6971[(32:u6):(128:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c698d = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x20 as u64); // (32:u6)
    let imm_c6995 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x80 as u64); // (128:u8)
    let _c697d = sys.create_slice(_c6971, imm_c698d, imm_c6995);
    // _c699d = bitcast _c697d to record { mem_ext: b2, link_pc: b1, is_offset_br: b1, is_branch: b1, flip: b1, cond: b16, alu: b16, memory: b2, imm_valid: b1, imm: b32, is_pc_calc: b1, is_mepc: b1, is_zimm: b1, csr_calculate: b1, csr_write: b1, csr_read: b1, rd_valid: b1, rd: b5, rs2_valid: b1, rs2: b5, rs1_valid: b1, rs1: b5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let _c699d = sys.create_bitcast(_c697d, assassyn::ir::DataType::bits_ty(97));
    // _c69a1 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c6991 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c69a1 = sys.create_array_read(array_ba53d, imm_c6991);
    // _c69ad = _c69a1[(0:u1):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c69b9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c69c1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _c69ad = sys.create_slice(_c69a1, imm_c69b9, imm_c69c1);
    // _c69c9 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c69bd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c69c9 = sys.create_array_read(array_ba53d, imm_c69bd);
    // _c69cd = _c69c9[(173:u8):(204:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c69dd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xad as u64); // (173:u8)
    let imm_c69e5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcc as u64); // (204:u8)
    let _c69cd = sys.create_slice(_c69c9, imm_c69dd, imm_c69e5);
    // _c69ed = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c69e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c69ed = sys.create_array_read(array_ba53d, imm_c69e1);
    // _c69f5 = _c69ed[(141:u8):(172:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c69fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8d as u64); // (141:u8)
    let imm_c6a05 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xac as u64); // (172:u8)
    let _c69f5 = sys.create_slice(_c69ed, imm_c69fd, imm_c6a05);
    // _ba7f9.signals.push((0:b97)) // handle = _c6a29, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c6a0d = sys.get_const_int(assassyn::ir::DataType::bits_ty(97), 0x0 as u64); // (0:b97)
    let push_c6a1d = sys.bind_arg(_c6a1d, "signals".into(), imm_c6a0d);
    // _ba7f9.fetch_addr.push((0:b32)) // handle = _c6a39, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c6a11 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let push_c6a1d = sys.bind_arg(_c6a1d, "fetch_addr".into(), imm_c6a11);
    // _c6a1d = _ba7f9.bind([_c6a29 /* _ba7f9.signals=(0:b97) */, _c6a39 /* _ba7f9.fetch_addr=(0:b32) */]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    // Already handled by `EmitBinds` {}
    // async_call _c6a1d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    sys.create_async_call(_c6a1d);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _c6a41 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6a25 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c6a41 = sys.create_array_read(array_ba53d, imm_c6a25);
    // _c6a3d = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6a49 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c6a3d = sys.create_array_read(array_ba53d, imm_c6a49);
    // _c6a4d = _c6a3d[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6a5d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c6a65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c6a4d = sys.create_slice(_c6a3d, imm_c6a5d, imm_c6a65);
    // _c6a61 = _c6a4d == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6a6d = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c6a61 = sys.create_eq(_c6a4d, imm_c6a6d);
    // _c6a71 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6a79 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c6a71 = sys.create_array_read(array_ba53d, imm_c6a79);
    // _c6a7d = _c6a71[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6a8d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c6a95 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c6a7d = sys.create_slice(_c6a71, imm_c6a8d, imm_c6a95);
    // _c6a9d = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6a91 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c6a9d = sys.create_array_read(array_ba53d, imm_c6a91);
    // _c6aa1 = _c6a9d[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6ab1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c6ab9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c6aa1 = sys.create_slice(_c6a9d, imm_c6ab1, imm_c6ab9);
    // _c6a21 = ~(1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:224
    let imm_c6919 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _c6a21 = sys.create_flip(imm_c6919);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c6ab5 = sys.create_conditional_block(_c6a21);
    sys.set_current_block(_c6ab5);
    // _c6ad1 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:225
    let imm_c6ad5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c6ad1 = sys.create_array_read(array_ba53d, imm_c6ad5);
    // _c6ad9 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c6ae1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c6ad9 = sys.create_array_read(array_ba53d, imm_c6ae1);
    // _c6ae5 = _c6ad9[(32:u6):(128:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c6af5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x20 as u64); // (32:u6)
    let imm_c6afd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x80 as u64); // (128:u8)
    let _c6ae5 = sys.create_slice(_c6ad9, imm_c6af5, imm_c6afd);
    // _c6b05 = bitcast _c6ae5 to record { mem_ext: b2, link_pc: b1, is_offset_br: b1, is_branch: b1, flip: b1, cond: b16, alu: b16, memory: b2, imm_valid: b1, imm: b32, is_pc_calc: b1, is_mepc: b1, is_zimm: b1, csr_calculate: b1, csr_write: b1, csr_read: b1, rd_valid: b1, rd: b5, rs2_valid: b1, rs2: b5, rs1_valid: b1, rs1: b5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let _c6b05 = sys.create_bitcast(_c6ae5, assassyn::ir::DataType::bits_ty(97));
    // _c6b09 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c6af9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c6b09 = sys.create_array_read(array_ba53d, imm_c6af9);
    // _c6b15 = _c6b09[(0:u1):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c6b21 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c6b29 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _c6b15 = sys.create_slice(_c6b09, imm_c6b21, imm_c6b29);
    // _c6b31 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c6b25 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c6b31 = sys.create_array_read(array_ba53d, imm_c6b25);
    // _c6b35 = _c6b31[(173:u8):(204:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c6b45 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xad as u64); // (173:u8)
    let imm_c6b4d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcc as u64); // (204:u8)
    let _c6b35 = sys.create_slice(_c6b31, imm_c6b45, imm_c6b4d);
    // _c6b55 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c6b49 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c6b55 = sys.create_array_read(array_ba53d, imm_c6b49);
    // _c6b5d = _c6b55[(141:u8):(172:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c6b65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8d as u64); // (141:u8)
    let imm_c6b6d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xac as u64); // (172:u8)
    let _c6b5d = sys.create_slice(_c6b55, imm_c6b65, imm_c6b6d);
    // _ba7f9.signals.push((0:b97)) // handle = _c6b91, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c6b75 = sys.get_const_int(assassyn::ir::DataType::bits_ty(97), 0x0 as u64); // (0:b97)
    let push_c6b85 = sys.bind_arg(_c6b85, "signals".into(), imm_c6b75);
    // _ba7f9.fetch_addr.push((0:b32)) // handle = _c6ba1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c6b79 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let push_c6b85 = sys.bind_arg(_c6b85, "fetch_addr".into(), imm_c6b79);
    // _c6b85 = _ba7f9.bind([_c6b91 /* _ba7f9.signals=(0:b97) */, _c6ba1 /* _ba7f9.fetch_addr=(0:b32) */]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    // Already handled by `EmitBinds` {}
    // async_call _c6b85, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    sys.create_async_call(_c6b85);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _c6ba9 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6b8d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c6ba9 = sys.create_array_read(array_ba53d, imm_c6b8d);
    // _c6ba5 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6bb1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c6ba5 = sys.create_array_read(array_ba53d, imm_c6bb1);
    // _c6bb5 = _c6ba5[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6bc5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c6bcd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c6bb5 = sys.create_slice(_c6ba5, imm_c6bc5, imm_c6bcd);
    // _c6bc9 = _c6bb5 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6bd5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c6bc9 = sys.create_eq(_c6bb5, imm_c6bd5);
    // _c6bd9 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6be1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c6bd9 = sys.create_array_read(array_ba53d, imm_c6be1);
    // _c6be5 = _c6bd9[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6bf5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c6bfd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c6be5 = sys.create_slice(_c6bd9, imm_c6bf5, imm_c6bfd);
    // _c6c09 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6c05 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c6c09 = sys.create_array_read(array_ba53d, imm_c6c05);
    // _c6c0d = _c6c09[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6c1d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c6c25 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c6c0d = sys.create_slice(_c6c09, imm_c6c1d, imm_c6c25);
    // _c278d = ~(1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:224
    let imm_c6a81 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _c278d = sys.create_flip(imm_c6a81);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c6c21 = sys.create_conditional_block(_c278d);
    sys.set_current_block(_c6c21);
    // _c6c3d = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:225
    let imm_c6c41 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c6c3d = sys.create_array_read(array_ba53d, imm_c6c41);
    // _c6c45 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c6c4d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c6c45 = sys.create_array_read(array_ba53d, imm_c6c4d);
    // _c6c51 = _c6c45[(32:u6):(128:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c6c61 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x20 as u64); // (32:u6)
    let imm_c6c69 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x80 as u64); // (128:u8)
    let _c6c51 = sys.create_slice(_c6c45, imm_c6c61, imm_c6c69);
    // _c6c71 = bitcast _c6c51 to record { mem_ext: b2, link_pc: b1, is_offset_br: b1, is_branch: b1, flip: b1, cond: b16, alu: b16, memory: b2, imm_valid: b1, imm: b32, is_pc_calc: b1, is_mepc: b1, is_zimm: b1, csr_calculate: b1, csr_write: b1, csr_read: b1, rd_valid: b1, rd: b5, rs2_valid: b1, rs2: b5, rs1_valid: b1, rs1: b5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let _c6c71 = sys.create_bitcast(_c6c51, assassyn::ir::DataType::bits_ty(97));
    // _c6c75 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c6c65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c6c75 = sys.create_array_read(array_ba53d, imm_c6c65);
    // _c6c81 = _c6c75[(0:u1):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c6c8d = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c6c95 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _c6c81 = sys.create_slice(_c6c75, imm_c6c8d, imm_c6c95);
    // _c6c9d = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c6c91 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c6c9d = sys.create_array_read(array_ba53d, imm_c6c91);
    // _c6ca1 = _c6c9d[(173:u8):(204:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c6cb1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xad as u64); // (173:u8)
    let imm_c6cb9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcc as u64); // (204:u8)
    let _c6ca1 = sys.create_slice(_c6c9d, imm_c6cb1, imm_c6cb9);
    // _c6cc1 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c6cb5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c6cc1 = sys.create_array_read(array_ba53d, imm_c6cb5);
    // _c6cc9 = _c6cc1[(141:u8):(172:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c6cd1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8d as u64); // (141:u8)
    let imm_c6cd9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xac as u64); // (172:u8)
    let _c6cc9 = sys.create_slice(_c6cc1, imm_c6cd1, imm_c6cd9);
    // _ba7f9.signals.push((0:b97)) // handle = _c6cfd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c6ce1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(97), 0x0 as u64); // (0:b97)
    let push_c6cf1 = sys.bind_arg(_c6cf1, "signals".into(), imm_c6ce1);
    // _ba7f9.fetch_addr.push((0:b32)) // handle = _c6d0d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c6ce5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let push_c6cf1 = sys.bind_arg(_c6cf1, "fetch_addr".into(), imm_c6ce5);
    // _c6cf1 = _ba7f9.bind([_c6cfd /* _ba7f9.signals=(0:b97) */, _c6d0d /* _ba7f9.fetch_addr=(0:b32) */]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    // Already handled by `EmitBinds` {}
    // async_call _c6cf1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    sys.create_async_call(_c6cf1);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _c6d15 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6cf9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c6d15 = sys.create_array_read(array_ba53d, imm_c6cf9);
    // _c6d11 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6d1d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c6d11 = sys.create_array_read(array_ba53d, imm_c6d1d);
    // _c6d21 = _c6d11[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6d31 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c6d39 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c6d21 = sys.create_slice(_c6d11, imm_c6d31, imm_c6d39);
    // _c6d35 = _c6d21 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6d41 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c6d35 = sys.create_eq(_c6d21, imm_c6d41);
    // _c6d45 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6d4d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c6d45 = sys.create_array_read(array_ba53d, imm_c6d4d);
    // _c6d51 = _c6d45[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6d61 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c6d69 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c6d51 = sys.create_slice(_c6d45, imm_c6d61, imm_c6d69);
    // _c6d71 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6d65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c6d71 = sys.create_array_read(array_ba53d, imm_c6d65);
    // _c6d75 = _c6d71[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6d85 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c6d8d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c6d75 = sys.create_slice(_c6d71, imm_c6d85, imm_c6d8d);
    // _c6cf5 = ~(1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:224
    let imm_c6be9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _c6cf5 = sys.create_flip(imm_c6be9);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c6d89 = sys.create_conditional_block(_c6cf5);
    sys.set_current_block(_c6d89);
    // _c6da5 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:225
    let imm_c6da9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c6da5 = sys.create_array_read(array_ba53d, imm_c6da9);
    // _c6dad = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c6db5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c6dad = sys.create_array_read(array_ba53d, imm_c6db5);
    // _c6db9 = _c6dad[(32:u6):(128:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c6dc9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x20 as u64); // (32:u6)
    let imm_c6dd1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x80 as u64); // (128:u8)
    let _c6db9 = sys.create_slice(_c6dad, imm_c6dc9, imm_c6dd1);
    // _c6dd9 = bitcast _c6db9 to record { mem_ext: b2, link_pc: b1, is_offset_br: b1, is_branch: b1, flip: b1, cond: b16, alu: b16, memory: b2, imm_valid: b1, imm: b32, is_pc_calc: b1, is_mepc: b1, is_zimm: b1, csr_calculate: b1, csr_write: b1, csr_read: b1, rd_valid: b1, rd: b5, rs2_valid: b1, rs2: b5, rs1_valid: b1, rs1: b5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let _c6dd9 = sys.create_bitcast(_c6db9, assassyn::ir::DataType::bits_ty(97));
    // _c6ddd = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c6dcd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c6ddd = sys.create_array_read(array_ba53d, imm_c6dcd);
    // _c6de9 = _c6ddd[(0:u1):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c6df5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c6dfd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _c6de9 = sys.create_slice(_c6ddd, imm_c6df5, imm_c6dfd);
    // _c6e05 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c6df9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c6e05 = sys.create_array_read(array_ba53d, imm_c6df9);
    // _c6e09 = _c6e05[(173:u8):(204:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c6e19 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xad as u64); // (173:u8)
    let imm_c6e21 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcc as u64); // (204:u8)
    let _c6e09 = sys.create_slice(_c6e05, imm_c6e19, imm_c6e21);
    // _c6e29 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c6e1d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c6e29 = sys.create_array_read(array_ba53d, imm_c6e1d);
    // _c6e31 = _c6e29[(141:u8):(172:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c6e39 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8d as u64); // (141:u8)
    let imm_c6e41 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xac as u64); // (172:u8)
    let _c6e31 = sys.create_slice(_c6e29, imm_c6e39, imm_c6e41);
    // _ba7f9.signals.push((0:b97)) // handle = _c6e65, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c6e49 = sys.get_const_int(assassyn::ir::DataType::bits_ty(97), 0x0 as u64); // (0:b97)
    let push_c6e59 = sys.bind_arg(_c6e59, "signals".into(), imm_c6e49);
    // _ba7f9.fetch_addr.push((0:b32)) // handle = _c6e75, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c6e4d = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let push_c6e59 = sys.bind_arg(_c6e59, "fetch_addr".into(), imm_c6e4d);
    // _c6e59 = _ba7f9.bind([_c6e65 /* _ba7f9.signals=(0:b97) */, _c6e75 /* _ba7f9.fetch_addr=(0:b32) */]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    // Already handled by `EmitBinds` {}
    // async_call _c6e59, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    sys.create_async_call(_c6e59);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _c6e7d = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6e61 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c6e7d = sys.create_array_read(array_ba53d, imm_c6e61);
    // _c6e79 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6e85 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c6e79 = sys.create_array_read(array_ba53d, imm_c6e85);
    // _c6e89 = _c6e79[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6e99 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c6ea1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c6e89 = sys.create_slice(_c6e79, imm_c6e99, imm_c6ea1);
    // _c6e9d = _c6e89 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6ea9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c6e9d = sys.create_eq(_c6e89, imm_c6ea9);
    // _c6ead = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6eb5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c6ead = sys.create_array_read(array_ba53d, imm_c6eb5);
    // _c6eb9 = _c6ead[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6ec9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c6ed1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c6eb9 = sys.create_slice(_c6ead, imm_c6ec9, imm_c6ed1);
    // _c6ed9 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6ecd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c6ed9 = sys.create_array_read(array_ba53d, imm_c6ecd);
    // _c6edd = _c6ed9[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c6eed = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c6ef5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c6edd = sys.create_slice(_c6ed9, imm_c6eed, imm_c6ef5);
    // _c6e5d = ~(1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:224
    let imm_c6d55 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _c6e5d = sys.create_flip(imm_c6d55);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c6ef1 = sys.create_conditional_block(_c6e5d);
    sys.set_current_block(_c6ef1);
    // _c6f0d = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:225
    let imm_c6f11 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c6f0d = sys.create_array_read(array_ba53d, imm_c6f11);
    // _c6f15 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c6f1d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c6f15 = sys.create_array_read(array_ba53d, imm_c6f1d);
    // _c6f21 = _c6f15[(32:u6):(128:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c6f31 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x20 as u64); // (32:u6)
    let imm_c6f39 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x80 as u64); // (128:u8)
    let _c6f21 = sys.create_slice(_c6f15, imm_c6f31, imm_c6f39);
    // _c6f41 = bitcast _c6f21 to record { mem_ext: b2, link_pc: b1, is_offset_br: b1, is_branch: b1, flip: b1, cond: b16, alu: b16, memory: b2, imm_valid: b1, imm: b32, is_pc_calc: b1, is_mepc: b1, is_zimm: b1, csr_calculate: b1, csr_write: b1, csr_read: b1, rd_valid: b1, rd: b5, rs2_valid: b1, rs2: b5, rs1_valid: b1, rs1: b5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let _c6f41 = sys.create_bitcast(_c6f21, assassyn::ir::DataType::bits_ty(97));
    // _c6f45 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c6f35 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c6f45 = sys.create_array_read(array_ba53d, imm_c6f35);
    // _c6f51 = _c6f45[(0:u1):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c6f5d = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c6f65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _c6f51 = sys.create_slice(_c6f45, imm_c6f5d, imm_c6f65);
    // _c6f6d = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c6f61 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c6f6d = sys.create_array_read(array_ba53d, imm_c6f61);
    // _c6f71 = _c6f6d[(173:u8):(204:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c6f81 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xad as u64); // (173:u8)
    let imm_c6f89 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcc as u64); // (204:u8)
    let _c6f71 = sys.create_slice(_c6f6d, imm_c6f81, imm_c6f89);
    // _c6f91 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c6f85 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c6f91 = sys.create_array_read(array_ba53d, imm_c6f85);
    // _c6f99 = _c6f91[(141:u8):(172:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c6fa1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8d as u64); // (141:u8)
    let imm_c6fa9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xac as u64); // (172:u8)
    let _c6f99 = sys.create_slice(_c6f91, imm_c6fa1, imm_c6fa9);
    // _ba7f9.signals.push((0:b97)) // handle = _c6fcd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c6fb1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(97), 0x0 as u64); // (0:b97)
    let push_c6fc1 = sys.bind_arg(_c6fc1, "signals".into(), imm_c6fb1);
    // _ba7f9.fetch_addr.push((0:b32)) // handle = _c6fdd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c6fb5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let push_c6fc1 = sys.bind_arg(_c6fc1, "fetch_addr".into(), imm_c6fb5);
    // _c6fc1 = _ba7f9.bind([_c6fcd /* _ba7f9.signals=(0:b97) */, _c6fdd /* _ba7f9.fetch_addr=(0:b32) */]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    // Already handled by `EmitBinds` {}
    // async_call _c6fc1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    sys.create_async_call(_c6fc1);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _c6fe5 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6fc9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c6fe5 = sys.create_array_read(array_ba53d, imm_c6fc9);
    // _c6fe1 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6fed = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c6fe1 = sys.create_array_read(array_ba53d, imm_c6fed);
    // _c6ff1 = _c6fe1[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6ffd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c780d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c6ff1 = sys.create_slice(_c6fe1, imm_c6ffd, imm_c780d);
    // _c7809 = _c6ff1 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c6f75 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c7809 = sys.create_eq(_c6ff1, imm_c6f75);
    // _c7819 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c7821 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c7819 = sys.create_array_read(array_ba53d, imm_c7821);
    // _c7825 = _c7819[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c7835 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c783d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c7825 = sys.create_slice(_c7819, imm_c7835, imm_c783d);
    // _c7845 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c7839 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c7845 = sys.create_array_read(array_ba53d, imm_c7839);
    // _c7849 = _c7845[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c7859 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c7861 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c7849 = sys.create_slice(_c7845, imm_c7859, imm_c7861);
    // _c6b89 = ~(1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:224
    let imm_c6ebd = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _c6b89 = sys.create_flip(imm_c6ebd);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c785d = sys.create_conditional_block(_c6b89);
    sys.set_current_block(_c785d);
    // _c7879 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:225
    let imm_c787d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c7879 = sys.create_array_read(array_ba53d, imm_c787d);
    // _c7881 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c7889 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c7881 = sys.create_array_read(array_ba53d, imm_c7889);
    // _c788d = _c7881[(32:u6):(128:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c789d = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x20 as u64); // (32:u6)
    let imm_c78a5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x80 as u64); // (128:u8)
    let _c788d = sys.create_slice(_c7881, imm_c789d, imm_c78a5);
    // _c78ad = bitcast _c788d to record { mem_ext: b2, link_pc: b1, is_offset_br: b1, is_branch: b1, flip: b1, cond: b16, alu: b16, memory: b2, imm_valid: b1, imm: b32, is_pc_calc: b1, is_mepc: b1, is_zimm: b1, csr_calculate: b1, csr_write: b1, csr_read: b1, rd_valid: b1, rd: b5, rs2_valid: b1, rs2: b5, rs1_valid: b1, rs1: b5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let _c78ad = sys.create_bitcast(_c788d, assassyn::ir::DataType::bits_ty(97));
    // _c78b1 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c78a1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c78b1 = sys.create_array_read(array_ba53d, imm_c78a1);
    // _c78bd = _c78b1[(0:u1):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c78c9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c78d1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _c78bd = sys.create_slice(_c78b1, imm_c78c9, imm_c78d1);
    // _c78d9 = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c78cd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c78d9 = sys.create_array_read(array_ba53d, imm_c78cd);
    // _c78dd = _c78d9[(173:u8):(204:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c78ed = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xad as u64); // (173:u8)
    let imm_c78f5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcc as u64); // (204:u8)
    let _c78dd = sys.create_slice(_c78d9, imm_c78ed, imm_c78f5);
    // _c78fd = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c78f1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c78fd = sys.create_array_read(array_ba53d, imm_c78f1);
    // _c7905 = _c78fd[(141:u8):(172:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c790d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8d as u64); // (141:u8)
    let imm_c7915 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xac as u64); // (172:u8)
    let _c7905 = sys.create_slice(_c78fd, imm_c790d, imm_c7915);
    // _ba7f9.signals.push((0:b97)) // handle = _c7939, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c791d = sys.get_const_int(assassyn::ir::DataType::bits_ty(97), 0x0 as u64); // (0:b97)
    let push_c792d = sys.bind_arg(_c792d, "signals".into(), imm_c791d);
    // _ba7f9.fetch_addr.push((0:b32)) // handle = _c7949, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c7921 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let push_c792d = sys.bind_arg(_c792d, "fetch_addr".into(), imm_c7921);
    // _c792d = _ba7f9.bind([_c7939 /* _ba7f9.signals=(0:b97) */, _c7949 /* _ba7f9.fetch_addr=(0:b32) */]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    // Already handled by `EmitBinds` {}
    // async_call _c792d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    sys.create_async_call(_c792d);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _c7955 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c7935 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c7955 = sys.create_array_read(array_ba53d, imm_c7935);
    // _c7951 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c795d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c7951 = sys.create_array_read(array_ba53d, imm_c795d);
    // _c7961 = _c7951[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c7971 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c7979 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c7961 = sys.create_slice(_c7951, imm_c7971, imm_c7979);
    // _c7975 = _c7961 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c7981 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c7975 = sys.create_eq(_c7961, imm_c7981);
    // _c7985 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c798d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c7985 = sys.create_array_read(array_ba53d, imm_c798d);
    // _c7991 = _c7985[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c79a1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c79a9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c7991 = sys.create_slice(_c7985, imm_c79a1, imm_c79a9);
    // _c79b1 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c79a5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c79b1 = sys.create_array_read(array_ba53d, imm_c79a5);
    // _c79b5 = _c79b1[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c79c5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c79cd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c79b5 = sys.create_slice(_c79b1, imm_c79c5, imm_c79cd);
    // _c7931 = ~(1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:224
    let imm_c6fa5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _c7931 = sys.create_flip(imm_c6fa5);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c79c9 = sys.create_conditional_block(_c7931);
    sys.set_current_block(_c79c9);
    // _c79e5 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:225
    let imm_c79e9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c79e5 = sys.create_array_read(array_ba53d, imm_c79e9);
    // _c79ed = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c79f5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c79ed = sys.create_array_read(array_ba53d, imm_c79f5);
    // _c79f9 = _c79ed[(32:u6):(128:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c7a09 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x20 as u64); // (32:u6)
    let imm_c7a11 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x80 as u64); // (128:u8)
    let _c79f9 = sys.create_slice(_c79ed, imm_c7a09, imm_c7a11);
    // _c7a19 = bitcast _c79f9 to record { mem_ext: b2, link_pc: b1, is_offset_br: b1, is_branch: b1, flip: b1, cond: b16, alu: b16, memory: b2, imm_valid: b1, imm: b32, is_pc_calc: b1, is_mepc: b1, is_zimm: b1, csr_calculate: b1, csr_write: b1, csr_read: b1, rd_valid: b1, rd: b5, rs2_valid: b1, rs2: b5, rs1_valid: b1, rs1: b5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let _c7a19 = sys.create_bitcast(_c79f9, assassyn::ir::DataType::bits_ty(97));
    // _c7a21 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c7a1d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c7a21 = sys.create_array_read(array_ba53d, imm_c7a1d);
    // _c7a2d = _c7a21[(0:u1):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c7a39 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c7a41 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _c7a2d = sys.create_slice(_c7a21, imm_c7a39, imm_c7a41);
    // _c7a49 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c7a3d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c7a49 = sys.create_array_read(array_ba53d, imm_c7a3d);
    // _c7a4d = _c7a49[(173:u8):(204:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c7a5d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xad as u64); // (173:u8)
    let imm_c7a65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcc as u64); // (204:u8)
    let _c7a4d = sys.create_slice(_c7a49, imm_c7a5d, imm_c7a65);
    // _c7a6d = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c7a61 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c7a6d = sys.create_array_read(array_ba53d, imm_c7a61);
    // _c7a75 = _c7a6d[(141:u8):(172:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c7a7d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8d as u64); // (141:u8)
    let imm_c7a85 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xac as u64); // (172:u8)
    let _c7a75 = sys.create_slice(_c7a6d, imm_c7a7d, imm_c7a85);
    // _ba7f9.signals.push((0:b97)) // handle = _c7aa9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c7a8d = sys.get_const_int(assassyn::ir::DataType::bits_ty(97), 0x0 as u64); // (0:b97)
    let push_c7a9d = sys.bind_arg(_c7a9d, "signals".into(), imm_c7a8d);
    // _ba7f9.fetch_addr.push((0:b32)) // handle = _c7abd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c7a91 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let push_c7a9d = sys.bind_arg(_c7a9d, "fetch_addr".into(), imm_c7a91);
    // _c7a9d = _ba7f9.bind([_c7aa9 /* _ba7f9.signals=(0:b97) */, _c7abd /* _ba7f9.fetch_addr=(0:b32) */]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    // Already handled by `EmitBinds` {}
    // async_call _c7a9d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    sys.create_async_call(_c7a9d);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _c7ac9 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c7aa5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c7ac9 = sys.create_array_read(array_ba53d, imm_c7aa5);
    // _c7ac5 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c7ad1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c7ac5 = sys.create_array_read(array_ba53d, imm_c7ad1);
    // _c7ad5 = _c7ac5[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c7ae5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c7aed = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c7ad5 = sys.create_slice(_c7ac5, imm_c7ae5, imm_c7aed);
    // _c7ae9 = _c7ad5 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c7af5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c7ae9 = sys.create_eq(_c7ad5, imm_c7af5);
    // _c7af9 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c7b01 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c7af9 = sys.create_array_read(array_ba53d, imm_c7b01);
    // _c7b05 = _c7af9[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c7b15 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c7b1d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c7b05 = sys.create_slice(_c7af9, imm_c7b15, imm_c7b1d);
    // _c7b25 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c7b19 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c7b25 = sys.create_array_read(array_ba53d, imm_c7b19);
    // _c7b29 = _c7b25[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c7b39 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c7b41 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c7b29 = sys.create_slice(_c7b25, imm_c7b39, imm_c7b41);
    // _c7aa1 = ~(1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:224
    let imm_c7995 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _c7aa1 = sys.create_flip(imm_c7995);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c7b3d = sys.create_conditional_block(_c7aa1);
    sys.set_current_block(_c7b3d);
    // _c7b59 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:225
    let imm_c7b5d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c7b59 = sys.create_array_read(array_ba53d, imm_c7b5d);
    // _c7b61 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c7b69 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c7b61 = sys.create_array_read(array_ba53d, imm_c7b69);
    // _c7b6d = _c7b61[(32:u6):(128:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c7b7d = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x20 as u64); // (32:u6)
    let imm_c7b85 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x80 as u64); // (128:u8)
    let _c7b6d = sys.create_slice(_c7b61, imm_c7b7d, imm_c7b85);
    // _c7b8d = bitcast _c7b6d to record { mem_ext: b2, link_pc: b1, is_offset_br: b1, is_branch: b1, flip: b1, cond: b16, alu: b16, memory: b2, imm_valid: b1, imm: b32, is_pc_calc: b1, is_mepc: b1, is_zimm: b1, csr_calculate: b1, csr_write: b1, csr_read: b1, rd_valid: b1, rd: b5, rs2_valid: b1, rs2: b5, rs1_valid: b1, rs1: b5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let _c7b8d = sys.create_bitcast(_c7b6d, assassyn::ir::DataType::bits_ty(97));
    // _c7b91 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c7b81 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c7b91 = sys.create_array_read(array_ba53d, imm_c7b81);
    // _c7b9d = _c7b91[(0:u1):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c7ba9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c7bb1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _c7b9d = sys.create_slice(_c7b91, imm_c7ba9, imm_c7bb1);
    // _c7bb9 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c7bad = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c7bb9 = sys.create_array_read(array_ba53d, imm_c7bad);
    // _c7bbd = _c7bb9[(173:u8):(204:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c7bcd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xad as u64); // (173:u8)
    let imm_c7bd5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcc as u64); // (204:u8)
    let _c7bbd = sys.create_slice(_c7bb9, imm_c7bcd, imm_c7bd5);
    // _c7bdd = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c7bd1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c7bdd = sys.create_array_read(array_ba53d, imm_c7bd1);
    // _c7be5 = _c7bdd[(141:u8):(172:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c7bed = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8d as u64); // (141:u8)
    let imm_c7bf5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xac as u64); // (172:u8)
    let _c7be5 = sys.create_slice(_c7bdd, imm_c7bed, imm_c7bf5);
    // _ba7f9.signals.push((0:b97)) // handle = _c801d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c7bfd = sys.get_const_int(assassyn::ir::DataType::bits_ty(97), 0x0 as u64); // (0:b97)
    let push_c8011 = sys.bind_arg(_c8011, "signals".into(), imm_c7bfd);
    // _ba7f9.fetch_addr.push((0:b32)) // handle = _c8031, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c7bf1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let push_c8011 = sys.bind_arg(_c8011, "fetch_addr".into(), imm_c7bf1);
    // _c8011 = _ba7f9.bind([_c801d /* _ba7f9.signals=(0:b97) */, _c8031 /* _ba7f9.fetch_addr=(0:b32) */]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    // Already handled by `EmitBinds` {}
    // async_call _c8011, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    sys.create_async_call(_c8011);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _c803d = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c8019 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c803d = sys.create_array_read(array_ba53d, imm_c8019);
    // _c8039 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c8045 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c8039 = sys.create_array_read(array_ba53d, imm_c8045);
    // _c8049 = _c8039[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c8059 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c8061 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c8049 = sys.create_slice(_c8039, imm_c8059, imm_c8061);
    // _c805d = _c8049 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c7bc1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c805d = sys.create_eq(_c8049, imm_c7bc1);
    // _c806d = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c8075 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c806d = sys.create_array_read(array_ba53d, imm_c8075);
    // _c8079 = _c806d[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c8089 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c8091 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c8079 = sys.create_slice(_c806d, imm_c8089, imm_c8091);
    // _c8099 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c808d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c8099 = sys.create_array_read(array_ba53d, imm_c808d);
    // _c809d = _c8099[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c80ad = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c80b5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c809d = sys.create_slice(_c8099, imm_c80ad, imm_c80b5);
    // _c8015 = ~(1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:224
    let imm_c7b09 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _c8015 = sys.create_flip(imm_c7b09);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c80b1 = sys.create_conditional_block(_c8015);
    sys.set_current_block(_c80b1);
    // _c80d1 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:225
    let imm_c80d5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c80d1 = sys.create_array_read(array_ba53d, imm_c80d5);
    // _c80d9 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c80e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c80d9 = sys.create_array_read(array_ba53d, imm_c80e1);
    // _c80e5 = _c80d9[(32:u6):(128:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c80f5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x20 as u64); // (32:u6)
    let imm_c80fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x80 as u64); // (128:u8)
    let _c80e5 = sys.create_slice(_c80d9, imm_c80f5, imm_c80fd);
    // _c8105 = bitcast _c80e5 to record { mem_ext: b2, link_pc: b1, is_offset_br: b1, is_branch: b1, flip: b1, cond: b16, alu: b16, memory: b2, imm_valid: b1, imm: b32, is_pc_calc: b1, is_mepc: b1, is_zimm: b1, csr_calculate: b1, csr_write: b1, csr_read: b1, rd_valid: b1, rd: b5, rs2_valid: b1, rs2: b5, rs1_valid: b1, rs1: b5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let _c8105 = sys.create_bitcast(_c80e5, assassyn::ir::DataType::bits_ty(97));
    // _c8109 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c80f9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c8109 = sys.create_array_read(array_ba53d, imm_c80f9);
    // _c8115 = _c8109[(0:u1):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c8121 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c8129 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _c8115 = sys.create_slice(_c8109, imm_c8121, imm_c8129);
    // _c8131 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c8125 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c8131 = sys.create_array_read(array_ba53d, imm_c8125);
    // _c8135 = _c8131[(173:u8):(204:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c8145 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xad as u64); // (173:u8)
    let imm_c814d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcc as u64); // (204:u8)
    let _c8135 = sys.create_slice(_c8131, imm_c8145, imm_c814d);
    // _c8155 = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c8149 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c8155 = sys.create_array_read(array_ba53d, imm_c8149);
    // _c815d = _c8155[(141:u8):(172:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c8165 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8d as u64); // (141:u8)
    let imm_c816d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xac as u64); // (172:u8)
    let _c815d = sys.create_slice(_c8155, imm_c8165, imm_c816d);
    // _ba7f9.signals.push((0:b97)) // handle = _c8191, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c8175 = sys.get_const_int(assassyn::ir::DataType::bits_ty(97), 0x0 as u64); // (0:b97)
    let push_c8185 = sys.bind_arg(_c8185, "signals".into(), imm_c8175);
    // _ba7f9.fetch_addr.push((0:b32)) // handle = _c81a5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c8179 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let push_c8185 = sys.bind_arg(_c8185, "fetch_addr".into(), imm_c8179);
    // _c8185 = _ba7f9.bind([_c8191 /* _ba7f9.signals=(0:b97) */, _c81a5 /* _ba7f9.fetch_addr=(0:b32) */]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    // Already handled by `EmitBinds` {}
    // async_call _c8185, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    sys.create_async_call(_c8185);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _c81b1 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c818d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c81b1 = sys.create_array_read(array_ba53d, imm_c818d);
    // _c81ad = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c81b9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c81ad = sys.create_array_read(array_ba53d, imm_c81b9);
    // _c81bd = _c81ad[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c81cd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c81d5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c81bd = sys.create_slice(_c81ad, imm_c81cd, imm_c81d5);
    // _c81d1 = _c81bd == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c81dd = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c81d1 = sys.create_eq(_c81bd, imm_c81dd);
    // _c81e1 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c81e9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c81e1 = sys.create_array_read(array_ba53d, imm_c81e9);
    // _c81ed = _c81e1[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c81fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c8205 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c81ed = sys.create_slice(_c81e1, imm_c81fd, imm_c8205);
    // _c820d = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c8201 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c820d = sys.create_array_read(array_ba53d, imm_c8201);
    // _c8211 = _c820d[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c8221 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c8229 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c8211 = sys.create_slice(_c820d, imm_c8221, imm_c8229);
    // _c8189 = ~(1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:224
    let imm_c807d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _c8189 = sys.create_flip(imm_c807d);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c8225 = sys.create_conditional_block(_c8189);
    sys.set_current_block(_c8225);
    // _c8245 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:225
    let imm_c8249 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c8245 = sys.create_array_read(array_ba53d, imm_c8249);
    // _c824d = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c8255 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c824d = sys.create_array_read(array_ba53d, imm_c8255);
    // _c8259 = _c824d[(32:u6):(128:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c8269 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x20 as u64); // (32:u6)
    let imm_c8271 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x80 as u64); // (128:u8)
    let _c8259 = sys.create_slice(_c824d, imm_c8269, imm_c8271);
    // _c8279 = bitcast _c8259 to record { mem_ext: b2, link_pc: b1, is_offset_br: b1, is_branch: b1, flip: b1, cond: b16, alu: b16, memory: b2, imm_valid: b1, imm: b32, is_pc_calc: b1, is_mepc: b1, is_zimm: b1, csr_calculate: b1, csr_write: b1, csr_read: b1, rd_valid: b1, rd: b5, rs2_valid: b1, rs2: b5, rs1_valid: b1, rs1: b5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let _c8279 = sys.create_bitcast(_c8259, assassyn::ir::DataType::bits_ty(97));
    // _c827d = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c826d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c827d = sys.create_array_read(array_ba53d, imm_c826d);
    // _c8289 = _c827d[(0:u1):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c8295 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c829d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _c8289 = sys.create_slice(_c827d, imm_c8295, imm_c829d);
    // _c82a5 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c8299 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c82a5 = sys.create_array_read(array_ba53d, imm_c8299);
    // _c82a9 = _c82a5[(173:u8):(204:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c82b9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xad as u64); // (173:u8)
    let imm_c82c1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcc as u64); // (204:u8)
    let _c82a9 = sys.create_slice(_c82a5, imm_c82b9, imm_c82c1);
    // _c82c9 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c82bd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c82c9 = sys.create_array_read(array_ba53d, imm_c82bd);
    // _c82d1 = _c82c9[(141:u8):(172:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c82d9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8d as u64); // (141:u8)
    let imm_c82e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xac as u64); // (172:u8)
    let _c82d1 = sys.create_slice(_c82c9, imm_c82d9, imm_c82e1);
    // _ba7f9.signals.push((0:b97)) // handle = _c8305, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c82e9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(97), 0x0 as u64); // (0:b97)
    let push_c82f9 = sys.bind_arg(_c82f9, "signals".into(), imm_c82e9);
    // _ba7f9.fetch_addr.push((0:b32)) // handle = _c8319, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c82ed = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let push_c82f9 = sys.bind_arg(_c82f9, "fetch_addr".into(), imm_c82ed);
    // _c82f9 = _ba7f9.bind([_c8305 /* _ba7f9.signals=(0:b97) */, _c8319 /* _ba7f9.fetch_addr=(0:b32) */]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    // Already handled by `EmitBinds` {}
    // async_call _c82f9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    sys.create_async_call(_c82f9);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _c8325 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c8301 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c8325 = sys.create_array_read(array_ba53d, imm_c8301);
    // _c8321 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c832d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c8321 = sys.create_array_read(array_ba53d, imm_c832d);
    // _c8331 = _c8321[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c8341 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c8349 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c8331 = sys.create_slice(_c8321, imm_c8341, imm_c8349);
    // _c8345 = _c8331 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c8351 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c8345 = sys.create_eq(_c8331, imm_c8351);
    // _c8355 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c835d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c8355 = sys.create_array_read(array_ba53d, imm_c835d);
    // _c8361 = _c8355[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c8371 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c8379 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c8361 = sys.create_slice(_c8355, imm_c8371, imm_c8379);
    // _c8381 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c8375 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c8381 = sys.create_array_read(array_ba53d, imm_c8375);
    // _c8385 = _c8381[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c8395 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c839d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c8385 = sys.create_slice(_c8381, imm_c8395, imm_c839d);
    // _c82fd = ~(1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:224
    let imm_c81f1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _c82fd = sys.create_flip(imm_c81f1);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c8399 = sys.create_conditional_block(_c82fd);
    sys.set_current_block(_c8399);
    // _c83b9 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:225
    let imm_c83bd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c83b9 = sys.create_array_read(array_ba53d, imm_c83bd);
    // _c83c1 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c83c9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c83c1 = sys.create_array_read(array_ba53d, imm_c83c9);
    // _c83cd = _c83c1[(32:u6):(128:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c83dd = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x20 as u64); // (32:u6)
    let imm_c83e5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x80 as u64); // (128:u8)
    let _c83cd = sys.create_slice(_c83c1, imm_c83dd, imm_c83e5);
    // _c83ed = bitcast _c83cd to record { mem_ext: b2, link_pc: b1, is_offset_br: b1, is_branch: b1, flip: b1, cond: b16, alu: b16, memory: b2, imm_valid: b1, imm: b32, is_pc_calc: b1, is_mepc: b1, is_zimm: b1, csr_calculate: b1, csr_write: b1, csr_read: b1, rd_valid: b1, rd: b5, rs2_valid: b1, rs2: b5, rs1_valid: b1, rs1: b5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let _c83ed = sys.create_bitcast(_c83cd, assassyn::ir::DataType::bits_ty(97));
    // _c83f1 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c83e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c83f1 = sys.create_array_read(array_ba53d, imm_c83e1);
    // _c83fd = _c83f1[(0:u1):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c8c0d = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c8c15 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _c83fd = sys.create_slice(_c83f1, imm_c8c0d, imm_c8c15);
    // _c8c1d = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c8c11 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c8c1d = sys.create_array_read(array_ba53d, imm_c8c11);
    // _c8c21 = _c8c1d[(173:u8):(204:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c8c31 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xad as u64); // (173:u8)
    let imm_c8c39 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcc as u64); // (204:u8)
    let _c8c21 = sys.create_slice(_c8c1d, imm_c8c31, imm_c8c39);
    // _c8c41 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c8c35 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c8c41 = sys.create_array_read(array_ba53d, imm_c8c35);
    // _c8c49 = _c8c41[(141:u8):(172:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c8c51 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8d as u64); // (141:u8)
    let imm_c8c59 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xac as u64); // (172:u8)
    let _c8c49 = sys.create_slice(_c8c41, imm_c8c51, imm_c8c59);
    // _ba7f9.signals.push((0:b97)) // handle = _c8c7d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c8389 = sys.get_const_int(assassyn::ir::DataType::bits_ty(97), 0x0 as u64); // (0:b97)
    let push_c8c71 = sys.bind_arg(_c8c71, "signals".into(), imm_c8389);
    // _ba7f9.fetch_addr.push((0:b32)) // handle = _c8c91, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c8c65 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let push_c8c71 = sys.bind_arg(_c8c71, "fetch_addr".into(), imm_c8c65);
    // _c8c71 = _ba7f9.bind([_c8c7d /* _ba7f9.signals=(0:b97) */, _c8c91 /* _ba7f9.fetch_addr=(0:b32) */]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    // Already handled by `EmitBinds` {}
    // async_call _c8c71, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    sys.create_async_call(_c8c71);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _c8c9d = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c8c79 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c8c9d = sys.create_array_read(array_ba53d, imm_c8c79);
    // _c8c99 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c8ca5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c8c99 = sys.create_array_read(array_ba53d, imm_c8ca5);
    // _c8ca9 = _c8c99[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c8cb9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c8cc1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c8ca9 = sys.create_slice(_c8c99, imm_c8cb9, imm_c8cc1);
    // _c8cbd = _c8ca9 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c8cc9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c8cbd = sys.create_eq(_c8ca9, imm_c8cc9);
    // _c8ccd = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c8cd5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c8ccd = sys.create_array_read(array_ba53d, imm_c8cd5);
    // _c8cd9 = _c8ccd[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c8ce9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c8cf1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c8cd9 = sys.create_slice(_c8ccd, imm_c8ce9, imm_c8cf1);
    // _c8cf9 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c8ced = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c8cf9 = sys.create_array_read(array_ba53d, imm_c8ced);
    // _c8cfd = _c8cf9[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c8d0d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c8d15 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c8cfd = sys.create_slice(_c8cf9, imm_c8d0d, imm_c8d15);
    // _c8c75 = ~(1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:224
    let imm_c8365 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _c8c75 = sys.create_flip(imm_c8365);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c8d11 = sys.create_conditional_block(_c8c75);
    sys.set_current_block(_c8d11);
    // _c8d31 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:225
    let imm_c8d35 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c8d31 = sys.create_array_read(array_ba53d, imm_c8d35);
    // _c8d39 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c8d41 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c8d39 = sys.create_array_read(array_ba53d, imm_c8d41);
    // _c8d45 = _c8d39[(32:u6):(128:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c8d55 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x20 as u64); // (32:u6)
    let imm_c8d5d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x80 as u64); // (128:u8)
    let _c8d45 = sys.create_slice(_c8d39, imm_c8d55, imm_c8d5d);
    // _c8d65 = bitcast _c8d45 to record { mem_ext: b2, link_pc: b1, is_offset_br: b1, is_branch: b1, flip: b1, cond: b16, alu: b16, memory: b2, imm_valid: b1, imm: b32, is_pc_calc: b1, is_mepc: b1, is_zimm: b1, csr_calculate: b1, csr_write: b1, csr_read: b1, rd_valid: b1, rd: b5, rs2_valid: b1, rs2: b5, rs1_valid: b1, rs1: b5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let _c8d65 = sys.create_bitcast(_c8d45, assassyn::ir::DataType::bits_ty(97));
    // _c8d69 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c8d59 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c8d69 = sys.create_array_read(array_ba53d, imm_c8d59);
    // _c8d75 = _c8d69[(0:u1):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c8d81 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c8d89 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _c8d75 = sys.create_slice(_c8d69, imm_c8d81, imm_c8d89);
    // _c8d91 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c8d85 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c8d91 = sys.create_array_read(array_ba53d, imm_c8d85);
    // _c8d95 = _c8d91[(173:u8):(204:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c8da5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xad as u64); // (173:u8)
    let imm_c8dad = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcc as u64); // (204:u8)
    let _c8d95 = sys.create_slice(_c8d91, imm_c8da5, imm_c8dad);
    // _c8db5 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c8da9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c8db5 = sys.create_array_read(array_ba53d, imm_c8da9);
    // _c8dbd = _c8db5[(141:u8):(172:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c8dc5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8d as u64); // (141:u8)
    let imm_c8dcd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xac as u64); // (172:u8)
    let _c8dbd = sys.create_slice(_c8db5, imm_c8dc5, imm_c8dcd);
    // _ba7f9.signals.push((0:b97)) // handle = _c8df1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c8dd5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(97), 0x0 as u64); // (0:b97)
    let push_c8de5 = sys.bind_arg(_c8de5, "signals".into(), imm_c8dd5);
    // _ba7f9.fetch_addr.push((0:b32)) // handle = _c8e05, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c8dd9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let push_c8de5 = sys.bind_arg(_c8de5, "fetch_addr".into(), imm_c8dd9);
    // _c8de5 = _ba7f9.bind([_c8df1 /* _ba7f9.signals=(0:b97) */, _c8e05 /* _ba7f9.fetch_addr=(0:b32) */]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    // Already handled by `EmitBinds` {}
    // async_call _c8de5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    sys.create_async_call(_c8de5);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _c8e11 = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c8ded = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c8e11 = sys.create_array_read(array_ba53d, imm_c8ded);
    // _c8e0d = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c8e19 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c8e0d = sys.create_array_read(array_ba53d, imm_c8e19);
    // _c8e1d = _c8e0d[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c8e2d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c8e35 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c8e1d = sys.create_slice(_c8e0d, imm_c8e2d, imm_c8e35);
    // _c8e31 = _c8e1d == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c8e3d = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c8e31 = sys.create_eq(_c8e1d, imm_c8e3d);
    // _c8e41 = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c8e49 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c8e41 = sys.create_array_read(array_ba53d, imm_c8e49);
    // _c8e4d = _c8e41[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c8e5d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c8e65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c8e4d = sys.create_slice(_c8e41, imm_c8e5d, imm_c8e65);
    // _c8e6d = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c8e61 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c8e6d = sys.create_array_read(array_ba53d, imm_c8e61);
    // _c8e71 = _c8e6d[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c8e81 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c8e89 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c8e71 = sys.create_slice(_c8e6d, imm_c8e81, imm_c8e89);
    // _c8de9 = ~(1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:224
    let imm_c8cdd = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _c8de9 = sys.create_flip(imm_c8cdd);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c8e85 = sys.create_conditional_block(_c8de9);
    sys.set_current_block(_c8e85);
    // _c8ea5 = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:225
    let imm_c8ea9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c8ea5 = sys.create_array_read(array_ba53d, imm_c8ea9);
    // _c8ead = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c8eb5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c8ead = sys.create_array_read(array_ba53d, imm_c8eb5);
    // _c8eb9 = _c8ead[(32:u6):(128:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c8ec9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x20 as u64); // (32:u6)
    let imm_c8ed1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x80 as u64); // (128:u8)
    let _c8eb9 = sys.create_slice(_c8ead, imm_c8ec9, imm_c8ed1);
    // _c8ed9 = bitcast _c8eb9 to record { mem_ext: b2, link_pc: b1, is_offset_br: b1, is_branch: b1, flip: b1, cond: b16, alu: b16, memory: b2, imm_valid: b1, imm: b32, is_pc_calc: b1, is_mepc: b1, is_zimm: b1, csr_calculate: b1, csr_write: b1, csr_read: b1, rd_valid: b1, rd: b5, rs2_valid: b1, rs2: b5, rs1_valid: b1, rs1: b5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let _c8ed9 = sys.create_bitcast(_c8eb9, assassyn::ir::DataType::bits_ty(97));
    // _c8edd = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c8ecd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c8edd = sys.create_array_read(array_ba53d, imm_c8ecd);
    // _c8ee9 = _c8edd[(0:u1):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c8ef5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c8efd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _c8ee9 = sys.create_slice(_c8edd, imm_c8ef5, imm_c8efd);
    // _c8f05 = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c8ef9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c8f05 = sys.create_array_read(array_ba53d, imm_c8ef9);
    // _c8f09 = _c8f05[(173:u8):(204:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c8f19 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xad as u64); // (173:u8)
    let imm_c8f21 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcc as u64); // (204:u8)
    let _c8f09 = sys.create_slice(_c8f05, imm_c8f19, imm_c8f21);
    // _c8f29 = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c8f1d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c8f29 = sys.create_array_read(array_ba53d, imm_c8f1d);
    // _c8f31 = _c8f29[(141:u8):(172:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c8f39 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8d as u64); // (141:u8)
    let imm_c8f41 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xac as u64); // (172:u8)
    let _c8f31 = sys.create_slice(_c8f29, imm_c8f39, imm_c8f41);
    // _ba7f9.signals.push((0:b97)) // handle = _c8f65, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c8f49 = sys.get_const_int(assassyn::ir::DataType::bits_ty(97), 0x0 as u64); // (0:b97)
    let push_c8f59 = sys.bind_arg(_c8f59, "signals".into(), imm_c8f49);
    // _ba7f9.fetch_addr.push((0:b32)) // handle = _c8f79, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c8f4d = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let push_c8f59 = sys.bind_arg(_c8f59, "fetch_addr".into(), imm_c8f4d);
    // _c8f59 = _ba7f9.bind([_c8f65 /* _ba7f9.signals=(0:b97) */, _c8f79 /* _ba7f9.fetch_addr=(0:b32) */]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    // Already handled by `EmitBinds` {}
    // async_call _c8f59, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    sys.create_async_call(_c8f59);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _c8f85 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c8f61 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c8f85 = sys.create_array_read(array_ba53d, imm_c8f61);
    // _c8f81 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c8f8d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c8f81 = sys.create_array_read(array_ba53d, imm_c8f8d);
    // _c8f91 = _c8f81[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c8fa1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c8fa9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c8f91 = sys.create_slice(_c8f81, imm_c8fa1, imm_c8fa9);
    // _c8fa5 = _c8f91 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c8fb1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c8fa5 = sys.create_eq(_c8f91, imm_c8fb1);
    // _c8fb5 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c8fbd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c8fb5 = sys.create_array_read(array_ba53d, imm_c8fbd);
    // _c8fc1 = _c8fb5[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c8fd1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c8fd9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c8fc1 = sys.create_slice(_c8fb5, imm_c8fd1, imm_c8fd9);
    // _c8fe1 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c8fd5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c8fe1 = sys.create_array_read(array_ba53d, imm_c8fd5);
    // _c8fe5 = _c8fe1[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c8ff5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c8ffd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c8fe5 = sys.create_slice(_c8fe1, imm_c8ff5, imm_c8ffd);
    // _c6fc5 = ~(1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:224
    let imm_c8e51 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _c6fc5 = sys.create_flip(imm_c8e51);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c9805 = sys.create_conditional_block(_c6fc5);
    sys.set_current_block(_c9805);
    // _c981d = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:225
    let imm_c9821 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c981d = sys.create_array_read(array_ba53d, imm_c9821);
    // _c9825 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c982d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c9825 = sys.create_array_read(array_ba53d, imm_c982d);
    // _c9831 = _c9825[(32:u6):(128:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c9841 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x20 as u64); // (32:u6)
    let imm_c9849 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x80 as u64); // (128:u8)
    let _c9831 = sys.create_slice(_c9825, imm_c9841, imm_c9849);
    // _c9851 = bitcast _c9831 to record { mem_ext: b2, link_pc: b1, is_offset_br: b1, is_branch: b1, flip: b1, cond: b16, alu: b16, memory: b2, imm_valid: b1, imm: b32, is_pc_calc: b1, is_mepc: b1, is_zimm: b1, csr_calculate: b1, csr_write: b1, csr_read: b1, rd_valid: b1, rd: b5, rs2_valid: b1, rs2: b5, rs1_valid: b1, rs1: b5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let _c9851 = sys.create_bitcast(_c9831, assassyn::ir::DataType::bits_ty(97));
    // _c9855 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c9845 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c9855 = sys.create_array_read(array_ba53d, imm_c9845);
    // _c9861 = _c9855[(0:u1):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c986d = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c9875 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _c9861 = sys.create_slice(_c9855, imm_c986d, imm_c9875);
    // _c987d = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c9871 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c987d = sys.create_array_read(array_ba53d, imm_c9871);
    // _c9881 = _c987d[(173:u8):(204:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c9891 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xad as u64); // (173:u8)
    let imm_c9899 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcc as u64); // (204:u8)
    let _c9881 = sys.create_slice(_c987d, imm_c9891, imm_c9899);
    // _c98a1 = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c9895 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c98a1 = sys.create_array_read(array_ba53d, imm_c9895);
    // _c98a9 = _c98a1[(141:u8):(172:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c98b1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8d as u64); // (141:u8)
    let imm_c98b9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xac as u64); // (172:u8)
    let _c98a9 = sys.create_slice(_c98a1, imm_c98b1, imm_c98b9);
    // _ba7f9.signals.push((0:b97)) // handle = _c98dd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c8fe9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(97), 0x0 as u64); // (0:b97)
    let push_c98d1 = sys.bind_arg(_c98d1, "signals".into(), imm_c8fe9);
    // _ba7f9.fetch_addr.push((0:b32)) // handle = _c98f1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c98c5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let push_c98d1 = sys.bind_arg(_c98d1, "fetch_addr".into(), imm_c98c5);
    // _c98d1 = _ba7f9.bind([_c98dd /* _ba7f9.signals=(0:b97) */, _c98f1 /* _ba7f9.fetch_addr=(0:b32) */]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    // Already handled by `EmitBinds` {}
    // async_call _c98d1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    sys.create_async_call(_c98d1);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _c98fd = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c98d9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c98fd = sys.create_array_read(array_ba53d, imm_c98d9);
    // _c98f9 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c9905 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c98f9 = sys.create_array_read(array_ba53d, imm_c9905);
    // _c9909 = _c98f9[(129:u8):(130:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c9919 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x81 as u64); // (129:u8)
    let imm_c9921 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x82 as u64); // (130:u8)
    let _c9909 = sys.create_slice(_c98f9, imm_c9919, imm_c9921);
    // _c991d = _c9909 == (0:b2), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:222
    let imm_c9929 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _c991d = sys.create_eq(_c9909, imm_c9929);
    // _c992d = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c9935 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c992d = sys.create_array_read(array_ba53d, imm_c9935);
    // _c9939 = _c992d[(206:u8):(206:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c9949 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let imm_c9951 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xce as u64); // (206:u8)
    let _c9939 = sys.create_slice(_c992d, imm_c9949, imm_c9951);
    // _c9959 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c994d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c9959 = sys.create_array_read(array_ba53d, imm_c994d);
    // _c995d = _c9959[(205:u8):(205:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:223
    let imm_c996d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let imm_c9975 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcd as u64); // (205:u8)
    let _c995d = sys.create_slice(_c9959, imm_c996d, imm_c9975);
    // _c98d5 = ~(1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:224
    let imm_c8fc5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _c98d5 = sys.create_flip(imm_c8fc5);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c9971 = sys.create_conditional_block(_c98d5);
    sys.set_current_block(_c9971);
    // _c9991 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:225
    let imm_c9995 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c9991 = sys.create_array_read(array_ba53d, imm_c9995);
    // _c9999 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c99a1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c9999 = sys.create_array_read(array_ba53d, imm_c99a1);
    // _c99a5 = _c9999[(32:u6):(128:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let imm_c99b5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(6), 0x20 as u64); // (32:u6)
    let imm_c99bd = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x80 as u64); // (128:u8)
    let _c99a5 = sys.create_slice(_c9999, imm_c99b5, imm_c99bd);
    // _c99c5 = bitcast _c99a5 to record { mem_ext: b2, link_pc: b1, is_offset_br: b1, is_branch: b1, flip: b1, cond: b16, alu: b16, memory: b2, imm_valid: b1, imm: b32, is_pc_calc: b1, is_mepc: b1, is_zimm: b1, csr_calculate: b1, csr_write: b1, csr_read: b1, rd_valid: b1, rd: b5, rs2_valid: b1, rs2: b5, rs1_valid: b1, rs1: b5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:227
    let _c99c5 = sys.create_bitcast(_c99a5, assassyn::ir::DataType::bits_ty(97));
    // _c99c9 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c99b9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c99c9 = sys.create_array_read(array_ba53d, imm_c99b9);
    // _c99d5 = _c99c9[(0:u1):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:228
    let imm_c99e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_c99e9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _c99d5 = sys.create_slice(_c99c9, imm_c99e1, imm_c99e9);
    // _c99f1 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c99e5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c99f1 = sys.create_array_read(array_ba53d, imm_c99e5);
    // _c99f5 = _c99f1[(173:u8):(204:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:229
    let imm_c9a05 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xad as u64); // (173:u8)
    let imm_c9a0d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xcc as u64); // (204:u8)
    let _c99f5 = sys.create_slice(_c99f1, imm_c9a05, imm_c9a0d);
    // _c9a15 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c9a09 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c9a15 = sys.create_array_read(array_ba53d, imm_c9a09);
    // _c9a1d = _c9a15[(141:u8):(172:u8)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:230
    let imm_c9a25 = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0x8d as u64); // (141:u8)
    let imm_c9a2d = sys.get_const_int(assassyn::ir::DataType::uint_ty(8), 0xac as u64); // (172:u8)
    let _c9a1d = sys.create_slice(_c9a15, imm_c9a25, imm_c9a2d);
    // _ba7f9.signals.push((0:b97)) // handle = _c9a51, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c9a35 = sys.get_const_int(assassyn::ir::DataType::bits_ty(97), 0x0 as u64); // (0:b97)
    let push_c9a45 = sys.bind_arg(_c9a45, "signals".into(), imm_c9a35);
    // _ba7f9.fetch_addr.push((0:b32)) // handle = _c9a65, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    let imm_c9a39 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let push_c9a45 = sys.bind_arg(_c9a45, "fetch_addr".into(), imm_c9a39);
    // _c9a45 = _ba7f9.bind([_c9a51 /* _ba7f9.signals=(0:b97) */, _c9a65 /* _ba7f9.fetch_addr=(0:b32) */]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    // Already handled by `EmitBinds` {}
    // async_call _c9a45, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:233
    sys.create_async_call(_c9a45);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // side effect intrinsic.wait_until({'(1:b1)'}), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:237
    let imm_c993d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    sys.create_wait_until(imm_c993d);
    // Fill in the body of _ba7b9
    sys.set_current_module(_ba7b9);
    // module root block
    // _c9a75 = _ba7b9.rdata.pop(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:252
    // Get port rdata
    let _ba7b9_rdata = {
        let module = _ba7b9.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("rdata").unwrap().upcast()
    };
    let _c9a75 = sys.create_fifo_pop(_ba7b9_rdata);
    // _c9a79 = _ba7b9.fetch_addr.pop(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:252
    // Get port fetch_addr
    let _ba7b9_fetch_addr = {
        let module = _ba7b9.as_ref::<assassyn::ir::Module>(&sys).unwrap();
        module.get_fifo("fetch_addr").unwrap().upcast()
    };
    let _c9a79 = sys.create_fifo_pop(_ba7b9_fetch_addr);
    // log('raw: 0x{:08x}  | addr: 0x{:05x} |', _c9a75, _c9a79), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:254
    let fmt = sys.get_str_literal("raw: 0x{:08x}  | addr: 0x{:05x} |".into());
    sys.create_log(fmt, vec![_c9a75, _c9a79]);
    // _ca075 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:156
    let imm_ca085 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_ca08d = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _ca075 = sys.create_slice(_c9a75, imm_ca085, imm_ca08d);
    // _ca0e9 = _ca075 == (111:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:156
    let imm_ca0a1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x6f as u64); // (111:b7)
    let _ca0e9 = sys.create_eq(_ca075, imm_ca0a1);
    // _ca061 = (0:b1) | _ca0e9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let imm_ca01d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _ca061 = sys.create_bitwise_or(imm_ca01d, _ca0e9);
    // _ca121 = (0:b1) | _ca0e9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let imm_ca04d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _ca121 = sys.create_bitwise_or(imm_ca04d, _ca0e9);
    // _ca135 = _ca0e9 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_ca119 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_ca131 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ca135 = sys.create_select(_ca0e9, imm_ca119, imm_ca131);
    // _ca13d = (0:b16) | _ca135, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_ca055 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ca13d = sys.create_bitwise_or(imm_ca055, _ca135);
    // _ca14d = _ca0e9 ? (2048:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_ca129 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x800 as u64); // (2048:b16)
    let imm_ca145 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ca14d = sys.create_select(_ca0e9, imm_ca129, imm_ca145);
    // _ca155 = (0:b16) | _ca14d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_ca05d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ca155 = sys.create_bitwise_or(imm_ca05d, _ca14d);
    // _ca169 = _ca0e9 ? (False:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_ca125 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (False:b1)
    let imm_ca165 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _ca169 = sys.create_select(_ca0e9, imm_ca125, imm_ca165);
    // _ca171 = (0:b1) | _ca169, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_ca065 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _ca171 = sys.create_bitwise_or(imm_ca065, _ca169);
    // _ca18d = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_ca199 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_ca1a1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _ca18d = sys.create_slice(_c9a75, imm_ca199, imm_ca1a1);
    // _ca185 = (0:b1) | _ca0e9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let imm_ca02d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _ca185 = sys.create_bitwise_or(imm_ca02d, _ca0e9);
    // _ca1b1 = _c9a75[(31:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:161
    let imm_ca1bd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let imm_ca1c5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _ca1b1 = sys.create_slice(_c9a75, imm_ca1bd, imm_ca1c5);
    // _ca1d1 = _c9a75[(12:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:161
    let imm_ca1dd = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_ca1e5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _ca1d1 = sys.create_slice(_c9a75, imm_ca1dd, imm_ca1e5);
    // _ca1e1 = _c9a75[(20:u5):(20:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:161
    let imm_ca1f9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_ca201 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let _ca1e1 = sys.create_slice(_c9a75, imm_ca1f9, imm_ca201);
    // _ca1fd = _c9a75[(21:u5):(30:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:161
    let imm_ca215 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x15 as u64); // (21:u5)
    let imm_ca21d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1e as u64); // (30:u5)
    let _ca1fd = sys.create_slice(_c9a75, imm_ca215, imm_ca21d);
    // _ca22d = { _ca1b1 _ca1d1 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:161
    let _ca22d = sys.create_concat(_ca1b1, _ca1d1);
    // _ca239 = { _ca22d _ca1e1 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:161
    let _ca239 = sys.create_concat(_ca22d, _ca1e1);
    // _ca23d = { _ca239 _ca1fd }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:161
    let _ca23d = sys.create_concat(_ca239, _ca1fd);
    // _ca241 = { _ca23d (0:b1) }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:161
    let imm_ca219 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _ca241 = sys.create_concat(_ca23d, imm_ca219);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _ca209 = sys.create_conditional_block(_ca0e9);
    sys.set_current_block(_ca209);
    // log('j.jal.1101111    | rd: x{:02}      |               |               |imm: 0x{:x}', _ca18d, _ca241), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "j.jal.1101111    | rd: x{:02}      |               |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_ca18d, _ca241]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _ca259 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:132
    let imm_ca265 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_ca26d = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _ca259 = sys.create_slice(_c9a75, imm_ca265, imm_ca26d);
    // _ca269 = _ca259 == (55:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:132
    let imm_ca275 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x37 as u64); // (55:b7)
    let _ca269 = sys.create_eq(_ca259, imm_ca275);
    // _ca249 = (0:b1) | _ca269, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let imm_ca015 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _ca249 = sys.create_bitwise_or(imm_ca015, _ca269);
    // _ca28d = _ca121 | _ca269, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _ca28d = sys.create_bitwise_or(_ca121, _ca269);
    // _ca299 = _ca269 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_ca289 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_ca149 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ca299 = sys.create_select(_ca269, imm_ca289, imm_ca149);
    // _ca2a5 = _ca13d | _ca299, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _ca2a5 = sys.create_bitwise_or(_ca13d, _ca299);
    // _ca2ad = _ca269 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_ca285 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_ca2a1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ca2ad = sys.create_select(_ca269, imm_ca285, imm_ca2a1);
    // _ca2b9 = _ca155 | _ca2ad, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _ca2b9 = sys.create_bitwise_or(_ca155, _ca2ad);
    // _ca2c1 = _ca269 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_ca291 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_ca2b5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _ca2c1 = sys.create_select(_ca269, imm_ca291, imm_ca2b5);
    // _ca2cd = _ca171 | _ca2c1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _ca2cd = sys.create_bitwise_or(_ca171, _ca2c1);
    // _ca2d5 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_ca2e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_ca2e9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _ca2d5 = sys.create_slice(_c9a75, imm_ca2e1, imm_ca2e9);
    // _ca17d = _ca185 | _ca269, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _ca17d = sys.create_bitwise_or(_ca185, _ca269);
    // _ca2e5 = _c9a75[(12:u4):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:136
    let imm_ca2fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_ca305 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _ca2e5 = sys.create_slice(_c9a75, imm_ca2fd, imm_ca305);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _ca2f1 = sys.create_conditional_block(_ca269);
    sys.set_current_block(_ca2f1);
    // log('u.lui.0110111    | rd: x{:02}      |               |               |imm: 0x{:x}', _ca2d5, _ca2e5), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "u.lui.0110111    | rd: x{:02}      |               |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_ca2d5, _ca2e5]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _ca321 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:53
    let imm_ca32d = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_ca335 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _ca321 = sys.create_slice(_c9a75, imm_ca32d, imm_ca335);
    // _ca331 = _ca321 == (51:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:53
    let imm_ca33d = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x33 as u64); // (51:b7)
    let _ca331 = sys.create_eq(_ca321, imm_ca33d);
    // _ca341 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:54
    let imm_ca351 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_ca359 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _ca341 = sys.create_slice(_c9a75, imm_ca351, imm_ca359);
    // _ca355 = _ca341 == (0:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:54
    let imm_ca361 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x0 as u64); // (0:b3)
    let _ca355 = sys.create_eq(_ca341, imm_ca361);
    // _ca365 = _c9a75[(25:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:55
    let imm_ca375 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x19 as u64); // (25:u5)
    let imm_ca37d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _ca365 = sys.create_slice(_c9a75, imm_ca375, imm_ca37d);
    // _ca379 = _ca365 == (0:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:55
    let imm_ca385 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x0 as u64); // (0:b7)
    let _ca379 = sys.create_eq(_ca365, imm_ca385);
    // _ca391 = _ca331 & _ca355, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let _ca391 = sys.create_bitwise_and(_ca331, _ca355);
    // _ca399 = _ca391 & _ca379, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let _ca399 = sys.create_bitwise_and(_ca391, _ca379);
    // _ca39d = _ca399 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let imm_ca389 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _ca39d = sys.create_bitwise_and(_ca399, imm_ca389);
    // _ca311 = (0:b1) | _ca39d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let imm_c9bf5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _ca311 = sys.create_bitwise_or(imm_c9bf5, _ca39d);
    // _ca3b1 = _ca28d | _ca39d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _ca3b1 = sys.create_bitwise_or(_ca28d, _ca39d);
    // _ca3bd = _ca39d ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_ca3ad = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_ca19d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ca3bd = sys.create_select(_ca39d, imm_ca3ad, imm_ca19d);
    // _ca3c9 = _ca2a5 | _ca3bd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _ca3c9 = sys.create_bitwise_or(_ca2a5, _ca3bd);
    // _ca3d1 = _ca39d ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_ca3a9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_ca3c5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ca3d1 = sys.create_select(_ca39d, imm_ca3a9, imm_ca3c5);
    // _ca3dd = _ca2b9 | _ca3d1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _ca3dd = sys.create_bitwise_or(_ca2b9, _ca3d1);
    // _ca3e5 = _ca39d ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_ca3b5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_ca3d9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _ca3e5 = sys.create_select(_ca39d, imm_ca3b5, imm_ca3d9);
    // _ca3f1 = _ca2cd | _ca3e5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _ca3f1 = sys.create_bitwise_or(_ca2cd, _ca3e5);
    // _ca3f9 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cac09 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cac11 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _ca3f9 = sys.create_slice(_c9a75, imm_cac09, imm_cac11);
    // _ca181 = _ca17d | _ca39d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _ca181 = sys.create_bitwise_or(_ca17d, _ca39d);
    // _cac0d = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cac25 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cac2d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cac0d = sys.create_slice(_c9a75, imm_cac25, imm_cac2d);
    // _ca3b9 = (0:b1) | _ca39d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let imm_ca035 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _ca3b9 = sys.create_bitwise_or(imm_ca035, _ca39d);
    // _cac29 = _c9a75[(20:u5):(24:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:65
    let imm_cac41 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cac49 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x18 as u64); // (24:u5)
    let _cac29 = sys.create_slice(_c9a75, imm_cac41, imm_cac49);
    // _ca3cd = (0:b1) | _ca39d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:66
    let imm_ca03d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _ca3cd = sys.create_bitwise_or(imm_ca03d, _ca39d);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _ca2c9 = sys.create_conditional_block(_ca39d);
    sys.set_current_block(_ca2c9);
    // log('r.add.0110011    | rd: x{:02}      | rs1: x{:02}      | rs2: x{:02}      ', _ca3f9, _cac0d, _cac29), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "r.add.0110011    | rd: x{:02}      | rs1: x{:02}      | rs2: x{:02}      ".into(),
    );
    sys.create_log(fmt, vec![_ca3f9, _cac0d, _cac29]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cac61 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:53
    let imm_cac6d = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cac75 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cac61 = sys.create_slice(_c9a75, imm_cac6d, imm_cac75);
    // _cac81 = _cac61 == (51:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:53
    let imm_cac71 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x33 as u64); // (51:b7)
    let _cac81 = sys.create_eq(_cac61, imm_cac71);
    // _cac85 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:54
    let imm_cac95 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cac9d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cac85 = sys.create_slice(_c9a75, imm_cac95, imm_cac9d);
    // _cac99 = _cac85 == (0:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:54
    let imm_caca5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x0 as u64); // (0:b3)
    let _cac99 = sys.create_eq(_cac85, imm_caca5);
    // _caca9 = _c9a75[(25:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:55
    let imm_cacb9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x19 as u64); // (25:u5)
    let imm_cacc1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _caca9 = sys.create_slice(_c9a75, imm_cacb9, imm_cacc1);
    // _cacbd = _caca9 == (32:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:55
    let imm_cacc9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x20 as u64); // (32:b7)
    let _cacbd = sys.create_eq(_caca9, imm_cacc9);
    // _cacd5 = _cac81 & _cac99, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let _cacd5 = sys.create_bitwise_and(_cac81, _cac99);
    // _cacdd = _cacd5 & _cacbd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let _cacdd = sys.create_bitwise_and(_cacd5, _cacbd);
    // _cace1 = _cacdd & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let imm_caccd = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cace1 = sys.create_bitwise_and(_cacdd, imm_caccd);
    // _ca38d = _ca311 | _cace1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _ca38d = sys.create_bitwise_or(_ca311, _cace1);
    // _cacf5 = _ca3b1 | _cace1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cacf5 = sys.create_bitwise_or(_ca3b1, _cace1);
    // _cad01 = _cace1 ? (2:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cacf1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x2 as u64); // (2:b16)
    let imm_cac55 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cad01 = sys.create_select(_cace1, imm_cacf1, imm_cac55);
    // _cad0d = _ca3c9 | _cad01, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cad0d = sys.create_bitwise_or(_ca3c9, _cad01);
    // _cad15 = _cace1 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_caced = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cad09 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cad15 = sys.create_select(_cace1, imm_caced, imm_cad09);
    // _cad21 = _ca3dd | _cad15, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cad21 = sys.create_bitwise_or(_ca3dd, _cad15);
    // _cad29 = _cace1 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cacf9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cad1d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cad29 = sys.create_select(_cace1, imm_cacf9, imm_cad1d);
    // _cad35 = _ca3f1 | _cad29, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cad35 = sys.create_bitwise_or(_ca3f1, _cad29);
    // _cad3d = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cad49 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cad51 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cad3d = sys.create_slice(_c9a75, imm_cad49, imm_cad51);
    // _cad39 = _ca181 | _cace1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _cad39 = sys.create_bitwise_or(_ca181, _cace1);
    // _cad4d = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cad65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cad6d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cad4d = sys.create_slice(_c9a75, imm_cad65, imm_cad6d);
    // _cad59 = _ca3b9 | _cace1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cad59 = sys.create_bitwise_or(_ca3b9, _cace1);
    // _cad69 = _c9a75[(20:u5):(24:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:65
    let imm_cad81 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cad89 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x18 as u64); // (24:u5)
    let _cad69 = sys.create_slice(_c9a75, imm_cad81, imm_cad89);
    // _ca15d = _ca3cd | _cace1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:66
    let _ca15d = sys.create_bitwise_or(_ca3cd, _cace1);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cad91 = sys.create_conditional_block(_cace1);
    sys.set_current_block(_cad91);
    // log('r.sub.0110011    | rd: x{:02}      | rs1: x{:02}      | rs2: x{:02}      ', _cad3d, _cad4d, _cad69), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "r.sub.0110011    | rd: x{:02}      | rs1: x{:02}      | rs2: x{:02}      ".into(),
    );
    sys.create_log(fmt, vec![_cad3d, _cad4d, _cad69]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cada1 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:53
    let imm_cadad = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cadb5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cada1 = sys.create_slice(_c9a75, imm_cadad, imm_cadb5);
    // _cadc1 = _cada1 == (51:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:53
    let imm_cadb1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x33 as u64); // (51:b7)
    let _cadc1 = sys.create_eq(_cada1, imm_cadb1);
    // _cadc5 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:54
    let imm_cadd5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_caddd = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cadc5 = sys.create_slice(_c9a75, imm_cadd5, imm_caddd);
    // _cadd9 = _cadc5 == (6:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:54
    let imm_cade5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x6 as u64); // (6:b3)
    let _cadd9 = sys.create_eq(_cadc5, imm_cade5);
    // _cade9 = _c9a75[(25:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:55
    let imm_cadf9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x19 as u64); // (25:u5)
    let imm_cae01 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cade9 = sys.create_slice(_c9a75, imm_cadf9, imm_cae01);
    // _cadfd = _cade9 == (0:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:55
    let imm_cae09 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x0 as u64); // (0:b7)
    let _cadfd = sys.create_eq(_cade9, imm_cae09);
    // _cae15 = _cadc1 & _cadd9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let _cae15 = sys.create_bitwise_and(_cadc1, _cadd9);
    // _cae1d = _cae15 & _cadfd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let _cae1d = sys.create_bitwise_and(_cae15, _cadfd);
    // _cae21 = _cae1d & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let imm_cae0d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cae21 = sys.create_bitwise_and(_cae1d, imm_cae0d);
    // _cad95 = _ca38d | _cae21, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cad95 = sys.create_bitwise_or(_ca38d, _cae21);
    // _cae35 = _cacf5 | _cae21, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cae35 = sys.create_bitwise_or(_cacf5, _cae21);
    // _cae41 = _cae21 ? (8:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cae31 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x8 as u64); // (8:b16)
    let imm_cacd1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cae41 = sys.create_select(_cae21, imm_cae31, imm_cacd1);
    // _cae4d = _cad0d | _cae41, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cae4d = sys.create_bitwise_or(_cad0d, _cae41);
    // _cae55 = _cae21 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cae2d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cae49 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cae55 = sys.create_select(_cae21, imm_cae2d, imm_cae49);
    // _cae61 = _cad21 | _cae55, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cae61 = sys.create_bitwise_or(_cad21, _cae55);
    // _cae69 = _cae21 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cae39 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cae5d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cae69 = sys.create_select(_cae21, imm_cae39, imm_cae5d);
    // _cae75 = _cad35 | _cae69, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cae75 = sys.create_bitwise_or(_cad35, _cae69);
    // _cae7d = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cae89 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cae91 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cae7d = sys.create_slice(_c9a75, imm_cae89, imm_cae91);
    // _cad25 = _cad39 | _cae21, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _cad25 = sys.create_bitwise_or(_cad39, _cae21);
    // _cae8d = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_caea5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_caead = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cae8d = sys.create_slice(_c9a75, imm_caea5, imm_caead);
    // _cae99 = _cad59 | _cae21, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cae99 = sys.create_bitwise_or(_cad59, _cae21);
    // _caea9 = _c9a75[(20:u5):(24:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:65
    let imm_caec1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_caec9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x18 as u64); // (24:u5)
    let _caea9 = sys.create_slice(_c9a75, imm_caec1, imm_caec9);
    // _cae79 = _ca15d | _cae21, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:66
    let _cae79 = sys.create_bitwise_or(_ca15d, _cae21);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _caed1 = sys.create_conditional_block(_cae21);
    sys.set_current_block(_caed1);
    // log('r.or.0110011     | rd: x{:02}      | rs1: x{:02}      | rs2: x{:02}      ', _cae7d, _cae8d, _caea9), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "r.or.0110011     | rd: x{:02}      | rs1: x{:02}      | rs2: x{:02}      ".into(),
    );
    sys.create_log(fmt, vec![_cae7d, _cae8d, _caea9]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _caee5 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_caef1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_caef9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _caee5 = sys.create_slice(_c9a75, imm_caef1, imm_caef9);
    // _caf05 = _caee5 == (103:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_caef5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x67 as u64); // (103:b7)
    let _caf05 = sys.create_eq(_caee5, imm_caef5);
    // _caf09 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_caf19 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_caf21 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _caf09 = sys.create_slice(_c9a75, imm_caf19, imm_caf21);
    // _caf1d = _caf09 == (0:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_caf29 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x0 as u64); // (0:b3)
    let _caf1d = sys.create_eq(_caf09, imm_caf29);
    // _caf3d = _caf05 & _caf1d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _caf3d = sys.create_bitwise_and(_caf05, _caf1d);
    // _caf45 = _caf3d & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_caf2d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _caf45 = sys.create_bitwise_and(_caf3d, imm_caf2d);
    // _caf49 = _caf45 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_caf39 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _caf49 = sys.create_bitwise_and(_caf45, imm_caf39);
    // _caed5 = (0:b1) | _caf49, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let imm_c9bfd = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _caed5 = sys.create_bitwise_or(imm_c9bfd, _caf49);
    // _caedd = _cae35 | _caf49, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _caedd = sys.create_bitwise_or(_cae35, _caf49);
    // _caf4d = _caf49 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_caf59 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cae11 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _caf4d = sys.create_select(_caf49, imm_caf59, imm_cae11);
    // _caf71 = _cae4d | _caf4d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _caf71 = sys.create_bitwise_or(_cae4d, _caf4d);
    // _caf79 = _caf49 ? (2048:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_caf69 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x800 as u64); // (2048:b16)
    let imm_caf6d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _caf79 = sys.create_select(_caf49, imm_caf69, imm_caf6d);
    // _caf85 = _cae61 | _caf79, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _caf85 = sys.create_bitwise_or(_cae61, _caf79);
    // _caf8d = _caf49 ? (False:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_caf65 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (False:b1)
    let imm_caf81 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _caf8d = sys.create_select(_caf49, imm_caf65, imm_caf81);
    // _caf99 = _cae75 | _caf8d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _caf99 = sys.create_bitwise_or(_cae75, _caf8d);
    // _cafa1 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cafad = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cafb5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cafa1 = sys.create_slice(_c9a75, imm_cafad, imm_cafb5);
    // _ca1ad = _cad25 | _caf49, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _ca1ad = sys.create_bitwise_or(_cad25, _caf49);
    // _cafb1 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cafc9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cafd1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cafb1 = sys.create_slice(_c9a75, imm_cafc9, imm_cafd1);
    // _cafbd = _cae99 | _caf49, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cafbd = sys.create_bitwise_or(_cae99, _caf49);
    // _cafcd = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:75
    let imm_cafe5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cafed = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cafcd = sys.create_slice(_c9a75, imm_cafe5, imm_cafed);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cafd9 = sys.create_conditional_block(_caf49);
    sys.set_current_block(_cafd9);
    // log('i.jalr.1100111   | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}', _cafa1, _cafb1, _cafcd), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "i.jalr.1100111   | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cafa1, _cafb1, _cafcd]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cb40d = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cb419 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cb421 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cb40d = sys.create_slice(_c9a75, imm_cb419, imm_cb421);
    // _cb41d = _cb40d == (19:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cb429 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x13 as u64); // (19:b7)
    let _cb41d = sys.create_eq(_cb40d, imm_cb429);
    // _cb42d = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cb43d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cb445 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cb42d = sys.create_slice(_c9a75, imm_cb43d, imm_cb445);
    // _cb441 = _cb42d == (0:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_caf89 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x0 as u64); // (0:b3)
    let _cb441 = sys.create_eq(_cb42d, imm_caf89);
    // _cb461 = _cb41d & _cb441, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _cb461 = sys.create_bitwise_and(_cb41d, _cb441);
    // _cb469 = _cb461 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cb451 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cb469 = sys.create_bitwise_and(_cb461, imm_cb451);
    // _cb46d = _cb469 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cb45d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cb46d = sys.create_bitwise_and(_cb469, imm_cb45d);
    // _caff9 = _caed5 | _cb46d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _caff9 = sys.create_bitwise_or(_caed5, _cb46d);
    // _cafe9 = _caedd | _cb46d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cafe9 = sys.create_bitwise_or(_caedd, _cb46d);
    // _cb489 = _cb46d ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cb47d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cae51 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cb489 = sys.create_select(_cb46d, imm_cb47d, imm_cae51);
    // _cb495 = _caf71 | _cb489, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cb495 = sys.create_bitwise_or(_caf71, _cb489);
    // _cb49d = _cb46d ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cb479 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cb491 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cb49d = sys.create_select(_cb46d, imm_cb479, imm_cb491);
    // _cb4a9 = _caf85 | _cb49d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cb4a9 = sys.create_bitwise_or(_caf85, _cb49d);
    // _cb4b1 = _cb46d ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cb485 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cb4a5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cb4b1 = sys.create_select(_cb46d, imm_cb485, imm_cb4a5);
    // _cb4bd = _caf99 | _cb4b1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cb4bd = sys.create_bitwise_or(_caf99, _cb4b1);
    // _cb4c9 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cb4d5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cb4dd = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cb4c9 = sys.create_slice(_c9a75, imm_cb4d5, imm_cb4dd);
    // _cb4c1 = _ca1ad | _cb46d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _cb4c1 = sys.create_bitwise_or(_ca1ad, _cb46d);
    // _cb4e5 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cb4ed = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cb4f5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cb4e5 = sys.create_slice(_c9a75, imm_cb4ed, imm_cb4f5);
    // _cb481 = _cafbd | _cb46d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cb481 = sys.create_bitwise_or(_cafbd, _cb46d);
    // _cb4f1 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:75
    let imm_cb509 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cb511 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cb4f1 = sys.create_slice(_c9a75, imm_cb509, imm_cb511);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cacfd = sys.create_conditional_block(_cb46d);
    sys.set_current_block(_cacfd);
    // log('i.addi.0010011   | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}', _cb4c9, _cb4e5, _cb4f1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "i.addi.0010011   | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cb4c9, _cb4e5, _cb4f1]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cb52d = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cb539 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cb541 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cb52d = sys.create_slice(_c9a75, imm_cb539, imm_cb541);
    // _cb53d = _cb52d == (19:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cb549 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x13 as u64); // (19:b7)
    let _cb53d = sys.create_eq(_cb52d, imm_cb549);
    // _cb54d = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cb55d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cb565 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cb54d = sys.create_slice(_c9a75, imm_cb55d, imm_cb565);
    // _cb561 = _cb54d == (4:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cb56d = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x4 as u64); // (4:b3)
    let _cb561 = sys.create_eq(_cb54d, imm_cb56d);
    // _cb581 = _cb53d & _cb561, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _cb581 = sys.create_bitwise_and(_cb53d, _cb561);
    // _cb589 = _cb581 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cb571 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cb589 = sys.create_bitwise_and(_cb581, imm_cb571);
    // _cb58d = _cb589 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cb57d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cb58d = sys.create_bitwise_and(_cb589, imm_cb57d);
    // _cb51d = _caff9 | _cb58d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cb51d = sys.create_bitwise_or(_caff9, _cb58d);
    // _cb525 = _cafe9 | _cb58d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cb525 = sys.create_bitwise_or(_cafe9, _cb58d);
    // _cb5a9 = _cb58d ? (4:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cb59d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x4 as u64); // (4:b16)
    let imm_cb44d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cb5a9 = sys.create_select(_cb58d, imm_cb59d, imm_cb44d);
    // _cb5b5 = _cb495 | _cb5a9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cb5b5 = sys.create_bitwise_or(_cb495, _cb5a9);
    // _cb5bd = _cb58d ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cb599 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cb5b1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cb5bd = sys.create_select(_cb58d, imm_cb599, imm_cb5b1);
    // _cb5c9 = _cb4a9 | _cb5bd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cb5c9 = sys.create_bitwise_or(_cb4a9, _cb5bd);
    // _cb5d1 = _cb58d ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cb5a5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cb5c5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cb5d1 = sys.create_select(_cb58d, imm_cb5a5, imm_cb5c5);
    // _cb5dd = _cb4bd | _cb5d1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cb5dd = sys.create_bitwise_or(_cb4bd, _cb5d1);
    // _cb5e5 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cb5f1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cb5f9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cb5e5 = sys.create_slice(_c9a75, imm_cb5f1, imm_cb5f9);
    // _cb5e1 = _cb4c1 | _cb58d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _cb5e1 = sys.create_bitwise_or(_cb4c1, _cb58d);
    // _cb5f5 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cb60d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cb615 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cb5f5 = sys.create_slice(_c9a75, imm_cb60d, imm_cb615);
    // _cb601 = _cb481 | _cb58d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cb601 = sys.create_bitwise_or(_cb481, _cb58d);
    // _cb611 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:75
    let imm_cb629 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cb631 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cb611 = sys.create_slice(_c9a75, imm_cb629, imm_cb631);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _ca319 = sys.create_conditional_block(_cb58d);
    sys.set_current_block(_ca319);
    // log('i.xori.0010011   | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}', _cb5e5, _cb5f5, _cb611), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "i.xori.0010011   | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cb5e5, _cb5f5, _cb611]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cb64d = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cb659 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cb661 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cb64d = sys.create_slice(_c9a75, imm_cb659, imm_cb661);
    // _cb65d = _cb64d == (3:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cb669 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x3 as u64); // (3:b7)
    let _cb65d = sys.create_eq(_cb64d, imm_cb669);
    // _cb66d = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cb67d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cb685 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cb66d = sys.create_slice(_c9a75, imm_cb67d, imm_cb685);
    // _cb681 = _cb66d == (2:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cb68d = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x2 as u64); // (2:b3)
    let _cb681 = sys.create_eq(_cb66d, imm_cb68d);
    // _cb6a1 = _cb65d & _cb681, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _cb6a1 = sys.create_bitwise_and(_cb65d, _cb681);
    // _cb6a9 = _cb6a1 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cb691 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cb6a9 = sys.create_bitwise_and(_cb6a1, imm_cb691);
    // _cb6ad = _cb6a9 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cb69d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cb6ad = sys.create_bitwise_and(_cb6a9, imm_cb69d);
    // _cb63d = _cb51d | _cb6ad, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cb63d = sys.create_bitwise_or(_cb51d, _cb6ad);
    // _cb645 = _cb525 | _cb6ad, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cb645 = sys.create_bitwise_or(_cb525, _cb6ad);
    // _cb6c9 = _cb6ad ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cb6bd = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cb4ad = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cb6c9 = sys.create_select(_cb6ad, imm_cb6bd, imm_cb4ad);
    // _cb6d5 = _cb5b5 | _cb6c9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cb6d5 = sys.create_bitwise_or(_cb5b5, _cb6c9);
    // _cb6dd = _cb6ad ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cb6b9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cb6d1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cb6dd = sys.create_select(_cb6ad, imm_cb6b9, imm_cb6d1);
    // _cb6e9 = _cb5c9 | _cb6dd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cb6e9 = sys.create_bitwise_or(_cb5c9, _cb6dd);
    // _cb6f1 = _cb6ad ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cb6c5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cb6e5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cb6f1 = sys.create_select(_cb6ad, imm_cb6c5, imm_cb6e5);
    // _cb6fd = _cb5dd | _cb6f1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cb6fd = sys.create_bitwise_or(_cb5dd, _cb6f1);
    // _cb705 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cb711 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cb719 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cb705 = sys.create_slice(_c9a75, imm_cb711, imm_cb719);
    // _cb499 = _cb5e1 | _cb6ad, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _cb499 = sys.create_bitwise_or(_cb5e1, _cb6ad);
    // _cb715 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cb72d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cb735 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cb715 = sys.create_slice(_c9a75, imm_cb72d, imm_cb735);
    // _cb721 = _cb601 | _cb6ad, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cb721 = sys.create_bitwise_or(_cb601, _cb6ad);
    // _cb731 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:75
    let imm_cb749 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cb751 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cb731 = sys.create_slice(_c9a75, imm_cb749, imm_cb751);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cb701 = sys.create_conditional_block(_cb6ad);
    sys.set_current_block(_cb701);
    // log('i.lw.0000011     | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}', _cb705, _cb715, _cb731), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "i.lw.0000011     | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cb705, _cb715, _cb731]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cb771 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cb77d = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cb785 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cb771 = sys.create_slice(_c9a75, imm_cb77d, imm_cb785);
    // _cb781 = _cb771 == (3:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cb78d = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x3 as u64); // (3:b7)
    let _cb781 = sys.create_eq(_cb771, imm_cb78d);
    // _cb791 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cb7a1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cb7a9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cb791 = sys.create_slice(_c9a75, imm_cb7a1, imm_cb7a9);
    // _cb7a5 = _cb791 == (4:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cb7b1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x4 as u64); // (4:b3)
    let _cb7a5 = sys.create_eq(_cb791, imm_cb7b1);
    // _cb7c5 = _cb781 & _cb7a5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _cb7c5 = sys.create_bitwise_and(_cb781, _cb7a5);
    // _cb7cd = _cb7c5 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cb7b5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cb7cd = sys.create_bitwise_and(_cb7c5, imm_cb7b5);
    // _cb7d1 = _cb7cd & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cb7c1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cb7d1 = sys.create_bitwise_and(_cb7cd, imm_cb7c1);
    // _cb75d = _cb63d | _cb7d1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cb75d = sys.create_bitwise_or(_cb63d, _cb7d1);
    // _cb769 = _cb645 | _cb7d1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cb769 = sys.create_bitwise_or(_cb645, _cb7d1);
    // _cb7ed = _cb7d1 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cb7e1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cb5cd = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cb7ed = sys.create_select(_cb7d1, imm_cb7e1, imm_cb5cd);
    // _cb7f9 = _cb6d5 | _cb7ed, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cb7f9 = sys.create_bitwise_or(_cb6d5, _cb7ed);
    // _cc005 = _cb7d1 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cb7dd = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cb7f5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cc005 = sys.create_select(_cb7d1, imm_cb7dd, imm_cb7f5);
    // _cc011 = _cb6e9 | _cc005, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cc011 = sys.create_bitwise_or(_cb6e9, _cc005);
    // _cc019 = _cb7d1 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cb7e9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cc00d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cc019 = sys.create_select(_cb7d1, imm_cb7e9, imm_cc00d);
    // _cc025 = _cb6fd | _cc019, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cc025 = sys.create_bitwise_or(_cb6fd, _cc019);
    // _cc02d = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cc039 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cc041 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cc02d = sys.create_slice(_c9a75, imm_cc039, imm_cc041);
    // _cb7e5 = _cb499 | _cb7d1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _cb7e5 = sys.create_bitwise_or(_cb499, _cb7d1);
    // _cc03d = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cc055 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cc05d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cc03d = sys.create_slice(_c9a75, imm_cc055, imm_cc05d);
    // _cb7fd = _cb721 | _cb7d1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cb7fd = sys.create_bitwise_or(_cb721, _cb7d1);
    // _cc059 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:75
    let imm_cc071 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cc079 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cc059 = sys.create_slice(_c9a75, imm_cc071, imm_cc079);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _caee1 = sys.create_conditional_block(_cb7d1);
    sys.set_current_block(_caee1);
    // log('i.lbu.0000011    | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}', _cc02d, _cc03d, _cc059), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "i.lbu.0000011    | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cc02d, _cc03d, _cc059]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cc095 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cc0a1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cc0a9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cc095 = sys.create_slice(_c9a75, imm_cc0a1, imm_cc0a9);
    // _cc0a5 = _cc095 == (115:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cc0b1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x73 as u64); // (115:b7)
    let _cc0a5 = sys.create_eq(_cc095, imm_cc0b1);
    // _cc0b5 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cc0c5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cc0cd = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cc0b5 = sys.create_slice(_c9a75, imm_cc0c5, imm_cc0cd);
    // _cc0c9 = _cc0b5 == (0:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cc0d5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x0 as u64); // (0:b3)
    let _cc0c9 = sys.create_eq(_cc0b5, imm_cc0d5);
    // _cc0d9 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:87
    let imm_cc0e9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cc0f1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cc0d9 = sys.create_slice(_c9a75, imm_cc0e9, imm_cc0f1);
    // _cc0ed = _cc0d9 == (1:b12), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:87
    let imm_cc0f9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(12), 0x1 as u64); // (1:b12)
    let _cc0ed = sys.create_eq(_cc0d9, imm_cc0f9);
    // _cc105 = _cc0a5 & _cc0c9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _cc105 = sys.create_bitwise_and(_cc0a5, _cc0c9);
    // _cc10d = _cc105 & _cc0ed, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _cc10d = sys.create_bitwise_and(_cc105, _cc0ed);
    // _cc111 = _cc10d & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cc0fd = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cc111 = sys.create_bitwise_and(_cc10d, imm_cc0fd);
    // _cb6ed = _cb75d | _cc111, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cb6ed = sys.create_bitwise_or(_cb75d, _cc111);
    // _cc08d = _cb769 | _cc111, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cc08d = sys.create_bitwise_or(_cb769, _cc111);
    // _cc12d = _cc111 ? (0:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cc119 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let imm_cc085 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cc12d = sys.create_select(_cc111, imm_cc119, imm_cc085);
    // _cc139 = _cb7f9 | _cc12d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cc139 = sys.create_bitwise_or(_cb7f9, _cc12d);
    // _cc141 = _cc111 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cc121 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cc135 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cc141 = sys.create_select(_cc111, imm_cc121, imm_cc135);
    // _cc14d = _cc011 | _cc141, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cc14d = sys.create_bitwise_or(_cc011, _cc141);
    // _cc155 = _cc111 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cc129 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cc149 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cc155 = sys.create_select(_cc111, imm_cc129, imm_cc149);
    // _cc161 = _cc025 | _cc155, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cc161 = sys.create_bitwise_or(_cc025, _cc155);
    // _cc15d = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cc171 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cc179 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cc15d = sys.create_slice(_c9a75, imm_cc171, imm_cc179);
    // _c83a9 = _cb7e5 | _cc111, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _c83a9 = sys.create_bitwise_or(_cb7e5, _cc111);
    // _cc175 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cc18d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cc195 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cc175 = sys.create_slice(_c9a75, imm_cc18d, imm_cc195);
    // _cc181 = _cb7fd | _cc111, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cc181 = sys.create_bitwise_or(_cb7fd, _cc111);
    // _cc191 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:75
    let imm_cc1a9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cc1b1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cc191 = sys.create_slice(_c9a75, imm_cc1a9, imm_cc1b1);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cc19d = sys.create_conditional_block(_cc111);
    sys.set_current_block(_cc19d);
    // log('i.ebreak.1110011 | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}', _cc15d, _cc175, _cc191), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "i.ebreak.1110011 | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cc15d, _cc175, _cc191]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cc1c9 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:110
    let imm_cc1d5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cc1dd = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cc1c9 = sys.create_slice(_c9a75, imm_cc1d5, imm_cc1dd);
    // _cc1d9 = _cc1c9 == (35:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:110
    let imm_cc1e5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x23 as u64); // (35:b7)
    let _cc1d9 = sys.create_eq(_cc1c9, imm_cc1e5);
    // _cc1e9 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:111
    let imm_cc1f9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cc201 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cc1e9 = sys.create_slice(_c9a75, imm_cc1f9, imm_cc201);
    // _cc1fd = _cc1e9 == (2:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:111
    let imm_cc209 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x2 as u64); // (2:b3)
    let _cc1fd = sys.create_eq(_cc1e9, imm_cc209);
    // _cc211 = _cc1d9 & _cc1fd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:112
    let _cc211 = sys.create_bitwise_and(_cc1d9, _cc1fd);
    // _cc1bd = (0:b1) | _cc211, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let imm_ca025 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cc1bd = sys.create_bitwise_or(imm_ca025, _cc211);
    // _cc225 = _cc08d | _cc211, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cc225 = sys.create_bitwise_or(_cc08d, _cc211);
    // _cc231 = _cc211 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cc221 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cc075 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cc231 = sys.create_select(_cc211, imm_cc221, imm_cc075);
    // _cc23d = _cc139 | _cc231, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cc23d = sys.create_bitwise_or(_cc139, _cc231);
    // _cc245 = _cc211 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cc21d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cc239 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cc245 = sys.create_select(_cc211, imm_cc21d, imm_cc239);
    // _cc251 = _cc14d | _cc245, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cc251 = sys.create_bitwise_or(_cc14d, _cc245);
    // _cc259 = _cc211 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cc229 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cc24d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cc259 = sys.create_select(_cc211, imm_cc229, imm_cc24d);
    // _cc265 = _cc161 | _cc259, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cc265 = sys.create_bitwise_or(_cc161, _cc259);
    // _cc271 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cc27d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cc285 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cc271 = sys.create_slice(_c9a75, imm_cc27d, imm_cc285);
    // _cc269 = _cc181 | _cc211, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cc269 = sys.create_bitwise_or(_cc181, _cc211);
    // _cc281 = _c9a75[(20:u5):(24:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:65
    let imm_cc299 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cc2a1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x18 as u64); // (24:u5)
    let _cc281 = sys.create_slice(_c9a75, imm_cc299, imm_cc2a1);
    // _cc28d = _cae79 | _cc211, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:66
    let _cc28d = sys.create_bitwise_or(_cae79, _cc211);
    // _cc29d = _c9a75[(25:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:116
    let imm_cc2b5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x19 as u64); // (25:u5)
    let imm_cc2bd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cc29d = sys.create_slice(_c9a75, imm_cc2b5, imm_cc2bd);
    // _cc2b9 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:116
    let imm_cc2d1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cc2d9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cc2b9 = sys.create_slice(_c9a75, imm_cc2d1, imm_cc2d9);
    // _cc261 = { _cc29d _cc2b9 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:116
    let _cc261 = sys.create_concat(_cc29d, _cc2b9);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cc2e1 = sys.create_conditional_block(_cc211);
    sys.set_current_block(_cc2e1);
    // log('s.sw.0100011     |              | rs1: x{:02}      | rs2: x{:02}      |imm: 0x{:x}', _cc271, _cc281, _cc261), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "s.sw.0100011     |              | rs1: x{:02}      | rs2: x{:02}      |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cc271, _cc281, _cc261]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cc2f1 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:183
    let imm_cc2fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cc305 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cc2f1 = sys.create_slice(_c9a75, imm_cc2fd, imm_cc305);
    // _cc301 = _cc2f1 == (99:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:183
    let imm_cc30d = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x63 as u64); // (99:b7)
    let _cc301 = sys.create_eq(_cc2f1, imm_cc30d);
    // _cc311 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:184
    let imm_cc321 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cc329 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cc311 = sys.create_slice(_c9a75, imm_cc321, imm_cc329);
    // _cc325 = _cc311 == (0:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:184
    let imm_cc331 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x0 as u64); // (0:b3)
    let _cc325 = sys.create_eq(_cc311, imm_cc331);
    // _cc339 = _cc301 & _cc325, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:185
    let _cc339 = sys.create_bitwise_and(_cc301, _cc325);
    // _cc2e5 = (0:b1) | _cc339, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let imm_ca00d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cc2e5 = sys.create_bitwise_or(imm_ca00d, _cc339);
    // _cc351 = _cc225 | _cc339, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cc351 = sys.create_bitwise_or(_cc225, _cc339);
    // _cc341 = _cc339 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cc349 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cc165 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cc341 = sys.create_select(_cc339, imm_cc349, imm_cc165);
    // _cc365 = _cc23d | _cc341, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cc365 = sys.create_bitwise_or(_cc23d, _cc341);
    // _cc36d = _cc339 ? (256:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cc359 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x100 as u64); // (256:b16)
    let imm_cc361 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cc36d = sys.create_select(_cc339, imm_cc359, imm_cc361);
    // _cc379 = _cc251 | _cc36d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cc379 = sys.create_bitwise_or(_cc251, _cc36d);
    // _cc381 = _cc339 ? (False:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cc355 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (False:b1)
    let imm_cc375 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cc381 = sys.create_select(_cc339, imm_cc355, imm_cc375);
    // _cc38d = _cc265 | _cc381, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cc38d = sys.create_bitwise_or(_cc265, _cc381);
    // _cc395 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cc3a1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cc3a9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cc395 = sys.create_slice(_c9a75, imm_cc3a1, imm_cc3a9);
    // _cc0b9 = _cc269 | _cc339, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cc0b9 = sys.create_bitwise_or(_cc269, _cc339);
    // _cc3a5 = _c9a75[(20:u5):(24:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:65
    let imm_cc3bd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cc3c5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x18 as u64); // (24:u5)
    let _cc3a5 = sys.create_slice(_c9a75, imm_cc3bd, imm_cc3c5);
    // _cc3b1 = _cc28d | _cc339, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:66
    let _cc3b1 = sys.create_bitwise_or(_cc28d, _cc339);
    // _cc3c1 = _c9a75[(31:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cc3d9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let imm_cc3e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cc3c1 = sys.create_slice(_c9a75, imm_cc3d9, imm_cc3e1);
    // _cc3dd = _c9a75[(7:u3):(7:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cc3f5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cc3fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let _cc3dd = sys.create_slice(_c9a75, imm_cc3f5, imm_cc3fd);
    // _cc809 = _c9a75[(25:u5):(30:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cc811 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x19 as u64); // (25:u5)
    let imm_cc819 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1e as u64); // (30:u5)
    let _cc809 = sys.create_slice(_c9a75, imm_cc811, imm_cc819);
    // _cc821 = _c9a75[(8:u4):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cc829 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0x8 as u64); // (8:u4)
    let imm_cc831 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cc821 = sys.create_slice(_c9a75, imm_cc829, imm_cc831);
    // _cc3cd = { _cc3c1 _cc3dd }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _cc3cd = sys.create_concat(_cc3c1, _cc3dd);
    // _cc1ad = { _cc3cd _cc809 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _cc1ad = sys.create_concat(_cc3cd, _cc809);
    // _cc82d = { _cc1ad _cc821 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _cc82d = sys.create_concat(_cc1ad, _cc821);
    // _cc841 = { _cc82d (0:b1) }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:190
    let imm_cc839 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cc841 = sys.create_concat(_cc82d, imm_cc839);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cc849 = sys.create_conditional_block(_cc339);
    sys.set_current_block(_cc849);
    // log('b.beq.1100011    |              | rs1: x{:02}      | rs2: x{:02}      |imm: 0x{:x}', _cc395, _cc3a5, _cc841), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "b.beq.1100011    |              | rs1: x{:02}      | rs2: x{:02}      |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cc395, _cc3a5, _cc841]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cc85d = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:183
    let imm_cc869 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cc871 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cc85d = sys.create_slice(_c9a75, imm_cc869, imm_cc871);
    // _cc86d = _cc85d == (99:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:183
    let imm_cc879 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x63 as u64); // (99:b7)
    let _cc86d = sys.create_eq(_cc85d, imm_cc879);
    // _cc87d = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:184
    let imm_cc88d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cc895 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cc87d = sys.create_slice(_c9a75, imm_cc88d, imm_cc895);
    // _cc891 = _cc87d == (1:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:184
    let imm_cc369 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x1 as u64); // (1:b3)
    let _cc891 = sys.create_eq(_cc87d, imm_cc369);
    // _cc8a5 = _cc86d & _cc891, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:185
    let _cc8a5 = sys.create_bitwise_and(_cc86d, _cc891);
    // _cc22d = _cc2e5 | _cc8a5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cc22d = sys.create_bitwise_or(_cc2e5, _cc8a5);
    // _cc8bd = _cc351 | _cc8a5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cc8bd = sys.create_bitwise_or(_cc351, _cc8a5);
    // _cc8ad = _cc8a5 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cc8b5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cc851 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cc8ad = sys.create_select(_cc8a5, imm_cc8b5, imm_cc851);
    // _cc8d1 = _cc365 | _cc8ad, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cc8d1 = sys.create_bitwise_or(_cc365, _cc8ad);
    // _cc8d9 = _cc8a5 ? (256:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cc8c5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x100 as u64); // (256:b16)
    let imm_cc8cd = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cc8d9 = sys.create_select(_cc8a5, imm_cc8c5, imm_cc8cd);
    // _cc8e5 = _cc379 | _cc8d9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cc8e5 = sys.create_bitwise_or(_cc379, _cc8d9);
    // _cc8ed = _cc8a5 ? (True:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cc8c1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (True:b1)
    let imm_cc8e1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cc8ed = sys.create_select(_cc8a5, imm_cc8c1, imm_cc8e1);
    // _cc8f9 = _cc38d | _cc8ed, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cc8f9 = sys.create_bitwise_or(_cc38d, _cc8ed);
    // _cc901 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cc90d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cc915 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cc901 = sys.create_slice(_c9a75, imm_cc90d, imm_cc915);
    // _cc151 = _cc0b9 | _cc8a5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cc151 = sys.create_bitwise_or(_cc0b9, _cc8a5);
    // _cc911 = _c9a75[(20:u5):(24:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:65
    let imm_cc929 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cc931 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x18 as u64); // (24:u5)
    let _cc911 = sys.create_slice(_c9a75, imm_cc929, imm_cc931);
    // _cc91d = _cc3b1 | _cc8a5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:66
    let _cc91d = sys.create_bitwise_or(_cc3b1, _cc8a5);
    // _cc92d = _c9a75[(31:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cc945 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let imm_cc94d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cc92d = sys.create_slice(_c9a75, imm_cc945, imm_cc94d);
    // _cc949 = _c9a75[(7:u3):(7:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cc961 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cc969 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let _cc949 = sys.create_slice(_c9a75, imm_cc961, imm_cc969);
    // _cc971 = _c9a75[(25:u5):(30:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cc979 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x19 as u64); // (25:u5)
    let imm_cc981 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1e as u64); // (30:u5)
    let _cc971 = sys.create_slice(_c9a75, imm_cc979, imm_cc981);
    // _cc989 = _c9a75[(8:u4):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cc991 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0x8 as u64); // (8:u4)
    let imm_cc999 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cc989 = sys.create_slice(_c9a75, imm_cc991, imm_cc999);
    // _cc2d5 = { _cc92d _cc949 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _cc2d5 = sys.create_concat(_cc92d, _cc949);
    // _cc9a1 = { _cc2d5 _cc971 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _cc9a1 = sys.create_concat(_cc2d5, _cc971);
    // _cc995 = { _cc9a1 _cc989 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _cc995 = sys.create_concat(_cc9a1, _cc989);
    // _cc9a9 = { _cc995 (0:b1) }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:190
    let imm_cc939 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cc9a9 = sys.create_concat(_cc995, imm_cc939);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cc9b1 = sys.create_conditional_block(_cc8a5);
    sys.set_current_block(_cc9b1);
    // log('b.bne.1100011    |              | rs1: x{:02}      | rs2: x{:02}      |imm: 0x{:x}', _cc901, _cc911, _cc9a9), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "b.bne.1100011    |              | rs1: x{:02}      | rs2: x{:02}      |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cc901, _cc911, _cc9a9]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cc9c5 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:183
    let imm_cc9d1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cc9d9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cc9c5 = sys.create_slice(_c9a75, imm_cc9d1, imm_cc9d9);
    // _cc9d5 = _cc9c5 == (99:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:183
    let imm_cc9e1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x63 as u64); // (99:b7)
    let _cc9d5 = sys.create_eq(_cc9c5, imm_cc9e1);
    // _cc9e5 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:184
    let imm_cc9f5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cc9fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cc9e5 = sys.create_slice(_c9a75, imm_cc9f5, imm_cc9fd);
    // _cc9f9 = _cc9e5 == (4:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:184
    let imm_cca05 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x4 as u64); // (4:b3)
    let _cc9f9 = sys.create_eq(_cc9e5, imm_cca05);
    // _cca0d = _cc9d5 & _cc9f9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:185
    let _cca0d = sys.create_bitwise_and(_cc9d5, _cc9f9);
    // _cc33d = _cc22d | _cca0d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cc33d = sys.create_bitwise_or(_cc22d, _cca0d);
    // _cca25 = _cc8bd | _cca0d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cca25 = sys.create_bitwise_or(_cc8bd, _cca0d);
    // _cca15 = _cca0d ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cca1d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cc9b9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cca15 = sys.create_select(_cca0d, imm_cca1d, imm_cc9b9);
    // _cca39 = _cc8d1 | _cca15, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cca39 = sys.create_bitwise_or(_cc8d1, _cca15);
    // _cca41 = _cca0d ? (512:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cca2d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x200 as u64); // (512:b16)
    let imm_cca35 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cca41 = sys.create_select(_cca0d, imm_cca2d, imm_cca35);
    // _cca4d = _cc8e5 | _cca41, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cca4d = sys.create_bitwise_or(_cc8e5, _cca41);
    // _cca55 = _cca0d ? (False:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cca29 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (False:b1)
    let imm_cca49 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cca55 = sys.create_select(_cca0d, imm_cca29, imm_cca49);
    // _cca61 = _cc8f9 | _cca55, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cca61 = sys.create_bitwise_or(_cc8f9, _cca55);
    // _cca69 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cca75 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cca7d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cca69 = sys.create_slice(_c9a75, imm_cca75, imm_cca7d);
    // _cc8fd = _cc151 | _cca0d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cc8fd = sys.create_bitwise_or(_cc151, _cca0d);
    // _cca79 = _c9a75[(20:u5):(24:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:65
    let imm_cca91 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cca99 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x18 as u64); // (24:u5)
    let _cca79 = sys.create_slice(_c9a75, imm_cca91, imm_cca99);
    // _cca85 = _cc91d | _cca0d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:66
    let _cca85 = sys.create_bitwise_or(_cc91d, _cca0d);
    // _cca95 = _c9a75[(31:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_ccaad = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let imm_ccab5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cca95 = sys.create_slice(_c9a75, imm_ccaad, imm_ccab5);
    // _ccab1 = _c9a75[(7:u3):(7:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_ccac9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_ccad1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let _ccab1 = sys.create_slice(_c9a75, imm_ccac9, imm_ccad1);
    // _ccad9 = _c9a75[(25:u5):(30:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_ccae1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x19 as u64); // (25:u5)
    let imm_ccae9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1e as u64); // (30:u5)
    let _ccad9 = sys.create_slice(_c9a75, imm_ccae1, imm_ccae9);
    // _ccaf1 = _c9a75[(8:u4):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_ccaf9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0x8 as u64); // (8:u4)
    let imm_ccb01 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _ccaf1 = sys.create_slice(_c9a75, imm_ccaf9, imm_ccb01);
    // _cc37d = { _cca95 _ccab1 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _cc37d = sys.create_concat(_cca95, _ccab1);
    // _ccb09 = { _cc37d _ccad9 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _ccb09 = sys.create_concat(_cc37d, _ccad9);
    // _ccafd = { _ccb09 _ccaf1 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _ccafd = sys.create_concat(_ccb09, _ccaf1);
    // _ccb11 = { _ccafd (0:b1) }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:190
    let imm_cc84d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _ccb11 = sys.create_concat(_ccafd, imm_cc84d);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _ccb19 = sys.create_conditional_block(_cca0d);
    sys.set_current_block(_ccb19);
    // log('b.blt.1100011    |              | rs1: x{:02}      | rs2: x{:02}      |imm: 0x{:x}', _cca69, _cca79, _ccb11), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "b.blt.1100011    |              | rs1: x{:02}      | rs2: x{:02}      |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cca69, _cca79, _ccb11]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _ccb2d = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:183
    let imm_ccb39 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_ccb41 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _ccb2d = sys.create_slice(_c9a75, imm_ccb39, imm_ccb41);
    // _ccb3d = _ccb2d == (99:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:183
    let imm_ccb49 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x63 as u64); // (99:b7)
    let _ccb3d = sys.create_eq(_ccb2d, imm_ccb49);
    // _ccb4d = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:184
    let imm_ccb5d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_ccb65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _ccb4d = sys.create_slice(_c9a75, imm_ccb5d, imm_ccb65);
    // _ccb61 = _ccb4d == (5:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:184
    let imm_ccb6d = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x5 as u64); // (5:b3)
    let _ccb61 = sys.create_eq(_ccb4d, imm_ccb6d);
    // _ccb75 = _ccb3d & _ccb61, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:185
    let _ccb75 = sys.create_bitwise_and(_ccb3d, _ccb61);
    // _ccb21 = _cc33d | _ccb75, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _ccb21 = sys.create_bitwise_or(_cc33d, _ccb75);
    // _ccb8d = _cca25 | _ccb75, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _ccb8d = sys.create_bitwise_or(_cca25, _ccb75);
    // _ccb7d = _ccb75 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_ccb85 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cc8a9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ccb7d = sys.create_select(_ccb75, imm_ccb85, imm_cc8a9);
    // _ccba1 = _cca39 | _ccb7d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _ccba1 = sys.create_bitwise_or(_cca39, _ccb7d);
    // _ccba9 = _ccb75 ? (512:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_ccb95 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x200 as u64); // (512:b16)
    let imm_ccb9d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ccba9 = sys.create_select(_ccb75, imm_ccb95, imm_ccb9d);
    // _ccbb5 = _cca4d | _ccba9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _ccbb5 = sys.create_bitwise_or(_cca4d, _ccba9);
    // _ccbbd = _ccb75 ? (True:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_ccb91 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (True:b1)
    let imm_ccbb1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _ccbbd = sys.create_select(_ccb75, imm_ccb91, imm_ccbb1);
    // _ccbc9 = _cca61 | _ccbbd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _ccbc9 = sys.create_bitwise_or(_cca61, _ccbbd);
    // _ccbd1 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_ccbdd = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_ccbe5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _ccbd1 = sys.create_slice(_c9a75, imm_ccbdd, imm_ccbe5);
    // _cc89d = _cc8fd | _ccb75, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cc89d = sys.create_bitwise_or(_cc8fd, _ccb75);
    // _ccbe1 = _c9a75[(20:u5):(24:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:65
    let imm_ccbf9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_ccbfd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x18 as u64); // (24:u5)
    let _ccbe1 = sys.create_slice(_c9a75, imm_ccbf9, imm_ccbfd);
    // _ccbed = _cca85 | _ccb75, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:66
    let _ccbed = sys.create_bitwise_or(_cca85, _ccb75);
    // _cd005 = _c9a75[(31:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cd019 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let imm_cd021 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cd005 = sys.create_slice(_c9a75, imm_cd019, imm_cd021);
    // _cd01d = _c9a75[(7:u3):(7:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cd035 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cd03d = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let _cd01d = sys.create_slice(_c9a75, imm_cd035, imm_cd03d);
    // _cd045 = _c9a75[(25:u5):(30:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cd04d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x19 as u64); // (25:u5)
    let imm_cd055 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1e as u64); // (30:u5)
    let _cd045 = sys.create_slice(_c9a75, imm_cd04d, imm_cd055);
    // _cd05d = _c9a75[(8:u4):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cd065 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0x8 as u64); // (8:u4)
    let imm_cd06d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cd05d = sys.create_slice(_c9a75, imm_cd065, imm_cd06d);
    // _ccba5 = { _cd005 _cd01d }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _ccba5 = sys.create_concat(_cd005, _cd01d);
    // _cc9b5 = { _ccba5 _cd045 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _cc9b5 = sys.create_concat(_ccba5, _cd045);
    // _cd069 = { _cc9b5 _cd05d }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _cd069 = sys.create_concat(_cc9b5, _cd05d);
    // _cd07d = { _cd069 (0:b1) }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:190
    let imm_cd075 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cd07d = sys.create_concat(_cd069, imm_cd075);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cd089 = sys.create_conditional_block(_ccb75);
    sys.set_current_block(_cd089);
    // log('b.bge.1100011    |              | rs1: x{:02}      | rs2: x{:02}      |imm: 0x{:x}', _ccbd1, _ccbe1, _cd07d), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "b.bge.1100011    |              | rs1: x{:02}      | rs2: x{:02}      |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_ccbd1, _ccbe1, _cd07d]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cd09d = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:183
    let imm_cd0a9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cd0b1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cd09d = sys.create_slice(_c9a75, imm_cd0a9, imm_cd0b1);
    // _cd0ad = _cd09d == (99:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:183
    let imm_cd0b9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x63 as u64); // (99:b7)
    let _cd0ad = sys.create_eq(_cd09d, imm_cd0b9);
    // _cd0bd = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:184
    let imm_cd0cd = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cd0d5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cd0bd = sys.create_slice(_c9a75, imm_cd0cd, imm_cd0d5);
    // _cd0d1 = _cd0bd == (7:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:184
    let imm_cd0dd = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x7 as u64); // (7:b3)
    let _cd0d1 = sys.create_eq(_cd0bd, imm_cd0dd);
    // _cd0e5 = _cd0ad & _cd0d1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:185
    let _cd0e5 = sys.create_bitwise_and(_cd0ad, _cd0d1);
    // _cca11 = _ccb21 | _cd0e5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cca11 = sys.create_bitwise_or(_ccb21, _cd0e5);
    // _cd0fd = _ccb8d | _cd0e5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cd0fd = sys.create_bitwise_or(_ccb8d, _cd0e5);
    // _cd0ed = _cd0e5 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cd0f5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cd091 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cd0ed = sys.create_select(_cd0e5, imm_cd0f5, imm_cd091);
    // _cd111 = _ccba1 | _cd0ed, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cd111 = sys.create_bitwise_or(_ccba1, _cd0ed);
    // _cd119 = _cd0e5 ? (1024:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cd105 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x400 as u64); // (1024:b16)
    let imm_cd10d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cd119 = sys.create_select(_cd0e5, imm_cd105, imm_cd10d);
    // _cd125 = _ccbb5 | _cd119, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cd125 = sys.create_bitwise_or(_ccbb5, _cd119);
    // _cd12d = _cd0e5 ? (True:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cd101 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (True:b1)
    let imm_cd121 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cd12d = sys.create_select(_cd0e5, imm_cd101, imm_cd121);
    // _cd139 = _ccbc9 | _cd12d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cd139 = sys.create_bitwise_or(_ccbc9, _cd12d);
    // _cd141 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cd14d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cd155 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cd141 = sys.create_slice(_c9a75, imm_cd14d, imm_cd155);
    // _cc8d5 = _cc89d | _cd0e5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cc8d5 = sys.create_bitwise_or(_cc89d, _cd0e5);
    // _cd151 = _c9a75[(20:u5):(24:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:65
    let imm_cd169 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cd171 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x18 as u64); // (24:u5)
    let _cd151 = sys.create_slice(_c9a75, imm_cd169, imm_cd171);
    // _cd15d = _ccbed | _cd0e5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:66
    let _cd15d = sys.create_bitwise_or(_ccbed, _cd0e5);
    // _cd16d = _c9a75[(31:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cd185 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let imm_cd18d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cd16d = sys.create_slice(_c9a75, imm_cd185, imm_cd18d);
    // _cd189 = _c9a75[(7:u3):(7:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cd1a1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cd1a9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let _cd189 = sys.create_slice(_c9a75, imm_cd1a1, imm_cd1a9);
    // _cd1b1 = _c9a75[(25:u5):(30:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cd1b9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x19 as u64); // (25:u5)
    let imm_cd1c1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1e as u64); // (30:u5)
    let _cd1b1 = sys.create_slice(_c9a75, imm_cd1b9, imm_cd1c1);
    // _cd1c9 = _c9a75[(8:u4):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cd1d1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0x8 as u64); // (8:u4)
    let imm_cd1d9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cd1c9 = sys.create_slice(_c9a75, imm_cd1d1, imm_cd1d9);
    // _ccb1d = { _cd16d _cd189 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _ccb1d = sys.create_concat(_cd16d, _cd189);
    // _cd1e1 = { _ccb1d _cd1b1 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _cd1e1 = sys.create_concat(_ccb1d, _cd1b1);
    // _cd1d5 = { _cd1e1 _cd1c9 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _cd1d5 = sys.create_concat(_cd1e1, _cd1c9);
    // _cd1e9 = { _cd1d5 (0:b1) }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:190
    let imm_cd179 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cd1e9 = sys.create_concat(_cd1d5, imm_cd179);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cd1f1 = sys.create_conditional_block(_cd0e5);
    sys.set_current_block(_cd1f1);
    // log('b.bgeu.1100011   |              | rs1: x{:02}      | rs2: x{:02}      |imm: 0x{:x}', _cd141, _cd151, _cd1e9), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "b.bgeu.1100011   |              | rs1: x{:02}      | rs2: x{:02}      |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cd141, _cd151, _cd1e9]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cd205 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:183
    let imm_cd211 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cd219 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cd205 = sys.create_slice(_c9a75, imm_cd211, imm_cd219);
    // _cd215 = _cd205 == (99:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:183
    let imm_cd221 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x63 as u64); // (99:b7)
    let _cd215 = sys.create_eq(_cd205, imm_cd221);
    // _cd225 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:184
    let imm_cd235 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cd23d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cd225 = sys.create_slice(_c9a75, imm_cd235, imm_cd23d);
    // _cd239 = _cd225 == (6:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:184
    let imm_cd245 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x6 as u64); // (6:b3)
    let _cd239 = sys.create_eq(_cd225, imm_cd245);
    // _cd24d = _cd215 & _cd239, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:185
    let _cd24d = sys.create_bitwise_and(_cd215, _cd239);
    // _ccb79 = _cca11 | _cd24d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _ccb79 = sys.create_bitwise_or(_cca11, _cd24d);
    // _cd265 = _cd0fd | _cd24d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cd265 = sys.create_bitwise_or(_cd0fd, _cd24d);
    // _cd255 = _cd24d ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cd25d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cd1f9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cd255 = sys.create_select(_cd24d, imm_cd25d, imm_cd1f9);
    // _cd279 = _cd111 | _cd255, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cd279 = sys.create_bitwise_or(_cd111, _cd255);
    // _cd281 = _cd24d ? (1024:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cd26d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x400 as u64); // (1024:b16)
    let imm_cd275 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cd281 = sys.create_select(_cd24d, imm_cd26d, imm_cd275);
    // _cd28d = _cd125 | _cd281, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cd28d = sys.create_bitwise_or(_cd125, _cd281);
    // _cd295 = _cd24d ? (False:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cd269 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (False:b1)
    let imm_cd289 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cd295 = sys.create_select(_cd24d, imm_cd269, imm_cd289);
    // _cd2a1 = _cd139 | _cd295, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cd2a1 = sys.create_bitwise_or(_cd139, _cd295);
    // _cd2a9 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cd2b5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cd2bd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cd2a9 = sys.create_slice(_c9a75, imm_cd2b5, imm_cd2bd);
    // _cd13d = _cc8d5 | _cd24d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cd13d = sys.create_bitwise_or(_cc8d5, _cd24d);
    // _cd2b9 = _c9a75[(20:u5):(24:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:65
    let imm_cd2d1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cd2d9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x18 as u64); // (24:u5)
    let _cd2b9 = sys.create_slice(_c9a75, imm_cd2d1, imm_cd2d9);
    // _cd2c5 = _cd15d | _cd24d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:66
    let _cd2c5 = sys.create_bitwise_or(_cd15d, _cd24d);
    // _cd2d5 = _c9a75[(31:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cd2ed = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let imm_cd2f5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cd2d5 = sys.create_slice(_c9a75, imm_cd2ed, imm_cd2f5);
    // _cd2f1 = _c9a75[(7:u3):(7:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cd309 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cd311 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let _cd2f1 = sys.create_slice(_c9a75, imm_cd309, imm_cd311);
    // _cd319 = _c9a75[(25:u5):(30:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cd321 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x19 as u64); // (25:u5)
    let imm_cd329 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1e as u64); // (30:u5)
    let _cd319 = sys.create_slice(_c9a75, imm_cd321, imm_cd329);
    // _cd331 = _c9a75[(8:u4):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_cd339 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0x8 as u64); // (8:u4)
    let imm_cd341 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cd331 = sys.create_slice(_c9a75, imm_cd339, imm_cd341);
    // _ccbb9 = { _cd2d5 _cd2f1 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _ccbb9 = sys.create_concat(_cd2d5, _cd2f1);
    // _cd349 = { _ccbb9 _cd319 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _cd349 = sys.create_concat(_ccbb9, _cd319);
    // _cd33d = { _cd349 _cd331 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _cd33d = sys.create_concat(_cd349, _cd331);
    // _cd351 = { _cd33d (0:b1) }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:190
    let imm_cd08d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cd351 = sys.create_concat(_cd33d, imm_cd08d);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cd359 = sys.create_conditional_block(_cd24d);
    sys.set_current_block(_cd359);
    // log('b.bltu.1100011   |              | rs1: x{:02}      | rs2: x{:02}      |imm: 0x{:x}', _cd2a9, _cd2b9, _cd351), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "b.bltu.1100011   |              | rs1: x{:02}      | rs2: x{:02}      |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cd2a9, _cd2b9, _cd351]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cd371 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cd37d = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cd385 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cd371 = sys.create_slice(_c9a75, imm_cd37d, imm_cd385);
    // _cd381 = _cd371 == (115:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cd38d = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x73 as u64); // (115:b7)
    let _cd381 = sys.create_eq(_cd371, imm_cd38d);
    // _cd391 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cd3a1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cd3a9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cd391 = sys.create_slice(_c9a75, imm_cd3a1, imm_cd3a9);
    // _cd3a5 = _cd391 == (2:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cd3b1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x2 as u64); // (2:b3)
    let _cd3a5 = sys.create_eq(_cd391, imm_cd3b1);
    // _cd3c5 = _cd381 & _cd3a5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _cd3c5 = sys.create_bitwise_and(_cd381, _cd3a5);
    // _cd3cd = _cd3c5 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cd3b5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cd3cd = sys.create_bitwise_and(_cd3c5, imm_cd3b5);
    // _cd3d1 = _cd3cd & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cd3c1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cd3d1 = sys.create_bitwise_and(_cd3cd, imm_cd3c1);
    // _cd361 = _cb6ed | _cd3d1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cd361 = sys.create_bitwise_or(_cb6ed, _cd3d1);
    // _cd369 = _cd265 | _cd3d1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cd369 = sys.create_bitwise_or(_cd265, _cd3d1);
    // _cd3ed = _cd3d1 ? (8:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cd3e1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x8 as u64); // (8:b16)
    let imm_cd0e9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cd3ed = sys.create_select(_cd3d1, imm_cd3e1, imm_cd0e9);
    // _cd3f9 = _cd279 | _cd3ed, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cd3f9 = sys.create_bitwise_or(_cd279, _cd3ed);
    // _cdc05 = _cd3d1 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cd3dd = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cd3f5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cdc05 = sys.create_select(_cd3d1, imm_cd3dd, imm_cd3f5);
    // _cdc11 = _cd28d | _cdc05, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cdc11 = sys.create_bitwise_or(_cd28d, _cdc05);
    // _cdc19 = _cd3d1 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cd3e9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cdc0d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cdc19 = sys.create_select(_cd3d1, imm_cd3e9, imm_cdc0d);
    // _cdc25 = _cd2a1 | _cdc19, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cdc25 = sys.create_bitwise_or(_cd2a1, _cdc19);
    // _cdc21 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cdc35 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cdc3d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cdc21 = sys.create_slice(_c9a75, imm_cdc35, imm_cdc3d);
    // _cd2a5 = _c83a9 | _cd3d1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _cd2a5 = sys.create_bitwise_or(_c83a9, _cd3d1);
    // _cdc39 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cdc51 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cdc59 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cdc39 = sys.create_slice(_c9a75, imm_cdc51, imm_cdc59);
    // _cd3e5 = _cd13d | _cd3d1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cd3e5 = sys.create_bitwise_or(_cd13d, _cd3d1);
    // _cdc55 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:75
    let imm_cdc6d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cdc75 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cdc55 = sys.create_slice(_c9a75, imm_cdc6d, imm_cdc75);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cd3fd = sys.create_conditional_block(_cd3d1);
    sys.set_current_block(_cd3fd);
    // log('i.csrrs.1110011  | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}', _cdc21, _cdc39, _cdc55), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "i.csrrs.1110011  | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cdc21, _cdc39, _cdc55]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cdc8d = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:132
    let imm_cdc99 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cdca1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cdc8d = sys.create_slice(_c9a75, imm_cdc99, imm_cdca1);
    // _cdc9d = _cdc8d == (23:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:132
    let imm_cdca9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x17 as u64); // (23:b7)
    let _cdc9d = sys.create_eq(_cdc8d, imm_cdca9);
    // _cd27d = _ca249 | _cdc9d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cd27d = sys.create_bitwise_or(_ca249, _cdc9d);
    // _cdcc1 = _cd369 | _cdc9d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cdcc1 = sys.create_bitwise_or(_cd369, _cdc9d);
    // _cdccd = _cdc9d ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cdcbd = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cdc81 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cdccd = sys.create_select(_cdc9d, imm_cdcbd, imm_cdc81);
    // _cdcd9 = _cd3f9 | _cdccd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cdcd9 = sys.create_bitwise_or(_cd3f9, _cdccd);
    // _cdce1 = _cdc9d ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cdcb9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cdcd5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cdce1 = sys.create_select(_cdc9d, imm_cdcb9, imm_cdcd5);
    // _cdced = _cdc11 | _cdce1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cdced = sys.create_bitwise_or(_cdc11, _cdce1);
    // _cdcf5 = _cdc9d ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cdcc5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cdce9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cdcf5 = sys.create_select(_cdc9d, imm_cdcc5, imm_cdce9);
    // _cdd01 = _cdc25 | _cdcf5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cdd01 = sys.create_bitwise_or(_cdc25, _cdcf5);
    // _cdd09 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cdd15 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cdd1d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cdd09 = sys.create_slice(_c9a75, imm_cdd15, imm_cdd1d);
    // _cdd05 = _cd2a5 | _cdc9d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _cdd05 = sys.create_bitwise_or(_cd2a5, _cdc9d);
    // _cdd19 = _c9a75[(12:u4):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:136
    let imm_cdd31 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cdd39 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cdd19 = sys.create_slice(_c9a75, imm_cdd31, imm_cdd39);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cdd25 = sys.create_conditional_block(_cdc9d);
    sys.set_current_block(_cdd25);
    // log('u.auipc.0010111  | rd: x{:02}      |               |               |imm: 0x{:x}', _cdd09, _cdd19), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "u.auipc.0010111  | rd: x{:02}      |               |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cdd09, _cdd19]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cdd59 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cdd65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cdd6d = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cdd59 = sys.create_slice(_c9a75, imm_cdd65, imm_cdd6d);
    // _cdd69 = _cdd59 == (115:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cdd75 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x73 as u64); // (115:b7)
    let _cdd69 = sys.create_eq(_cdd59, imm_cdd75);
    // _cdd79 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cdd89 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cdd91 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cdd79 = sys.create_slice(_c9a75, imm_cdd89, imm_cdd91);
    // _cdd8d = _cdd79 == (1:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cd115 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x1 as u64); // (1:b3)
    let _cdd8d = sys.create_eq(_cdd79, imm_cd115);
    // _cddad = _cdd69 & _cdd8d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _cddad = sys.create_bitwise_and(_cdd69, _cdd8d);
    // _cddb5 = _cddad & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cdd9d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cddb5 = sys.create_bitwise_and(_cddad, imm_cdd9d);
    // _cddb9 = _cddb5 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cdda9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cddb9 = sys.create_bitwise_and(_cddb5, imm_cdda9);
    // _cc0dd = _cd361 | _cddb9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cc0dd = sys.create_bitwise_or(_cd361, _cddb9);
    // _cdd51 = _cdcc1 | _cddb9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cdd51 = sys.create_bitwise_or(_cdcc1, _cddb9);
    // _cddd5 = _cddb9 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cddc9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cdd45 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cddd5 = sys.create_select(_cddb9, imm_cddc9, imm_cdd45);
    // _cdde1 = _cdcd9 | _cddd5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cdde1 = sys.create_bitwise_or(_cdcd9, _cddd5);
    // _cdde9 = _cddb9 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cddc5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cdddd = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cdde9 = sys.create_select(_cddb9, imm_cddc5, imm_cdddd);
    // _cddf5 = _cdced | _cdde9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cddf5 = sys.create_bitwise_or(_cdced, _cdde9);
    // _cddfd = _cddb9 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cddd1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cddf1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cddfd = sys.create_select(_cddb9, imm_cddd1, imm_cddf1);
    // _cde09 = _cdd01 | _cddfd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cde09 = sys.create_bitwise_or(_cdd01, _cddfd);
    // _cde11 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cde1d = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cde25 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cde11 = sys.create_slice(_c9a75, imm_cde1d, imm_cde25);
    // _cdcc9 = _cdd05 | _cddb9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _cdcc9 = sys.create_bitwise_or(_cdd05, _cddb9);
    // _cde21 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cde39 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cde41 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cde21 = sys.create_slice(_c9a75, imm_cde39, imm_cde41);
    // _cde2d = _cd3e5 | _cddb9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cde2d = sys.create_bitwise_or(_cd3e5, _cddb9);
    // _cde3d = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:75
    let imm_cde55 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cde5d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cde3d = sys.create_slice(_c9a75, imm_cde55, imm_cde5d);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cdc15 = sys.create_conditional_block(_cddb9);
    sys.set_current_block(_cdc15);
    // log('i.csrrw.1110011  | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}', _cde11, _cde21, _cde3d), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "i.csrrw.1110011  | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cde11, _cde21, _cde3d]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cde79 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cde85 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cde8d = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cde79 = sys.create_slice(_c9a75, imm_cde85, imm_cde8d);
    // _cde89 = _cde79 == (115:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cde95 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x73 as u64); // (115:b7)
    let _cde89 = sys.create_eq(_cde79, imm_cde95);
    // _cde99 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cdea9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cdeb1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cde99 = sys.create_slice(_c9a75, imm_cdea9, imm_cdeb1);
    // _cdead = _cde99 == (5:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cdeb9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x5 as u64); // (5:b3)
    let _cdead = sys.create_eq(_cde99, imm_cdeb9);
    // _cdecd = _cde89 & _cdead, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _cdecd = sys.create_bitwise_and(_cde89, _cdead);
    // _cded5 = _cdecd & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cdebd = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cded5 = sys.create_bitwise_and(_cdecd, imm_cdebd);
    // _cded9 = _cded5 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cdec9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cded9 = sys.create_bitwise_and(_cded5, imm_cdec9);
    // _cde69 = _cc0dd | _cded9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cde69 = sys.create_bitwise_or(_cc0dd, _cded9);
    // _cde71 = _cdd51 | _cded9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cde71 = sys.create_bitwise_or(_cdd51, _cded9);
    // _cdef5 = _cded9 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cdee9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cdd99 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cdef5 = sys.create_select(_cded9, imm_cdee9, imm_cdd99);
    // _cdf01 = _cdde1 | _cdef5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cdf01 = sys.create_bitwise_or(_cdde1, _cdef5);
    // _cdf09 = _cded9 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cdee5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cdefd = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cdf09 = sys.create_select(_cded9, imm_cdee5, imm_cdefd);
    // _cdf15 = _cddf5 | _cdf09, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cdf15 = sys.create_bitwise_or(_cddf5, _cdf09);
    // _cdf1d = _cded9 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cdef1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cdf11 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cdf1d = sys.create_select(_cded9, imm_cdef1, imm_cdf11);
    // _cdf29 = _cde09 | _cdf1d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cdf29 = sys.create_bitwise_or(_cde09, _cdf1d);
    // _cdf31 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cdf3d = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cdf45 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cdf31 = sys.create_slice(_c9a75, imm_cdf3d, imm_cdf45);
    // _cdde5 = _cdcc9 | _cded9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _cdde5 = sys.create_bitwise_or(_cdcc9, _cded9);
    // _cdf41 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cdf59 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cdf61 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cdf41 = sys.create_slice(_c9a75, imm_cdf59, imm_cdf61);
    // _cdf4d = _cde2d | _cded9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cdf4d = sys.create_bitwise_or(_cde2d, _cded9);
    // _cdf5d = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:75
    let imm_cdf75 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cdf7d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cdf5d = sys.create_slice(_c9a75, imm_cdf75, imm_cdf7d);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cde0d = sys.create_conditional_block(_cded9);
    sys.set_current_block(_cde0d);
    // log('i.csrrwi.1110011 | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}', _cdf31, _cdf41, _cdf5d), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "i.csrrwi.1110011 | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cdf31, _cdf41, _cdf5d]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cdf99 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cdfa5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cdfad = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cdf99 = sys.create_slice(_c9a75, imm_cdfa5, imm_cdfad);
    // _cdfa9 = _cdf99 == (19:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cdfb5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x13 as u64); // (19:b7)
    let _cdfa9 = sys.create_eq(_cdf99, imm_cdfb5);
    // _cdfb9 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cdfc9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cdfd1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cdfb9 = sys.create_slice(_c9a75, imm_cdfc9, imm_cdfd1);
    // _cdfcd = _cdfb9 == (1:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cdfd9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x1 as u64); // (1:b3)
    let _cdfcd = sys.create_eq(_cdfb9, imm_cdfd9);
    // _cdfe9 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:92
    let imm_cdff5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cdffd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cdfe9 = sys.create_slice(_c9a75, imm_cdff5, imm_cdffd);
    // _cac89 = _cdfe9[(6:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:92
    let imm_ce40d = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let imm_ce415 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cac89 = sys.create_slice(_cdfe9, imm_ce40d, imm_ce415);
    // _ce421 = _cac89 == (0:b6), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:92
    let imm_cdf19 = sys.get_const_int(assassyn::ir::DataType::bits_ty(6), 0x0 as u64); // (0:b6)
    let _ce421 = sys.create_eq(_cac89, imm_cdf19);
    // _ce429 = _cdfa9 & _cdfcd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _ce429 = sys.create_bitwise_and(_cdfa9, _cdfcd);
    // _ce42d = _ce429 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cdfdd = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _ce42d = sys.create_bitwise_and(_ce429, imm_cdfdd);
    // _ce431 = _ce42d & _ce421, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _ce431 = sys.create_bitwise_and(_ce42d, _ce421);
    // _cdf89 = _cde69 | _ce431, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cdf89 = sys.create_bitwise_or(_cde69, _ce431);
    // _cdf91 = _cde71 | _ce431, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cdf91 = sys.create_bitwise_or(_cde71, _ce431);
    // _ce44d = _ce431 ? (32:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_ce441 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x20 as u64); // (32:b16)
    let imm_cddf9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ce44d = sys.create_select(_ce431, imm_ce441, imm_cddf9);
    // _ce459 = _cdf01 | _ce44d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _ce459 = sys.create_bitwise_or(_cdf01, _ce44d);
    // _ce461 = _ce431 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_ce43d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_ce455 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ce461 = sys.create_select(_ce431, imm_ce43d, imm_ce455);
    // _ce46d = _cdf15 | _ce461, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _ce46d = sys.create_bitwise_or(_cdf15, _ce461);
    // _ce475 = _ce431 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_ce449 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_ce469 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _ce475 = sys.create_select(_ce431, imm_ce449, imm_ce469);
    // _ce481 = _cdf29 | _ce475, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _ce481 = sys.create_bitwise_or(_cdf29, _ce475);
    // _ce48d = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_ce499 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_ce4a1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _ce48d = sys.create_slice(_c9a75, imm_ce499, imm_ce4a1);
    // _ce47d = _cdde5 | _ce431, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _ce47d = sys.create_bitwise_or(_cdde5, _ce431);
    // _ce49d = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_ce4b5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_ce4bd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _ce49d = sys.create_slice(_c9a75, imm_ce4b5, imm_ce4bd);
    // _ce4a9 = _cdf4d | _ce431, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _ce4a9 = sys.create_bitwise_or(_cdf4d, _ce431);
    // _ce4b9 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:75
    let imm_ce4d1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_ce4d9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _ce4b9 = sys.create_slice(_c9a75, imm_ce4d1, imm_ce4d9);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cdfbd = sys.create_conditional_block(_ce431);
    sys.set_current_block(_cdfbd);
    // log('i.slli.0010011   | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}', _ce48d, _ce49d, _ce4b9), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "i.slli.0010011   | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_ce48d, _ce49d, _ce4b9]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _ce4f1 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:53
    let imm_ce4fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_ce505 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _ce4f1 = sys.create_slice(_c9a75, imm_ce4fd, imm_ce505);
    // _ce501 = _ce4f1 == (51:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:53
    let imm_ce50d = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x33 as u64); // (51:b7)
    let _ce501 = sys.create_eq(_ce4f1, imm_ce50d);
    // _ce511 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:54
    let imm_ce521 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_ce529 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _ce511 = sys.create_slice(_c9a75, imm_ce521, imm_ce529);
    // _ce525 = _ce511 == (1:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:54
    let imm_ce531 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x1 as u64); // (1:b3)
    let _ce525 = sys.create_eq(_ce511, imm_ce531);
    // _ce535 = _c9a75[(25:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:55
    let imm_ce545 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x19 as u64); // (25:u5)
    let imm_ce54d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _ce535 = sys.create_slice(_c9a75, imm_ce545, imm_ce54d);
    // _ce549 = _ce535 == (0:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:55
    let imm_ce555 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x0 as u64); // (0:b7)
    let _ce549 = sys.create_eq(_ce535, imm_ce555);
    // _ce561 = _ce501 & _ce525, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let _ce561 = sys.create_bitwise_and(_ce501, _ce525);
    // _ce569 = _ce561 & _ce549, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let _ce569 = sys.create_bitwise_and(_ce561, _ce549);
    // _ce56d = _ce569 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let imm_ce559 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _ce56d = sys.create_bitwise_and(_ce569, imm_ce559);
    // _ce4e5 = _cad95 | _ce56d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _ce4e5 = sys.create_bitwise_or(_cad95, _ce56d);
    // _ce585 = _cdf91 | _ce56d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _ce585 = sys.create_bitwise_or(_cdf91, _ce56d);
    // _ce591 = _ce56d ? (32:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_ce581 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x20 as u64); // (32:b16)
    let imm_ce411 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ce591 = sys.create_select(_ce56d, imm_ce581, imm_ce411);
    // _ce59d = _ce459 | _ce591, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _ce59d = sys.create_bitwise_or(_ce459, _ce591);
    // _ce5a5 = _ce56d ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_ce57d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_ce599 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ce5a5 = sys.create_select(_ce56d, imm_ce57d, imm_ce599);
    // _ce5b1 = _ce46d | _ce5a5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _ce5b1 = sys.create_bitwise_or(_ce46d, _ce5a5);
    // _ce5b9 = _ce56d ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_ce589 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_ce5ad = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _ce5b9 = sys.create_select(_ce56d, imm_ce589, imm_ce5ad);
    // _ce5c5 = _ce481 | _ce5b9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _ce5c5 = sys.create_bitwise_or(_ce481, _ce5b9);
    // _ce5cd = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_ce5d9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_ce5e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _ce5cd = sys.create_slice(_c9a75, imm_ce5d9, imm_ce5e1);
    // _ce485 = _ce47d | _ce56d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _ce485 = sys.create_bitwise_or(_ce47d, _ce56d);
    // _ce5dd = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_ce5f5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_ce5fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _ce5dd = sys.create_slice(_c9a75, imm_ce5f5, imm_ce5fd);
    // _ce5e9 = _ce4a9 | _ce56d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _ce5e9 = sys.create_bitwise_or(_ce4a9, _ce56d);
    // _ce5f9 = _c9a75[(20:u5):(24:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:65
    let imm_ce611 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_ce619 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x18 as u64); // (24:u5)
    let _ce5f9 = sys.create_slice(_c9a75, imm_ce611, imm_ce619);
    // _ce605 = _cd2c5 | _ce56d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:66
    let _ce605 = sys.create_bitwise_or(_cd2c5, _ce56d);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _ce621 = sys.create_conditional_block(_ce56d);
    sys.set_current_block(_ce621);
    // log('r.sll.0110011    | rd: x{:02}      | rs1: x{:02}      | rs2: x{:02}      ', _ce5cd, _ce5dd, _ce5f9), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "r.sll.0110011    | rd: x{:02}      | rs1: x{:02}      | rs2: x{:02}      ".into(),
    );
    sys.create_log(fmt, vec![_ce5cd, _ce5dd, _ce5f9]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _ce635 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_ce641 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_ce649 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _ce635 = sys.create_slice(_c9a75, imm_ce641, imm_ce649);
    // _ce655 = _ce635 == (19:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_ce645 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x13 as u64); // (19:b7)
    let _ce655 = sys.create_eq(_ce635, imm_ce645);
    // _ce659 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_ce669 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_ce671 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _ce659 = sys.create_slice(_c9a75, imm_ce669, imm_ce671);
    // _ce66d = _ce659 == (5:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_ce679 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x5 as u64); // (5:b3)
    let _ce66d = sys.create_eq(_ce659, imm_ce679);
    // _ce689 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:92
    let imm_ce695 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_ce69d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _ce689 = sys.create_slice(_c9a75, imm_ce695, imm_ce69d);
    // _ce681 = _ce689[(6:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:92
    let imm_ce6a9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let imm_ce6b1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _ce681 = sys.create_slice(_ce689, imm_ce6a9, imm_ce6b1);
    // _ce6bd = _ce681 == (16:b6), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:92
    let imm_ce6ad = sys.get_const_int(assassyn::ir::DataType::bits_ty(6), 0x10 as u64); // (16:b6)
    let _ce6bd = sys.create_eq(_ce681, imm_ce6ad);
    // _ce6c5 = _ce655 & _ce66d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _ce6c5 = sys.create_bitwise_and(_ce655, _ce66d);
    // _ce6c9 = _ce6c5 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_ce67d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _ce6c9 = sys.create_bitwise_and(_ce6c5, imm_ce67d);
    // _ce6cd = _ce6c9 & _ce6bd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _ce6cd = sys.create_bitwise_and(_ce6c9, _ce6bd);
    // _ce571 = _cdf89 | _ce6cd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _ce571 = sys.create_bitwise_or(_cdf89, _ce6cd);
    // _ce55d = _ce585 | _ce6cd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _ce55d = sys.create_bitwise_or(_ce585, _ce6cd);
    // _ce6e1 = _ce6cd ? (128:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_ce6dd = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x80 as u64); // (128:b16)
    let imm_ce625 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ce6e1 = sys.create_select(_ce6cd, imm_ce6dd, imm_ce625);
    // _ce6f1 = _ce59d | _ce6e1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _ce6f1 = sys.create_bitwise_or(_ce59d, _ce6e1);
    // _ce6f9 = _ce6cd ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_ce6d9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_ce6ed = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ce6f9 = sys.create_select(_ce6cd, imm_ce6d9, imm_ce6ed);
    // _ce705 = _ce5b1 | _ce6f9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _ce705 = sys.create_bitwise_or(_ce5b1, _ce6f9);
    // _ce70d = _ce6cd ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_ce6e5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_ce701 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _ce70d = sys.create_select(_ce6cd, imm_ce6e5, imm_ce701);
    // _ce719 = _ce5c5 | _ce70d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _ce719 = sys.create_bitwise_or(_ce5c5, _ce70d);
    // _ce721 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_ce72d = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_ce735 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _ce721 = sys.create_slice(_c9a75, imm_ce72d, imm_ce735);
    // _ce71d = _ce485 | _ce6cd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _ce71d = sys.create_bitwise_or(_ce485, _ce6cd);
    // _ce731 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_ce749 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_ce751 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _ce731 = sys.create_slice(_c9a75, imm_ce749, imm_ce751);
    // _ce73d = _ce5e9 | _ce6cd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _ce73d = sys.create_bitwise_or(_ce5e9, _ce6cd);
    // _ce74d = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:75
    let imm_ce765 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_ce76d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _ce74d = sys.create_slice(_c9a75, imm_ce765, imm_ce76d);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _ce759 = sys.create_conditional_block(_ce6cd);
    sys.set_current_block(_ce759);
    // log('i.srai.0010011   | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}', _ce721, _ce731, _ce74d), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "i.srai.0010011   | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_ce721, _ce731, _ce74d]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _ce789 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_ce795 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_ce79d = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _ce789 = sys.create_slice(_c9a75, imm_ce795, imm_ce79d);
    // _ce799 = _ce789 == (19:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_ce7a5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x13 as u64); // (19:b7)
    let _ce799 = sys.create_eq(_ce789, imm_ce7a5);
    // _ce7a9 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_ce7b9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_ce7c1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _ce7a9 = sys.create_slice(_c9a75, imm_ce7b9, imm_ce7c1);
    // _ce7bd = _ce7a9 == (5:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_ce7c9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x5 as u64); // (5:b3)
    let _ce7bd = sys.create_eq(_ce7a9, imm_ce7c9);
    // _ce7d9 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:92
    let imm_ce7e5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_ce7ed = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _ce7d9 = sys.create_slice(_c9a75, imm_ce7e5, imm_ce7ed);
    // _ce5c1 = _ce7d9[(6:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:92
    let imm_ce7f9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let imm_ce7fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _ce5c1 = sys.create_slice(_ce7d9, imm_ce7f9, imm_ce7fd);
    // _cec11 = _ce5c1 == (0:b6), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:92
    let imm_ce709 = sys.get_const_int(assassyn::ir::DataType::bits_ty(6), 0x0 as u64); // (0:b6)
    let _cec11 = sys.create_eq(_ce5c1, imm_ce709);
    // _cec19 = _ce799 & _ce7bd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _cec19 = sys.create_bitwise_and(_ce799, _ce7bd);
    // _cec1d = _cec19 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_ce7cd = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cec1d = sys.create_bitwise_and(_cec19, imm_ce7cd);
    // _cec21 = _cec1d & _cec11, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _cec21 = sys.create_bitwise_and(_cec1d, _cec11);
    // _ce779 = _ce571 | _cec21, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _ce779 = sys.create_bitwise_or(_ce571, _cec21);
    // _ce781 = _ce55d | _cec21, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _ce781 = sys.create_bitwise_or(_ce55d, _cec21);
    // _cec3d = _cec21 ? (4096:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cec31 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1000 as u64); // (4096:b16)
    let imm_ce5a1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cec3d = sys.create_select(_cec21, imm_cec31, imm_ce5a1);
    // _cec49 = _ce6f1 | _cec3d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cec49 = sys.create_bitwise_or(_ce6f1, _cec3d);
    // _cec51 = _cec21 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cec2d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cec45 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cec51 = sys.create_select(_cec21, imm_cec2d, imm_cec45);
    // _cec5d = _ce705 | _cec51, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cec5d = sys.create_bitwise_or(_ce705, _cec51);
    // _cec65 = _cec21 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cec39 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cec59 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cec65 = sys.create_select(_cec21, imm_cec39, imm_cec59);
    // _cec71 = _ce719 | _cec65, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cec71 = sys.create_bitwise_or(_ce719, _cec65);
    // _cec79 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cec85 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cec8d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cec79 = sys.create_slice(_c9a75, imm_cec85, imm_cec8d);
    // _cec75 = _ce71d | _cec21, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _cec75 = sys.create_bitwise_or(_ce71d, _cec21);
    // _cec89 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_ceca1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_ceca9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cec89 = sys.create_slice(_c9a75, imm_ceca1, imm_ceca9);
    // _cec95 = _ce73d | _cec21, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cec95 = sys.create_bitwise_or(_ce73d, _cec21);
    // _ceca5 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:75
    let imm_cecbd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cecc5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _ceca5 = sys.create_slice(_c9a75, imm_cecbd, imm_cecc5);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _ce7ad = sys.create_conditional_block(_cec21);
    sys.set_current_block(_ce7ad);
    // log('i.srli.0010011   | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}', _cec79, _cec89, _ceca5), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "i.srli.0010011   | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cec79, _cec89, _ceca5]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cecdd = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:53
    let imm_cece9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cecf1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cecdd = sys.create_slice(_c9a75, imm_cece9, imm_cecf1);
    // _ceced = _cecdd == (51:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:53
    let imm_cecf9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x33 as u64); // (51:b7)
    let _ceced = sys.create_eq(_cecdd, imm_cecf9);
    // _cecfd = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:54
    let imm_ced0d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_ced15 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cecfd = sys.create_slice(_c9a75, imm_ced0d, imm_ced15);
    // _ced11 = _cecfd == (3:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:54
    let imm_ced1d = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x3 as u64); // (3:b3)
    let _ced11 = sys.create_eq(_cecfd, imm_ced1d);
    // _ced21 = _c9a75[(25:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:55
    let imm_ced31 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x19 as u64); // (25:u5)
    let imm_ced39 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _ced21 = sys.create_slice(_c9a75, imm_ced31, imm_ced39);
    // _ced35 = _ced21 == (0:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:55
    let imm_ced41 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x0 as u64); // (0:b7)
    let _ced35 = sys.create_eq(_ced21, imm_ced41);
    // _ced4d = _ceced & _ced11, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let _ced4d = sys.create_bitwise_and(_ceced, _ced11);
    // _ced55 = _ced4d & _ced35, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let _ced55 = sys.create_bitwise_and(_ced4d, _ced35);
    // _ced59 = _ced55 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let imm_ced45 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _ced59 = sys.create_bitwise_and(_ced55, imm_ced45);
    // _cecd1 = _ce4e5 | _ced59, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cecd1 = sys.create_bitwise_or(_ce4e5, _ced59);
    // _ced6d = _ce781 | _ced59, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _ced6d = sys.create_bitwise_or(_ce781, _ced59);
    // _ced79 = _ced59 ? (1024:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_ced69 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x400 as u64); // (1024:b16)
    let imm_cec05 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ced79 = sys.create_select(_ced59, imm_ced69, imm_cec05);
    // _ced85 = _cec49 | _ced79, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _ced85 = sys.create_bitwise_or(_cec49, _ced79);
    // _ced8d = _ced59 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_ced65 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_ced81 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ced8d = sys.create_select(_ced59, imm_ced65, imm_ced81);
    // _ced99 = _cec5d | _ced8d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _ced99 = sys.create_bitwise_or(_cec5d, _ced8d);
    // _ceda1 = _ced59 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_ced71 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_ced95 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _ceda1 = sys.create_select(_ced59, imm_ced71, imm_ced95);
    // _cedad = _cec71 | _ceda1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cedad = sys.create_bitwise_or(_cec71, _ceda1);
    // _cedb5 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cedc1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cedc9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cedb5 = sys.create_slice(_c9a75, imm_cedc1, imm_cedc9);
    // _cedb1 = _cec75 | _ced59, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _cedb1 = sys.create_bitwise_or(_cec75, _ced59);
    // _cedc5 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_ceddd = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cede5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cedc5 = sys.create_slice(_c9a75, imm_ceddd, imm_cede5);
    // _cedd1 = _cec95 | _ced59, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cedd1 = sys.create_bitwise_or(_cec95, _ced59);
    // _cede1 = _c9a75[(20:u5):(24:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:65
    let imm_cedf9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cee01 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x18 as u64); // (24:u5)
    let _cede1 = sys.create_slice(_c9a75, imm_cedf9, imm_cee01);
    // _ceded = _ce605 | _ced59, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:66
    let _ceded = sys.create_bitwise_or(_ce605, _ced59);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cee09 = sys.create_conditional_block(_ced59);
    sys.set_current_block(_cee09);
    // log('r.sltu.0110011   | rd: x{:02}      | rs1: x{:02}      | rs2: x{:02}      ', _cedb5, _cedc5, _cede1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "r.sltu.0110011   | rd: x{:02}      | rs1: x{:02}      | rs2: x{:02}      ".into(),
    );
    sys.create_log(fmt, vec![_cedb5, _cedc5, _cede1]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cee19 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:53
    let imm_cee25 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cee2d = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cee19 = sys.create_slice(_c9a75, imm_cee25, imm_cee2d);
    // _cee39 = _cee19 == (51:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:53
    let imm_cee29 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x33 as u64); // (51:b7)
    let _cee39 = sys.create_eq(_cee19, imm_cee29);
    // _cee3d = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:54
    let imm_cee4d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cee55 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cee3d = sys.create_slice(_c9a75, imm_cee4d, imm_cee55);
    // _cee51 = _cee3d == (5:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:54
    let imm_cee5d = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x5 as u64); // (5:b3)
    let _cee51 = sys.create_eq(_cee3d, imm_cee5d);
    // _cee61 = _c9a75[(25:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:55
    let imm_cee71 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x19 as u64); // (25:u5)
    let imm_cee79 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cee61 = sys.create_slice(_c9a75, imm_cee71, imm_cee79);
    // _cee75 = _cee61 == (0:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:55
    let imm_cee81 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x0 as u64); // (0:b7)
    let _cee75 = sys.create_eq(_cee61, imm_cee81);
    // _cee8d = _cee39 & _cee51, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let _cee8d = sys.create_bitwise_and(_cee39, _cee51);
    // _cee95 = _cee8d & _cee75, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let _cee95 = sys.create_bitwise_and(_cee8d, _cee75);
    // _cee99 = _cee95 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let imm_cee85 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cee99 = sys.create_bitwise_and(_cee95, imm_cee85);
    // _cee0d = _cecd1 | _cee99, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cee0d = sys.create_bitwise_or(_cecd1, _cee99);
    // _ceead = _ced6d | _cee99, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _ceead = sys.create_bitwise_or(_ced6d, _cee99);
    // _ceeb9 = _cee99 ? (4096:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_ceea9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1000 as u64); // (4096:b16)
    let imm_ced49 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ceeb9 = sys.create_select(_cee99, imm_ceea9, imm_ced49);
    // _ceec5 = _ced85 | _ceeb9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _ceec5 = sys.create_bitwise_or(_ced85, _ceeb9);
    // _ceecd = _cee99 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_ceea5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_ceec1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ceecd = sys.create_select(_cee99, imm_ceea5, imm_ceec1);
    // _ceed9 = _ced99 | _ceecd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _ceed9 = sys.create_bitwise_or(_ced99, _ceecd);
    // _ceee1 = _cee99 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_ceeb1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_ceed5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _ceee1 = sys.create_select(_cee99, imm_ceeb1, imm_ceed5);
    // _ceeed = _cedad | _ceee1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _ceeed = sys.create_bitwise_or(_cedad, _ceee1);
    // _ceef5 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cef01 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cef09 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _ceef5 = sys.create_slice(_c9a75, imm_cef01, imm_cef09);
    // _ced9d = _cedb1 | _cee99, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _ced9d = sys.create_bitwise_or(_cedb1, _cee99);
    // _cef05 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cef1d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cef25 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cef05 = sys.create_slice(_c9a75, imm_cef1d, imm_cef25);
    // _cef11 = _cedd1 | _cee99, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cef11 = sys.create_bitwise_or(_cedd1, _cee99);
    // _cef21 = _c9a75[(20:u5):(24:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:65
    let imm_cef39 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cef41 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x18 as u64); // (24:u5)
    let _cef21 = sys.create_slice(_c9a75, imm_cef39, imm_cef41);
    // _ceef1 = _ceded | _cee99, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:66
    let _ceef1 = sys.create_bitwise_or(_ceded, _cee99);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cef49 = sys.create_conditional_block(_cee99);
    sys.set_current_block(_cef49);
    // log('r.srl.0110011    | rd: x{:02}      | rs1: x{:02}      | rs2: x{:02}      ', _ceef5, _cef05, _cef21), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "r.srl.0110011    | rd: x{:02}      | rs1: x{:02}      | rs2: x{:02}      ".into(),
    );
    sys.create_log(fmt, vec![_ceef5, _cef05, _cef21]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cef59 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:53
    let imm_cef65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cef6d = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cef59 = sys.create_slice(_c9a75, imm_cef65, imm_cef6d);
    // _cef79 = _cef59 == (51:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:53
    let imm_cef69 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x33 as u64); // (51:b7)
    let _cef79 = sys.create_eq(_cef59, imm_cef69);
    // _cef7d = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:54
    let imm_cef8d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cef95 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cef7d = sys.create_slice(_c9a75, imm_cef8d, imm_cef95);
    // _cef91 = _cef7d == (5:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:54
    let imm_cef9d = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x5 as u64); // (5:b3)
    let _cef91 = sys.create_eq(_cef7d, imm_cef9d);
    // _cefa1 = _c9a75[(25:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:55
    let imm_cefb1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x19 as u64); // (25:u5)
    let imm_cefb9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cefa1 = sys.create_slice(_c9a75, imm_cefb1, imm_cefb9);
    // _cefb5 = _cefa1 == (32:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:55
    let imm_cefc1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x20 as u64); // (32:b7)
    let _cefb5 = sys.create_eq(_cefa1, imm_cefc1);
    // _cefcd = _cef79 & _cef91, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let _cefcd = sys.create_bitwise_and(_cef79, _cef91);
    // _cefd5 = _cefcd & _cefb5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let _cefd5 = sys.create_bitwise_and(_cefcd, _cefb5);
    // _cefd9 = _cefd5 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let imm_cefc5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cefd9 = sys.create_bitwise_and(_cefd5, imm_cefc5);
    // _cef4d = _cee0d | _cefd9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cef4d = sys.create_bitwise_or(_cee0d, _cefd9);
    // _cefed = _ceead | _cefd9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cefed = sys.create_bitwise_or(_ceead, _cefd9);
    // _ceff9 = _cefd9 ? (128:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cefe9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x80 as u64); // (128:b16)
    let imm_cee89 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _ceff9 = sys.create_select(_cefd9, imm_cefe9, imm_cee89);
    // _cf409 = _ceec5 | _ceff9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cf409 = sys.create_bitwise_or(_ceec5, _ceff9);
    // _cf411 = _cefd9 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cefe5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cf405 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cf411 = sys.create_select(_cefd9, imm_cefe5, imm_cf405);
    // _cf41d = _ceed9 | _cf411, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cf41d = sys.create_bitwise_or(_ceed9, _cf411);
    // _cf425 = _cefd9 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_ceff1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cf419 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cf425 = sys.create_select(_cefd9, imm_ceff1, imm_cf419);
    // _cf431 = _ceeed | _cf425, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cf431 = sys.create_bitwise_or(_ceeed, _cf425);
    // _cf439 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cf445 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cf44d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cf439 = sys.create_slice(_c9a75, imm_cf445, imm_cf44d);
    // _ceff5 = _ced9d | _cefd9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _ceff5 = sys.create_bitwise_or(_ced9d, _cefd9);
    // _cf449 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cf461 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cf469 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cf449 = sys.create_slice(_c9a75, imm_cf461, imm_cf469);
    // _cf455 = _cef11 | _cefd9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cf455 = sys.create_bitwise_or(_cef11, _cefd9);
    // _cf465 = _c9a75[(20:u5):(24:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:65
    let imm_cf47d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cf485 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x18 as u64); // (24:u5)
    let _cf465 = sys.create_slice(_c9a75, imm_cf47d, imm_cf485);
    // _ce515 = _ceef1 | _cefd9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:66
    let _ce515 = sys.create_bitwise_or(_ceef1, _cefd9);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cf48d = sys.create_conditional_block(_cefd9);
    sys.set_current_block(_cf48d);
    // log('r.sra.0110011    | rd: x{:02}      | rs1: x{:02}      | rs2: x{:02}      ', _cf439, _cf449, _cf465), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "r.sra.0110011    | rd: x{:02}      | rs1: x{:02}      | rs2: x{:02}      ".into(),
    );
    sys.create_log(fmt, vec![_cf439, _cf449, _cf465]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cf49d = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:53
    let imm_cf4a9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cf4b1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cf49d = sys.create_slice(_c9a75, imm_cf4a9, imm_cf4b1);
    // _cf4bd = _cf49d == (115:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:53
    let imm_cf4ad = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x73 as u64); // (115:b7)
    let _cf4bd = sys.create_eq(_cf49d, imm_cf4ad);
    // _cf4c1 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:54
    let imm_cf4d1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cf4d9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cf4c1 = sys.create_slice(_c9a75, imm_cf4d1, imm_cf4d9);
    // _cf4d5 = _cf4c1 == (0:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:54
    let imm_cf4e1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x0 as u64); // (0:b3)
    let _cf4d5 = sys.create_eq(_cf4c1, imm_cf4e1);
    // _cf4e5 = _c9a75[(25:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:55
    let imm_cf4f5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x19 as u64); // (25:u5)
    let imm_cf4fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cf4e5 = sys.create_slice(_c9a75, imm_cf4f5, imm_cf4fd);
    // _cf4f9 = _cf4e5 == (24:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:55
    let imm_cf505 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x18 as u64); // (24:b7)
    let _cf4f9 = sys.create_eq(_cf4e5, imm_cf505);
    // _cf509 = _c9a75[(20:u5):(24:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:57
    let imm_cf519 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cf521 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x18 as u64); // (24:u5)
    let _cf509 = sys.create_slice(_c9a75, imm_cf519, imm_cf521);
    // _cf51d = _cf509 == (2:b5), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:57
    let imm_cf529 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x2 as u64); // (2:b5)
    let _cf51d = sys.create_eq(_cf509, imm_cf529);
    // _cf531 = _cf4bd & _cf4d5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let _cf531 = sys.create_bitwise_and(_cf4bd, _cf4d5);
    // _cf535 = _cf531 & _cf4f9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let _cf535 = sys.create_bitwise_and(_cf531, _cf4f9);
    // _cf539 = _cf535 & _cf51d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let _cf539 = sys.create_bitwise_and(_cf535, _cf51d);
    // _cefc9 = _cef4d | _cf539, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cefc9 = sys.create_bitwise_or(_cef4d, _cf539);
    // _cf54d = _cefed | _cf539, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cf54d = sys.create_bitwise_or(_cefed, _cf539);
    // _cf559 = _cf539 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cf549 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cf491 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cf559 = sys.create_select(_cf539, imm_cf549, imm_cf491);
    // _cf565 = _cf409 | _cf559, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cf565 = sys.create_bitwise_or(_cf409, _cf559);
    // _cf56d = _cf539 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cf545 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cf561 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cf56d = sys.create_select(_cf539, imm_cf545, imm_cf561);
    // _cf579 = _cf41d | _cf56d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cf579 = sys.create_bitwise_or(_cf41d, _cf56d);
    // _cf581 = _cf539 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cf551 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cf575 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cf581 = sys.create_select(_cf539, imm_cf551, imm_cf575);
    // _cf58d = _cf431 | _cf581, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cf58d = sys.create_bitwise_or(_cf431, _cf581);
    // _cf595 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cf5a1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cf5a9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cf595 = sys.create_slice(_c9a75, imm_cf5a1, imm_cf5a9);
    // _ceedd = _ceff5 | _cf539, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _ceedd = sys.create_bitwise_or(_ceff5, _cf539);
    // _cf5a5 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cf5bd = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cf5c5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cf5a5 = sys.create_slice(_c9a75, imm_cf5bd, imm_cf5c5);
    // _cf5b1 = _cf455 | _cf539, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cf5b1 = sys.create_bitwise_or(_cf455, _cf539);
    // _cf5c1 = _c9a75[(20:u5):(24:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:65
    let imm_cf5d9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cf5e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x18 as u64); // (24:u5)
    let _cf5c1 = sys.create_slice(_c9a75, imm_cf5d9, imm_cf5e1);
    // _cdeed = _ce515 | _cf539, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:66
    let _cdeed = sys.create_bitwise_or(_ce515, _cf539);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cf5e9 = sys.create_conditional_block(_cf539);
    sys.set_current_block(_cf5e9);
    // log('r.mret.1110011   | rd: x{:02}      | rs1: x{:02}      | rs2: x{:02}      ', _cf595, _cf5a5, _cf5c1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "r.mret.1110011   | rd: x{:02}      | rs1: x{:02}      | rs2: x{:02}      ".into(),
    );
    sys.create_log(fmt, vec![_cf595, _cf5a5, _cf5c1]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cf5fd = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cf609 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cf611 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cf5fd = sys.create_slice(_c9a75, imm_cf609, imm_cf611);
    // _cf61d = _cf5fd == (15:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cf60d = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0xf as u64); // (15:b7)
    let _cf61d = sys.create_eq(_cf5fd, imm_cf60d);
    // _cf621 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cf631 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cf639 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cf621 = sys.create_slice(_c9a75, imm_cf631, imm_cf639);
    // _cf635 = _cf621 == (0:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cf641 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x0 as u64); // (0:b3)
    let _cf635 = sys.create_eq(_cf621, imm_cf641);
    // _cf655 = _cf61d & _cf635, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _cf655 = sys.create_bitwise_and(_cf61d, _cf635);
    // _cf65d = _cf655 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cf645 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cf65d = sys.create_bitwise_and(_cf655, imm_cf645);
    // _cf661 = _cf65d & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cf651 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cf661 = sys.create_bitwise_and(_cf65d, imm_cf651);
    // _cf5ed = _ce779 | _cf661, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cf5ed = sys.create_bitwise_or(_ce779, _cf661);
    // _cf5f5 = _cf54d | _cf661, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cf5f5 = sys.create_bitwise_or(_cf54d, _cf661);
    // _cf67d = _cf661 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cf671 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cf499 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cf67d = sys.create_select(_cf661, imm_cf671, imm_cf499);
    // _cf689 = _cf565 | _cf67d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cf689 = sys.create_bitwise_or(_cf565, _cf67d);
    // _cf691 = _cf661 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cf66d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cf685 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cf691 = sys.create_select(_cf661, imm_cf66d, imm_cf685);
    // _cf69d = _cf579 | _cf691, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cf69d = sys.create_bitwise_or(_cf579, _cf691);
    // _cf6a5 = _cf661 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cf679 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cf699 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cf6a5 = sys.create_select(_cf661, imm_cf679, imm_cf699);
    // _cf6b1 = _cf58d | _cf6a5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cf6b1 = sys.create_bitwise_or(_cf58d, _cf6a5);
    // _cf6ad = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cf6c1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cf6c9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cf6ad = sys.create_slice(_c9a75, imm_cf6c1, imm_cf6c9);
    // _cdf25 = _ceedd | _cf661, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _cdf25 = sys.create_bitwise_or(_ceedd, _cf661);
    // _cf6c5 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cf6dd = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cf6e5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cf6c5 = sys.create_slice(_c9a75, imm_cf6dd, imm_cf6e5);
    // _cf6d1 = _cf5b1 | _cf661, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cf6d1 = sys.create_bitwise_or(_cf5b1, _cf661);
    // _cf6e1 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:75
    let imm_cf6f9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cf701 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cf6e1 = sys.create_slice(_c9a75, imm_cf6f9, imm_cf701);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cf6ed = sys.create_conditional_block(_cf661);
    sys.set_current_block(_cf6ed);
    // log('i.fence.0001111  | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}', _cf6ad, _cf6c5, _cf6e1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "i.fence.0001111  | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cf6ad, _cf6c5, _cf6e1]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cf71d = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cf729 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cf731 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cf71d = sys.create_slice(_c9a75, imm_cf729, imm_cf731);
    // _cf72d = _cf71d == (115:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cf739 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x73 as u64); // (115:b7)
    let _cf72d = sys.create_eq(_cf71d, imm_cf739);
    // _cf73d = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cf74d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cf755 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cf73d = sys.create_slice(_c9a75, imm_cf74d, imm_cf755);
    // _cf751 = _cf73d == (0:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cf75d = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x0 as u64); // (0:b3)
    let _cf751 = sys.create_eq(_cf73d, imm_cf75d);
    // _cf761 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:87
    let imm_cf771 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cf779 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cf761 = sys.create_slice(_c9a75, imm_cf771, imm_cf779);
    // _cf775 = _cf761 == (0:b12), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:87
    let imm_cf781 = sys.get_const_int(assassyn::ir::DataType::bits_ty(12), 0x0 as u64); // (0:b12)
    let _cf775 = sys.create_eq(_cf761, imm_cf781);
    // _cf78d = _cf72d & _cf751, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _cf78d = sys.create_bitwise_and(_cf72d, _cf751);
    // _cf795 = _cf78d & _cf775, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _cf795 = sys.create_bitwise_and(_cf78d, _cf775);
    // _cf799 = _cf795 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cf785 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cf799 = sys.create_bitwise_and(_cf795, imm_cf785);
    // _cf70d = _cf5ed | _cf799, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cf70d = sys.create_bitwise_or(_cf5ed, _cf799);
    // _cf715 = _cf5f5 | _cf799, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cf715 = sys.create_bitwise_or(_cf5f5, _cf799);
    // _cf7b5 = _cf799 ? (0:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cf7a1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let imm_cf569 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cf7b5 = sys.create_select(_cf799, imm_cf7a1, imm_cf569);
    // _cf7c1 = _cf689 | _cf7b5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cf7c1 = sys.create_bitwise_or(_cf689, _cf7b5);
    // _cf7c9 = _cf799 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cf7a9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cf7bd = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cf7c9 = sys.create_select(_cf799, imm_cf7a9, imm_cf7bd);
    // _cf7d5 = _cf69d | _cf7c9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cf7d5 = sys.create_bitwise_or(_cf69d, _cf7c9);
    // _cf7dd = _cf799 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cf7b1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cf7d1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cf7dd = sys.create_select(_cf799, imm_cf7b1, imm_cf7d1);
    // _cf7e9 = _cf6b1 | _cf7dd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cf7e9 = sys.create_bitwise_or(_cf6b1, _cf7dd);
    // _cf7f1 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cf7fd = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cfc09 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cf7f1 = sys.create_slice(_c9a75, imm_cf7fd, imm_cfc09);
    // _cf625 = _cdf25 | _cf799, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _cf625 = sys.create_bitwise_or(_cdf25, _cf799);
    // _cfc05 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cfc1d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cfc25 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cfc05 = sys.create_slice(_c9a75, imm_cfc1d, imm_cfc25);
    // _cf7ad = _cf6d1 | _cf799, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cf7ad = sys.create_bitwise_or(_cf6d1, _cf799);
    // _cfc21 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:75
    let imm_cfc39 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cfc41 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cfc21 = sys.create_slice(_c9a75, imm_cfc39, imm_cfc41);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cf7c5 = sys.create_conditional_block(_cf799);
    sys.set_current_block(_cf7c5);
    // log('i.ecall.1110011  | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}', _cf7f1, _cfc05, _cfc21), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "i.ecall.1110011  | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cf7f1, _cfc05, _cfc21]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cfc59 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:53
    let imm_cfc65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cfc6d = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cfc59 = sys.create_slice(_c9a75, imm_cfc65, imm_cfc6d);
    // _cfc69 = _cfc59 == (51:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:53
    let imm_cfc75 = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x33 as u64); // (51:b7)
    let _cfc69 = sys.create_eq(_cfc59, imm_cfc75);
    // _cfc79 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:54
    let imm_cfc89 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cfc91 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cfc79 = sys.create_slice(_c9a75, imm_cfc89, imm_cfc91);
    // _cfc8d = _cfc79 == (7:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:54
    let imm_cf7d9 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x7 as u64); // (7:b3)
    let _cfc8d = sys.create_eq(_cfc79, imm_cf7d9);
    // _cfc9d = _c9a75[(25:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:55
    let imm_cfcad = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x19 as u64); // (25:u5)
    let imm_cfcb5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cfc9d = sys.create_slice(_c9a75, imm_cfcad, imm_cfcb5);
    // _cfcb1 = _cfc9d == (0:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:55
    let imm_cfcbd = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x0 as u64); // (0:b7)
    let _cfcb1 = sys.create_eq(_cfc9d, imm_cfcbd);
    // _cfcc9 = _cfc69 & _cfc8d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let _cfcc9 = sys.create_bitwise_and(_cfc69, _cfc8d);
    // _cfcd1 = _cfcc9 & _cfcb1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let _cfcd1 = sys.create_bitwise_and(_cfcc9, _cfcb1);
    // _cfcd5 = _cfcd1 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:60
    let imm_cfcc1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cfcd5 = sys.create_bitwise_and(_cfcd1, imm_cfcc1);
    // _cf481 = _cefc9 | _cfcd5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cf481 = sys.create_bitwise_or(_cefc9, _cfcd5);
    // _cfce9 = _cf715 | _cfcd5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cfce9 = sys.create_bitwise_or(_cf715, _cfcd5);
    // _cfcf5 = _cfcd5 ? (16:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cfce5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x10 as u64); // (16:b16)
    let imm_cfc4d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cfcf5 = sys.create_select(_cfcd5, imm_cfce5, imm_cfc4d);
    // _cfd01 = _cf7c1 | _cfcf5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cfd01 = sys.create_bitwise_or(_cf7c1, _cfcf5);
    // _cfd09 = _cfcd5 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cfce1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cfcfd = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cfd09 = sys.create_select(_cfcd5, imm_cfce1, imm_cfcfd);
    // _cfd15 = _cf7d5 | _cfd09, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cfd15 = sys.create_bitwise_or(_cf7d5, _cfd09);
    // _cfd1d = _cfcd5 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cfced = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cfd11 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cfd1d = sys.create_select(_cfcd5, imm_cfced, imm_cfd11);
    // _cfd29 = _cf7e9 | _cfd1d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cfd29 = sys.create_bitwise_or(_cf7e9, _cfd1d);
    // _cfd35 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cfd41 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cfd49 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cfd35 = sys.create_slice(_c9a75, imm_cfd41, imm_cfd49);
    // _cfd2d = _cf625 | _cfcd5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _cfd2d = sys.create_bitwise_or(_cf625, _cfcd5);
    // _cfd45 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cfd5d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cfd65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cfd45 = sys.create_slice(_c9a75, imm_cfd5d, imm_cfd65);
    // _cfd51 = _cf7ad | _cfcd5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cfd51 = sys.create_bitwise_or(_cf7ad, _cfcd5);
    // _cfd61 = _c9a75[(20:u5):(24:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:65
    let imm_cfd79 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cfd81 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x18 as u64); // (24:u5)
    let _cfd61 = sys.create_slice(_c9a75, imm_cfd79, imm_cfd81);
    // _cfd6d = _cdeed | _cfcd5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:66
    let _cfd6d = sys.create_bitwise_or(_cdeed, _cfcd5);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cfd89 = sys.create_conditional_block(_cfcd5);
    sys.set_current_block(_cfd89);
    // log('r.and.0110011    | rd: x{:02}      | rs1: x{:02}      | rs2: x{:02}      ', _cfd35, _cfd45, _cfd61), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "r.and.0110011    | rd: x{:02}      | rs1: x{:02}      | rs2: x{:02}      ".into(),
    );
    sys.create_log(fmt, vec![_cfd35, _cfd45, _cfd61]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cfd9d = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cfda9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cfdb1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cfd9d = sys.create_slice(_c9a75, imm_cfda9, imm_cfdb1);
    // _cfdbd = _cfd9d == (19:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cfdad = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x13 as u64); // (19:b7)
    let _cfdbd = sys.create_eq(_cfd9d, imm_cfdad);
    // _cfdc1 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cfdd1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cfdd9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cfdc1 = sys.create_slice(_c9a75, imm_cfdd1, imm_cfdd9);
    // _cfdd5 = _cfdc1 == (7:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cfde1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x7 as u64); // (7:b3)
    let _cfdd5 = sys.create_eq(_cfdc1, imm_cfde1);
    // _cfdf5 = _cfdbd & _cfdd5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _cfdf5 = sys.create_bitwise_and(_cfdbd, _cfdd5);
    // _cfdfd = _cfdf5 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cfde5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cfdfd = sys.create_bitwise_and(_cfdf5, imm_cfde5);
    // _cfe01 = _cfdfd & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cfdf1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cfe01 = sys.create_bitwise_and(_cfdfd, imm_cfdf1);
    // _cfd8d = _cf70d | _cfe01, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cfd8d = sys.create_bitwise_or(_cf70d, _cfe01);
    // _cfd95 = _cfce9 | _cfe01, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cfd95 = sys.create_bitwise_or(_cfce9, _cfe01);
    // _cfe1d = _cfe01 ? (16:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cfe11 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x10 as u64); // (16:b16)
    let imm_cfcc5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cfe1d = sys.create_select(_cfe01, imm_cfe11, imm_cfcc5);
    // _cfe29 = _cfd01 | _cfe1d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cfe29 = sys.create_bitwise_or(_cfd01, _cfe1d);
    // _cfe31 = _cfe01 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cfe0d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cfe25 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cfe31 = sys.create_select(_cfe01, imm_cfe0d, imm_cfe25);
    // _cfe3d = _cfd15 | _cfe31, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cfe3d = sys.create_bitwise_or(_cfd15, _cfe31);
    // _cfe45 = _cfe01 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cfe19 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cfe39 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cfe45 = sys.create_select(_cfe01, imm_cfe19, imm_cfe39);
    // _cfe51 = _cfd29 | _cfe45, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cfe51 = sys.create_bitwise_or(_cfd29, _cfe45);
    // _cfe59 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cfe65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cfe6d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cfe59 = sys.create_slice(_c9a75, imm_cfe65, imm_cfe6d);
    // _cfe55 = _cfd2d | _cfe01, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _cfe55 = sys.create_bitwise_or(_cfd2d, _cfe01);
    // _cfe69 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cfe81 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cfe89 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cfe69 = sys.create_slice(_c9a75, imm_cfe81, imm_cfe89);
    // _cfe75 = _cfd51 | _cfe01, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cfe75 = sys.create_bitwise_or(_cfd51, _cfe01);
    // _cfe85 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:75
    let imm_cfe9d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cfea5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cfe85 = sys.create_slice(_c9a75, imm_cfe9d, imm_cfea5);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cfe91 = sys.create_conditional_block(_cfe01);
    sys.set_current_block(_cfe91);
    // log('i.andi.0010011   | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}', _cfe59, _cfe69, _cfe85), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "i.andi.0010011   | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cfe59, _cfe69, _cfe85]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cfec1 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cfecd = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cfed5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cfec1 = sys.create_slice(_c9a75, imm_cfecd, imm_cfed5);
    // _cfed1 = _cfec1 == (19:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cfedd = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x13 as u64); // (19:b7)
    let _cfed1 = sys.create_eq(_cfec1, imm_cfedd);
    // _cfee1 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cfef1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_cfef9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _cfee1 = sys.create_slice(_c9a75, imm_cfef1, imm_cfef9);
    // _cfef5 = _cfee1 == (6:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cff01 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x6 as u64); // (6:b3)
    let _cfef5 = sys.create_eq(_cfee1, imm_cff01);
    // _cff15 = _cfed1 & _cfef5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _cff15 = sys.create_bitwise_and(_cfed1, _cfef5);
    // _cff1d = _cff15 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cff05 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cff1d = sys.create_bitwise_and(_cff15, imm_cff05);
    // _cff21 = _cff1d & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_cff11 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _cff21 = sys.create_bitwise_and(_cff1d, imm_cff11);
    // _cfeb1 = _cfd8d | _cff21, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cfeb1 = sys.create_bitwise_or(_cfd8d, _cff21);
    // _cfeb9 = _cfd95 | _cff21, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cfeb9 = sys.create_bitwise_or(_cfd95, _cff21);
    // _cff3d = _cff21 ? (8192:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_cff31 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x2000 as u64); // (8192:b16)
    let imm_cfd05 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cff3d = sys.create_select(_cff21, imm_cff31, imm_cfd05);
    // _cff49 = _cfe29 | _cff3d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _cff49 = sys.create_bitwise_or(_cfe29, _cff3d);
    // _cff51 = _cff21 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_cff2d = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_cff45 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _cff51 = sys.create_select(_cff21, imm_cff2d, imm_cff45);
    // _cff5d = _cfe3d | _cff51, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _cff5d = sys.create_bitwise_or(_cfe3d, _cff51);
    // _cff65 = _cff21 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_cff39 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_cff59 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _cff65 = sys.create_select(_cff21, imm_cff39, imm_cff59);
    // _cff71 = _cfe51 | _cff65, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _cff71 = sys.create_bitwise_or(_cfe51, _cff65);
    // _cff79 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_cff85 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_cff8d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _cff79 = sys.create_slice(_c9a75, imm_cff85, imm_cff8d);
    // _cfd25 = _cfe55 | _cff21, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _cfd25 = sys.create_bitwise_or(_cfe55, _cff21);
    // _cff89 = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_cffa1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_cffa9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _cff89 = sys.create_slice(_c9a75, imm_cffa1, imm_cffa9);
    // _cff95 = _cfe75 | _cff21, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _cff95 = sys.create_bitwise_or(_cfe75, _cff21);
    // _cffa5 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:75
    let imm_cffbd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_cffc5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _cffa5 = sys.create_slice(_c9a75, imm_cffbd, imm_cffc5);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cff75 = sys.create_conditional_block(_cff21);
    sys.set_current_block(_cff75);
    // log('i.ori.0010011    | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}', _cff79, _cff89, _cffa5), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "i.ori.0010011    | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_cff79, _cff89, _cffa5]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _cffe1 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cffed = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_cfff5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _cffe1 = sys.create_slice(_c9a75, imm_cffed, imm_cfff5);
    // _cfff1 = _cffe1 == (19:b7), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:84
    let imm_cfffd = sys.get_const_int(assassyn::ir::DataType::bits_ty(7), 0x13 as u64); // (19:b7)
    let _cfff1 = sys.create_eq(_cffe1, imm_cfffd);
    // _d0c05 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_d0c15 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_d0c1d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _d0c05 = sys.create_slice(_c9a75, imm_d0c15, imm_d0c1d);
    // _d0c19 = _d0c05 == (4:b3), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:85
    let imm_cff61 = sys.get_const_int(assassyn::ir::DataType::bits_ty(3), 0x4 as u64); // (4:b3)
    let _d0c19 = sys.create_eq(_d0c05, imm_cff61);
    // _d0c39 = _cfff1 & _d0c19, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let _d0c39 = sys.create_bitwise_and(_cfff1, _d0c19);
    // _d0c41 = _d0c39 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_d0c29 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _d0c41 = sys.create_bitwise_and(_d0c39, imm_d0c29);
    // _d0c45 = _d0c41 & (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:97
    let imm_d0c35 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _d0c45 = sys.create_bitwise_and(_d0c41, imm_d0c35);
    // _cffd1 = _cfeb1 | _d0c45, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:29
    let _cffd1 = sys.create_bitwise_or(_cfeb1, _d0c45);
    // _cffd9 = _cfeb9 | _d0c45, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:31
    let _cffd9 = sys.create_bitwise_or(_cfeb9, _d0c45);
    // _d0c61 = _d0c45 ? (4:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let imm_d0c55 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x4 as u64); // (4:b16)
    let imm_cfe41 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _d0c61 = sys.create_select(_d0c45, imm_d0c55, imm_cfe41);
    // _d0c6d = _cff49 | _d0c61, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:34
    let _d0c6d = sys.create_bitwise_or(_cff49, _d0c61);
    // _d0c75 = _d0c45 ? (1:b16) : (0:b16), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let imm_d0c51 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x1 as u64); // (1:b16)
    let imm_d0c69 = sys.get_const_int(assassyn::ir::DataType::bits_ty(16), 0x0 as u64); // (0:b16)
    let _d0c75 = sys.create_select(_d0c45, imm_d0c51, imm_d0c69);
    // _d0c81 = _cff5d | _d0c75, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:35
    let _d0c81 = sys.create_bitwise_or(_cff5d, _d0c75);
    // _d0c89 = _d0c45 ? (0:b1) : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let imm_d0c5d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_d0c7d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _d0c89 = sys.create_select(_d0c45, imm_d0c5d, imm_d0c7d);
    // _d0c95 = _cff71 | _d0c89, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:36
    let _d0c95 = sys.create_bitwise_or(_cff71, _d0c89);
    // _d0c9d = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:51
    let imm_d0ca9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_d0cb1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _d0c9d = sys.create_slice(_c9a75, imm_d0ca9, imm_d0cb1);
    // _d0c99 = _cfd25 | _d0c45, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:52
    let _d0c99 = sys.create_bitwise_or(_cfd25, _d0c45);
    // _d0cad = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:58
    let imm_d0cc5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_d0ccd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _d0cad = sys.create_slice(_c9a75, imm_d0cc5, imm_d0ccd);
    // _d0cb9 = _cff95 | _d0c45, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:59
    let _d0cb9 = sys.create_bitwise_or(_cff95, _d0c45);
    // _d0cc9 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:75
    let imm_d0ce1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_d0ce9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _d0cc9 = sys.create_slice(_c9a75, imm_d0ce1, imm_d0ce9);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _cf591 = sys.create_conditional_block(_d0c45);
    sys.set_current_block(_cf591);
    // log('i.xori.0010011   | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}', _d0c9d, _d0cad, _d0cc9), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:76
    let fmt = sys.get_str_literal(
        "i.xori.0010011   | rd: x{:02}      | rs1: x{:02}      |               |imm: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_d0c9d, _d0cad, _d0cc9]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _d0c25 = ~_cffd9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:78
    let _d0c25 = sys.create_flip(_cffd9);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _d0ce5 = sys.create_conditional_block(_d0c25);
    sys.set_current_block(_d0ce5);
    // _d0d0d = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:80
    let imm_d0d19 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_d0d21 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _d0d0d = sys.create_slice(_c9a75, imm_d0d19, imm_d0d21);
    // _d0d1d = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:80
    let imm_d0d35 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_d0d3d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _d0d1d = sys.create_slice(_c9a75, imm_d0d35, imm_d0d3d);
    // _d0d45 = _c9a75[(25:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:80
    let imm_d0d4d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x19 as u64); // (25:u5)
    let imm_d0d55 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _d0d45 = sys.create_slice(_c9a75, imm_d0d4d, imm_d0d55);
    // log('Unsupported instruction: opcode = 0x{:x} funct3: 0x{:x} funct7: 0x{:x}', _d0d0d, _d0d1d, _d0d45), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:80
    let fmt = sys.get_str_literal(
        "Unsupported instruction: opcode = 0x{:x} funct3: 0x{:x} funct7: 0x{:x}".into(),
    );
    sys.create_log(fmt, vec![_d0d0d, _d0d1d, _d0d45]);
    // side effect intrinsic.assert({'(0:b1)'}), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:81
    let imm_d0d09 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    sys.create_assert(imm_d0d09);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _d0d05 = _cb6ad | _cb7d1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:85
    let _d0d05 = sys.create_bitwise_or(_cb6ad, _cb7d1);
    // _d0d6d = { _cc211 _d0d05 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:85
    let _d0d6d = sys.create_concat(_cc211, _d0d05);
    // _d0d75 = { _cb7d1 _cb7d1 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:87
    let _d0d75 = sys.create_concat(_cb7d1, _cb7d1);
    // _d0d79 = _ccb79 | _ca061, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:90
    let _d0d79 = sys.create_bitwise_or(_ccb79, _ca061);
    // _d0d7d = _d0d79 | _cc111, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:90
    let _d0d7d = sys.create_bitwise_or(_d0d79, _cc111);
    // _d0d81 = _d0d7d | _caf49, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:90
    let _d0d81 = sys.create_bitwise_or(_d0d7d, _caf49);
    // _d0d85 = _d0d81 | _cf539, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:90
    let _d0d85 = sys.create_bitwise_or(_d0d81, _cf539);
    // _d0d89 = _ccb79 | _ca0e9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:91
    let _d0d89 = sys.create_bitwise_or(_ccb79, _ca0e9);
    // _d0d8d = _caf49 | _ca0e9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:92
    let _d0d8d = sys.create_bitwise_or(_caf49, _ca0e9);
    // _d0d95 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:97
    let imm_d0da1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_d0da9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _d0d95 = sys.create_slice(_c9a75, imm_d0da1, imm_d0da9);
    // _d0db1 = _d0c99 ? _d0d95 : (0:b5), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:97
    let imm_d0d91 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x0 as u64); // (0:b5)
    let _d0db1 = sys.create_select(_d0c99, _d0d95, imm_d0d91);
    // _d0dbd = _c9a75[(15:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:98
    let imm_d0dc9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xf as u64); // (15:u4)
    let imm_d0dd1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _d0dbd = sys.create_slice(_c9a75, imm_d0dc9, imm_d0dd1);
    // _d0dd9 = _d0cb9 ? _d0dbd : (0:b5), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:98
    let imm_d0d01 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x0 as u64); // (0:b5)
    let _d0dd9 = sys.create_select(_d0cb9, _d0dbd, imm_d0d01);
    // _d0de5 = _c9a75[(20:u5):(24:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:99
    let imm_d0df1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_d0df9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x18 as u64); // (24:u5)
    let _d0de5 = sys.create_slice(_c9a75, imm_d0df1, imm_d0df9);
    // _d0e01 = _cfd6d ? _d0de5 : (0:b5), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:99
    let imm_d0de1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x0 as u64); // (0:b5)
    let _d0e01 = sys.create_select(_cfd6d, _d0de5, imm_d0de1);
    // _d0e09 = _cffd1 | _cd27d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:102
    let _d0e09 = sys.create_bitwise_or(_cffd1, _cd27d);
    // _d0e0d = _d0e09 | _ccb79, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:102
    let _d0e0d = sys.create_bitwise_or(_d0e09, _ccb79);
    // _d0e11 = _d0e0d | _ca061, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:102
    let _d0e11 = sys.create_bitwise_or(_d0e0d, _ca061);
    // _d0e15 = _d0e11 | _cc1bd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:102
    let _d0e15 = sys.create_bitwise_or(_d0e11, _cc1bd);
    // _d0e45 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:75
    let imm_d0e51 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_d0e59 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _d0e45 = sys.create_slice(_c9a75, imm_d0e51, imm_d0e59);
    // _d0e3d = _d0e45[(11:u4):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:77
    let imm_d0e65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let imm_d0e6d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _d0e3d = sys.create_slice(_d0e45, imm_d0e65, imm_d0e6d);
    // _d0e81 = _d0e3d ? (1048575:b20) : (0:b20), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:78
    let imm_cf7ed = sys.get_const_int(assassyn::ir::DataType::bits_ty(20), 0xfffff as u64); // (1048575:b20)
    let imm_d0e7d = sys.get_const_int(assassyn::ir::DataType::bits_ty(20), 0x0 as u64); // (0:b20)
    let _d0e81 = sys.create_select(_d0e3d, imm_cf7ed, imm_d0e7d);
    // _d0e8d = { _d0e81 _d0e45 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:79
    let _d0e8d = sys.create_concat(_d0e81, _d0e45);
    // _d0e91 = _cffd1 ? _d0e8d : (0:b32), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:114
    let imm_ca039 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let _d0e91 = sys.create_select(_cffd1, _d0e8d, imm_ca039);
    // _d0e9d = _c9a75[(31:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_d0ea9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let imm_d0eb1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _d0e9d = sys.create_slice(_c9a75, imm_d0ea9, imm_d0eb1);
    // _d0eb9 = _c9a75[(7:u3):(7:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_d0ec1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_d0ec9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let _d0eb9 = sys.create_slice(_c9a75, imm_d0ec1, imm_d0ec9);
    // _d0ed1 = _c9a75[(25:u5):(30:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_d0ed9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x19 as u64); // (25:u5)
    let imm_d0ee1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1e as u64); // (30:u5)
    let _d0ed1 = sys.create_slice(_c9a75, imm_d0ed9, imm_d0ee1);
    // _d0ee9 = _c9a75[(8:u4):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let imm_d0ef1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0x8 as u64); // (8:u4)
    let imm_d0ef9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _d0ee9 = sys.create_slice(_c9a75, imm_d0ef1, imm_d0ef9);
    // _d0e99 = { _d0e9d _d0eb9 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _d0e99 = sys.create_concat(_d0e9d, _d0eb9);
    // _d0f01 = { _d0e99 _d0ed1 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _d0f01 = sys.create_concat(_d0e99, _d0ed1);
    // _d0ef5 = { _d0f01 _d0ee9 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:189
    let _d0ef5 = sys.create_concat(_d0f01, _d0ee9);
    // _d0f09 = { _d0ef5 (0:b1) }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:190
    let imm_d0c91 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _d0f09 = sys.create_concat(_d0ef5, imm_d0c91);
    // _d0f11 = _d0f09[(12:u4):(12:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:192
    let imm_d0f19 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_d0f21 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let _d0f11 = sys.create_slice(_d0f09, imm_d0f19, imm_d0f21);
    // _d0f35 = _d0f11 ? (524287:b19) : (0:b19), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:193
    let imm_d0f1d = sys.get_const_int(assassyn::ir::DataType::bits_ty(19), 0x7ffff as u64); // (524287:b19)
    let imm_d0f31 = sys.get_const_int(assassyn::ir::DataType::bits_ty(19), 0x0 as u64); // (0:b19)
    let _d0f35 = sys.create_select(_d0f11, imm_d0f1d, imm_d0f31);
    // _d0f41 = { _d0f35 _d0f09 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:194
    let _d0f41 = sys.create_concat(_d0f35, _d0f09);
    // _d0f45 = _ccb79 ? _d0f41 : _d0e91, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:114
    let _d0f45 = sys.create_select(_ccb79, _d0f41, _d0e91);
    // _d0f51 = _c9a75[(12:u4):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:136
    let imm_d0f5d = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_d0f65 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _d0f51 = sys.create_slice(_c9a75, imm_d0f5d, imm_d0f65);
    // _d0f4d = { (0:b12) _d0f51 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:138
    let imm_d0db5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(12), 0x0 as u64); // (0:b12)
    let _d0f4d = sys.create_concat(imm_d0db5, _d0f51);
    // _d0f61 = _cd27d ? _d0f4d : _d0f45, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:114
    let _d0f61 = sys.create_select(_cd27d, _d0f4d, _d0f45);
    // _d0f81 = _c9a75[(31:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:161
    let imm_d0f8d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let imm_d0f95 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _d0f81 = sys.create_slice(_c9a75, imm_d0f8d, imm_d0f95);
    // _d0f9d = _c9a75[(12:u4):(19:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:161
    let imm_d0fa5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_d0fad = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x13 as u64); // (19:u5)
    let _d0f9d = sys.create_slice(_c9a75, imm_d0fa5, imm_d0fad);
    // _d0fb5 = _c9a75[(20:u5):(20:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:161
    let imm_d0fbd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_d0fc5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let _d0fb5 = sys.create_slice(_c9a75, imm_d0fbd, imm_d0fc5);
    // _d0fcd = _c9a75[(21:u5):(30:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:161
    let imm_d0fd5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x15 as u64); // (21:u5)
    let imm_d0fdd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1e as u64); // (30:u5)
    let _d0fcd = sys.create_slice(_c9a75, imm_d0fd5, imm_d0fdd);
    // _d0fd9 = { _d0f81 _d0f9d }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:161
    let _d0fd9 = sys.create_concat(_d0f81, _d0f9d);
    // _d0fed = { _d0fd9 _d0fb5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:161
    let _d0fed = sys.create_concat(_d0fd9, _d0fb5);
    // _d0ff1 = { _d0fed _d0fcd }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:161
    let _d0ff1 = sys.create_concat(_d0fed, _d0fcd);
    // _d0ff5 = { _d0ff1 (0:b1) }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:161
    let imm_d0fe5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _d0ff5 = sys.create_concat(_d0ff1, imm_d0fe5);
    // _d0ff9 = _d0ff5[(20:u5):(20:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:163
    let imm_d0ffd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_d140d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let _d0ff9 = sys.create_slice(_d0ff5, imm_d0ffd, imm_d140d);
    // _d1421 = _d0ff9 ? (2047:b11) : (0:b11), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:164
    let imm_d0f79 = sys.get_const_int(assassyn::ir::DataType::bits_ty(11), 0x7ff as u64); // (2047:b11)
    let imm_d141d = sys.get_const_int(assassyn::ir::DataType::bits_ty(11), 0x0 as u64); // (0:b11)
    let _d1421 = sys.create_select(_d0ff9, imm_d0f79, imm_d141d);
    // _d142d = { _d1421 _d0ff5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:165
    let _d142d = sys.create_concat(_d1421, _d0ff5);
    // _d0db9 = _ca061 ? _d142d : _d0f61, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:114
    let _d0db9 = sys.create_select(_ca061, _d142d, _d0f61);
    // _d1439 = _c9a75[(25:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:116
    let imm_d1445 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x19 as u64); // (25:u5)
    let imm_d144d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _d1439 = sys.create_slice(_c9a75, imm_d1445, imm_d144d);
    // _d1455 = _c9a75[(7:u3):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:116
    let imm_d145d = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x7 as u64); // (7:u3)
    let imm_d1465 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _d1455 = sys.create_slice(_c9a75, imm_d145d, imm_d1465);
    // _d0f49 = { _d1439 _d1455 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:116
    let _d0f49 = sys.create_concat(_d1439, _d1455);
    // _d146d = _d0f49[(11:u4):(11:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:118
    let imm_d1471 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let imm_d1479 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xb as u64); // (11:u4)
    let _d146d = sys.create_slice(_d0f49, imm_d1471, imm_d1479);
    // _d148d = _d146d ? (1048575:b20) : (0:b20), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:119
    let imm_d1475 = sys.get_const_int(assassyn::ir::DataType::bits_ty(20), 0xfffff as u64); // (1048575:b20)
    let imm_d1489 = sys.get_const_int(assassyn::ir::DataType::bits_ty(20), 0x0 as u64); // (0:b20)
    let _d148d = sys.create_select(_d146d, imm_d1475, imm_d1489);
    // _d1499 = { _d148d _d0f49 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:120
    let _d1499 = sys.create_concat(_d148d, _d0f49);
    // _d149d = _cc1bd ? _d1499 : _d0db9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:114
    let _d149d = sys.create_select(_cc1bd, _d1499, _d0db9);
    // _d14a9 = _c9a75[(12:u4):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:136
    let imm_d14b5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_d14bd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _d14a9 = sys.create_slice(_c9a75, imm_d14b5, imm_d14bd);
    // _d14c5 = { _d14a9 (0:b12) }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:115
    let imm_d14a5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(12), 0x0 as u64); // (0:b12)
    let _d14c5 = sys.create_concat(_d14a9, imm_d14a5);
    // _d14c9 = _ca269 ? _d14c5 : _d149d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:115
    let _d14c9 = sys.create_select(_ca269, _d14c5, _d149d);
    // _d14d5 = _c9a75[(12:u4):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/instructions.py:136
    let imm_d14e1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_d14e9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _d14d5 = sys.create_slice(_c9a75, imm_d14e1, imm_d14e9);
    // _d14f1 = { _d14d5 (0:b12) }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:116
    let imm_cfe2d = sys.get_const_int(assassyn::ir::DataType::bits_ty(12), 0x0 as u64); // (0:b12)
    let _d14f1 = sys.create_concat(_d14d5, imm_cfe2d);
    // _d14f5 = _cdc9d ? _d14f1 : _d14c9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:116
    let _d14f5 = sys.create_select(_cdc9d, _d14f1, _d14c9);
    // _d14fd = _cd3d1 | _cf539, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:118
    let _d14fd = sys.create_bitwise_or(_cd3d1, _cf539);
    // _d0e1d = _cddb9 | _cded9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:120
    let _d0e1d = sys.create_bitwise_or(_cddb9, _cded9);
    // _d0e2d = _d14fd | _d0e1d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:124
    let _d0e2d = sys.create_bitwise_or(_d14fd, _d0e1d);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _d0e25 = sys.create_conditional_block(_d0e2d);
    sys.set_current_block(_d0e25);
    // _d0e05 = _c9a75[(0:u1):(6:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:126
    let imm_d14a1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let imm_d14f9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x6 as u64); // (6:u3)
    let _d0e05 = sys.create_slice(_c9a75, imm_d14a1, imm_d14f9);
    // _d1509 = _c9a75[(12:u4):(14:u4)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:126
    let imm_d1511 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xc as u64); // (12:u4)
    let imm_d1519 = sys.get_const_int(assassyn::ir::DataType::uint_ty(4), 0xe as u64); // (14:u4)
    let _d1509 = sys.create_slice(_c9a75, imm_d1511, imm_d1519);
    // _d1521 = _c9a75[(20:u5):(31:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:126
    let imm_d1529 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x14 as u64); // (20:u5)
    let imm_d1531 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1f as u64); // (31:u5)
    let _d1521 = sys.create_slice(_c9a75, imm_d1529, imm_d1531);
    // log('CSR instruction: opcode = 0x{:x} funct3: 0x{:x} csr_addr: 0x{:x}', _d0e05, _d1509, _d1521), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:126
    let fmt = sys
        .get_str_literal("CSR instruction: opcode = 0x{:x} funct3: 0x{:x} csr_addr: 0x{:x}".into());
    sys.create_log(fmt, vec![_d0e05, _d1509, _d1521]);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _d1585 = { _d0dd9 _d0cb9 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d1585 = sys.create_concat(_d0dd9, _d0cb9);
    // _d1599 = { _d1585 _d0e01 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d1599 = sys.create_concat(_d1585, _d0e01);
    // _d159d = { _d1599 _cfd6d }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d159d = sys.create_concat(_d1599, _cfd6d);
    // _d15a5 = { _d159d _d0db1 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d15a5 = sys.create_concat(_d159d, _d0db1);
    // _d15b1 = { _d15a5 _d0c99 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d15b1 = sys.create_concat(_d15a5, _d0c99);
    // _d15b5 = { _d15b1 _d14fd }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d15b5 = sys.create_concat(_d15b1, _d14fd);
    // _d15b9 = { _d15b5 _d0e1d }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d15b9 = sys.create_concat(_d15b5, _d0e1d);
    // _d15bd = { _d15b9 _cd3d1 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d15bd = sys.create_concat(_d15b9, _cd3d1);
    // _d15c1 = { _d15bd _cded9 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d15c1 = sys.create_concat(_d15bd, _cded9);
    // _d15c5 = { _d15c1 _cf539 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d15c5 = sys.create_concat(_d15c1, _cf539);
    // _d15c9 = { _d15c5 _cdc9d }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d15c9 = sys.create_concat(_d15c5, _cdc9d);
    // _d15cd = { _d15c9 _d14f5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d15cd = sys.create_concat(_d15c9, _d14f5);
    // _d15d1 = { _d15cd _d0e15 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d15d1 = sys.create_concat(_d15cd, _d0e15);
    // _d15d5 = { _d15d1 _d0d6d }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d15d5 = sys.create_concat(_d15d1, _d0d6d);
    // _d15d9 = { _d15d5 _d0c6d }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d15d9 = sys.create_concat(_d15d5, _d0c6d);
    // _d15dd = { _d15d9 _d0c81 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d15dd = sys.create_concat(_d15d9, _d0c81);
    // _d15e1 = { _d15dd _d0c95 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d15e1 = sys.create_concat(_d15dd, _d0c95);
    // _d15e5 = { _d15e1 _d0d85 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d15e5 = sys.create_concat(_d15e1, _d0d85);
    // _d15e9 = { _d15e5 _d0d89 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d15e9 = sys.create_concat(_d15e5, _d0d89);
    // _d15ed = { _d15e9 _d0d8d }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d15ed = sys.create_concat(_d15e9, _d0d8d);
    // _d15f1 = { _d15ed _d0d75 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/decoder.py:128
    let _d15f1 = sys.create_concat(_d15ed, _d0d75);
    // _cf741 = _d15f1[(4:u3):(4:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:257
    let imm_c8ee5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x4 as u64); // (4:u3)
    let imm_40471 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x4 as u64); // (4:u3)
    let _cf741 = sys.create_slice(_d15f1, imm_c8ee5, imm_40471);
    // array_c24b1[(0:u1)] = _cf741, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:257
    let imm_c8281 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    sys.create_array_write(array_c24b1, imm_c8281, _cf741);
    // _c9add = array_ba53d[(0:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:262
    let imm_c9ae9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x0 as u64); // (0:u5)
    let _c9add = sys.create_array_read(array_ba53d, imm_c9ae9);
    // _c9ac5 = array_ba53d[(1:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:262
    let imm_c9ad9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x1 as u64); // (1:u5)
    let _c9ac5 = sys.create_array_read(array_ba53d, imm_c9ad9);
    // _c98b5 = array_ba53d[(2:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:262
    let imm_c9a7d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x2 as u64); // (2:u5)
    let _c98b5 = sys.create_array_read(array_ba53d, imm_c9a7d);
    // _c99f9 = array_ba53d[(3:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:262
    let imm_c9be9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x3 as u64); // (3:u5)
    let _c99f9 = sys.create_array_read(array_ba53d, imm_c9be9);
    // _c9bf1 = array_ba53d[(4:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:262
    let imm_c9be5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x4 as u64); // (4:u5)
    let _c9bf1 = sys.create_array_read(array_ba53d, imm_c9be5);
    // _c9bc5 = array_ba53d[(5:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:262
    let imm_c9bcd = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x5 as u64); // (5:u5)
    let _c9bc5 = sys.create_array_read(array_ba53d, imm_c9bcd);
    // _c9bad = array_ba53d[(6:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:262
    let imm_c9bb5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x6 as u64); // (6:u5)
    let _c9bad = sys.create_array_read(array_ba53d, imm_c9bb5);
    // _c9b65 = array_ba53d[(7:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:262
    let imm_c9b8d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x7 as u64); // (7:u5)
    let _c9b65 = sys.create_array_read(array_ba53d, imm_c9b8d);
    // _c9b79 = array_ba53d[(8:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:262
    let imm_c9b71 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x8 as u64); // (8:u5)
    let _c9b79 = sys.create_array_read(array_ba53d, imm_c9b71);
    // _c9b7d = array_ba53d[(9:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:262
    let imm_c9b75 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x9 as u64); // (9:u5)
    let _c9b7d = sys.create_array_read(array_ba53d, imm_c9b75);
    // _c9b69 = array_ba53d[(10:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:262
    let imm_c9a41 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xa as u64); // (10:u5)
    let _c9b69 = sys.create_array_read(array_ba53d, imm_c9a41);
    // _c9b45 = array_ba53d[(11:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:262
    let imm_c9b55 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xb as u64); // (11:u5)
    let _c9b45 = sys.create_array_read(array_ba53d, imm_c9b55);
    // _c9b61 = array_ba53d[(12:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:262
    let imm_c9b59 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xc as u64); // (12:u5)
    let _c9b61 = sys.create_array_read(array_ba53d, imm_c9b59);
    // _c9b3d = array_ba53d[(13:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:262
    let imm_c9981 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xd as u64); // (13:u5)
    let _c9b3d = sys.create_array_read(array_ba53d, imm_c9981);
    // _c9afd = array_ba53d[(14:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:262
    let imm_c98ed = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xe as u64); // (14:u5)
    let _c9afd = sys.create_array_read(array_ba53d, imm_c98ed);
    // _c9b35 = array_ba53d[(15:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:262
    let imm_c9b2d = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0xf as u64); // (15:u5)
    let _c9b35 = sys.create_array_read(array_ba53d, imm_c9b2d);
    // side effect intrinsic.wait_until({'(0:b1)'}), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:266
    let imm_c9aed = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    sys.create_wait_until(imm_c9aed);
    // _c9b29 = _d15f1[(80:u7):(84:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:32
    let imm_c9b0d = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x50 as u64); // (80:u7)
    let imm_c9b05 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x54 as u64); // (84:u7)
    let _c9b29 = sys.create_slice(_d15f1, imm_c9b0d, imm_c9b05);
    // _c9af5 = _d15f1[(92:u7):(96:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:33
    let imm_c9b19 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x5c as u64); // (92:u7)
    let imm_c9ab9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x60 as u64); // (96:u7)
    let _c9af5 = sys.create_slice(_d15f1, imm_c9b19, imm_c9ab9);
    // _c9ab1 = _d15f1[(86:u7):(90:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:34
    let imm_c9aa9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x56 as u64); // (86:u7)
    let imm_c26f5 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x5a as u64); // (90:u7)
    let _c9ab1 = sys.create_slice(_d15f1, imm_c9aa9, imm_c26f5);
    // _ca179 = _d15f1[(91:u7):(91:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:37
    let imm_d0f05 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x5b as u64); // (91:u7)
    let imm_d0e19 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x5b as u64); // (91:u7)
    let _ca179 = sys.create_slice(_d15f1, imm_d0f05, imm_d0e19);
    // _c9af9 = array_ba589[_c9af5], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:37
    let _c9af9 = sys.create_array_read(array_ba589, _c9af5);
    // _d0e39 = _c9af9 != (16:b5), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:37
    let imm_c9a9d = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64); // (16:b5)
    let _d0e39 = sys.create_neq(_c9af9, imm_c9a9d);
    // _d0c09 = _ca179 & _d0e39, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:37
    let _d0c09 = sys.create_bitwise_and(_ca179, _d0e39);
    // _d15ad = _d0c09 ? (0:b1) : (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:37
    let imm_d1485 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_d15f5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _d15ad = sys.create_select(_d0c09, imm_d1485, imm_d15f5);
    // _d1601 = ~_d15ad, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:38
    let _d1601 = sys.create_flip(_d15ad);
    // _d1605 = array_ba589[_c9af5], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:38
    let _d1605 = sys.create_array_read(array_ba589, _c9af5);
    // _d1609 = _d1601 ? _d1605 : (16:b5), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:38
    let imm_c9a9d = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64); // (16:b5)
    let _d1609 = sys.create_select(_d1601, _d1605, imm_c9a9d);
    // _d160d = _d15f1[(92:u7):(96:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:39
    let imm_d161d = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x5c as u64); // (92:u7)
    let imm_d1625 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x60 as u64); // (96:u7)
    let _d160d = sys.create_slice(_d15f1, imm_d161d, imm_d1625);
    // _c9b31 = array_b9a49[_d160d], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:39
    let _c9b31 = sys.create_array_read(array_b9a49, _d160d);
    // _d1631 = _d15ad ? _c9b31 : (0:b32), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:39
    let imm_d162d = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let _d1631 = sys.create_select(_d15ad, _c9b31, imm_d162d);
    // _d1639 = _d15f1[(85:u7):(85:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:41
    let imm_d1649 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x55 as u64); // (85:u7)
    let imm_d1651 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x55 as u64); // (85:u7)
    let _d1639 = sys.create_slice(_d15f1, imm_d1649, imm_d1651);
    // _d15a9 = array_ba589[_c9ab1], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:41
    let _d15a9 = sys.create_array_read(array_ba589, _c9ab1);
    // _d164d = _d15a9 != (16:b5), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:41
    let imm_c9a9d = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64); // (16:b5)
    let _d164d = sys.create_neq(_d15a9, imm_c9a9d);
    // _d165d = _d1639 & _d164d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:41
    let _d165d = sys.create_bitwise_and(_d1639, _d164d);
    // _d166d = _d165d ? (0:b1) : (1:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:41
    let imm_d1659 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_d1669 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _d166d = sys.create_select(_d165d, imm_d1659, imm_d1669);
    // _d1679 = ~_d166d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:42
    let _d1679 = sys.create_flip(_d166d);
    // _d167d = array_ba589[_c9ab1], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:42
    let _d167d = sys.create_array_read(array_ba589, _c9ab1);
    // _d1681 = _d1679 ? _d167d : (16:b5), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:42
    let imm_c9a9d = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x10 as u64); // (16:b5)
    let _d1681 = sys.create_select(_d1679, _d167d, imm_c9a9d);
    // _d1685 = _d15f1[(86:u7):(90:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:43
    let imm_d1695 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x56 as u64); // (86:u7)
    let imm_d169d = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x5a as u64); // (90:u7)
    let _d1685 = sys.create_slice(_d15f1, imm_d1695, imm_d169d);
    // _d1621 = array_b9a49[_d1685], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:43
    let _d1621 = sys.create_array_read(array_b9a49, _d1685);
    // _d16a9 = _d166d ? _d1621 : (0:b32), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:43
    let imm_d16a5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    let _d16a9 = sys.create_select(_d166d, _d1621, imm_d16a5);
    // _d16b1 = _d15f1[(79:u7):(79:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:45
    let imm_d16c1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4f as u64); // (79:u7)
    let imm_d16c9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4f as u64); // (79:u7)
    let _d16b1 = sys.create_slice(_d15f1, imm_d16c1, imm_d16c9);
    // _d1665 = bitcast (0:b5) to b5, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:46
    let imm_c9ae5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x0 as u64); // (0:b5)
    let _d1665 = sys.create_bitcast(imm_c9ae5, assassyn::ir::DataType::bits_ty(5));
    // array_ba589[_c9b29] = _d1665, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:46
    sys.create_array_write(array_ba589, _c9b29, _d1665);
    // _d16dd = array_ba589[_c9b29], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:50
    let _d16dd = sys.create_array_read(array_ba589, _c9b29);
    // _d16e1 = _d15f1[(79:u7):(79:u7)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:50
    let imm_d16f1 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4f as u64); // (79:u7)
    let imm_d16f9 = sys.get_const_int(assassyn::ir::DataType::uint_ty(7), 0x4f as u64); // (79:u7)
    let _d16e1 = sys.create_slice(_d15f1, imm_d16f1, imm_d16f9);
    // log('reg map table rd:{}  {} , rd_valid:{}, Index:{}', _c9b29, _d16dd, _d16e1, (0:b5)), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:50
    let fmt = sys.get_str_literal("reg map table rd:{}  {} , rd_valid:{}, Index:{}".into());
    let imm_c9ae5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x0 as u64); // (0:b5)
    sys.create_log(fmt, vec![_c9b29, _d16dd, _d16e1, imm_c9ae5]);
    // log('Bundle values: valid={},  rd={}, rs1={}, rs2={}, rs1_ready={}, rs2_ready={}, rs1_value={}, rs2_value={},             rs1_dep={}, rs2_dep={}, status={} ', (1:b1), _c9b29, _c9af5, _c9ab1, _d15ad, _d166d, _d1631, _d16a9, _d1609, _d1681, (0:b2)), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:52
    let fmt = sys.get_str_literal("Bundle values: valid={},  rd={}, rs1={}, rs2={}, rs1_ready={}, rs2_ready={}, rs1_value={}, rs2_value={},             rs1_dep={}, rs2_dep={}, status={} ".into());
    let imm_c9b25 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let imm_d16c5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    sys.create_log(
        fmt,
        vec![
            imm_c9b25, _c9b29, _c9af5, _c9ab1, _d15ad, _d166d, _d1631, _d16a9, _d1609, _d1681,
            imm_d16c5,
        ],
    );
    // _d1705 = { (1:b1) _c9b29 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:57
    let imm_c9b25 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x1 as u64); // (1:b1)
    let _d1705 = sys.create_concat(imm_c9b25, _c9b29);
    // _d1711 = { _d1705 _c9af5 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:57
    let _d1711 = sys.create_concat(_d1705, _c9af5);
    // _d1715 = { _d1711 _c9ab1 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:57
    let _d1715 = sys.create_concat(_d1711, _c9ab1);
    // _d171d = { _d1715 _d15ad }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:57
    let _d171d = sys.create_concat(_d1715, _d15ad);
    // _d1729 = { _d171d _d166d }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:57
    let _d1729 = sys.create_concat(_d171d, _d166d);
    // _d172d = { _d1729 _d1631 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:57
    let _d172d = sys.create_concat(_d1729, _d1631);
    // _d1731 = { _d172d _d16a9 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:57
    let _d1731 = sys.create_concat(_d172d, _d16a9);
    // _d1735 = { _d1731 _d1609 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:57
    let _d1735 = sys.create_concat(_d1731, _d1609);
    // _d1739 = { _d1735 _d1681 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:57
    let _d1739 = sys.create_concat(_d1735, _d1681);
    // _d173d = { _d1739 (0:b2) }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:57
    let imm_d16c5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(2), 0x0 as u64); // (0:b2)
    let _d173d = sys.create_concat(_d1739, imm_d16c5);
    // _d1741 = { _d173d _d15f1 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:57
    let _d1741 = sys.create_concat(_d173d, _d15f1);
    // _d1745 = { _d1741 _c9a79 }, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/scoreboard.py:57
    let _d1745 = sys.create_concat(_d1741, _c9a79);
    // array_ba53d[(0:b5)] = _d1745, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:267
    let imm_c9ae5 = sys.get_const_int(assassyn::ir::DataType::bits_ty(5), 0x0 as u64); // (0:b5)
    sys.create_array_write(array_ba53d, imm_c9ae5, _d1745);
    // _d1749 = _bedf5.bind([]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:275
    // Already handled by `EmitBinds` {}
    // async_call _d1749, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:275
    sys.create_async_call(_d1749);
    // _d174d = _d15f1[(4:u3):(4:u3)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:278
    let imm_d1759 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x4 as u64); // (4:u3)
    let imm_d1761 = sys.get_const_int(assassyn::ir::DataType::uint_ty(3), 0x4 as u64); // (4:u3)
    let _d174d = sys.create_slice(_d15f1, imm_d1759, imm_d1761);
    // Fill in the body of _c9a29
    sys.set_current_module(_c9a29);
    // module root block
    // _d24d1 = _b9a3d.bind([]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:342
    // Already handled by `EmitBinds` {}
    // async_call _d24d1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:342
    sys.create_async_call(_d24d1);
    // Set FIFO depths
    // Emit downstream modules
    // Module F1
    sys.set_current_module(_25d8d);
    // Fill in the body of _25d8d
    sys.set_current_module(_25d8d);
    // module root block
    // _d176d = _d174d.valid(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:312
    let _d176d = sys.create_value_valid(_d174d);
    // _d1781 = _d176d ? _d174d : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:312
    let imm_d175d = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _d1781 = sys.create_select(_d176d, _d174d, imm_d175d);
    // _d178d = array_c24b1[(0:u1)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:312
    let imm_d1791 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let _d178d = sys.create_array_read(array_c24b1, imm_d1791);
    // _d1795 = _d1781 | _d178d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:312
    let _d1795 = sys.create_bitwise_or(_d1781, _d178d);
    // _d17a1 = ~_d1795, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:313
    let _d17a1 = sys.create_flip(_d1795);
    // _d179d = _c25f5.valid(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:313
    let _d179d = sys.create_value_valid(_c25f5);
    // _d17a9 = _d17a1 | _d179d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:313
    let _d17a9 = sys.create_bitwise_or(_d17a1, _d179d);
    // _d17b5 = _c25f5.valid(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:314
    let _d17b5 = sys.create_value_valid(_c25f5);
    // _d17b1 = _d17b5 ? _c25f5 : _b9a61, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:314
    let _d17b1 = sys.create_select(_d17b5, _c25f5, _b9a61);
    // _d17c1 = array_d0e35[(0:u1)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:318
    let imm_d17cd = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    let _d17c1 = sys.create_array_read(array_d0e35, imm_d17cd);
    // _d17d9 = _bcd69.valid(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:318
    let _d17d9 = sys.create_value_valid(_bcd69);
    // _d17dd = _d17d9 ? _bcd69 : (0:b1), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:318
    let imm_d17d1 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _d17dd = sys.create_select(_d17d9, _bcd69, imm_d17d1);
    // _d17f9 = _d17dd ? (1:i8) : (0:i8), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:318
    let imm_d17e9 = sys.get_const_int(assassyn::ir::DataType::int_ty(8), 0x1 as u64); // (1:i8)
    let imm_d17f5 = sys.get_const_int(assassyn::ir::DataType::int_ty(8), 0x0 as u64); // (0:i8)
    let _d17f9 = sys.create_select(_d17dd, imm_d17e9, imm_d17f5);
    // _d2409 = _d17c1 - _d17f9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:318
    let _d2409 = sys.create_sub(_d17c1, _d17f9);
    // _d2411 = _d2409 < (5:i8), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:319
    let imm_d2405 = sys.get_const_int(assassyn::ir::DataType::int_ty(8), 0x5 as u64); // (5:i8)
    let _d2411 = sys.create_ilt(_d2409, imm_d2405);
    // _d2419 = _d17a9 & _d2411, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:319
    let _d2419 = sys.create_bitwise_and(_d17a9, _d2411);
    // _d2421 = _d17b1[(2:u2):(17:u5)], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:321
    let imm_d242d = sys.get_const_int(assassyn::ir::DataType::uint_ty(2), 0x2 as u64); // (2:u2)
    let imm_d2435 = sys.get_const_int(assassyn::ir::DataType::uint_ty(5), 0x11 as u64); // (17:u5)
    let _d2421 = sys.create_slice(_d17b1, imm_d242d, imm_d2435);
    // _d2431 = bitcast _d2421 to i16, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:321
    let _d2431 = sys.create_bitcast(_d2421, assassyn::ir::DataType::int_ty(16));
    // _d1771 = _c25f5.valid(), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:323
    let _d1771 = sys.create_value_valid(_c25f5);
    // log('on_br: {}         | ex_by: {}     | fetch: {}      | addr: 0x{:05x} | ongoing: {}', _d1795, _d1771, _d2419, _d17b1, _d2409), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:322
    let fmt = sys.get_str_literal(
        "on_br: {}         | ex_by: {}     | fetch: {}      | addr: 0x{:05x} | ongoing: {}".into(),
    );
    sys.create_log(fmt, vec![_d1795, _d1771, _d2419, _d17b1, _d2409]);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _d2449 = sys.create_conditional_block(_d2419);
    sys.set_current_block(_d2449);
    // _ba7b9.fetch_addr.push(_d17b1) // handle = _d24c1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:326
    let push_d2495 = sys.bind_arg(_d2495, "fetch_addr".into(), _d17b1);
    // async_call _d2495, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:326
    sys.create_async_call(_d2495);
    // _d246d = bitcast _d17b1 to i32, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:327
    let _d246d = sys.create_bitcast(_d17b1, assassyn::ir::DataType::int_ty(32));
    // _d24d5 = _d246d + (4:i32), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:327
    let imm_d24cd = sys.get_const_int(assassyn::ir::DataType::int_ty(32), 0x4 as u64); // (4:i32)
    let _d24d5 = sys.create_add(_d246d, imm_d24cd);
    // _d24dd = bitcast _d24d5 to b32, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:327
    let _d24dd = sys.create_bitcast(_d24d5, assassyn::ir::DataType::bits_ty(32));
    // array_b9a51[(0:u1)] = _d24dd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:327
    let imm_d24ed = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    sys.create_array_write(array_b9a51, imm_d24ed, _d24dd);
    // _d24f9 = _d2409 + (1:i8), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:328
    let imm_d24f1 = sys.get_const_int(assassyn::ir::DataType::int_ty(8), 0x1 as u64); // (1:i8)
    let _d24f9 = sys.create_add(_d2409, imm_d24f1);
    // array_d0e35[(0:u1)] = _d24f9, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:328
    let imm_d2505 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    sys.create_array_write(array_d0e35, imm_d2505, _d24f9);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // _d2499 = ~_d2419, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:330
    let _d2499 = sys.create_flip(_d2419);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _d2509 = sys.create_conditional_block(_d2499);
    sys.set_current_block(_d2509);
    // array_b9a51[(0:u1)] = _d17b1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:331
    let imm_d251d = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    sys.create_array_write(array_b9a51, imm_d251d, _d17b1);
    // array_d0e35[(0:u1)] = _d2409, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:332
    let imm_d2529 = sys.get_const_int(assassyn::ir::DataType::uint_ty(1), 0x0 as u64); // (0:u1)
    sys.create_array_write(array_d0e35, imm_d2529, _d2409);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // Module dcache
    sys.set_current_module(_c0ba1);
    _c0ba1
        .as_mut::<assassyn::ir::Module>(&mut sys)
        .unwrap()
        .add_attr(assassyn::ir::module::Attribute::MemoryParams(
            assassyn::ir::module::attrs::MemoryParams::new(
                32,                         // width
                65536,                      // depth
                1..=1,                      // lat
                Some("0to100.data".into()), // init-file
                assassyn::ir::module::attrs::MemoryPins::new(
                    array_c2731, // array
                    _bee61,      // re
                    _bee81,      // we
                    _c2701,      // addr
                    _becdd,      // wdata
                ),
            ),
        ));
    // Fill in the body of _c0ba1
    sys.set_current_module(_c0ba1);
    // module root block
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c273d = sys.create_conditional_block(_bee81);
    sys.set_current_block(_c273d);
    // array_c2731[_c2701] = _becdd, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:191
    sys.create_array_write(array_c2731, _c2701, _becdd);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _c2791 = sys.create_conditional_block(_bee61);
    sys.set_current_block(_c2791);
    // _c279d = array_c2731[_c2701], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:191
    let _c279d = sys.create_array_read(array_c2731, _c2701);
    // _ba5e5.rdata.push(_c279d) // handle = _c27a1, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:191
    let push_c27a5 = sys.bind_arg(_c27a5, "rdata".into(), _c279d);
    // _c27a5 = _ba5e5.bind([_c27a1 /* _ba5e5.rdata=_c279d */, _c26cd /* _ba5e5.rd=_be701 */]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:191
    // Already handled by `EmitBinds` {}
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // Module icache
    sys.set_current_module(_d16e5);
    let imm_d2415 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let imm_d2441 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    _d16e5
        .as_mut::<assassyn::ir::Module>(&mut sys)
        .unwrap()
        .add_attr(assassyn::ir::module::Attribute::MemoryParams(
            assassyn::ir::module::attrs::MemoryParams::new(
                32,                        // width
                65536,                     // depth
                1..=1,                     // lat
                Some("0to100.exe".into()), // init-file
                assassyn::ir::module::attrs::MemoryPins::new(
                    array_d17c9, // array
                    _d2419,      // re
                    imm_d2415,   // we
                    _d2431,      // addr
                    imm_d2441,   // wdata
                ),
            ),
        ));
    // Fill in the body of _d16e5
    sys.set_current_module(_d16e5);
    // module root block
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let imm_d2415 = sys.get_const_int(assassyn::ir::DataType::bits_ty(1), 0x0 as u64); // (0:b1)
    let _d2455 = sys.create_conditional_block(imm_d2415);
    sys.set_current_block(_d2455);
    // array_d17c9[_d2431] = (0:b32), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:321
    let imm_d2441 = sys.get_const_int(assassyn::ir::DataType::bits_ty(32), 0x0 as u64); // (0:b32)
    sys.create_array_write(array_d17c9, _d2431, imm_d2441);
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);
    // restore current block
    block_stack.push(sys.get_current_block().unwrap().upcast());
    // conditional block
    let _d247d = sys.create_conditional_block(_d2419);
    sys.set_current_block(_d247d);
    // _d248d = array_d17c9[_d2431], /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:321
    let _d248d = sys.create_array_read(array_d17c9, _d2431);
    // _ba7b9.rdata.push(_d248d) // handle = _d249d, /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:321
    let push_d2495 = sys.bind_arg(_d2495, "rdata".into(), _d248d);
    // _d2495 = _ba7b9.bind([_d249d /* _ba7b9.rdata=_d248d */, _d24c1 /* _ba7b9.fetch_addr=_d17b1 */]), /Users/lychee/DAY1029/assassyn/examples/minor-cpu/src/tomasulo.py:321
    // Already handled by `EmitBinds` {}
    let restore = block_stack.pop().unwrap();
    sys.set_current_block(restore);

    let mut config = assassyn::backend::common::Config {
        idle_threshold: 1000000,
        sim_threshold: 1000000,
        random: false,
        resource_base: PathBuf::from("/Users/lychee/DAY1029/assassyn/examples/minor-cpu/workloads"),
        ..Default::default()
    };

    println!("{}", sys);
    assassyn::xform::basic(
        &mut sys,
        &assassyn::xform::Config {
            rewrite_wait_until: true,
        },
    );
    config.base_dir = (env!("CARGO_MANIFEST_DIR").to_string()).into();
    assassyn::backend::simulator::elaborate(&sys, &config).unwrap();
}
