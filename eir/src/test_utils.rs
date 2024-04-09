use std::process::{Command, Output};

pub fn compile(src: &String, exe: &String) {
  let output = Command::new("rustc")
    .arg(src)
    .arg("-o")
    .arg(exe)
    .output()
    .expect("Failed to compile");
  assert!(
    output.status.success(),
    "Failed to compile: {} to {}",
    src,
    exe
  );
  println!("Successfully compiled to {}", exe);
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

pub fn run(exe: &String) -> Output {
  let output = Command::new(exe).output().expect("Failed to run");
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
