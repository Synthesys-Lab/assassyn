use crate::{builder::system::SysBuilder, module_builder};


#[test]
fn parser() {
  let mut SysBuilder = SysBuilder::new("main");
  module_builder!(sys, driver);
}
