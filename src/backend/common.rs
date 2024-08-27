use std::{collections::HashSet, env, fs, path::PathBuf};

use crate::{
  builder::SysBuilder,
  ir::{
    node::{BaseNode, ModuleRef},
    Expr,
  },
};

use super::verilog;

pub struct Config {
  /// The name of the file to dump simulation code to a temporary directory.
  pub base_dir: PathBuf,
  /// If true, the elaborator will remove the existing dump file before writing to it.
  pub override_dump: bool,
  /// The number of cycles to simulate
  pub sim_threshold: usize,
  /// The number of cycles allowed to be idle before the simulation is stopped
  pub idle_threshold: usize,
  /// If true, the order of simulators will be randomized.
  pub random: bool,
  /// The base directory of memory initialization files
  pub resource_base: PathBuf,
  /// The simulator to use for verilog simulation
  pub verilog: verilog::Simulator,
}

impl Default for Config {
  fn default() -> Self {
    Config {
      base_dir: env::temp_dir(),
      override_dump: true,
      sim_threshold: 100,
      idle_threshold: 100,
      random: false,
      resource_base: PathBuf::new(),
      verilog: verilog::Simulator::default(),
    }
  }
}

impl Config {
  /// The name of the file to which the elaboration code is dumped.
  pub fn fname(&self, sys: &SysBuilder, suffix: &str) -> PathBuf {
    let fname = format!("{}.{}", sys.get_name(), suffix);
    self.base_dir.join(fname)
  }

  /// The name of the directory with a custom suffix.
  pub fn dirname(&self, sys: &SysBuilder, suf: &str) -> PathBuf {
    let name_with_suffix = format!("{}_{}", sys.get_name(), suf);
    self.base_dir.join(name_with_suffix)
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

pub(super) fn namify(name: &str) -> String {
  name.replace('.', "_")
}

pub(super) fn upstreams(m: &ModuleRef<'_>) -> HashSet<BaseNode> {
  assert!(m.is_downstream());
  let mut res = HashSet::new();
  for (interf, _) in m.ext_interf_iter() {
    if let Ok(expr) = interf.as_ref::<Expr>(m.sys) {
      res.insert(expr.get_block().get_module());
    }
  }
  res
}
