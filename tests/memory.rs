use eda4eda::module_builder;
use eir::{
  builder::SysBuilder,
  test_utils::{self, parse_cycle},
};

module_builder!(
  mem_sink[rdata:int<32>][] {
    rdata  = rdata.pop();
    log("rdata: {}", rdata);
  }
);

fn sram() -> SysBuilder {
  module_builder!(
    driver[][sink, mem] {
      cnt = array(int<32>, 1);
      v = cnt[0];
      v = v.slice(0, 9);
      async mem { raddr: v, r: sink };
      plused = v.add(1);
      cnt[0] = plused;
    }
  );

  let mut sys = SysBuilder::new("main");
  let sink = mem_sink_builder(&mut sys);
  // TODO: data is a Bits, not Int
  let memory = sys.create_memory(
    "sram",
    eir::ir::DataType::Int(32),
    1024,
    /* latency: [min, max] */ (1, 1),
    None,
  );
  let _driver = driver_builder(&mut sys, sink, memory);
  sys
}

fn testit(fname: &str, mut sys: SysBuilder) {
  let config = eir::sim::Config {
    fname: test_utils::temp_dir(&format!("{}.rs", fname)),
    sim_threshold: 200,
    idle_threshold: 200,
  };
  println!("{}", sys);

  eir::builder::verify(&sys);
  eir::xform::basic(&mut sys);
  eir::builder::verify(&sys);

  println!("{}", sys);

  let verilog_name = test_utils::temp_dir(&format!("{}.sv", fname));
  let verilog_config = eir::verilog::Config {
    fname: verilog_name,
    sim_threshold: config.sim_threshold,
  };
  eir::verilog::elaborate(&sys, &verilog_config).unwrap();
  eir::sim::elaborate(&sys, &config).unwrap();
  let exec_name = test_utils::temp_dir(&fname.to_string());
  test_utils::compile(&config.fname, &exec_name);
  // TODO(@were): Make a time timeout here.
  let raw = test_utils::run(&exec_name);
  String::from_utf8(raw.stdout)
    .unwrap()
    .lines()
    .for_each(|l| {
      if l.contains("agent move on") {
        let (cycle, _) = parse_cycle(l);
        assert!(
          cycle % 4 == 1 || cycle % 4 == 2,
          "agent move on {} % 4 = {}",
          cycle,
          cycle % 4
        );
      }
      if l.contains("squarer") {
        let (cycle, _) = parse_cycle(l);
        assert!(cycle % 4 == 2 || cycle % 4 == 3, "{}", l);
      }
    });
}

#[test]
fn memory() {
  let sram = sram();
  testit("sram", sram);
}
