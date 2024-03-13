pub mod pred_propa;
pub mod spin_trigger;

pub use pred_propa::propagate_predications;
pub use spin_trigger::rewrite_spin_triggers;

use crate::builder::system::SysBuilder;

pub fn basic(sys: &mut SysBuilder) {
  propagate_predications(sys);
  rewrite_spin_triggers(sys);
}
