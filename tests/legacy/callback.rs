use crate::{frontend::*, module_builder};

#[test]
fn callback() {
  module_builder!(
    callbacked[callback:module(int<32>, int<32>),][] {
    }
  );

  fn module_with_callback(sys: &mut SysBuilder) -> BaseNode {
    let int32 = DataType::int(32);
    let port = PortInfo::new("addr", int32.clone());
    let callback = PortInfo::new("callback", DataType::module(vec![DataType::int(32)]));
    let res = sys.create_module("module_with_callback", vec![port, callback]);
    sys.set_current_module(&res);
    let (data, callback) = {
      let module = sys.get_current_module().unwrap();
      (
        module.get_input(0).unwrap().clone(),
        module.get_input(1).unwrap().clone(),
      )
    };
    let data = sys.create_fifo_pop(&data, None);
    let callback = sys.create_fifo_pop(&callback, None);
    sys.create_bundled_trigger(&callback, vec![data]);
    res
  }

  fn driver(sys: &mut SysBuilder, mwcb: BaseNode, sqr: BaseNode) {
    let driver = sys.create_module("driver", vec![]);
    sys.set_current_module(&driver);
    let int32 = DataType::int(32);
    let cnt = sys.create_array(&int32, "cnt", 1);
    let zero = sys.get_const_int(&int32, 0);
    let a0ptr = sys.create_array_ptr(&cnt, &zero);
    let a0 = sys.create_array_read(&a0ptr);
    let one = sys.get_const_int(&int32, 1);
    let plused = sys.create_add(None, &a0, &one);
    sys.create_array_write(&a0ptr, &plused);
    sys.create_bundled_trigger(&mwcb, vec![a0, sqr]);
  }

  fn squarer(sys: &mut SysBuilder) -> BaseNode {
    let int32 = DataType::int(32);
    let port = PortInfo::new("i0", int32.clone());
    let module = sys.create_module("square", vec![port]);
    sys.set_current_module(&module);
    let i0 = {
      let module = module.as_ref::<Module>(sys).unwrap();
      let i0_port = module.get_input(0).unwrap().clone();
      sys.create_fifo_pop(&i0_port, None)
    };
    sys.create_mul(Some(int32), &i0, &i0);
    module
  }

  let mut sys = SysBuilder::new("main");
  let sqr = squarer(&mut sys);
  let mwcb = module_with_callback(&mut sys);
  driver(&mut sys, mwcb, sqr);
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
