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
