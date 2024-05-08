use crate::builder::system::SysBuilder;

pub mod callback;
pub mod common_read;
pub mod rewrite_wait_until;

pub struct Config {
  pub rewrite_wait_until: bool,
}

pub fn basic(sys: &mut SysBuilder, config: &Config) {
  callback::rewrite_fifos(sys);
  common_read::unify_common_reads(sys);
  if config.rewrite_wait_until {
    rewrite_wait_until::rewrite_wait_until(sys);
  }
}
