use assassyn::module_builder;
use eir::builder::SysBuilder;

module_builder!(
  driver(fetcher)() {
    async_call fetcher ();
  }
);

module_builder!(
  fetcher(inst_buffer, pc)() {
    to_fetch = pc[0].slice(0, 9).bitcast(uint<10>);
    async_call inst_buffer { write: 0.bits<1>, wdata: 0.bits<32>, addr: to_fetch };
  }
);

module_builder!(
  inst_buffer(we, inst, decoder)() {
    async_call decoder { inst: inst };
    // pc = array(int<33>, 1);
    // opcode = inst.slice(6, 0);
    // not_br = opcode.neq(0b1100111);
    // when not_br {
    //   pc[0] = pc[0].add(4);
    // }
  }
);

module_builder!(
  decoder()(inst: bits<32>) {
    pc = array(int<33>, 1);
    pc[0] = pc[0].add(1.int<33>);
    log("instruction: {:x}", inst);
  }.expose(pc)
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

  let (decoder, pc) = decoder_builder(&mut sys);

  let inst_buffer = sys.create_memory(
    "inst_buffer",
    32,
    1024,
    1..=1,
    Some("binary.mem".into()),
    |sys, module, write, rdata| {
      inst_buffer_impl(sys, module, write, rdata, decoder);
    },
  );

  let fetcher = fetcher_builder(&mut sys, inst_buffer, pc);

  driver_builder(&mut sys, fetcher);

  println!("{}", sys);
}
