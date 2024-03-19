use eda4eda::module_builder;

#[test]
fn foo() {
  // module_builder!(driver[][] {});

  module_builder!(sqr[a:int<32>][] {
    a = a.pop();
    b = a.mul(a);
  });

  // let mut sys = eir::frontend::SysBuilder::new("a");
  // sqr_builder(&mut sys);
}
