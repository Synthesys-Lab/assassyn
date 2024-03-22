use crate::builder::system::SysBuilder;

mod spin_trigger;

pub fn basic(sys: &mut SysBuilder) {
  spin_trigger::rewrite_spin_triggers(sys);
}
