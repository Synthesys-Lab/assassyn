pub mod spin_trigger;

pub use spin_trigger::rewrite_spin_triggers;

use crate::builder::system::SysBuilder;

pub fn basic(sys: &mut SysBuilder) {
  rewrite_spin_triggers(sys);
}
