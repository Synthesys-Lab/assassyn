use std::path::PathBuf;

use assassyn::module_builder;
use eir::{backend::simulator::elaborate, builder::SysBuilder, ir::data::ArrayAttr};

module_builder!(
  driver(fetcher)() {
    async_call fetcher ();
  }
);

module_builder!(
  fetcher(decoder, pc, on_branch)() {
    when on_branch[0].flip() {
      log("fetching inst from: {:x}", pc[0]);
      to_fetch = pc[0].slice(0, 9).bitcast(uint<10>);
      async_call decoder { write: 0.bits<1>, wdata: 0.bits<32>, addr: to_fetch };
      pc[0] = pc[0].bitcast(int<32>).add(1.int<32>).bitcast(bits<32>);
    }
    when on_branch[0] {
      log("on a branch, stall fetching");
    }
  }
);

module_builder!(
  decoder(we, inst, pc, on_branch, register_file, reg_onwrite, exec)() {
    when on_branch[0].flip() {
      opcode = inst.slice(0, 6);
      // funct = inst.slice(12, 14);
      rd    = inst.slice(7, 11);
      rs1   = inst.slice(15, 19);
      rs2   = inst.slice(20, 24);
      u_imm = inst.slice(12, 31);
      i_imm = inst.slice(20, 31);

      is_lui  = opcode.eq(0b0110111.bits<7>);
      is_addi = opcode.eq(0b0010011.bits<7>);
      is_add  = opcode.eq(0b0110011.bits<7>);
      is_li   = opcode.eq(0b0000011.bits<7>);
      is_bne  = opcode.eq(0b1100011.bits<7>);
      is_ret  = opcode.eq(0b1100111.bits<7>);

      supported  = bitwise_or(is_lui, is_addi, is_li, is_add, is_bne, is_ret);
      write_rd   = bitwise_or(is_lui, is_addi, is_li, is_add);
      read_rs1   = bitwise_or(is_lui, is_addi, is_li, is_add, is_bne);
      read_rs2   = is_add;
      read_i_imm = bitwise_or(is_li, is_addi, is_bne);
      read_u_imm = is_lui;

      when write_rd {
        reg_onwrite[rd] = 1.bits<1>;
      }

      value_a = read_rs1.select(register_file[rs1], 0.bits<32>);
      reg_a   = read_rs1.select(rs1, 0.bits<5>);

      value_b = read_rs2.select(register_file[rs2], read_i_imm.select(i_imm, read_u_imm.select(u_imm, 0.bits<32>)));
      reg_b   = read_rs2.select(rs2, 0.bits<5>);

      rd_reg = write_rd.select(rd, 0.bits<5>);

      async_call exec(opcode, value_a, value_b, reg_a, reg_b, rd_reg);

      when is_lui  { log("lui:  rd: x{}, imm: {:b}", rd, u_imm); }
      when is_addi { log("addi: rd: x{}, rs1: x{}, imm: {:b}", rd, rs1, i_imm); }
      when is_add  { log("add:  rd: x{}, rs1: x{}, rs2: x{}", rd, rs1, rs2); }
      when is_li   { log("li:   rd: x{}, rs1: x{}, imm: {:b}", rd, rs1, i_imm); }
      when is_bne  { log("bne:           rs1: x{}, imm: {:b}", rs1, i_imm); }
      when is_ret  { log("ret"); }

      when supported.flip() {
        log("unsupported opcode: {:b}, raw_inst: {:x}", opcode, inst);
      }
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
    _a_reg: bits<5>,
    _b_reg: bits<5>,
    rd_reg: bits<5>
  ) {
    wait_until {
      a_valid = reg_onwrite[_a_reg.peek()].flip();
      b_valid = reg_onwrite[_b_reg.peek()].flip();
      c_valid = reg_onwrite[rd_reg.peek()].flip();
      valid = a_valid.bitwise_and(b_valid).bitwise_and(c_valid);
      valid
    } {

      is_addi = opcode.eq(0b0010011.bits<7>);
      is_add  = opcode.eq(0b0110011.bits<7>);
      is_lui  = opcode.eq(0b0110111.bits<7>);
      is_li   = opcode.eq(0b0000011.bits<7>);
      is_bne  = opcode.eq(0b1100011.bits<7>);
      // is_ret  = opcode.eq(0b1100111.bits<7>);

      invoke_adder = bitwise_or(is_addi, is_add, is_li, is_lui);

      is_branch = is_bne;
      add_res = a.bitcast(int<32>).add(b.bitcast(int<32>)).bitcast(bits<32>);
      result = invoke_adder.select(add_res, 0.bits<32>);

      when is_branch {
        on_branch[0] = 0.bits<1>;
      }

      when is_bne {
        new_pc = pc[0].bitcast(int<32>).add(1.int<32>).bitcast(bits<32>);
        pc[0] = a.neq(0.bits<32>).select(b, new_pc);
      }

      async_call memory { write: 0.bits<1>, addr: result, data: a };
      wb = bind writeback { rd: rd_reg };
    }
  }.expose(wb)
);

module_builder!(
  memory_access(we, data, writeback)() {
  }
);

module_builder!(
  writeback()() {
  }
);

fn main() {
  let mut sys = SysBuilder::new("minor_cpu");

  let bits1 = eir::ir::DataType::Bits(1);
  let bits32 = eir::ir::DataType::Bits(32);
  let pc = sys.create_array(bits32.clone(), "pc", 1, None, vec![]);

  let on_branch = sys.create_array(bits1.clone(), "on_branch", 1, None, vec![]);

  let reg_file = sys.create_array(bits32.clone(), "rf.data", 32, None, vec![]);

  let reg_onwrite = {
    let fully_partitioned = vec![ArrayAttr::FullyPartitioned];
    sys.create_array(bits1.clone(), "rf.onwrite", 32, None, fully_partitioned)
  };

  let writeback = writeback_builder(&mut sys);

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

  elaborate(&mut sys, &config).unwrap();
}
