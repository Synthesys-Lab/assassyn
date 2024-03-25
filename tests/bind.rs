use eda4eda::module_builder;

#[test]
fn bind() {
  module_builder!(adder[a:int<32>, b:int<32>][] {
    a  = a.pop();
    b  = b.pop();
    _c = a.add(b);
  });

  module_builder!(driver[][lhs, rhs] {
    cnt = array(int<32>, 1);
    k = cnt[0.int<32>];
    v = k.add(1);
    cnt[0] = v;
    async lhs { a: v };
    async rhs { a: v };
  });

  module_builder!(
    lhs[a:int<32>][adder] {
      v = a.pop();
      aa = bind adder { a: v };
    }.expose[aa]
  );

  module_builder!(
    rhs[a:int<32>][aa] {
      v = a.pop();
      async aa { b: v };
    }
  );

  let mut sys = eir::frontend::SysBuilder::new("main");
  let adder = adder_builder(&mut sys);
  let (lhs, aa) = lhs_builder(&mut sys, adder);
  let rhs = rhs_builder(&mut sys, aa);
  driver_builder(&mut sys, lhs, rhs);
  println!("{}", sys);
}
