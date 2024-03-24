use eda4eda::module_builder;

#[test]
fn callback() {
  module_builder!(
    driver[][] {
    }
  );

  module_builder!(
    callback[][] {
    }
  );

  module_builder!(
    memory_read[addr:int<32>, callback: module(int<32>)][] {
      addr = addr.pop();
      callback = callback.pop();
      // wait for 200 cycles
      // fabricate some data.
    }
  );
}
