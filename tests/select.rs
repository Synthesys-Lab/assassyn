use eda4eda::module_builder;
use eir::{builder::SysBuilder, test_utils};

#[test]
fn mux() {
  module_builder!(driver[][adder] {
    rng1 = array(int<32>, 1);
    rng2 = array(int<32>, 1);

    v0 = rng1[0];
    v1 = rng2[0];

    v0 = v0.mul(123456.int<32>);
    v1 = v1.mul(654321.int<32>);

    rand0 = v0.add(789012.int<32>);
    rand1 = v1.add(210987.int<32>);

    rand0 = rand0.slice(0, 31);
    rand1 = rand1.slice(0, 31);

    gt = rand0.igt(rand1);
    mux = default rand1.case(gt, rand0);

    log("muxed max: {}", mux);
  });
}
