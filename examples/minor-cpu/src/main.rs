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
      log("on a branch, stall fetching");
    }
  }
);

module_builder!(
  decoder(we, inst, pc, on_branch, register_file, reg_onwrite, exec)() {
    when on_branch[0].flip() {
      // Slice the fields.
      opcode = inst.slice(0, 6);
      // funct = inst.slice(12, 14);
      rd    = inst.slice(7, 11);
      rs1   = inst.slice(15, 19);
      rs2   = inst.slice(20, 24);
      u_imm = inst.slice(12, 31);
      i_imm = inst.slice(20, 31);

      sign = inst.slice(31, 31);
      b_imm = concat(sign, inst.slice(7, 7), inst.slice(25, 30), inst.slice(8, 11), 0.bits<1>);
      b_imm = sign.select(0x7ffff.bits<19>, 0.bits<19>).concat(b_imm);

      is_lui  = opcode.eq(0b0110111.bits<7>);
      is_addi = opcode.eq(0b0010011.bits<7>);
      is_add  = opcode.eq(0b0110011.bits<7>);
      is_li   = opcode.eq(0b0000011.bits<7>);
      is_bne  = opcode.eq(0b1100011.bits<7>);
      is_ret  = opcode.eq(0b1100111.bits<7>);

      supported  = bitwise_or(is_lui, is_addi, is_li, is_add, is_bne, is_ret);
      write_rd   = bitwise_or(is_lui, is_addi, is_li, is_add);
      read_rs1   = bitwise_or(is_lui, is_addi, is_li, is_add, is_bne);
      read_rs2   = bitwise_or(is_add, is_bne);
      read_i_imm = bitwise_or(is_li, is_addi);
      read_u_imm = is_lui;
      read_b_imm = is_bne;

      when is_bne {
        on_branch[0] = 1.bits<1>;
      }

      value_a = read_rs1.select(register_file[rs1], 0.bits<32>);
      reg_a   = read_rs1.select(rs1, 0.bits<5>);


      value_b = read_rs2.select(register_file[rs2], 0.bits<32>);
      reg_b   = read_rs2.select(rs2, 0.bits<5>);

      no_imm = bitwise_or(read_i_imm, read_u_imm, read_b_imm).flip();
      imm_cond = concat(read_i_imm, read_u_imm, read_b_imm, no_imm);
      imm_value = imm_cond.select_1hot(i_imm.zext(bits<32>), u_imm.zext(bits<32>), b_imm, 0.bits<32>);

      rd_reg  = write_rd.select(rd, 0.bits<5>);

      async_call exec(opcode, value_a, value_b, imm_value, reg_a, reg_b, rd_reg);

      when is_lui  { log("lui:  rd: x{}, imm: {:x}", rd, u_imm); }
      when is_addi { log("addi: rd: x{}, rs1: x{}, imm: {}", rd, rs1, i_imm); }
      when is_add  { log("add:  rd: x{}, rs1: x{}, rs2: {}", rd, rs1, rs2); }
      when is_li   { log("li:   rd: x{}, rs1: x{}, imm: {:x}", rd, rs1, i_imm); }
      when is_bne  { log("bne:  rs1:x{}, rs2: x{}, imm: {}, set on_branch reg", rs1, rs2, b_imm.bitcast(int<32>)); }
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
    memory,
    writeback
  )(
    opcode: bits<7>,
    a: bits<32>,
    b: bits<32>,
    c: bits<32>,
    // FIXME: value used in wait_until is NOT considered as used. Warnings are casted.
    _a_reg: bits<5>,
    _b_reg: bits<5>,
    rd_reg: bits<5>
  ) {
    wait_until {
      // handle read after write
      a_valid = reg_onwrite[_a_reg.peek()].flip();
      b_valid = reg_onwrite[_b_reg.peek()].flip();
      c_valid = reg_onwrite[rd_reg.peek()].flip();
      log("x{}.a_valid: {}, x{}.b_valid: {}, x{}.rd_valid: {}",
          _a_reg.peek(), a_valid,
          _b_reg.peek(), b_valid,
          rd_reg.peek(), c_valid);
      valid = bitwise_and(a_valid, b_valid, c_valid);
      valid
    } {

      when rd_reg.neq(0.bits<5>) {
        reg_onwrite[rd_reg] = 1.bits<1>;
        log("set x{} onwrite", rd_reg);
      }

      is_addi = opcode.eq(0b0010011.bits<7>);
      is_add  = opcode.eq(0b0110011.bits<7>);
      is_lui  = opcode.eq(0b0110111.bits<7>);
      is_li   = opcode.eq(0b0000011.bits<7>);
      is_bne  = opcode.eq(0b1100011.bits<7>);
      // is_ret  = opcode.eq(0b1100111.bits<7>);

      // instruction attributes
      uses_imm = bitwise_or(is_addi, is_lui, is_li, is_bne);
      is_branch = is_bne;

      rhs = uses_imm.select(c, b);

      invoke_adder = bitwise_or(is_addi, is_add, is_li, is_lui);

      result = a.bitcast(int<32>).add(rhs.bitcast(int<32>)).bitcast(bits<32>);
      log("adder: a: {:x}, b: {:x}, res: {:x}", a, b, result);
      result = invoke_adder.select(result, 0.bits<32>);

      when is_branch {
        on_branch[0] = 0.bits<1>;
        log("reset on_branch reg");
      }

      when is_bne {
        new_pc  = pc[0].bitcast(int<32>).sub(4.int<32>).add(c.bitcast(int<32>)).bitcast(bits<32>);
        br_dest = a.neq(b).select(new_pc, pc[0]);
        log("if {} != 0: branch to {}; br_dest", a, b);
        pc[0] = br_dest;
      }

      async_call memory { write: 0.bits<1>, addr: result.slice(0, 9).bitcast(uint<10>), wdata: a };
      wb = bind writeback { rd: rd_reg, result: result };
    }
  }.expose(wb)
);

module_builder!(
  memory_access(we, data, writeback)() {
    async_call writeback();
  }
);

module_builder!(
  writeback(reg_file, reg_onwrite)(rd: bits<5>, result: bits<32>) {
    when rd.neq(0.bits<5>) {
      log("writeback: x{} = {:x}", rd, result);
      reg_file[rd] = result;
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

  let memory_access = sys.declare_memory("memory_access", 32, 1024, 1..=1, None);

  let (exec, wb) = execution_builder(
    &mut sys,
    reg_onwrite,
    on_branch,
    pc,
    memory_access,
    writeback,
  );

  sys.impl_memory(memory_access, |sys, module, write, rdata| {
    memory_access_impl(sys, module, write, rdata, wb);
  });

  let decoder = sys.create_memory(
    "decoder",
    32,
    1024,
    1..=1,
    Some("binaries/0to100.mem".into()),
    |sys, module, write, rdata| {
      decoder_impl(
        sys,
        module,
        write,
        rdata,
        pc,
        on_branch,
        reg_file,
        reg_onwrite,
        exec,
      );
    },
  );

  let fetcher = fetcher_builder(&mut sys, decoder, pc, on_branch);

  driver_builder(&mut sys, fetcher);

  println!("{}", sys);

  let config = eir::backend::common::Config {
    resource_base: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
    sim_threshold: 20,
    ..Default::default()
  };

  let o1 = eir::xform::Config{
    rewrite_wait_until: true,
  };
  xform::basic(&mut sys, &o1);
  println!("{}", sys);

  elaborate(&mut sys, &config).unwrap();
}
