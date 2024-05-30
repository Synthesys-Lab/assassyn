use std::path::PathBuf;

use assassyn::module_builder;
use eir::{backend::simulator::elaborate, builder::SysBuilder, ir::data::ArrayAttr, xform};

module_builder!(
  driver(fetcher)() {
    async_call fetcher ();
  }
);

module_builder!(
  fetcher(decoder, pc, on_branch)() {
    when on_branch[0].flip() {
      log("fetching inst from: 0x{:x}", pc[0]);
      to_fetch = pc[0].slice(2, 11).bitcast(uint<10>);
      async_call decoder { write: 0.bits<1>, wdata: 0.bits<32>, addr: to_fetch };
      pc[0] = pc[0].bitcast(int<32>).add(4.int<32>).bitcast(bits<32>);
    }
    when on_branch[0] {
      log("on a branch, stall fetching, pc freeze @ 0x{:x}", pc[0]);
    }
  }
);

module_builder!(
  decoder(we, inst, pc, on_branch, exec)() {
    when on_branch[0].flip() {
      // Slice the fields.
      opcode = inst.slice(0, 6);
      log("decoding: {:b}", opcode);
      // funct = inst.slice(12, 14);
      rd    = inst.slice(7, 11);
      rs1   = inst.slice(15, 19);
      rs2   = inst.slice(20, 24);
      i_imm = inst.slice(20, 31);

      u_imm = inst.slice(12, 31).concat(0.bits<12>);

      sign = inst.slice(31, 31);
      b_imm = concat(sign, inst.slice(7, 7), inst.slice(25, 30), inst.slice(8, 11), 0.bits<1>);
      b_imm = sign.select(0x7ffff.bits<19>, 0.bits<19>).concat(b_imm);

      is_lui  = opcode.eq(0b0110111.bits<7>);
      is_addi = opcode.eq(0b0010011.bits<7>);
      is_add  = opcode.eq(0b0110011.bits<7>);
      is_lw   = opcode.eq(0b0000011.bits<7>);
      is_bne  = opcode.eq(0b1100011.bits<7>);
      is_ret  = opcode.eq(0b1100111.bits<7>);

      supported  = bitwise_or(is_lui, is_addi, is_add, is_lw, is_bne, is_ret);
      write_rd   = bitwise_or(is_lui, is_addi, is_add, is_lw);
      read_rs1   = bitwise_or(is_lui, is_addi, is_add, is_bne, is_lw);
      read_rs2   = bitwise_or(is_add, is_bne);
      read_i_imm = bitwise_or(is_addi, is_lw);
      read_u_imm = is_lui;
      read_b_imm = is_bne;

      when is_bne {
        log("set on-branch!");
        on_branch[0] = 1.bits<1>;
      }

      reg_a   = read_rs1.select(rs1, 0.bits<5>);
      reg_b   = read_rs2.select(rs2, 0.bits<5>);

      no_imm = bitwise_or(read_i_imm, read_u_imm, read_b_imm).flip();
      imm_cond = concat(read_i_imm, read_u_imm, read_b_imm, no_imm);
      imm_value = imm_cond.select_1hot(i_imm.zext(bits<32>), u_imm, b_imm, 0.bits<32>);

      rd_reg  = write_rd.select(rd, 0.bits<5>);

      async_call exec(opcode, imm_value, reg_a, reg_b, rd_reg);

      when is_lui  { log("lui:  rd: x{}, imm: 0x{:x}", rd, u_imm); }
      when is_lw   { log("lw:   rd: x{}, rs1: x{}, imm: {}", rd, rs1, i_imm); }
      when is_addi { log("addi: rd: x{}, rs1: x{}, imm: {}", rd, rs1, i_imm); }
      when is_add  { log("add:  rd: x{}, rs1: x{}, rs2: x{}", rd, rs1, rs2); }
      when is_bne  { log("bne:  rs1:x{}, rs2: x{}, imm: {}", rs1, rs2, b_imm.bitcast(int<32>)); }
      when is_ret  { log("ret"); }

      when supported.flip() {
        log("unsupported opcode: {:b}, raw_inst: {:x}", opcode, inst);
      }
    }
    when on_branch[0] {
      log("on a branch, fetched instruction may wrong, stall decoding");
    }
  }
);

module_builder!(
  execution(
    reg_onwrite,
    on_branch,
    pc,
    exec_bypass_reg,
    exec_bypass_data,
    mem_bypass_reg,
    mem_bypass_data,
    rf,
    memory,
    writeback
  )(
    opcode: bits<7>,
    imm_value: bits<32>,
    // FIXME: value used in wait_until is NOT considered as used. Warnings are casted.
    a_reg: bits<5>,
    b_reg: bits<5>,
    rd_reg: bits<5>
  ) {
    wait_until {
      // handle read after write
      a_valid = bitwise_or(reg_onwrite[a_reg.peek()].flip(), exec_bypass_reg[0].eq(a_reg.peek()), mem_bypass_reg[0].eq(a_reg.peek()));
      b_valid = bitwise_or(reg_onwrite[b_reg.peek()].flip(), exec_bypass_reg[0].eq(b_reg.peek()), mem_bypass_reg[0].eq(b_reg.peek()));
      rd_valid = reg_onwrite[rd_reg.peek()].flip();
      valid = bitwise_and(a_valid, b_valid, rd_valid);
      when valid.flip() {
        log("operand not ready, stall execution, x{}: {}, x{}: {}, x{}: {}",
            a_reg.peek(), a_valid, b_reg.peek(), b_valid, rd_reg.peek(), rd_valid);
      }
      valid
    } {

      log("executing: {:07b}", opcode);

      when rd_reg.neq(0.bits<5>) {
        reg_onwrite[rd_reg] = 1.bits<1>;
        log("set x{} onwrite", rd_reg);
      }

      is_lui  = opcode.eq(0b0110111.bits<7>);
      is_addi = opcode.eq(0b0010011.bits<7>);
      is_add  = opcode.eq(0b0110011.bits<7>);
      is_lw   = opcode.eq(0b0000011.bits<7>);
      is_bne  = opcode.eq(0b1100011.bits<7>);
      // is_ret  = opcode.eq(0b1100111.bits<7>);

      // instruction attributes
      uses_imm = bitwise_or(is_addi, is_bne);
      is_branch = is_bne;

      a = exec_bypass_reg[0].eq(a_reg).select(
                exec_bypass_data[0],
                mem_bypass_reg[0].eq(a_reg).select(mem_bypass_data[0], rf[a_reg]));
      b = exec_bypass_reg[0].eq(b_reg).select(
                exec_bypass_data[0],
                mem_bypass_reg[0].eq(b_reg).select(mem_bypass_data[0], rf[b_reg]));

      rhs = uses_imm.select(imm_value, b);

      invoke_adder = bitwise_or(is_addi, is_add, is_lw);

      result = a.bitcast(int<32>).add(rhs.bitcast(int<32>)).bitcast(bits<32>);
      result = concat(invoke_adder, is_lui, is_branch).select_1hot(result, imm_value, 0.bits<32>);
      log("{:07b}: a: {:x}, b: {:x}, res: {:x}", opcode, a, rhs, result);

      produced_by_exec   = bitwise_or(is_lui, is_addi, is_add);

      exec_bypass_reg[0] = produced_by_exec.select(rd_reg, 0.bits<5>);
      exec_bypass_data[0] = produced_by_exec.select(result, 0.bits<32>);

      when is_branch {
        on_branch[0] = 0.bits<1>;
        log("reset on-branch");
      }

      when is_bne {
        new_pc  = pc[0].bitcast(int<32>).sub(8.int<32>).add(imm_value.bitcast(int<32>)).bitcast(bits<32>);
        br_dest = a.neq(b).select(new_pc, pc[0]);
        log("if {} != {}: branch to {}; actual: {}", a, b, new_pc, br_dest);
        pc[0] = br_dest;
      }

      is_memory = is_lw;
      is_memory_read = is_lw;

      request_addr = is_memory.select(result.slice(2, 19).bitcast(uint<17>), 0.uint<17>);

      mem_bypass_reg[0] = is_memory_read.select(rd_reg, 0.bits<5>);

      when is_memory {
        log("addr: {:x}, lineno: {:x}", result, request_addr);
      }

      async_call memory { write: 0.bits<1>, addr: request_addr, wdata: a };
      wb = bind writeback { opcode: opcode, rd: rd_reg, result: result };
    }
  }.expose(wb)
);

module_builder!(
  memory_access(we, data, writeback, mem_bypass_reg, mem_bypass_data)() {
    log("mem-data: 0x{:x}", data);
    async_call writeback { mdata: data };
    when mem_bypass_reg[0].neq(0.bits<5>) {
      log("bypassing memory data: x{} = {}", mem_bypass_reg[0], data);
    }
    mem_bypass_data[0] = mem_bypass_reg[0].neq(0.bits<5>).select(data, 0.bits<32>);
  }
);

module_builder!(
  writeback(reg_file, reg_onwrite)(opcode: bits<7>, rd: bits<5>, result: bits<32>, mdata: bits<32>) {
    is_lui  = opcode.eq(0b0110111.bits<7>);
    is_addi = opcode.eq(0b0010011.bits<7>);
    is_add  = opcode.eq(0b0110011.bits<7>);
    is_lw   = opcode.eq(0b0000011.bits<7>);
    is_bne  = opcode.eq(0b1100011.bits<7>);
    // is_ret  = opcode.eq(0b1100111.bits<7>);

    is_result = bitwise_or(is_addi, is_add, is_lui, is_bne);
    is_memory = is_lw;
    cond = is_memory.concat(is_result);
    data = cond.select_1hot(mdata, result);

    when rd.neq(0.bits<5>) {
      log("opcode: {:07b}, writeback: x{} = {:x}", opcode, rd, data);
      reg_file[rd] = data;
      reg_onwrite[rd] = 0.bits<1>;
    }
  }
);

fn main() {
  let mut sys = SysBuilder::new("minor_cpu");

  let bits1 = eir::ir::DataType::Bits(1);
  let bits32 = eir::ir::DataType::Bits(32);

  // Declare data structures
  let pc = sys.create_array(bits32.clone(), "pc", 1, None, vec![]);
  let on_branch = sys.create_array(bits1.clone(), "on_branch", 1, None, vec![]);
  let reg_file = sys.create_array(bits32.clone(), "rf.data", 32, None, vec![]);
  let reg_onwrite = {
    let fully_partitioned = vec![ArrayAttr::FullyPartitioned];
    sys.create_array(bits1.clone(), "rf.onwrite", 32, None, fully_partitioned)
  };

  // Top function
  let writeback = writeback_builder(&mut sys, reg_file, reg_onwrite);

  let memory_access = sys.declare_memory(
    "memory_access",
    32,
    65536 * 2,
    1..=1,
    Some("resources/0to100.data".into()),
  );

  let bits5 = eir::ir::DataType::Bits(5);
  let (exec_bypass_reg, exec_bypass_data) = {
    (sys.create_array(bits5.clone(), "bypass_reg", 1, None, vec![]),
    sys.create_array(bits32.clone(), "bypass_data", 1, None, vec![]),)
  };

  let (mem_bypass_reg, mem_bypass_data) = {
    (sys.create_array(bits5, "bypass_reg", 1, None, vec![]),
    sys.create_array(bits32, "bypass_data", 1, None, vec![]),)
  };

  let (exec, wb) = execution_builder(
    &mut sys,
    reg_onwrite,
    on_branch,
    pc,
    exec_bypass_reg,
    exec_bypass_data,
    mem_bypass_reg,
    mem_bypass_data,
    reg_file,
    memory_access,
    writeback,
  );

  sys.impl_memory(memory_access, |sys, module, write, rdata| {
    memory_access_impl(sys, module, write, rdata, wb, mem_bypass_reg, mem_bypass_data);
  });

  let decoder = sys.create_memory(
    "decoder",
    32,
    1024,
    1..=1,
    Some("resources/0to100.exe".into()),
    |sys, module, write, rdata| {
      decoder_impl(sys, module, write, rdata, pc, on_branch, exec);
    },
  );

  let fetcher = fetcher_builder(&mut sys, decoder, pc, on_branch);

  driver_builder(&mut sys, fetcher);

  println!("{}", sys);

  let config = eir::backend::common::Config {
    resource_base: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
    sim_threshold: 1000,
    ..Default::default()
  };

  let o1 = eir::xform::Config {
    rewrite_wait_until: true,
  };
  xform::basic(&mut sys, &o1);
  println!("{}", sys);

  elaborate(&mut sys, &config).unwrap();
}
