use eda4eda::module_builder;
use eir::builder::SysBuilder;

#[test]
fn helloworld() {
  module_builder!(driver()() {
    log("{}, {}!", "Hello", "world");
  });
  let mut sys = SysBuilder::new("helloworld");
  driver_builder(&mut sys);

  println!("{}", sys);

  let config = eir::backend::common::Config {
    temp_dir: true,
    sim_threshold: 2,
    idle_threshold: 2,
  };
  eir::backend::verilog::elaborate(&sys, &config).unwrap();

  eir::builder::verify(&sys);
  eir::test_utils::run_simulator(
    &sys,
    &config,
    Some((
      |x| {
        if x.contains("driver") {
          assert!(x.contains("Hello, world!"));
        }
        false
      },
      None,
    )),
  );
}
