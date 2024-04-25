use eda4eda::module_builder;
use eir::{
  builder::SysBuilder,
  sim::elaborate,
  test_utils::{self, temp_dir},
  xform,
};

#[test]
fn select() {
  module_builder!(driver[][] {
    rng0 = array(int<32>, 1);
    rng1 = array(int<32>, 1);

    v0 = rng0[0];
    v1 = rng1[0];

    v0 = v0.mul(12345);
    v1 = v1.mul(54321);

    rand0 = v0.add(67890);
    rand1 = v1.add(09876);

    rand0 = rand0.slice(0, 31);
    rand1 = rand1.slice(0, 31);

    gt = rand0.igt(rand1);
    mux = default rand1.case(gt, rand0);

    rng0[0] = rand0;
    rng1[0] = rand1;

    log("{} >? {} = {}", rand0, rand1, mux);
  });

  let mut sys = SysBuilder::new("select");
  driver_builder(&mut sys);
  xform::basic(&mut sys);
  eir::builder::verify(&sys);

  let fname = temp_dir(&"select.rs".to_string());
  let config = eir::sim::Config {
    fname,
    sim_threshold: 101,
    idle_threshold: 100,
  };
  elaborate(&sys, &config).unwrap();
  let exec_name = test_utils::temp_dir(&"select".to_string());
  test_utils::compile(&config.fname, &exec_name);
  let output = test_utils::run(&exec_name);
  let output = String::from_utf8(output.stdout).unwrap();
  output.lines().for_each(|line| {
    if line.contains(">?") {
      let toks = line.split_whitespace().collect::<Vec<&str>>();
      let a = toks[toks.len() - 5].parse::<i32>().unwrap();
      let b = toks[toks.len() - 3].parse::<i32>().unwrap();
      let c = toks[toks.len() - 1].parse::<i32>().unwrap();
      assert_eq!(c, if a > b { a } else { b });
    }
  });
}
