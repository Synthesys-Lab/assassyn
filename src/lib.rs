pub mod builder;
pub mod sim;
pub mod xform;
pub mod ir;

pub use ir::expr;

pub use ir::reference;
pub use ir::reference::Reference;

pub use ir::data::DataType;
pub use ir::data::IntImm;
pub use ir::data;

pub use ir::module::Module;

pub use ir::port;

#[cfg(test)]
mod tests;
