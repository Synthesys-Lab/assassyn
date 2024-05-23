use std::path::PathBuf;

use assassyn::module_builder;
use eir::{backend::simulator::elaborate, builder::SysBuilder};

module_builder!(
  driver(fetcher)() {
    async_call fetcher ();
  }
);

module_builder!(
  fetcher(decoder, pc)() {
    to_fetch = pc[0].slice(0, 9).bitcast(uint<10>);
    async_call decoder { write: 0.bits<1>, wdata: 0.bits<32>, addr: to_fetch };
  }
);

module_builder!(
  decoder(we, inst, pc)() {
    new_pc = pc[0].slice(0, 31).bitcast(int<32>).add(1.int<32>);
    pc[0] = 0.int<1>.concat(new_pc);
    log("instruction: {:x}", inst);
    opcode = inst.slice(0, 6);
    log("opcode: {:07b}", opcode);
    // not_br = opcode.neq(0b1100111);
    // when not_br {
    //   pc[0] = pc[0].add(4);
    // }
  }
);

// module_builder!(
//   execution()(opcode: int<6>, a: int<32>, b: int<32>) {
//   }
// );
//
// module_builder!(
//   memory_access()(addr: int<32>, data: int<32>, we: bits<1>) {
//   }
// );
//
// module_builder!(
//   writeback()(addr: int<32>, data: int<32>) {
//   }
// );

fn main() {

  let mut sys = SysBuilder::new("minor_cpu");
  
  let pc = sys.create_array(eir::ir::DataType::Bits(33), "pc", 1, None, vec![]);

  let decoder = sys.create_memory(
    "decoder",
    32,
    1024,
    1..=1,
    Some("binaries/0to100.mem".into()),
    |sys, module, write, rdata| {
      decoder_impl(sys, module, write, rdata, pc);
    },
  );

  let fetcher = fetcher_builder(&mut sys, decoder, pc);

  driver_builder(&mut sys, fetcher);

  println!("{}", sys);

  let config = eir::backend::common::Config {
    resource_base: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
    ..Default::default()
  };

  elaborate(&mut sys, &config).unwrap();
}
