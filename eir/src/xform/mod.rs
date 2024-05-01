use crate::builder::system::SysBuilder;

mod callback;
mod rewrite_wait_until;
mod spin_trigger;

pub struct Config {
  pub rewrite_wait_until: bool,
}

pub fn basic(sys: &mut SysBuilder, config: &Config) {
  callback::rewrite_fifos(sys);
  spin_trigger::rewrite_spin_triggers(sys);
  if config.rewrite_wait_until {
    rewrite_wait_until::rewrite_wait_until(sys);
  }
}
