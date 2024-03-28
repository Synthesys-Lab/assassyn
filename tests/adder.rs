use eda4eda::module_builder;
use eir::test_utils;

#[test]
fn adder() {
  module_builder!(adder[a:int<32>, b:int<32>][] {
    a = a.pop();
    b = b.pop();
    c = a.add(b);
    log("adder: {} + {} = {}", a, b, c);
  });

  module_builder!(driver[][adder] {
    cnt = array(int<32>, 1);
    k = cnt[0.int<32>];
    v = k.add(1);
    cnt[0] = v;
    async adder { a: v, b: v };
  });

  let mut sys = eir::frontend::SysBuilder::new("main");
  let adder = adder_builder(&mut sys);
  driver_builder(&mut sys, adder);

  let src_name = test_utils::temp_dir(&"adder.rs".to_string());
  let config = eir::sim::Config {
    fname: src_name,
    sim_threshold: 100,
    idle_threshold: 100,
  };

  eir::sim::elaborate(&sys, &config).unwrap();

  let exec_name = test_utils::temp_dir(&"adder".to_string());
  test_utils::compile(&config.fname, &exec_name);

  let output = test_utils::run(&exec_name);
  let times_invoked = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .filter(|x| {
      if x.contains("adder") {
        let raw = x.split(" ").collect::<Vec<&str>>();
        let len = raw.len();
        let a = raw[len - 5].parse::<i32>().unwrap();
        let b = raw[len - 3].parse::<i32>().unwrap();
        let c = raw[len - 1].parse::<i32>().unwrap();
        assert_eq!(c, a + b);
        true
      } else {
        false
      }
    })
    .count();
  assert_eq!(times_invoked, 100);
}
