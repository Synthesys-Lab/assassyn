use eda4eda::module_builder;
use eir;

#[test]
fn foo() {
  module_builder!(driver[][] {
    cnt = array(int<32>, 1);
    v = cnt[0];
    v = v.add(1);
    cnt[0] = v;
  });

  module_builder!(sqr[a:int<32>][] {
    a = a.pop();
    bind func { a: a };
    async sqr(a);
    b = a.mul(a);
  });
  let mut sys = eir::frontend::SysBuilder::new("a");
  sqr_builder(&mut sys);
}
