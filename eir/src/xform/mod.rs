use crate::builder::system::SysBuilder;

pub mod arbiter;
pub mod cse;
pub mod rewrite_wait_until;
pub mod array_partition;

pub struct Config {
  pub rewrite_wait_until: bool,
}

pub fn basic(sys: &mut SysBuilder, config: &Config) {
  array_partition::rewrite_array_partitions(sys);
  arbiter::inject_arbiter(sys);
  cse::common_code_elimination(sys);
  if config.rewrite_wait_until {
    rewrite_wait_until::rewrite_wait_until(sys);
  }
}
