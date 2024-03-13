use crate::{
  builder::system::{PortInfo, SysBuilder},
  node::IsElement,
  sim::{self, elaborate},
  tests::utils,
  xform, BaseNode, DataType,
};

#[test]
fn callback() {
  fn memory(sys: &mut SysBuilder) -> BaseNode {
    let int32 = DataType::int(32);
    let port = PortInfo::new("addr", int32.clone());
    let callback = PortInfo::new("callback", DataType::module(vec![DataType::int(32)]));
    let res = sys.create_module("memory", vec![port, callback]);
    sys.set_current_module(&res);
    res
  }

  fn driver(sys: &mut SysBuilder, memory: BaseNode) {
    let driver = sys.get_driver().upcast();
    sys.set_current_module(&driver);
    let int32 = DataType::int(32);
    let cnt = sys.create_array(&int32, "cnt", 1);
    let zero = sys.get_const_int(&int32, 0);
    let a0ptr = sys.create_array_ptr(&cnt, &zero);
    let a0 = sys.create_array_read(&a0ptr, None);
    let one = sys.get_const_int(&int32, 1);
    let plused = sys.create_add(None, &a0, &one, None);
    sys.create_array_write(&a0ptr, &plused, None);
    sys.create_bundled_trigger(&memory, vec![a0], None);
  }

  let mut sys = SysBuilder::new("main");
  let memory = memory(&mut sys);
  driver(&mut sys, memory);
  println!("{}", sys);
  // xform::basic(&mut sys);
  // println!("{}", sys);

  // let config = sim::Config {
  //   fname: utils::temp_dir(&String::from("spin_trigger.rs")),
  //   sim_threshold: 200,
  //   idle_threshold: 200,
  // };

  // elaborate(&sys, &config).unwrap();
  // let exec_name = utils::temp_dir(&"trigger".to_string());
  // utils::compile(&config.fname, &exec_name);

  // // TODO(@were): Make a time timeout here.
  // utils::run(&exec_name);
}
