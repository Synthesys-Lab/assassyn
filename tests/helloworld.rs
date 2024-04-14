use eda4eda::module_builder;
use eir::{builder::SysBuilder, test_utils};

#[test]
fn helloworld() {
  module_builder!(driver[][] {
    log("{}, {}!", "Hello", "world");
  });
  let mut sys = SysBuilder::new("hw");
  driver_builder(&mut sys);

  println!("{}", sys);

  let src_name = test_utils::temp_dir(&"helloworld.rs".to_string());
  let config = eir::sim::Config {
    fname: src_name,
    sim_threshold: 1,
    idle_threshold: 1,
  };

  eir::builder::verify(&sys);
  eir::sim::elaborate(&sys, &config).unwrap();

  let exec_name = test_utils::temp_dir(&"helloworld".to_string());
  test_utils::compile(&config.fname, &exec_name);

  let output = test_utils::run(&exec_name);
  let raw = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .next()
    .unwrap()
    .to_string();
  let reference = "Hello, world!";
  assert_eq!(raw[raw.len() - reference.len()..raw.len()], *reference);
}
