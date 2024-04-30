use std::process::{Command, Output};

use crate::{
  backend::{self, common::Config},
  builder::SysBuilder,
};

pub fn compile(src: &String, exe: Option<&String>) -> String {
  let obj = exe.map_or_else(|| src[..src.len() - 3].to_string(), |x| x.clone());
  let output = Command::new("rustc")
    .arg(src)
    .arg("-o")
    .arg(&obj)
    .output()
    .expect("Failed to compile");
  assert!(
    output.status.success(),
    "Failed to compile: {} to {}",
    src,
    obj
  );
  println!("Successfully compiled to {}", obj);
  return obj;
}

/// Put the given file into a temporary directory and return the path.
///
/// # Arguments
/// * `fname` - The name of the file to put into the temporary directory.
pub fn temp_dir(fname: &String) -> String {
  let dir = std::env::temp_dir();
  let fname = dir.join(fname);
  fname.to_str().unwrap().to_string()
}

pub fn run_exec(exe: &String) -> Output {
  let exe = if !exe.contains("/") {
    format!("./{}", exe)
  } else {
    exe.clone()
  };
  let output = Command::new(&exe)
    .output()
    .unwrap_or_else(|_| panic!("Failed to run \"{}\"", exe));
  assert!(output.status.success(), "Failed to run: {}", exe);
  output
}

pub fn parse_cycle(raw_line: &str) -> (usize, usize) {
  let toks = raw_line.split_whitespace().collect::<Vec<_>>();
  let cycle_tok_idx = 2;
  let len = toks[cycle_tok_idx].len();
  let cycle = toks[cycle_tok_idx][1..len - 4].parse::<usize>().unwrap();
  let half = toks[cycle_tok_idx][len - 3..len - 1]
    .parse::<usize>()
    .unwrap();
  (cycle, half)
}

pub fn run_simulator(
  sys: &SysBuilder,
  config: &Config,
  cond: Option<(fn(&&str) -> bool, Option<usize>)>,
) -> String {
  let simulator_rs = backend::simulator::elaborate(&sys, &config).unwrap();
  let simulator_bin = compile(&simulator_rs, None);
  let output = run_exec(&simulator_bin);
  let raw_output = String::from_utf8(output.stdout).unwrap();
  if let Some((func, cond_cnt)) = cond {
    let actual = raw_output.lines().filter(func).count();
    if let Some(expected) = cond_cnt {
      assert_eq!(actual, expected);
    }
  }
  raw_output
}
