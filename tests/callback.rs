// use eda4eda::module_builder;
// use eir::{builder::SysBuilder, sim, test_utils, xform};
//
// #[test]
// fn callback() {
//   module_builder!(
//     driver[][sqr, memory_read] {
//       cnt = array(int<32>, 1);
//       v = cnt[0];
//       async memory_read { v: v, func: sqr };
//       plused = v.add(1);
//       cnt[0] = plused;
//     }
//   );
//
//   module_builder!(
//     sqr[a:int<32>][] {
//       a = a.pop();
//       b = a.mul(a);
//       log("sqr: {}^2 = {}", a, b);
//     }
//   );
//
//   module_builder!(
//     agent[v:int<32>, func: module(int<32>)][] {
//       v = v.pop();
//       func = func.pop();
//       async func(v);
//     }
//   );
//
//   let mut sys = SysBuilder::new("callback");
//   let agent = agent_builder(&mut sys);
//   let sqr = sqr_builder(&mut sys);
//   let _ = driver_builder(&mut sys, sqr, agent);
//
//   println!("{}", sys);
//
//   let src_name = test_utils::temp_dir(&"callback.rs".to_string());
//   let exec_name = test_utils::temp_dir(&"callback".to_string());
//   let config = sim::Config {
//     fname: src_name,
//     idle_threshold: 100,
//     sim_threshold: 100,
//   };
//
//   xform::basic(&mut sys);
//   sim::elaborate(&sys, &config).unwrap();
//   test_utils::compile(&config.fname, &exec_name);
//   let output = test_utils::run(&exec_name);
//   let times_invoked = String::from_utf8(output.stdout)
//     .unwrap()
//     .lines()
//     .filter(|x| {
//       if x.contains("sqr: ") {
//         let raw = x.split(" ").collect::<Vec<&str>>();
//         let len = raw.len();
//         let raw_len = raw[len - 3].len();
//         let a = raw[len - 3][0..(raw_len - 2)].parse::<i32>().unwrap();
//         let b = raw[len - 1].parse::<i32>().unwrap();
//         assert_eq!(b, a * a);
//         true
//       } else {
//         false
//       }
//     })
//     .count();
//   assert_eq!(times_invoked, 99);
// }
