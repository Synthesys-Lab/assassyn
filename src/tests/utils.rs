use std::process::Command;


pub(super) fn compile(src: &String, exe: &String) {
  let output = Command::new("rustc")
    .arg(src)
    .arg("-o")
    .arg(exe)
    .output()
    .expect("Failed to compile");
  assert!(output.status.success(), "Failed to compile: {} to {}", src, exe);
}

pub(super) fn temp_dir(fname: &String) -> String {
  let dir = std::env::temp_dir();
  let fname = dir.join(fname);
  fname.to_str().unwrap().to_string()
}

