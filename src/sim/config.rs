
pub struct Config {
  /// The name of the file to dump simulation code to
  pub fname: String,
  /// The number of cycles to tolerate the idle
  pub idle_threshold: usize,
  /// The number of cycles to simulate
  pub sim_threshold: usize,
}



