use crate::builder::system::SysBuilder;

mod callback;
mod hoist_wait_until;
mod spin_trigger;

pub fn basic(sys: &mut SysBuilder) {
  callback::rewrite_fifos(sys);
  spin_trigger::rewrite_spin_triggers(sys);
  hoist_wait_until::hoist_wait_until(sys);
}
