use eda4eda::module_builder;

#[test]
fn foo() {
  module_builder!(adder[a:int::<32>, b:int::<32>][] {
    a  = a.pop();
    b  = b.pop();
    _c = a.add(b);
  });

  module_builder!(driver[][] {
    a = array(int::<32>, 1);
  });

  // let mut sys = eir::frontend::SysBuilder::new("a");
  // sqr_builder(&mut sys);
}
