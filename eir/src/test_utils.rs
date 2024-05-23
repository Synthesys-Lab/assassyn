use std::process::Command;

use crate::{
  backend::{self, common::Config},
  builder::SysBuilder,
};

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
  backend::simulator::elaborate(&sys, &config).unwrap();
  let dir_name = config.dir_name(sys);
  let manifest = dir_name.join("Cargo.toml");
  let output = Command::new("cargo")
    .arg("run")
    .arg("--release")
    .arg("--manifest-path")
    .arg(&manifest)
    .output()
    .unwrap_or_else(|_| panic!("Failed to run \"{}\"", dir_name.to_str().unwrap()));
  assert!(output.status.success());
  let raw_output = String::from_utf8(output.stdout).unwrap();
  println!("{}", raw_output);
  assert!(
    output.status.success(),
    "Failed to run \"{}\"",
    dir_name.to_str().unwrap()
  );
  if let Some((func, cond_cnt)) = cond {
    let actual = raw_output.lines().filter(func).count();
    if let Some(expected) = cond_cnt {
      assert_eq!(actual, expected);
    }
  }
  raw_output
}
