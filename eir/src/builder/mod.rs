// The module of the IR and the IR builder.

pub mod system;
pub mod verify;

pub use system::{InsertPoint, PortInfo, SysBuilder};
pub use verify::verify;
