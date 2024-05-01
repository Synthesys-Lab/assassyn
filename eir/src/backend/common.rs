use crate::{builder::SysBuilder, test_utils};

pub struct Config {
  /// The name of the file to dump simulation code to
  pub temp_dir: bool,
  /// The number of cycles to simulate
  pub sim_threshold: usize,
  /// The number of cycles allowed to be idle before the simulation is stopped
  pub idle_threshold: usize,
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
}
