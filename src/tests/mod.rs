use std::env::temp_dir;

use crate::builder::system::{PortInfo, SysBuilder};
use crate::reference::IsElement;
use crate::{sim, DataType, Module, Reference};

#[test]
fn helloworld() {
  println!("Hello, world!");
}

#[test]
fn trigger() {
  let dir = temp_dir();

  let fname = dir.join("trigger.rs");

  fn a_plus_b(sys: &mut SysBuilder) -> Reference {
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
      let i0 = module.get_input(0).unwrap();
      let i1 = module.get_input(1).unwrap();
      (i0.clone(), i1.clone())
    };
    sys.create_add(None, a, b, None);
    module
  }

  fn build_driver(sys: &mut SysBuilder, plus: Reference) {
    let driver_module = sys.get_driver();
    sys.set_current_module(driver_module.upcast());
    let a = sys.create_array(DataType::int(32), "cnt", 1);
    let int32 = DataType::int(32);
    let zero = sys.get_const_int(int32.clone(), 0);
    let one = sys.get_const_int(int32, 1);
    let a0 = sys.create_array_read(a.clone(), zero.clone(), None);
    let acc = sys.create_add(None, a0, one, None);
    sys.create_array_write(a.clone(), zero.clone(), acc, None);
    let plused = sys.create_array_read(a, zero, None);
    let hundred = sys.get_const_int(DataType::int(32), 100);
    let cond = sys.create_ilt(None, plused.clone(), hundred, None);
    sys.create_trigger(plus, vec![plused.clone(), plused.clone()], Some(cond));
  }

  let mut sys = SysBuilder::new("main");

  // Create a trivial module.
  let m1 = a_plus_b(&mut sys);

  // Build the driver module.
  build_driver(&mut sys, m1);

  println!("{}", sys);

  let fname = fname.to_str().unwrap().to_string();

  println!("Writing simulator code to {}", fname);

  let config = sim::Config {
    fname,
    sim_threshold: 200,
    idle_threshold: 200,
  };

  sim::elaborate(&sys, &config).unwrap();
}
