use crate::builder::system::SysBuilder;

mod callback;
mod rewrite_wait_until;
mod spin_trigger;

pub fn basic(sys: &mut SysBuilder) {
  callback::rewrite_fifos(sys);
  spin_trigger::rewrite_spin_triggers(sys);
  rewrite_wait_until::rewrite_wait_until(sys);
}
