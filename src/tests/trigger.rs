use super::utils;
use crate::builder::system::{PortInfo, SysBuilder};
use crate::node::IsElement;
use crate::{sim, DataType, Module, BaseNode};

#[test]
fn trigger() {
  fn a_plus_b(sys: &mut SysBuilder) -> BaseNode {
    let int32 = DataType::int(32);
    let module = sys.create_module(
      "a_plus_b",
      vec![
        PortInfo::new("a", int32.clone()),
        PortInfo::new("b", int32.clone()),
      ],
    );
    let (a, b) = {
      let module = module.as_ref::<Module>(&sys).unwrap();
      let i0 = module.get_input(0).unwrap().clone();
      let i1 = module.get_input(1).unwrap().clone();
      let a = sys.create_fifo_pop(&i0, None, None);
      let b = sys.create_fifo_pop(&i1, None, None);
      (a, b)
    };
    sys.create_add(None, &a, &b, None);
    module
  }

  fn build_driver(sys: &mut SysBuilder, plus: BaseNode) {
    let driver_module = sys.get_driver();
    sys.set_current_module(driver_module.upcast());
    let int32 = DataType::int(32);
    let a = sys.create_array(&int32, "cnt", 1);
    let zero = sys.get_const_int(&int32, 0);
    let one = sys.get_const_int(&int32, 1);
    let a0 = sys.create_array_read(&a, &zero, None);
    let hundred = sys.get_const_int(&int32, 100);
    let cond = sys.create_ilt(None, &a0, &hundred, None);
    sys.create_trigger(&plus, vec![a0.clone(), a0.clone()], Some(cond));
    let acc = sys.create_add(None, &a0, &one, None);
    sys.create_array_write(&a, &zero, &acc, None);
  }

  let mut sys = SysBuilder::new("main");

  // Create a trivial module.
  let m1 = a_plus_b(&mut sys);

  // Build the driver module.
  build_driver(&mut sys, m1);

  println!("{}", sys);

  let src_name = utils::temp_dir(&"trigger.rs".to_string());

  println!("Writing simulator code to {}", src_name);

  let config = sim::Config {
    fname: src_name,
    sim_threshold: 200,
    idle_threshold: 200,
  };

  sim::elaborate(&sys, &config).unwrap();

  let exec_name = utils::temp_dir(&"trigger".to_string());
  utils::compile(&config.fname, &exec_name);

  let output = utils::run(&exec_name);
  let times_invoked = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .filter(|x| x.contains("Simulating module a_plus_b"))
    .count();
  assert_eq!(times_invoked, 100);
}
