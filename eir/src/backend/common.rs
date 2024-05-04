use crate::{builder::SysBuilder, test_utils};

pub struct Config {
  /// The name of the file to dump simulation code to a temporary directory.
  pub temp_dir: bool,
  /// If true, the elaborator will remove the existing dump file before writing to it.
  pub override_dump: bool,
  /// The number of cycles to simulate
  pub sim_threshold: usize,
  /// The number of cycles allowed to be idle before the simulation is stopped
  pub idle_threshold: usize,
}

impl Default for Config {
  fn default() -> Self {
    Config {
      temp_dir: true,
      override_dump: true,
      sim_threshold: 100,
      idle_threshold: 100,
    }
  }
}

impl Config {
  pub fn fname(&self, sys: &SysBuilder, suffix: &str) -> String {
    let fname = format!("{}.{}", sys.get_name(), suffix);
    return if self.temp_dir {
      test_utils::temp_dir(&fname)
    } else {
      fname
    };
  }

  pub fn dir_name(&self, sys: &SysBuilder) -> String {
    return if self.temp_dir {
      test_utils::temp_dir(&sys.get_name().to_string())
    } else {
      sys.get_name().to_string()
    };
  }
}
