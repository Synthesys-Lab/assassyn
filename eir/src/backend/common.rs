use std::{env, fs, path::PathBuf};

use crate::builder::SysBuilder;

pub struct Config {
  /// The name of the file to dump simulation code to a temporary directory.
  pub base_dir: PathBuf,
  /// If true, the elaborator will remove the existing dump file before writing to it.
  pub override_dump: bool,
  /// The number of cycles to simulate
  pub sim_threshold: usize,
  /// The number of cycles allowed to be idle before the simulation is stopped
  pub idle_threshold: usize,
  /// The base directory of memory initialization files
  pub resource_base: PathBuf,
}

impl Default for Config {
  fn default() -> Self {
    Config {
      base_dir: env::temp_dir(),
      override_dump: true,
      sim_threshold: 100,
      idle_threshold: 100,
      resource_base: PathBuf::new(),
    }
  }
}

impl Config {
  /// The name of the file to which the elaboration code is dumped.
  pub fn fname(&self, sys: &SysBuilder, suffix: &str) -> PathBuf {
    let fname = format!("{}.{}", sys.get_name(), suffix);
    self.base_dir.join(fname).into()
  }

  /// The name of the directory to which the elaboration code is dumped.
  pub fn dir_name(&self, sys: &SysBuilder) -> PathBuf {
    self.base_dir.join(sys.get_name())
  }
}

pub(super) fn create_and_clean_dir(dir: PathBuf, override_dir: bool) {
  if !dir.exists() {
    fs::create_dir_all(&dir).unwrap();
  }
  assert!(dir.is_dir());
  let files = fs::read_dir(&dir).unwrap();
  if override_dir {
    for elem in files {
      let path = elem.unwrap().path();
      if path.is_dir() {
        fs::remove_dir_all(path).unwrap();
      } else {
        fs::remove_file(path).unwrap();
      }
    }
  } else {
    assert!(files.count() == 0);
  }
}
