use inventory::module_builder;

#[test]
fn foo() {
  module_builder!(sqr[a:int<32>][] {
  });
  let mut sys = eda4eda::frontend::SysBuilder::new("a");
  sqr_builder(&mut sys);
}
