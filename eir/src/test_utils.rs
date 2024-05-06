use std::process::Command;

use crate::{
  backend::{self, common::Config},
  builder::SysBuilder,
};

/// Put the given file into a temporary directory and return the path.
///
/// # Arguments
/// * `fname` - The name of the file to put into the temporary directory.
pub fn temp_dir(fname: &String) -> String {
  let dir = std::env::temp_dir();
  let fname = dir.join(fname);
  fname.to_str().unwrap().to_string()
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
  backend::simulator::elaborate(&sys, &config).unwrap();
  let dir_name = config.dir_name(sys);
  let manifest = format!("{}/Cargo.toml", dir_name);
  let output = Command::new("cargo")
    .arg("run")
    .arg("--release")
    .arg("--manifest-path")
    .arg(&manifest)
    .output()
    .unwrap_or_else(|_| panic!("Failed to run \"{}\"", config.dir_name(sys)));
  let raw_output = String::from_utf8(output.stdout).unwrap();
  println!("{}", raw_output);
  if let Some((func, cond_cnt)) = cond {
    let actual = raw_output.lines().filter(func).count();
    if let Some(expected) = cond_cnt {
      assert_eq!(actual, expected);
    }
  }
  raw_output
}
