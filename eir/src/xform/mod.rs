use crate::builder::system::SysBuilder;

mod callback;
mod spin_trigger;

pub fn basic(sys: &mut SysBuilder) {
  callback::rewrite_fifos(sys);
  spin_trigger::rewrite_spin_triggers(sys);
}
