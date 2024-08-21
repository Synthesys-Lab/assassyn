pub mod elaborate;
pub(super) mod gather;
pub(super) mod utils;

pub use elaborate::elaborate;

#[derive(Default)]
pub enum Simulator {
  VCS,
  Verilator,
  #[default]
  None,
}
