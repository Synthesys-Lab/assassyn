use crate::builder::system::SysBuilder;

mod callback;
mod rewrite_wait_until;

pub struct Config {
  pub rewrite_wait_until: bool,
}

pub fn basic(sys: &mut SysBuilder, config: &Config) {
  callback::rewrite_fifos(sys);
  if config.rewrite_wait_until {
    rewrite_wait_until::rewrite_wait_until(sys);
  }
}
