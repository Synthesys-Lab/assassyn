use crate::builder::system::SysBuilder;

pub mod arbiter;
pub mod array_partition;
pub mod cse;
pub mod erase_metadata;
pub mod rewrite_wait_until;

pub struct Config {
  pub rewrite_wait_until: bool,
}

pub fn basic(sys: &mut SysBuilder, config: &Config) {
  arbiter::inject_arbiter(sys);
  array_partition::rewrite_array_partitions(sys);
  cse::common_code_elimination(sys);
  if config.rewrite_wait_until {
    rewrite_wait_until::rewrite_wait_until(sys);
  }
}
