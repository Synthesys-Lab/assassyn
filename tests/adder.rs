use eda4eda::module_builder;

#[test]
fn foo() {
  module_builder!(adder[a:int<32>, b:int<32>][] {
    a  = a.pop();
    b  = b.pop();
    _c = a.add(b);
  });

  module_builder!(driver[][adder] {
    cnt = array(int<32>, 1);
    k = cnt[0.int<32>];
    v = k.add(1);
    cnt[0] = v;
    async adder { a: v, b: v };
  });

  let mut sys = eir::frontend::SysBuilder::new("main");
  let adder = adder_builder(&mut sys);
  driver_builder(&mut sys, adder);
}
