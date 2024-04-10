use crate::builder::system::SysBuilder;

mod fifo;
mod spin_trigger;

pub fn basic(sys: &mut SysBuilder) {
  fifo::rewrite_fifos(sys);
  spin_trigger::rewrite_spin_triggers(sys);
}
