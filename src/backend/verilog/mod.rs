pub mod elaborate;
pub(super) mod gather;

pub use elaborate::elaborate;

#[derive(Default)]
pub enum Simulator {
  VCS,
  Verilator,
  #[default]
  None,
}

