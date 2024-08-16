// The module of the IR and the IR builder.

pub(crate) mod symbol_table;
pub mod system;
pub mod verify;

pub use system::{PortInfo, SysBuilder};
pub use verify::verify;
