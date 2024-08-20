pub mod elaborate;
pub(super) mod gather;

pub use elaborate::elaborate;

pub enum Simulator {
  VCS,
  Verilator,
}
